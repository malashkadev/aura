use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::{Manager, Emitter, Runtime};

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
    let Some(dir) = exe_path.parent() else { return false };
    let check = |d: &Path| {
        ["whisper.dll", "ggml.dll"].iter().all(|name| {
            fs::metadata(d.join(name)).map(|m| m.len() > 1024).unwrap_or(false)
        })
    };
    check(dir) || check(&dir.join("binaries"))
}

/// Locate the whisper sidecar executable under the resources directory or fallback paths.
/// Candidates with the runtime DLLs beside them are preferred; broken copies
/// (e.g. a bare exe in target/debug without its DLLs) are used only as a last resort.
pub fn find_sidecar<R: Runtime>(app_handle: &tauri::AppHandle<R>) -> Result<PathBuf, String> {
    // In release bundles Tauri strips the target triple from externalBin names
    let target_names = [
        "whisper-sidecar-x86_64-pc-windows-msvc.exe",
        "whisper-sidecar.exe",
    ];

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

    let url = format!(
        "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/{}",
        filename
    );

    let client = crate::ai_client::build_http_client();
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

    let mut file = tokio::fs::File::create(&temp_path)
        .await
        .map_err(|e| format!("Failed to create temp file: {}", e))?;

    let mut downloaded: u64 = 0;
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|e| format!("Error while downloading chunk: {}", e))?
    {
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

    fs::rename(&temp_path, &dest_path)
        .map_err(|e| format!("Failed to rename temp file to destination: {}", e))?;

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

    let mut args: Vec<String> = vec![
        "-m".to_string(),
        short_model_path.to_str().ok_or("Invalid model path encoding")?.to_string(),
        "-f".to_string(),
        short_wav_path.to_str().ok_or("Invalid wav path encoding")?.to_string(),
        "-l".to_string(),
        lang.to_string(),
        "-nt".to_string(),
        "-np".to_string(),
    ];

    let dict = dictionary.trim();
    if !dict.is_empty() {
        args.push("--prompt".to_string());
        args.push(dict.to_string());
    }

    let mut cmd = Command::new(&short_sidecar_path);
    cmd.current_dir(&short_sidecar_dir);
    cmd.args(&args);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW (hides the black console window)
    }

    // Prepend sidecar executable directory and resource DLLs directory to PATH for Windows DLL resolution
    if let Some(path_env) = std::env::var_os("PATH") {
        let mut paths = std::env::split_paths(&path_env).collect::<Vec<_>>();
        paths.insert(0, short_sidecar_dir.to_path_buf());
        paths.insert(0, short_dlls_dir.to_path_buf());
        if let Ok(new_path) = std::env::join_paths(paths) {
            cmd.env("PATH", new_path);
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

#[cfg(target_os = "windows")]
fn get_short_path(path: &Path) -> Result<PathBuf, String> {
    use std::os::windows::ffi::OsStrExt;
    use std::ffi::OsString;
    use std::os::windows::ffi::OsStringExt;
    
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
    let mut buffer = vec![0u16; 1024];
    
    let len = unsafe {
        windows_sys::Win32::Storage::FileSystem::GetShortPathNameW(
            wide.as_ptr(),
            buffer.as_mut_ptr(),
            buffer.len() as u32,
        )
    };
    
    if len == 0 {
        return Err(format!("GetShortPathNameW failed for path {:?}", path));
    }
    
    if len > buffer.len() as u32 {
        buffer.resize(len as usize, 0);
        let len2 = unsafe {
            windows_sys::Win32::Storage::FileSystem::GetShortPathNameW(
                wide.as_ptr(),
                buffer.as_mut_ptr(),
                buffer.len() as u32,
            )
        };
        if len2 == 0 || len2 > buffer.len() as u32 {
            return Err(format!("GetShortPathNameW failed on second try for path {:?}", path));
        }
    }
    
    let short_str = OsString::from_wide(&buffer[..len as usize]);
    Ok(PathBuf::from(short_str))
}

#[cfg(not(target_os = "windows"))]
fn get_short_path(path: &Path) -> Result<PathBuf, String> {
    Ok(path.to_path_buf())
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
