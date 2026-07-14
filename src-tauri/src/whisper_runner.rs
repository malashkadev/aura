use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::{Manager, Emitter, Runtime};
use std::collections::HashSet;
use std::sync::{Mutex, OnceLock};

/// Models whose in-flight download the user asked to cancel (keyed by model name).
static DOWNLOAD_CANCEL: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

fn cancel_set() -> &'static Mutex<HashSet<String>> {
    DOWNLOAD_CANCEL.get_or_init(|| Mutex::new(HashSet::new()))
}

/// Requests cancellation of the running download for `model_name`.
pub fn request_cancel_download(model_name: &str) {
    if let Ok(mut set) = cancel_set().lock() {
        set.insert(model_name.to_string());
    }
}

fn is_cancel_requested(model_name: &str) -> bool {
    cancel_set().lock().map(|s| s.contains(model_name)).unwrap_or(false)
}

fn clear_cancel(model_name: &str) {
    if let Ok(mut set) = cancel_set().lock() {
        set.remove(model_name);
    }
}

#[derive(Clone, serde::Serialize)]
pub struct DownloadProgress {
    pub model: String,
    pub downloaded: u64,
    pub total: Option<u64>,
    pub percentage: f64,
    pub done: bool,
}

/// Helper to search recursively for the sidecar file if not in direct paths
fn find_file_recursive(dir: &Path, target_name: &str) -> Option<PathBuf> {
    if dir.is_dir() {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(found) = find_file_recursive(&path, target_name) {
                        return Some(found);
                    }
                } else if path.file_name().is_some_and(|name| name == target_name) {
                    return Some(path);
                }
            }
        }
    }
    None
}

/// Format the model filename correctly to prevent duplicates (e.g., ggml-ggml-tiny.bin)
fn format_model_filename(model_name: &str) -> String {
    let name_without_ggml = model_name.strip_prefix("ggml-").unwrap_or(model_name);
    let name_without_bin = name_without_ggml.strip_suffix(".bin").unwrap_or(name_without_ggml);
    format!("ggml-{}.bin", name_without_bin)
}

/// Checks that the whisper runtime DLLs live next to the sidecar (or in its
/// `binaries/` subfolder) and are real files, not zero-byte placeholders.
fn dir_has_runtime_dlls(exe_path: &Path) -> bool {
    #[cfg(target_os = "macos")]
    {
        let _ = exe_path;
        true // macOS does not use DLLs
    }
    #[cfg(not(target_os = "macos"))]
    {
        let Some(dir) = exe_path.parent() else { return false };
        let check = |d: &Path| {
            ["whisper.dll", "ggml.dll"].iter().all(|name| {
                fs::metadata(d.join(name)).map(|m| m.len() > 1024).unwrap_or(false)
            })
        };
        check(dir) || check(&dir.join("binaries")) || check(&dir.join("resources").join("binaries"))
    }
}

/// Locate the whisper sidecar executable under the resources directory or fallback paths.
/// Candidates with the runtime DLLs beside them are preferred; broken copies
/// (e.g. a bare exe in target/debug without its DLLs) are used only as a last resort.
pub fn find_sidecar<R: Runtime>(app_handle: &tauri::AppHandle<R>) -> Result<PathBuf, String> {
    // In release bundles Tauri strips the target triple from externalBin names
    #[cfg(target_os = "windows")]
    let target_names = [
        "whisper-sidecar-x86_64-pc-windows-msvc.exe",
        "whisper-sidecar.exe",
    ];
    #[cfg(target_os = "macos")]
    let target_names = [
        "whisper-sidecar-x86_64-apple-darwin",
        "whisper-sidecar-aarch64-apple-darwin",
        "whisper-sidecar",
    ];
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let target_names = ["whisper-sidecar"];

    let resource_dir = app_handle
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?;

    let mut candidates: Vec<PathBuf> = Vec::new();

    // Dev builds: the source binaries folder always has the exe plus all DLLs
    #[cfg(debug_assertions)]
    for name in &target_names {
        candidates.push(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("binaries").join(name),
        );
    }

    for name in &target_names {
        candidates.push(resource_dir.join("binaries").join(name));
        candidates.push(resource_dir.join("_up_").join("binaries").join(name));
        candidates.push(resource_dir.join(name));
        // CWD-relative fallbacks (dev mode launched from the workspace root)
        candidates.push(PathBuf::from("binaries").join(name));
        candidates.push(PathBuf::from("src-tauri").join("binaries").join(name));
    }

    // Bundled apps place externalBin next to the main executable
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            for name in &target_names {
                candidates.push(exe_dir.join(name));
            }
        }
    }

    let existing: Vec<PathBuf> = candidates.into_iter().filter(|p| p.exists()).collect();

    // Prefer a copy that has its runtime DLLs; fall back to any existing copy
    if let Some(path) = existing.iter().find(|p| dir_has_runtime_dlls(p)) {
        return Ok(path.clone());
    }
    if let Some(path) = existing.first() {
        #[cfg(target_os = "windows")]
        eprintln!(
            "Aura Dev Log WARNING: sidecar found at {:?} but whisper.dll/ggml.dll are missing next to it.",
            path
        );
        return Ok(path.clone());
    }

    // Last resort: recursive search of the resource dir
    for name in &target_names {
        if let Some(path) = find_file_recursive(&resource_dir, name) {
            return Ok(path);
        }
    }

    Err(format!(
        "Could not find sidecar executable in resource dir ({:?}) or current working directory.",
        resource_dir
    ))
}

struct DeleteOnDrop {
    path: std::path::PathBuf,
    active: bool,
}

impl Drop for DeleteOnDrop {
    fn drop(&mut self) {
        if self.active && self.path.exists() {
            let _ = std::fs::remove_file(&self.path);
        }
    }
}

/// Download a GGML model file from huggingface if it doesn't already exist on disk
pub async fn download_model<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
    model_name: &str,
) -> Result<PathBuf, String> {
    let filename = format_model_filename(model_name);

    let app_local_data = app_handle
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app local data dir: {}", e))?;
    let models_dir = app_local_data.join("models");

    if !models_dir.exists() {
        fs::create_dir_all(&models_dir)
            .map_err(|e| format!("Failed to create models directory: {}", e))?;
    }

    let dest_path = models_dir.join(&filename);
    if dest_path.exists() {
        let size = fs::metadata(&dest_path).map(|m| m.len()).unwrap_or(0);
        if size > 0 {
            let _ = app_handle.emit(
                "model-download-progress",
                DownloadProgress {
                    model: model_name.to_string(),
                    downloaded: size,
                    total: Some(size),
                    percentage: 100.0,
                    done: true,
                },
            );
            return Ok(dest_path);
        }
    }

    // Clear any stale cancel flag from a previous attempt for this model
    clear_cancel(model_name);

    let url = format!(
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}",
        filename
    );

    let client = crate::ai_client::build_download_client();
    let mut response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| format!("Failed to send request: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "Failed to download model: HTTP status {}",
            response.status()
        ));
    }

    let total_size = response.content_length();
    let temp_path = models_dir.join(format!("{}.tmp", filename));

    if temp_path.exists() {
        let _ = fs::remove_file(&temp_path);
    }

    let mut delete_guard = DeleteOnDrop {
        path: temp_path.clone(),
        active: true,
    };

    let mut file = tokio::fs::File::create(&temp_path)
        .await
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    let mut downloaded: u64 = 0;
    while let Some(chunk) = tokio::time::timeout(std::time::Duration::from_secs(30), response.chunk())
        .await
        .map_err(|_| "Download timed out".to_string())?
        .map_err(|e| format!("Error while downloading chunk: {}", e))?
    {
        // Abort promptly if the user pressed the cancel (×) button
        if is_cancel_requested(model_name) {
            drop(file);
            let _ = fs::remove_file(&temp_path);
            clear_cancel(model_name);
            return Err("Download cancelled".to_string());
        }

        use tokio::io::AsyncWriteExt;
        file.write_all(&chunk)
            .await
            .map_err(|e| format!("Failed to write chunk: {}", e))?;
        downloaded += chunk.len() as u64;

        let percentage = if let Some(total) = total_size {
            (downloaded as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        let _ = app_handle.emit(
            "model-download-progress",
            DownloadProgress {
                model: model_name.to_string(),
                downloaded,
                total: total_size,
                percentage,
                done: false,
            },
        );
    }

    use tokio::io::AsyncWriteExt;
    file.flush()
        .await
        .map_err(|e| format!("Failed to flush temp file: {}", e))?;
    drop(file);

    delete_guard.active = false;

    fs::rename(&temp_path, &dest_path)
        .map_err(|e| format!("Failed to rename temp file to destination: {}", e))?;

    clear_cancel(model_name);

    let _ = app_handle.emit(
        "model-download-progress",
        DownloadProgress {
            model: model_name.to_string(),
            downloaded,
            total: total_size,
            percentage: 100.0,
            done: true,
        },
    );

    Ok(dest_path)
}

/// Run transcription using the local Whisper sidecar binary and return the result.
/// `language` accepts "ru"/"en" to force a language; anything else auto-detects.
/// `dictionary` (comma-separated terms) is passed as the initial prompt to bias recognition.
pub fn run_local_whisper<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
    model_name: &str,
    wav_path: &str,
    language: &str,
    dictionary: &str,
) -> Result<String, String> {
    // 1. Locate sidecar binary
    let sidecar_path = find_sidecar(app_handle)?;
    let short_sidecar_path = get_short_path(&sidecar_path)?;
    let sidecar_dir = sidecar_path
        .parent()
        .ok_or_else(|| "Invalid sidecar path".to_string())?;
    let short_sidecar_dir = get_short_path(sidecar_dir)?;

    // Resolve short path of resource DLLs folder (under resource_dir/binaries/)
    let resource_dir = app_handle
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?;
    let dlls_dir = resource_dir.join("binaries");
    let short_dlls_dir = get_short_path(&dlls_dir)?;

    // 2. Resolve model path
    let filename = format_model_filename(model_name);

    let app_local_data = app_handle
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app local data dir: {}", e))?;
    let model_path = app_local_data.join("models").join(&filename);

    if !model_path.exists() {
        return Err(format!(
            "Model file not found at: {:?}. Please download it first.",
            model_path
        ));
    }

    // Convert model and wav paths to short 8.3 representations
    let short_model_path = get_short_path(&model_path)?;
    let short_wav_path = get_short_path(Path::new(wav_path))?;

    let lang = match language {
        "ru" | "en" | "de" | "es" | "fr" | "it" | "zh" | "pt" | "tr" => language,
        _ => "auto",
    };

    // whisper.cpp defaults to only 4 threads; on modern many-core CPUs that leaves
    // most of the machine idle (~20% usage). Use all available logical cores so
    // transcription runs several times faster.
    let n_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4);

    let mut args: Vec<String> = vec![
        "-m".to_string(),
        short_model_path.to_str().ok_or("Invalid model path encoding")?.to_string(),
        "-f".to_string(),
        short_wav_path.to_str().ok_or("Invalid wav path encoding")?.to_string(),
        "-l".to_string(),
        lang.to_string(),
        "-t".to_string(),
        n_threads.to_string(),
        "-nt".to_string(),
        "-np".to_string(),
        // Greedy decoding (beam_size=1): ~20-40% faster than default beam-search
        // with negligible quality loss for real-time dictation use cases.
        "--best-of".to_string(),
        "1".to_string(),
        "--beam-size".to_string(),
        "-1".to_string(),
    ];

    let dict = dictionary.trim();
    if !dict.is_empty() {
        args.push("--prompt".to_string());
        args.push(dict.to_string());
    }

    let mut cmd = Command::new(&short_sidecar_path);
    cmd.current_dir(&short_dlls_dir);
    cmd.args(&args);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW (hides the black console window)
    }

    #[cfg(target_os = "windows")]
    {
        // Prepend sidecar executable directory, resources, and dll paths to PATH for Windows DLL resolution
        let path_key = std::env::vars_os()
            .map(|(k, _)| k.to_string_lossy().into_owned())
            .find(|name| name.eq_ignore_ascii_case("path"))
            .unwrap_or_else(|| "PATH".to_string());

        let mut paths = if let Some(path_env) = std::env::var_os(&path_key) {
            std::env::split_paths(&path_env).collect::<Vec<_>>()
        } else {
            Vec::new()
        };
        
        // Add all possible location candidates of whisper.dll / ggml.dll to guarantee resolution
        if let Some(parent) = sidecar_path.parent() {
            paths.insert(0, parent.join("resources").join("binaries"));
            paths.insert(0, parent.to_path_buf());
        }
        paths.insert(0, dlls_dir.clone());
        paths.insert(0, short_sidecar_dir.to_path_buf());
        paths.insert(0, short_dlls_dir.to_path_buf());

        if let Ok(new_path) = std::env::join_paths(paths) {
            cmd.env(&path_key, new_path);
        }
    }

    let output = cmd
        .output()
        .map_err(|e| format!("Failed to run sidecar command: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // An empty stderr with an odd exit code usually means missing DLLs
        // (STATUS_DLL_NOT_FOUND = 0xC0000135) — include the code for diagnostics.
        return Err(format!(
            "Whisper sidecar exited with error (code {:?}, path {:?}): {}",
            output.status.code(),
            sidecar_path,
            stderr
        ));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    Ok(text.trim().to_string())
}

pub fn find_sherpa_sidecar<R: Runtime>(app_handle: &tauri::AppHandle<R>) -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    let target_names = [
        "sherpa-onnx-offline-x86_64-pc-windows-msvc.exe",
        "sherpa-onnx-offline.exe",
    ];
    #[cfg(target_os = "macos")]
    let target_names = [
        "sherpa-onnx-offline-aarch64-apple-darwin",
        "sherpa-onnx-offline-x86_64-apple-darwin",
        "sherpa-onnx-offline",
    ];
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let target_names = ["sherpa-onnx-offline"];

    let resource_dir = app_handle
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?;

    let mut candidates: Vec<PathBuf> = Vec::new();

    #[cfg(debug_assertions)]
    for name in &target_names {
        candidates.push(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("binaries").join(name),
        );
    }

    for name in &target_names {
        candidates.push(resource_dir.join("binaries").join(name));
        candidates.push(resource_dir.join("_up_").join("binaries").join(name));
        candidates.push(resource_dir.join(name));
        candidates.push(PathBuf::from("binaries").join(name));
        candidates.push(PathBuf::from("src-tauri").join("binaries").join(name));
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            for name in &target_names {
                candidates.push(exe_dir.join(name));
            }
        }
    }

    let existing: Vec<PathBuf> = candidates.into_iter().filter(|p| p.exists()).collect();

    if let Some(path) = existing.first() {
        return Ok(path.clone());
    }

    for name in &target_names {
        if let Some(path) = find_file_recursive(&resource_dir, name) {
            return Ok(path);
        }
    }

    Err(format!(
        "Could not find sherpa-onnx sidecar executable in resource dir ({:?}).",
        resource_dir
    ))
}

pub async fn download_parakeet_model<R: Runtime>(
    app_handle: &tauri::AppHandle<R>
) -> Result<PathBuf, String> {
    let app_local_data = app_handle
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app local data dir: {}", e))?;
    let parakeet_dir = app_local_data.join("models").join("parakeet-v3");

    if !parakeet_dir.exists() {
        fs::create_dir_all(&parakeet_dir)
            .map_err(|e| format!("Failed to create parakeet-v3 directory: {}", e))?;
    }

    let files = [
        ("encoder.int8.onnx", "encoder.onnx"),
        ("decoder.int8.onnx", "decoder.onnx"),
        ("joiner.int8.onnx", "joiner.onnx"),
        ("tokens.txt", "tokens.txt"),
    ];

    let all_exist = files.iter().all(|(_, local)| parakeet_dir.join(local).exists());
    if all_exist {
        let _ = app_handle.emit(
            "model-download-progress",
            DownloadProgress {
                model: "parakeet-v3".to_string(),
                downloaded: 670_000_000,
                total: Some(670_000_000),
                percentage: 100.0,
                done: true,
            },
        );
        return Ok(parakeet_dir);
    }

    clear_cancel("parakeet-v3");

    let total_estimated_size: u64 = 670_000_000;
    let mut total_downloaded: u64 = 0;

    let client = crate::ai_client::build_download_client();

    for (remote_file, local_file) in &files {
        let url = format!(
            "https://huggingface.co/csukuangfj/sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8/resolve/main/{}",
            remote_file
        );
        let mut response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Failed to download {}: {}", remote_file, e))?;

        if !response.status().is_success() {
            return Err(format!(
                "Failed to download {}: HTTP status {}",
                remote_file,
                response.status()
            ));
        }

        let dest_path = parakeet_dir.join(local_file);
        let temp_path = parakeet_dir.join(format!("{}.tmp", local_file));

        if temp_path.exists() {
            let _ = fs::remove_file(&temp_path);
        }

        let mut delete_guard = DeleteOnDrop {
            path: temp_path.clone(),
            active: true,
        };

        let mut file = tokio::fs::File::create(&temp_path)
            .await
            .map_err(|e| format!("Failed to create temp file for {}: {}", local_file, e))?;

        while let Some(chunk) = tokio::time::timeout(std::time::Duration::from_secs(30), response.chunk())
            .await
            .map_err(|_| "Download timed out".to_string())?
            .map_err(|e| format!("Error while downloading chunk of {}: {}", remote_file, e))?
        {
            if is_cancel_requested("parakeet-v3") {
                drop(file);
                let _ = fs::remove_file(&temp_path);
                clear_cancel("parakeet-v3");
                return Err("Download cancelled".to_string());
            }

            use tokio::io::AsyncWriteExt;
            file.write_all(&chunk)
                .await
                .map_err(|e| format!("Failed to write chunk of {}: {}", local_file, e))?;
            total_downloaded += chunk.len() as u64;

            let percentage = (total_downloaded as f64 / total_estimated_size as f64 * 100.0).min(99.9);

            let _ = app_handle.emit(
                "model-download-progress",
                DownloadProgress {
                    model: "parakeet-v3".to_string(),
                    downloaded: total_downloaded,
                    total: Some(total_estimated_size),
                    percentage,
                    done: false,
                },
            );
        }

        use tokio::io::AsyncWriteExt;
        file.flush()
            .await
            .map_err(|e| format!("Failed to flush temp file for {}: {}", local_file, e))?;
        drop(file);

        delete_guard.active = false;

        fs::rename(&temp_path, &dest_path)
            .map_err(|e| format!("Failed to rename temp file for {} to destination: {}", local_file, e))?;
    }

    clear_cancel("parakeet-v3");

    let _ = app_handle.emit(
        "model-download-progress",
        DownloadProgress {
            model: "parakeet-v3".to_string(),
            downloaded: total_estimated_size,
            total: Some(total_estimated_size),
            percentage: 100.0,
            done: true,
        },
    );

    Ok(parakeet_dir)
}

pub fn find_sherpa_websocket_server<R: Runtime>(app_handle: &tauri::AppHandle<R>) -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    let target_names = [
        "sherpa-onnx-offline-websocket-server.exe",
    ];
    #[cfg(not(target_os = "windows"))]
    let target_names = ["sherpa-onnx-offline-websocket-server"];

    let resource_dir = app_handle
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?;

    let mut candidates: Vec<PathBuf> = Vec::new();

    #[cfg(debug_assertions)]
    for name in &target_names {
        candidates.push(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("binaries").join(name),
        );
    }

    for name in &target_names {
        candidates.push(resource_dir.join("binaries").join(name));
        candidates.push(resource_dir.join("_up_").join("binaries").join(name));
        candidates.push(resource_dir.join(name));
        candidates.push(PathBuf::from("binaries").join(name));
        candidates.push(PathBuf::from("src-tauri").join("binaries").join(name));
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            for name in &target_names {
                candidates.push(exe_dir.join(name));
            }
        }
    }

    let existing: Vec<PathBuf> = candidates.into_iter().filter(|p| p.exists()).collect();

    if let Some(path) = existing.first() {
        return Ok(path.clone());
    }

    for name in &target_names {
        if let Some(path) = find_file_recursive(&resource_dir, name) {
            return Ok(path);
        }
    }

    Err(format!(
        "Failed to find sidecar file '{}' or alternatives in candidates.",
        target_names[0]
    ))
}

fn get_free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .ok()
        .and_then(|listener| listener.local_addr().ok())
        .map(|addr| addr.port())
        .unwrap_or(3033)
}

pub fn start_parakeet_server<R: Runtime>(app_handle: &tauri::AppHandle<R>) -> Result<(), String> {
    if let Some(state) = app_handle.try_state::<crate::AppState>() {
        let mut server_guard = state.parakeet_server.lock().unwrap();
        if server_guard.is_some() {
            return Ok(());
        }

        eprintln!("Aura Dev Log: Starting Parakeet WebSocket server...");

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            let _ = Command::new("taskkill")
                .args(&["/F", "/IM", "sherpa-onnx-offline-websocket-server.exe"])
                .creation_flags(0x08000000)
                .output();
        }

        let server_path = find_sherpa_websocket_server(app_handle)?;
        let short_server_path = get_short_path(&server_path)?;

        let app_local_data = app_handle
            .path()
            .app_local_data_dir()
            .map_err(|e| format!("Failed to get app local data dir: {}", e))?;
        let model_dir = app_local_data.join("models").join("parakeet-v3");

        if !model_dir.exists() {
            return Err("Parakeet model not found. Please download it first.".to_string());
        }

        let encoder_path = model_dir.join("encoder.onnx");
        let decoder_path = model_dir.join("decoder.onnx");
        let joiner_path = model_dir.join("joiner.onnx");
        let tokens_path = model_dir.join("tokens.txt");

        if !encoder_path.exists()
            || !decoder_path.exists()
            || !joiner_path.exists()
            || !tokens_path.exists()
        {
            return Err("Parakeet model files are incomplete. Please redownload.".to_string());
        }

        let short_encoder = get_short_path(&encoder_path)?;
        let short_decoder = get_short_path(&decoder_path)?;
        let short_joiner = get_short_path(&joiner_path)?;
        let short_tokens = get_short_path(&tokens_path)?;

        let n_threads = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4);

        let port = get_free_port();
        state.parakeet_port.store(port, std::sync::atomic::Ordering::SeqCst);

        let log_file_path = app_local_data.join("parakeet_server_log.txt");
        if log_file_path.exists() {
            let _ = std::fs::remove_file(&log_file_path);
        }

        let settings = crate::settings::load_settings(app_handle)
            .unwrap_or_else(|_| crate::settings::Settings::default());

        let dict = settings.dictionary.trim();
        let hotwords_path = model_dir.join("hotwords.txt");


        if !dict.is_empty() {
            let hotwords: Vec<&str> = dict.split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            if !hotwords.is_empty() {
                let content = hotwords.join("\n");
                if let Err(e) = std::fs::write(&hotwords_path, content) {
                    eprintln!("Aura Dev Log ERROR: Failed to write hotwords.txt: {}", e);
                }
            } else {
                let _ = std::fs::remove_file(&hotwords_path);
            }
        } else {
            let _ = std::fs::remove_file(&hotwords_path);
        }

        let args = vec![
            format!("--encoder={}", short_encoder.to_string_lossy()),
            format!("--decoder={}", short_decoder.to_string_lossy()),
            format!("--joiner={}", short_joiner.to_string_lossy()),
            format!("--tokens={}", short_tokens.to_string_lossy()),
            format!("--port={}", port),
            "--feat-dim=128".to_string(),
            format!("--num-work-threads={}", n_threads),
            format!("--log-file={}", log_file_path.to_string_lossy()),
        ];

        // Note: We deliberately DO NOT pass --hotwords-file here.
        // sherpa-onnx requires --decoding-method=modified_beam_search for hotwords,
        // but the NeMo Parakeet transducer model ONLY supports greedy_search.
        // Thus, hotwords are fundamentally incompatible with Parakeet in sherpa-onnx.

        let mut cmd = Command::new(&short_server_path);
        cmd.args(&args);

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }

        let child = cmd.spawn().map_err(|e| format!("Failed to spawn Parakeet server: {}", e))?;
        *server_guard = Some(child);
        eprintln!("Aura Dev Log: Parakeet WebSocket server process spawned on port {}.", port);
    }
    Ok(())
}

pub fn stop_parakeet_server<R: Runtime>(app_handle: &tauri::AppHandle<R>) {
    if let Some(state) = app_handle.try_state::<crate::AppState>() {
        let mut server_guard = state.parakeet_server.lock().unwrap();
        if let Some(mut child) = server_guard.take() {
            eprintln!("Aura Dev Log: Stopping background Parakeet server...");
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

pub fn ensure_parakeet_server_state<R: Runtime>(app_handle: &tauri::AppHandle<R>, settings: &crate::settings::Settings) {
    if settings.transcription_mode == "local" && settings.local_engine == "parakeet" {
        if let Err(e) = start_parakeet_server(app_handle) {
            eprintln!("Aura Dev Log ERROR: Failed to start Parakeet server: {}", e);
        }
    } else {
        stop_parakeet_server(app_handle);
    }
}

/// Transcribes a 16 kHz mono WAV via the resident Parakeet WebSocket server.
///
/// Note: `language` and `dictionary` are intentionally unused. Parakeet v3 auto-detects the
/// language, and the custom dictionary (hotwords) can't be applied here because the server is a
/// long-lived daemon started once with fixed args — biasing would require a `--hotwords-file`
/// baked in at server start plus a restart whenever the dictionary changes. Left as a documented
/// limitation rather than shipping an unverified server flag that could stop the daemon from
/// starting. The dictionary still works on the Whisper engine and the cloud providers.
pub fn run_parakeet<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
    wav_path: &str,
    _language: &str,
    _dictionary: &str,
) -> Result<String, String> {
    if let Err(e) = start_parakeet_server(app_handle) {
        return Err(format!("Parakeet server is not running and failed to start: {}", e));
    }

    let mut reader = hound::WavReader::open(wav_path)
        .map_err(|e| format!("Failed to open WAV file: {}", e))?;
    let spec = reader.spec();
    if spec.sample_rate != 16000 || spec.channels != 1 {
        return Err(format!(
            "Unsupported WAV format: channels={}, sample_rate={}",
            spec.channels, spec.sample_rate
        ));
    }

    let samples_i16: Vec<i16> = reader
        .samples::<i16>()
        .collect::<Result<Vec<i16>, hound::Error>>()
        .map_err(|e| format!("Failed to read WAV samples: {}", e))?;

    let samples_f32: Vec<f32> = samples_i16
        .iter()
        .map(|&s| s as f32 / 32768.0)
        .collect();

    let port = if let Some(state) = app_handle.try_state::<crate::AppState>() {
        state.parakeet_port.load(std::sync::atomic::Ordering::SeqCst)
    } else {
        3033
    };
    let url = format!("ws://127.0.0.1:{}", port);
    let mut socket = {
        let start_connect = std::time::Instant::now();
        let mut notified = false;
        loop {
            match tungstenite::connect(&url) {
                Ok((s, _)) => {
                    break s;
                }
                Err(e) => {
                    if start_connect.elapsed().as_secs() > 15 {
                        return Err(format!("Parakeet server connection timeout: {}", e));
                    }
                    // First failed connect = the server is still loading the model into RAM.
                    // Tell the user so the wait doesn't look like a freeze.
                    if !notified {
                        notified = true;
                        let _ = app_handle.emit("recording-state", "notice:Загрузка модели…");
                    }
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
            }
        }
    };

    // Guard against a hung/dead server: without a read timeout, socket.read() below
    // would block this dictation forever, leaving the overlay stuck on "processing".
    if let tungstenite::stream::MaybeTlsStream::Plain(stream) = socket.get_ref() {
        let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(30)));
    }

    let sample_rate = spec.sample_rate as i32;
    let expected_byte_size = (samples_f32.len() * 4) as i32;

    let mut payload = Vec::with_capacity(8 + expected_byte_size as usize);
    payload.extend_from_slice(&sample_rate.to_le_bytes());
    payload.extend_from_slice(&expected_byte_size.to_le_bytes());

    for &sample in &samples_f32 {
        payload.extend_from_slice(&sample.to_le_bytes());
    }

    socket.send(tungstenite::Message::Binary(payload))
        .map_err(|e| format!("Failed to send audio data: {}", e))?;

    let msg = socket.read()
        .map_err(|e| format!("Failed to read transcription response: {}", e))?;

    let response_text = match msg {
        tungstenite::Message::Text(text) => text,
        _ => return Err("Unexpected message format from server".to_string()),
    };

    let _ = socket.send(tungstenite::Message::Text("Done".to_string()));

    let mut transcript = response_text.clone();
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&response_text) {
        if let Some(t) = val.get("text").and_then(|v| v.as_str()) {
            transcript = t.trim().to_string();
        }
    }

    // Workaround for Sherpa ONNX Nemo Parakeet model outputting <unk> instead of 'ё'.
    // Only replace with 'ё' when the transcript is in Cyrillic; otherwise just strip
    // the tag, since <unk> in Latin/Chinese text signals a genuinely unknown token.
    let has_cyrillic = transcript
        .chars()
        .any(|c| ('\u{0400}'..='\u{04FF}').contains(&c));
    transcript = if has_cyrillic {
        transcript.replace("<unk>", "ё")
    } else {
        transcript.replace("<unk>", "")
    };

    Ok(transcript)
}

pub fn find_sherpa_punctuation_exe<R: Runtime>(app_handle: &tauri::AppHandle<R>) -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    let target_names = [
        "sherpa-onnx-offline-punctuation.exe",
    ];
    #[cfg(not(target_os = "windows"))]
    let target_names = ["sherpa-onnx-offline-punctuation"];

    let resource_dir = app_handle
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?;

    let mut candidates: Vec<PathBuf> = Vec::new();

    #[cfg(debug_assertions)]
    for name in &target_names {
        candidates.push(
            Path::new(env!("CARGO_MANIFEST_DIR")).join("binaries").join(name),
        );
    }

    for name in &target_names {
        candidates.push(resource_dir.join("binaries").join(name));
        candidates.push(resource_dir.join("_up_").join("binaries").join(name));
        candidates.push(resource_dir.join(name));
        candidates.push(PathBuf::from("binaries").join(name));
        candidates.push(PathBuf::from("src-tauri").join("binaries").join(name));
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            for name in &target_names {
                candidates.push(exe_dir.join(name));
            }
        }
    }

    let existing: Vec<PathBuf> = candidates.into_iter().filter(|p| p.exists()).collect();

    if let Some(path) = existing.first() {
        return Ok(path.clone());
    }

    for name in &target_names {
        if let Some(path) = find_file_recursive(&resource_dir, name) {
            return Ok(path);
        }
    }

    Err(format!(
        "Failed to find punctuation binary '{}' in candidates.",
        target_names[0]
    ))
}

pub async fn download_punctuation_model<R: Runtime>(app_handle: &tauri::AppHandle<R>) -> Result<(), String> {
    let app_local_data = app_handle
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app local data dir: {}", e))?;
    let punc_dir = app_local_data.join("models").join("punctuation");
    
    if punc_dir.join("model.int8.onnx").exists() {
        return Ok(()); // Already downloaded
    }
    
    std::fs::create_dir_all(&punc_dir).map_err(|e| format!("Failed to create punctuation dir: {}", e))?;
    
    eprintln!("Aura Dev Log: Downloading local punctuation model...");
    let url = "https://github.com/k2-fsa/sherpa-onnx/releases/download/punctuation-models/sherpa-onnx-punct-ct-transformer-zh-en-vocab272727-2024-04-12-int8.tar.bz2";
    
    let response = reqwest::get(url).await.map_err(|e| format!("Failed to download punctuation model: {}", e))?;
    let bytes = response.bytes().await.map_err(|e| format!("Failed to read punctuation bytes: {}", e))?;
    
    let temp_tar_path = punc_dir.join("temp_punc.tar.bz2");
    std::fs::write(&temp_tar_path, &bytes).map_err(|e| format!("Failed to write temp punctuation archive: {}", e))?;
    
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        let mut cmd = Command::new("tar");
        cmd.args(&[
            "-xf",
            temp_tar_path.to_str().unwrap(),
            "-C",
            punc_dir.to_str().unwrap(),
        ]);
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        let _ = cmd.output();
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = Command::new("tar")
            .args(&[
                "-xf",
                temp_tar_path.to_str().unwrap(),
                "-C",
                punc_dir.to_str().unwrap(),
            ])
            .output();
    }
    
    let _ = std::fs::remove_file(&temp_tar_path);
    
    let ext_dir = punc_dir.join("sherpa-onnx-punct-ct-transformer-zh-en-vocab272727-2024-04-12-int8");
    if ext_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&ext_dir) {
            for entry in entries.flatten() {
                let dest_path = punc_dir.join(entry.file_name());
                let _ = std::fs::rename(entry.path(), dest_path);
            }
        }
        let _ = std::fs::remove_dir_all(&ext_dir);
    }
    
    eprintln!("Aura Dev Log: Punctuation model downloaded successfully.");
    Ok(())
}

pub fn run_punctuation<R: Runtime>(app_handle: &tauri::AppHandle<R>, text: &str) -> Result<String, String> {
    // Windows CreateProcess limit is ~32 767 chars; long dictations would silently fail.
    // At >4 000 chars the marginal value of offline punctuation is also low (LLM/cloud
    // mode handles its own punctuation), so return the text as-is for safety.
    const MAX_CLI_CHARS: usize = 4_000;
    if text.chars().count() > MAX_CLI_CHARS {
        eprintln!(
            "Aura Dev Log: run_punctuation skipped — text too long ({} chars > {})",
            text.chars().count(),
            MAX_CLI_CHARS
        );
        return Ok(text.to_string());
    }

    let app_local_data = app_handle
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app local data dir: {}", e))?;
    let punc_dir = app_local_data.join("models").join("punctuation");
    let model_path = punc_dir.join("model.int8.onnx");
    
    if !model_path.exists() {
        return Err("Punctuation model not found".to_string());
    }
    
    let punc_exe = find_sherpa_punctuation_exe(app_handle)?;
    let short_punc_exe = get_short_path(&punc_exe)?;
    let short_model_path = get_short_path(&model_path)?;
    
    let mut cmd = Command::new(&short_punc_exe);
    cmd.args(&[
        format!("--ct-transformer={}", short_model_path.to_string_lossy()),
        text.to_string(),
    ]);
    
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }
    
    let output = cmd.output().map_err(|e| format!("Failed to run punctuation tool: {}", e))?;
    if !output.status.success() {
        return Err("Punctuation process failed".to_string());
    }
    
    let stdout_str = String::from_utf8_lossy(&output.stdout);
    
    for line in stdout_str.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("Output text:") {
            let punc_text = trimmed.replace("Output text:", "").trim().to_string();
            let normalized = punc_text
                .replace("？", "? ")
                .replace("。", ". ")
                .replace("，", ", ")
                .replace("；", "; ")
                .replace("：", ": ")
                .replace("！", "! ")
                .replace(" ,", ",")
                .replace(" .", ".")
                .replace(" ?", "?")
                .replace(" !", "!")
                .replace("  ", " ");
            return Ok(normalized.trim().to_string());
        }
    }
    
    Err("Failed to parse punctuation output".to_string())
}

fn get_short_path(path: &Path) -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::ffi::OsStrExt;
        use std::os::windows::ffi::OsStringExt;
        use windows_sys::Win32::Storage::FileSystem::GetShortPathNameW;

        let wide_path: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        
        unsafe {
            let size = GetShortPathNameW(wide_path.as_ptr(), std::ptr::null_mut(), 0);
            if size == 0 {
                return Ok(path.to_path_buf());
            }
            
            let mut buffer: Vec<u16> = vec![0; size as usize];
            let written = GetShortPathNameW(wide_path.as_ptr(), buffer.as_mut_ptr(), size);
            if written == 0 || written >= size {
                return Ok(path.to_path_buf());
            }
            
            let short_str = std::ffi::OsString::from_wide(&buffer[..written as usize]);
            Ok(PathBuf::from(short_str))
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(path.to_path_buf())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filename_parsing() {
        assert_eq!(format_model_filename("small"), "ggml-small.bin");
        assert_eq!(format_model_filename("ggml-base.bin"), "ggml-base.bin");
        assert_eq!(format_model_filename("base.bin"), "ggml-base.bin");
        assert_eq!(format_model_filename("ggml-tiny"), "ggml-tiny.bin");
    }
}
