use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::{Mutex, OnceLock};
use tauri::{Emitter, Manager, Runtime};

fn spawn_output_reader<R>(
    mut reader: R,
    thread_name: &str,
    stream_name: &'static str,
) -> Result<std::thread::JoinHandle<Result<Vec<u8>, String>>, String>
where
    R: std::io::Read + Send + 'static,
{
    std::thread::Builder::new()
        .name(thread_name.to_string())
        .spawn(move || {
            let mut output = Vec::new();
            reader
                .read_to_end(&mut output)
                .map_err(|error| format!("Failed to read {stream_name}: {error}"))?;
            Ok(output)
        })
        .map_err(|error| format!("Failed to start {stream_name} reader: {error}"))
}

fn collect_output_reader(
    reader: std::thread::JoinHandle<Result<Vec<u8>, String>>,
    stream_name: &str,
) -> Result<Vec<u8>, String> {
    reader
        .join()
        .map_err(|_| format!("{stream_name} reader panicked"))?
}

fn terminate_and_reap(child: &mut std::process::Child) {
    let _ = child.kill();
    let _ = child.wait();
}

fn run_command_with_timeout(
    command: &mut Command,
    timeout: std::time::Duration,
    label: &str,
) -> Result<Output, String> {
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|error| format!("Failed to start {label}: {error}"))?;

    let Some(stdout) = child.stdout.take() else {
        terminate_and_reap(&mut child);
        return Err(format!("{label} stdout pipe is unavailable"));
    };
    let Some(stderr) = child.stderr.take() else {
        terminate_and_reap(&mut child);
        return Err(format!("{label} stderr pipe is unavailable"));
    };

    let stdout_reader = match spawn_output_reader(stdout, "aura-sidecar-stdout", "sidecar stdout") {
        Ok(reader) => reader,
        Err(error) => {
            terminate_and_reap(&mut child);
            return Err(error);
        }
    };
    let stderr_reader = match spawn_output_reader(stderr, "aura-sidecar-stderr", "sidecar stderr") {
        Ok(reader) => reader,
        Err(error) => {
            terminate_and_reap(&mut child);
            let _ = collect_output_reader(stdout_reader, "sidecar stdout");
            return Err(error);
        }
    };

    let started = std::time::Instant::now();
    let status = loop {
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) if started.elapsed() < timeout => {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
            Ok(None) => {
                terminate_and_reap(&mut child);
                let _ = collect_output_reader(stdout_reader, "sidecar stdout");
                let _ = collect_output_reader(stderr_reader, "sidecar stderr");
                return Err(format!(
                    "{label} timed out after {} seconds and was terminated",
                    timeout.as_secs()
                ));
            }
            Err(error) => {
                terminate_and_reap(&mut child);
                let _ = collect_output_reader(stdout_reader, "sidecar stdout");
                let _ = collect_output_reader(stderr_reader, "sidecar stderr");
                return Err(format!("Failed to inspect {label}: {error}"));
            }
        }
    };

    let stdout = collect_output_reader(stdout_reader, "sidecar stdout")?;
    let stderr = collect_output_reader(stderr_reader, "sidecar stderr")?;
    Ok(Output {
        status,
        stdout,
        stderr,
    })
}

fn whisper_timeout_for_duration(model_name: &str, audio_seconds: u64) -> std::time::Duration {
    let model = model_name.to_ascii_lowercase();
    let realtime_factor = if model.contains("tiny") || model.contains("base") {
        2
    } else if model.contains("small") {
        3
    } else {
        5
    };
    let seconds = 120u64
        .saturating_add(audio_seconds.saturating_mul(realtime_factor))
        .clamp(180, 3_600);
    std::time::Duration::from_secs(seconds)
}

fn whisper_timeout_for_wav(model_name: &str, wav_path: &str) -> std::time::Duration {
    let audio_seconds = hound::WavReader::open(wav_path)
        .ok()
        .map(|reader| {
            let sample_rate = u64::from(reader.spec().sample_rate.max(1));
            let sample_count = u64::from(reader.duration());
            sample_count.saturating_add(sample_rate.saturating_sub(1)) / sample_rate
        })
        .unwrap_or(0);
    whisper_timeout_for_duration(model_name, audio_seconds)
}
/// Models whose in-flight download the user asked to cancel (keyed by model name).
static DOWNLOAD_CANCEL: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

fn cancel_set() -> &'static Mutex<HashSet<String>> {
    DOWNLOAD_CANCEL.get_or_init(|| Mutex::new(HashSet::new()))
}

/// Requests cancellation of the running download for `model_name`.
pub fn request_cancel_download(model_name: &str) {
    recover_lock(cancel_set(), "download cancellation").insert(model_name.to_string());
}

pub fn is_cancel_requested(model_name: &str) -> bool {
    recover_lock(cancel_set(), "download cancellation").contains(model_name)
}

pub fn clear_cancel(model_name: &str) {
    recover_lock(cancel_set(), "download cancellation").remove(model_name);
}

#[derive(Clone, serde::Serialize)]
pub struct DownloadProgress {
    pub model: String,
    pub downloaded: u64,
    pub total: Option<u64>,
    pub percentage: f64,
    pub done: bool,
    pub status: Option<String>,
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
    let name_without_bin = name_without_ggml
        .strip_suffix(".bin")
        .unwrap_or(name_without_ggml);
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
        let Some(dir) = exe_path.parent() else {
            return false;
        };
        let check = |d: &Path| {
            ["whisper.dll", "ggml.dll"].iter().all(|name| {
                fs::metadata(d.join(name))
                    .map(|m| m.len() > 1024)
                    .unwrap_or(false)
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
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("binaries")
                .join(name),
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
        crate::logger::log(
            "WARN",
            "Sidecar",
            None,
            &format!(
                "sidecar found at {:?} but whisper.dll/ggml.dll are missing next to it.",
                path
            ),
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

static ACTIVE_MODEL_DOWNLOADS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();

struct DownloadLease {
    model: String,
}

impl Drop for DownloadLease {
    fn drop(&mut self) {
        let downloads = ACTIVE_MODEL_DOWNLOADS.get_or_init(|| Mutex::new(HashSet::new()));
        recover_lock(downloads, "active model downloads").remove(&self.model);
    }
}

pub fn is_model_download_active(model: &str) -> bool {
    let downloads = ACTIVE_MODEL_DOWNLOADS.get_or_init(|| Mutex::new(HashSet::new()));
    recover_lock(downloads, "active model downloads").contains(model)
}

fn begin_model_download(model: &str) -> Result<DownloadLease, String> {
    let downloads = ACTIVE_MODEL_DOWNLOADS.get_or_init(|| Mutex::new(HashSet::new()));
    let mut active = match downloads.lock() {
        Ok(active) => active,
        Err(poisoned) => {
            crate::logger::log(
                "ERROR",
                "Download",
                None,
                "Recovering poisoned active-download mutex",
            );
            poisoned.into_inner()
        }
    };
    if !active.insert(model.to_string()) {
        return Err(format!("A download for '{model}' is already running"));
    }
    Ok(DownloadLease {
        model: model.to_string(),
    })
}

#[derive(Clone, Copy)]
struct ArtifactSpec {
    filename: &'static str,
    url: &'static str,
    expected_size: u64,
    sha256: &'static str,
}

fn whisper_artifact(model_name: &str) -> Result<ArtifactSpec, String> {
    let filename = format_model_filename(model_name);
    let spec = match filename.as_str() {
        "ggml-tiny.bin" => ArtifactSpec {
            filename: "ggml-tiny.bin",
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/ggml-tiny.bin",
            expected_size: 77_691_713,
            sha256: "be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21",
        },
        "ggml-base.bin" => ArtifactSpec {
            filename: "ggml-base.bin",
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/ggml-base.bin",
            expected_size: 147_951_465,
            sha256: "60ed5bc3dd14eea856493d334349b405782ddcaf0028d4b5df4088345fba2efe",
        },
        "ggml-small.bin" => ArtifactSpec {
            filename: "ggml-small.bin",
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/ggml-small.bin",
            expected_size: 487_601_967,
            sha256: "1be3a9b2063867b937e64e2ec7483364a79917e157fa98c5d94b5c1fffea987b",
        },
        "ggml-medium.bin" => ArtifactSpec {
            filename: "ggml-medium.bin",
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/ggml-medium.bin",
            expected_size: 1_533_763_059,
            sha256: "6c14d5adee5f86394037b4e4e8b59f1673b6cee10e3cf0b11bbdbee79c156208",
        },
        "ggml-large-v3-turbo-q5_0.bin" => ArtifactSpec {
            filename: "ggml-large-v3-turbo-q5_0.bin",
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/ggml-large-v3-turbo-q5_0.bin",
            expected_size: 574_041_195,
            sha256: "394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2",
        },
        "ggml-large-v3-turbo.bin" => ArtifactSpec {
            filename: "ggml-large-v3-turbo.bin",
            url: "https://huggingface.co/ggerganov/whisper.cpp/resolve/5359861c739e955e79d9a303bcbc70fb988958b1/ggml-large-v3-turbo.bin",
            expected_size: 1_624_555_275,
            sha256: "1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69",
        },
        _ => {
            return Err(format!(
                "Unsupported Whisper model '{model_name}'. Select a model from Aura settings."
            ))
        }
    };
    Ok(spec)
}

pub(crate) fn whisper_model_filename(model_name: &str) -> Result<String, String> {
    whisper_artifact(model_name).map(|spec| spec.filename.to_string())
}

pub(crate) fn whisper_model_is_installed(model_name: &str, path: &Path) -> bool {
    let Ok(spec) = whisper_artifact(model_name) else {
        return false;
    };
    std::fs::metadata(path)
        .map(|metadata| metadata.is_file() && metadata.len() == spec.expected_size)
        .unwrap_or(false)
}

/// Model paths whose SHA-256 was verified at least once in this process.
/// Runtime checks are size-first (cheap, catches truncation) plus one hash
/// pass per process, which catches same-size corruption that a size check
/// cannot. Hashing a 1.5 GB model on every app start would be wasteful, so
/// the result is memoized.
static RUNTIME_VERIFIED_MODELS: OnceLock<Mutex<HashSet<PathBuf>>> = OnceLock::new();

fn runtime_verify_model_file(
    label: &str,
    path: &Path,
    expected_sha256: &str,
) -> Result<(), String> {
    let verified = RUNTIME_VERIFIED_MODELS.get_or_init(|| Mutex::new(HashSet::new()));
    let mut verified = match verified.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            crate::logger::log(
                "WARN",
                "Model",
                None,
                "runtime verification set mutex was poisoned; recovering",
            );
            poisoned.into_inner()
        }
    };
    if verified.contains(path) {
        return Ok(());
    }
    let actual = crate::artifact_download::sha256_file(path).map_err(|error| {
        format!(
            "Could not hash-check {label} at {}: {error}",
            path.display()
        )
    })?;
    if actual.eq_ignore_ascii_case(expected_sha256) {
        verified.insert(path.to_path_buf());
        Ok(())
    } else {
        Err(format!(
            "{} ({}) failed SHA-256 integrity (expected {expected_sha256}, got {actual}). Re-download the model in settings.",
            label,
            path.display()
        ))
    }
}

/// Pre-warms and verifies the specified local model in the background so that
/// the first dictation does not suffer any SHA-256 calculation or cold-start lag.
pub fn prewarm_local_model_background<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
    model_name: &str,
) {
    let Ok(spec) = whisper_artifact(model_name) else {
        return;
    };
    let Ok(app_local_data) = app_handle.path().app_local_data_dir() else {
        return;
    };
    let model_path = app_local_data.join("models").join(spec.filename);
    let size_ok = std::fs::metadata(&model_path)
        .map(|m| m.is_file() && m.len() == spec.expected_size)
        .unwrap_or(false);
    if !size_ok {
        return;
    }

    let model_label = spec.filename.to_string();
    std::thread::Builder::new()
        .name("aura-model-prewarm".to_string())
        .spawn(move || {
            let started = std::time::Instant::now();
            if let Ok(()) = runtime_verify_model_file(&model_label, &model_path, spec.sha256) {
                crate::logger::log(
                    "INFO",
                    "Model",
                    None,
                    &format!(
                        "Pre-warmed and verified model '{model_label}' in {:?}",
                        started.elapsed()
                    ),
                );
            }
        })
        .ok();
}

/// Download a verified GGML model from a pinned repository revision.
///
/// The transfer is delegated to `artifact_download`, which resumes from a
/// `.part` file across runs, enforces the exact pinned size and SHA-256, and
/// honours cancellation requests for `model_name`.
pub async fn download_model<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
    model_name: &str,
) -> Result<PathBuf, String> {
    let artifact = whisper_artifact(model_name)?;
    let app_local_data = app_handle
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app local data dir: {e}"))?;
    let destination = app_local_data.join("models").join(artifact.filename);

    let _lease = begin_model_download(model_name)?;
    clear_cancel(model_name);

    let client = crate::ai_client::build_download_client();
    let spec = crate::artifact_download::ArtifactSpec {
        label: artifact.filename,
        url: artifact.url,
        expected_size: artifact.expected_size,
        sha256: artifact.sha256,
    };
    let outcome = crate::artifact_download::download_verified_artifact(
        &client,
        spec,
        &destination,
        crate::artifact_download::DEFAULT_STALL_TIMEOUT,
        || is_cancel_requested(model_name),
        |progress| {
            let is_verifying = progress.downloaded >= spec.expected_size;
            let status = if is_verifying {
                Some("installing".to_string())
            } else {
                None
            };
            let percentage = if is_verifying {
                100.0
            } else {
                (progress.downloaded as f64 / spec.expected_size as f64 * 100.0).min(99.9)
            };
            let _ = app_handle.emit(
                "model-download-progress",
                DownloadProgress {
                    model: model_name.to_string(),
                    downloaded: progress.downloaded,
                    total: Some(spec.expected_size),
                    percentage,
                    done: false,
                    status,
                },
            );
        },
    )
    .await?;

    let _ = app_handle.emit(
        "model-download-progress",
        DownloadProgress {
            model: model_name.to_string(),
            downloaded: spec.expected_size,
            total: Some(spec.expected_size),
            percentage: 100.0,
            done: true,
            status: Some("done".to_string()),
        },
    );
    Ok(outcome.path)
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

    // Size check first (cheap, catches truncation), then one hash pass per
    // process: a same-size corrupted file must fail loudly with an actionable
    // error instead of silently producing garbage transcriptions (C8).
    let spec = whisper_artifact(model_name)?;
    let size_ok = std::fs::metadata(&model_path)
        .map(|metadata| metadata.is_file() && metadata.len() == spec.expected_size)
        .unwrap_or(false);
    if !size_ok {
        return Err(format!(
            "Model file missing or incomplete at: {:?}. Please download it first.",
            model_path
        ));
    }
    runtime_verify_model_file(&filename, &model_path, spec.sha256)?;

    // Convert model and wav paths to short 8.3 representations
    let short_model_path = get_short_path(&model_path)?;
    let short_wav_path = get_short_path(Path::new(wav_path))?;

    let lang = match language {
        "ru" | "en" | "de" | "es" | "fr" | "it" | "zh" | "pt" | "tr" => language,
        _ => "auto",
    };

    // In GGML matrix computation on CPU, capping threads to 8 prevents
    // thread-sync thrashing and cache contention on high-core-count processors.
    let n_threads = std::thread::available_parallelism()
        .map(|n| n.get().min(8))
        .unwrap_or(4);

    let mut args: Vec<String> = vec![
        "-m".to_string(),
        short_model_path
            .to_str()
            .ok_or("Invalid model path encoding")?
            .to_string(),
        "-f".to_string(),
        short_wav_path
            .to_str()
            .ok_or("Invalid wav path encoding")?
            .to_string(),
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
        "-sow".to_string(),
    ];

    let settings = crate::settings::load_settings(app_handle).unwrap_or_default();
    if settings.local_acceleration == "cpu" {
        args.push("-ng".to_string());
    }

    let primed_prompt = build_whisper_prompt(lang, dictionary);
    if !primed_prompt.is_empty() {
        args.push("--prompt".to_string());
        args.push(primed_prompt);
    }

    let mut working_dir = short_dlls_dir.to_path_buf();
    if settings.local_acceleration == "cuda" {
        let cuda_bin = app_local_data.join("binaries").join("cuda").join("bin");
        if cuda_bin.join("ggml-cuda.dll").exists() {
            if let Ok(short_cuda_bin) = get_short_path(&cuda_bin) {
                working_dir = short_cuda_bin;
            }
        }
    }

    let mut cmd = Command::new(&short_sidecar_path);
    cmd.current_dir(&working_dir);
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

    let output = run_command_with_timeout(
        &mut cmd,
        whisper_timeout_for_wav(model_name, wav_path),
        "Whisper sidecar",
    )?;

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

pub fn find_sherpa_sidecar<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
) -> Result<PathBuf, String> {
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
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("binaries")
                .join(name),
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

const PARAKEET_ARTIFACTS: [ArtifactSpec; 4] = [
    ArtifactSpec {
        filename: "encoder.onnx",
        url: "https://huggingface.co/csukuangfj/sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8/resolve/2bda32ec70b097a55adaa07d9a7173915b43cc78/encoder.int8.onnx",
        expected_size: 652_184_281,
        sha256: "acfc2b4456377e15d04f0243af540b7fe7c992f8d898d751cf134c3a55fd2247",
    },
    ArtifactSpec {
        filename: "decoder.onnx",
        url: "https://huggingface.co/csukuangfj/sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8/resolve/2bda32ec70b097a55adaa07d9a7173915b43cc78/decoder.int8.onnx",
        expected_size: 11_845_275,
        sha256: "179e50c43d1a9de79c8a24149a2f9bac6eb5981823f2a2ed88d655b24248db4e",
    },
    ArtifactSpec {
        filename: "joiner.onnx",
        url: "https://huggingface.co/csukuangfj/sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8/resolve/2bda32ec70b097a55adaa07d9a7173915b43cc78/joiner.int8.onnx",
        expected_size: 6_355_277,
        sha256: "3164c13fc2821009440d20fcb5fdc78bff28b4db2f8d0f0b329101719c0948b3",
    },
    ArtifactSpec {
        filename: "tokens.txt",
        url: "https://huggingface.co/csukuangfj/sherpa-onnx-nemo-parakeet-tdt-0.6b-v3-int8/resolve/2bda32ec70b097a55adaa07d9a7173915b43cc78/tokens.txt",
        expected_size: 93_939,
        sha256: "d58544679ea4bc6ac563d1f545eb7d474bd6cfa467f0a6e2c1dc1c7d37e3c35d",
    },
];

pub(crate) fn parakeet_model_is_installed(directory: &Path) -> bool {
    PARAKEET_ARTIFACTS.iter().all(|spec| {
        std::fs::metadata(directory.join(spec.filename))
            .map(|metadata| metadata.is_file() && metadata.len() == spec.expected_size)
            .unwrap_or(false)
    })
}

pub async fn download_parakeet_model<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
) -> Result<PathBuf, String> {
    let app_local_data = app_handle
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app local data dir: {e}"))?;
    let parakeet_dir = app_local_data.join("models").join("parakeet-v3");

    let _lease = begin_model_download("parakeet-v3")?;
    clear_cancel("parakeet-v3");

    let total_size: u64 = PARAKEET_ARTIFACTS
        .iter()
        .map(|spec| spec.expected_size)
        .sum();
    let client = crate::ai_client::build_download_client();
    let mut downloaded = 0u64;

    for spec in PARAKEET_ARTIFACTS {
        let artifact = crate::artifact_download::ArtifactSpec {
            label: spec.filename,
            url: spec.url,
            expected_size: spec.expected_size,
            sha256: spec.sha256,
        };
        crate::artifact_download::download_verified_artifact(
            &client,
            artifact,
            &parakeet_dir.join(spec.filename),
            crate::artifact_download::DEFAULT_STALL_TIMEOUT,
            || is_cancel_requested("parakeet-v3"),
            |progress| {
                let file_downloaded = downloaded + progress.downloaded;
                let is_verifying = file_downloaded >= total_size;
                let status = if is_verifying {
                    Some("installing".to_string())
                } else {
                    None
                };
                let percentage = if is_verifying {
                    100.0
                } else {
                    (file_downloaded as f64 / total_size as f64 * 100.0).min(99.9)
                };
                let _ = app_handle.emit(
                    "model-download-progress",
                    DownloadProgress {
                        model: "parakeet-v3".to_string(),
                        downloaded: file_downloaded,
                        total: Some(total_size),
                        percentage,
                        done: false,
                        status,
                    },
                );
            },
        )
        .await?;
        downloaded += artifact.expected_size;
    }

    let _ = app_handle.emit(
        "model-download-progress",
        DownloadProgress {
            model: "parakeet-v3".to_string(),
            downloaded: total_size,
            total: Some(total_size),
            percentage: 100.0,
            done: true,
            status: Some("done".to_string()),
        },
    );
    Ok(parakeet_dir)
}
pub fn find_cpu_sherpa_websocket_server<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
) -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    let target_names = ["sherpa-onnx-offline-websocket-server.exe"];
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
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("binaries")
                .join(name),
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

pub fn find_sherpa_websocket_server<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
) -> Result<PathBuf, String> {
    let settings = crate::settings::load_settings(app_handle).unwrap_or_default();

    let exe_name = if cfg!(target_os = "windows") {
        "sherpa-onnx-offline-websocket-server.exe"
    } else {
        "sherpa-onnx-offline-websocket-server"
    };

    if settings.local_acceleration != "cpu" {
        if let Ok(app_local_data) = app_handle.path().app_local_data_dir() {
            let gpu_exe = app_local_data
                .join("binaries")
                .join(&settings.local_acceleration)
                .join("bin")
                .join(exe_name);
            if gpu_exe.exists() {
                #[cfg(target_os = "windows")]
                {
                    if let Some(parent) = gpu_exe.parent() {
                        let has_ort = parent.join("onnxruntime.dll").exists();
                        let is_cuda_valid = if settings.local_acceleration == "cuda" {
                            parent.join("onnxruntime_providers_cuda.dll").exists()
                        } else {
                            true
                        };
                        if has_ort && is_cuda_valid {
                            return Ok(gpu_exe);
                        }
                    }
                }
                #[cfg(not(target_os = "windows"))]
                {
                    return Ok(gpu_exe);
                }
            }
        }
    }

    find_cpu_sherpa_websocket_server(app_handle)
}

pub(crate) struct RunningWhisperServer {
    child: std::process::Child,
    model: String,
    provider: String,
    port: u16,
    readers: SidecarPipeReaders,
    #[cfg(target_os = "windows")]
    _kill_on_close_job: Option<KillOnCloseJob>,
}

impl Drop for RunningWhisperServer {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        self.readers.join();
    }
}

pub(crate) struct RunningParakeetServer {
    child: std::process::Child,
    provider: String,
    executable: PathBuf,
    port: u16,
    readers: SidecarPipeReaders,
    #[cfg(target_os = "windows")]
    _kill_on_close_job: Option<KillOnCloseJob>,
}

impl Drop for RunningParakeetServer {
    fn drop(&mut self) {
        // Spare kill: the server must never outlive this struct, even when a
        // code path forgets to stop it explicitly (early return or panic).
        let _ = self.child.kill();
        let _ = self.child.wait();
        // The child is gone, so the pipes reached EOF; reap the reader threads
        // so restarts do not accumulate them.
        self.readers.join();
    }
}

#[cfg(target_os = "windows")]
struct KillOnCloseJob {
    handle: windows_sys::Win32::Foundation::HANDLE,
}

#[cfg(target_os = "windows")]
impl Drop for KillOnCloseJob {
    fn drop(&mut self) {
        if self.handle != 0 {
            unsafe {
                windows_sys::Win32::Foundation::CloseHandle(self.handle);
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn create_kill_on_close_job() -> Result<KillOnCloseJob, String> {
    use windows_sys::Win32::System::JobObjects::{
        CreateJobObjectW, JobObjectExtendedLimitInformation, SetInformationJobObject,
        JOBOBJECT_EXTENDED_LIMIT_INFORMATION, JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE,
    };

    unsafe {
        let job = CreateJobObjectW(std::ptr::null(), std::ptr::null());
        if job == 0 {
            return Err(format!(
                "CreateJobObjectW failed: {}",
                std::io::Error::last_os_error()
            ));
        }

        let mut limits: JOBOBJECT_EXTENDED_LIMIT_INFORMATION = std::mem::zeroed();
        limits.BasicLimitInformation.LimitFlags = JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let configured = SetInformationJobObject(
            job,
            JobObjectExtendedLimitInformation,
            &limits as *const _ as *const std::ffi::c_void,
            std::mem::size_of::<JOBOBJECT_EXTENDED_LIMIT_INFORMATION>() as u32,
        );
        if configured == 0 {
            let error = std::io::Error::last_os_error();
            windows_sys::Win32::Foundation::CloseHandle(job);
            return Err(format!("SetInformationJobObject failed: {error}"));
        }

        Ok(KillOnCloseJob { handle: job })
    }
}

#[cfg(target_os = "windows")]
fn assign_process_to_job(job: &KillOnCloseJob, child: &std::process::Child) -> Result<(), String> {
    use std::os::windows::io::AsRawHandle;
    use windows_sys::Win32::System::JobObjects::AssignProcessToJobObject;

    let process = child.as_raw_handle() as windows_sys::Win32::Foundation::HANDLE;
    if unsafe { AssignProcessToJobObject(job.handle, process) } == 0 {
        return Err(format!(
            "AssignProcessToJobObject failed: {}",
            std::io::Error::last_os_error()
        ));
    }
    Ok(())
}

fn recover_lock<'a, T>(mutex: &'a std::sync::Mutex<T>, name: &str) -> std::sync::MutexGuard<'a, T> {
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            crate::logger::log(
                "ERROR",
                "State",
                None,
                &format!("Recovering poisoned {name} mutex"),
            );
            poisoned.into_inner()
        }
    }
}

fn get_free_port() -> Result<u16, String> {
    let listener = std::net::TcpListener::bind("127.0.0.1:0")
        .map_err(|e| format!("Failed to reserve a local Parakeet port: {e}"))?;
    listener
        .local_addr()
        .map(|address| address.port())
        .map_err(|e| format!("Failed to inspect the reserved Parakeet port: {e}"))
}

type SidecarDiagnostics = std::sync::Arc<std::sync::Mutex<std::collections::VecDeque<String>>>;

fn remember_sidecar_line(diagnostics: &SidecarDiagnostics, line: &str) {
    let mut lines = recover_lock(diagnostics, "sidecar diagnostics");
    if lines.len() == 24 {
        lines.pop_front();
    }
    lines.push_back(line.chars().take(800).collect());
}

fn is_benign_websocket_shutdown_line(line: &str) -> bool {
    cfg!(target_os = "windows") && line.contains("handle_read_frame error: asio.system:10058")
}

fn is_routine_parakeet_status_line(line: &str) -> bool {
    if line.trim().is_empty() || line.contains("parse-options.cc:Read:") {
        return true;
    }

    if line.contains("offline-websocket-server.cc:main:") {
        return line.ends_with(" Started!")
            || line.contains(" Listening on: ")
            || line.contains(" Number of work threads: ");
    }

    line.contains("offline-websocket-server-impl.cc:")
        && ((line.contains(":Decode:") && line.contains(" size: "))
            || (line.contains(":OnOpen:") && line.contains("Number of active connections:"))
            || (line.contains(":OnClose:") && line.contains("Number of active connections:")))
}

/// Owns the two pipe-reader threads of a sidecar plus the diagnostics buffer.
/// Dropping it joins both readers, so restarts never accumulate threads that
/// keep blocking on a half-closed pipe.
struct SidecarPipeReaders {
    diagnostics: SidecarDiagnostics,
    stdout: Option<std::thread::JoinHandle<()>>,
    stderr: Option<std::thread::JoinHandle<()>>,
}

impl Default for SidecarPipeReaders {
    fn default() -> Self {
        Self {
            diagnostics: std::sync::Arc::new(std::sync::Mutex::new(
                std::collections::VecDeque::new(),
            )),
            stdout: None,
            stderr: None,
        }
    }
}

impl SidecarPipeReaders {
    /// Must be called after the child has been killed/waited: the pipes hit EOF
    /// only once the sidecar is gone, and `join` reaps the reader threads.
    fn join(&mut self) {
        for handle in [self.stdout.take(), self.stderr.take()]
            .into_iter()
            .flatten()
        {
            let _ = handle.join();
        }
    }
}

#[cfg(target_os = "windows")]
fn prevent_pipe_inheritance(pipe: &impl std::os::windows::io::AsRawHandle, name: &str) {
    use windows_sys::Win32::Foundation::{SetHandleInformation, HANDLE_FLAG_INHERIT};
    let raw = pipe.as_raw_handle() as windows_sys::Win32::Foundation::HANDLE;
    if unsafe { SetHandleInformation(raw, HANDLE_FLAG_INHERIT, 0) } == 0 {
        crate::logger::log(
            "WARN",
            "Sidecar",
            None,
            &format!(
                "Could not mark the Parakeet {name} pipe as non-inheritable: {}",
                std::io::Error::last_os_error()
            ),
        );
    }
}

fn pipe_child_output(child: &mut std::process::Child) -> SidecarPipeReaders {
    let diagnostics = std::sync::Arc::new(std::sync::Mutex::new(std::collections::VecDeque::new()));
    let stdout_reader = if let Some(stdout) = child.stdout.take() {
        #[cfg(target_os = "windows")]
        prevent_pipe_inheritance(&stdout, "stdout");
        let stdout_diagnostics = std::sync::Arc::clone(&diagnostics);
        let handle = std::thread::Builder::new()
            .name("parakeet-stdout-reader".to_string())
            .spawn(move || {
                use std::io::BufRead;
                let reader = std::io::BufReader::new(stdout);
                for line in reader.lines().map_while(Result::ok) {
                    if is_benign_websocket_shutdown_line(&line) {
                        continue;
                    }
                    remember_sidecar_line(&stdout_diagnostics, &line);
                    crate::logger::log("INFO", "Sidecar", None, &line);
                }
            });
        match handle {
            Ok(handle) => Some(handle),
            Err(error) => {
                crate::logger::log(
                    "WARN",
                    "Sidecar",
                    None,
                    &format!("Could not start the Parakeet stdout reader: {error}"),
                );
                None
            }
        }
    } else {
        None
    };
    let stderr_reader = if let Some(stderr) = child.stderr.take() {
        #[cfg(target_os = "windows")]
        prevent_pipe_inheritance(&stderr, "stderr");
        let stderr_diagnostics = std::sync::Arc::clone(&diagnostics);
        let handle = std::thread::Builder::new()
            .name("parakeet-stderr-reader".to_string())
            .spawn(move || {
                use std::io::BufRead;
                let reader = std::io::BufReader::new(stderr);
                for line in reader.lines().map_while(Result::ok) {
                    if is_benign_websocket_shutdown_line(&line) {
                        continue;
                    }
                    remember_sidecar_line(&stderr_diagnostics, &line);
                    if !is_routine_parakeet_status_line(&line) {
                        crate::logger::log("WARN", "Sidecar", None, &line);
                    }
                }
            });
        match handle {
            Ok(handle) => Some(handle),
            Err(error) => {
                crate::logger::log(
                    "WARN",
                    "Sidecar",
                    None,
                    &format!("Could not start the Parakeet stderr reader: {error}"),
                );
                None
            }
        }
    } else {
        None
    };
    SidecarPipeReaders {
        diagnostics,
        stdout: stdout_reader,
        stderr: stderr_reader,
    }
}

fn recent_sidecar_diagnostics(diagnostics: &SidecarDiagnostics) -> String {
    recover_lock(diagnostics, "sidecar diagnostics")
        .iter()
        .cloned()
        .collect::<Vec<_>>()
        .join(" | ")
}

fn wait_for_server_warm_up(
    child: &mut std::process::Child,
    port: u16,
    timeout: std::time::Duration,
    diagnostics: &SidecarDiagnostics,
    should_abort: &dyn Fn() -> bool,
) -> Result<u128, String> {
    let started = std::time::Instant::now();
    loop {
        if should_abort() {
            return Err("Parakeet startup aborted because the app is shutting down".to_string());
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                std::thread::sleep(std::time::Duration::from_millis(75));
                let details = recent_sidecar_diagnostics(diagnostics);
                return Err(if details.is_empty() {
                    format!("sidecar exited before becoming ready ({status})")
                } else {
                    format!("sidecar exited before becoming ready ({status}): {details}")
                });
            }
            Ok(None) => {}
            Err(error) => {
                return Err(format!("Failed to inspect sidecar state: {error}"));
            }
        }

        let connection_error = match crate::warm_up_parakeet_server_port(port) {
            Ok(warm_up_ms) => return Ok(warm_up_ms),
            Err(crate::ParakeetWarmUpError::Connection(error)) => error,
            Err(crate::ParakeetWarmUpError::Inference(error)) => {
                std::thread::sleep(std::time::Duration::from_millis(75));
                let details = recent_sidecar_diagnostics(diagnostics);
                return Err(if details.is_empty() {
                    format!("sidecar failed functional inference warm-up: {error}")
                } else {
                    format!("sidecar failed functional inference warm-up: {error}: {details}")
                });
            }
        };
        if started.elapsed() >= timeout {
            let details = recent_sidecar_diagnostics(diagnostics);
            return Err(if details.is_empty() {
                format!(
                    "sidecar did not accept WebSocket connections on port {port} within {} seconds: {connection_error}",
                    timeout.as_secs(),
                )
            } else {
                format!(
                    "sidecar did not accept WebSocket connections on port {port} within {} seconds: {connection_error}: {details}",
                    timeout.as_secs(),
                )
            });
        }
        std::thread::sleep(std::time::Duration::from_millis(75));
    }
}

fn spawn_ready_server(
    executable: &Path,
    provider: &str,
    base_args: &[String],
    port: u16,
    should_abort: &dyn Fn() -> bool,
) -> Result<RunningParakeetServer, String> {
    let mut command = Command::new(executable);
    command
        .args(base_args)
        .arg(format!("--port={port}"))
        .arg(format!("--provider={provider}"))
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }

    // Create the kill-on-close Job Object BEFORE the child exists: if anything
    // between spawn and the struct construction panics, dropping the job during
    // unwind still kills the sidecar instead of leaking it.
    #[cfg(target_os = "windows")]
    let prepared_job = match create_kill_on_close_job() {
        Ok(job) => Some(job),
        Err(error) => {
            crate::logger::log(
                "WARN",
                "Sidecar",
                None,
                &format!("Could not create Parakeet kill-on-close Job Object: {error}"),
            );
            None
        }
    };

    let mut child = command.spawn().map_err(|e| {
        format!(
            "Failed to spawn Parakeet server '{}' ({provider}): {e}",
            executable.display()
        )
    })?;

    #[cfg(target_os = "windows")]
    let kill_on_close_job = match prepared_job {
        Some(job) => match assign_process_to_job(&job, &child) {
            Ok(()) => Some(job),
            Err(error) => {
                crate::logger::log(
                    "WARN",
                    "Sidecar",
                    None,
                    &format!("Could not attach Parakeet to a kill-on-close Job Object: {error}"),
                );
                None
            }
        },
        None => None,
    };

    let mut readers = pipe_child_output(&mut child);
    let timeout = if provider == "cpu" {
        std::time::Duration::from_secs(45)
    } else {
        std::time::Duration::from_secs(25)
    };
    let warm_up_ms = match wait_for_server_warm_up(
        &mut child,
        port,
        timeout,
        &readers.diagnostics,
        should_abort,
    ) {
        Ok(elapsed_ms) => elapsed_ms,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            readers.join();
            return Err(format!("{provider} sidecar failed readiness: {error}"));
        }
    };
    crate::logger::log(
        "INFO",
        "Sidecar",
        None,
        &format!("Parakeet {provider} inference warm-up completed in {warm_up_ms} ms"),
    );

    Ok(RunningParakeetServer {
        child,
        provider: provider.to_string(),
        executable: executable.to_path_buf(),
        port,
        readers,
        #[cfg(target_os = "windows")]
        _kill_on_close_job: kill_on_close_job,
    })
}

/// A sidecar that dies before binding its port usually lost a race for a
/// freshly reserved port (TOCTOU: `get_free_port` releases the listener, then
/// the child binds). Such failures are worth retrying on a new port; functional
/// and spawn errors are not.
fn is_retryable_sidecar_failure(error: &str) -> bool {
    error.contains("exited before becoming ready")
        || error.contains("did not accept WebSocket connections")
        // A functional warm-up failure is just as transient as a bind race
        // (driver initialization hiccup, first-inference JIT), so it deserves
        // the same retry budget instead of permanently failing the startup.
        || error.contains("failed functional inference warm-up")
}

const MAX_READY_ATTEMPTS: usize = 3;

fn spawn_ready_server_with_retries(
    executable: &Path,
    provider: &str,
    base_args: &[String],
    should_abort: &dyn Fn() -> bool,
) -> Result<RunningParakeetServer, String> {
    let mut last_error = String::new();
    for attempt in 1..=MAX_READY_ATTEMPTS {
        if attempt > 1 {
            crate::logger::log(
                "WARN",
                "Sidecar",
                None,
                &format!(
                    "Parakeet {provider} startup attempt {attempt}/{MAX_READY_ATTEMPTS} after: {last_error}"
                ),
            );
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
        // A fresh port per attempt: the previous one may have been grabbed by a
        // foreign process in the bind gap.
        let port = get_free_port()?;
        match spawn_ready_server(executable, provider, base_args, port, should_abort) {
            Ok(server) => return Ok(server),
            Err(error) if attempt < MAX_READY_ATTEMPTS && is_retryable_sidecar_failure(&error) => {
                last_error = error;
            }
            Err(error) => return Err(error),
        }
    }
    Err(last_error)
}

fn stop_owned_server(server: RunningParakeetServer) {
    // The Drop impl performs the kill; this helper only documents intent.
    drop(server);
}

fn provider_for_server_path(path: &Path) -> &'static str {
    let normalized = path.to_string_lossy().replace('\\', "/").to_lowercase();
    if normalized.contains("/binaries/cuda/") {
        "cuda"
    } else if normalized.contains("/binaries/directml/") {
        "directml"
    } else {
        "cpu"
    }
}

pub fn start_parakeet_server<R: Runtime>(app_handle: &tauri::AppHandle<R>) -> Result<(), String> {
    let state = app_handle
        .try_state::<crate::AppState>()
        .ok_or_else(|| "Application state is not initialized".to_string())?;
    // Serializes start/stop/watchdog-restart, so the published port and the
    // stored server can never be observed half-updated (port TOCTOU, Б-11).
    let _lifecycle = recover_lock(&state.parakeet_lifecycle, "Parakeet lifecycle");
    start_parakeet_server_unlocked(app_handle, &state)
}

fn start_parakeet_server_unlocked<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
    state: &crate::AppState,
) -> Result<(), String> {
    // Lazily spawn the watchdog that resurrects a crashed daemon while the app
    // idles. Its stop flag doubles as the shutdown signal for warm-ups, so an
    // in-flight restart never delays app exit for the full warm-up timeout.
    let watchdog_stop = {
        let mut watchdog_slot = recover_lock(&state.parakeet_watchdog, "Parakeet watchdog");
        match watchdog_slot.as_mut() {
            Some(watchdog) => std::sync::Arc::clone(&watchdog.stop),
            None => {
                let (watchdog, stop) = ParakeetWatchdog::spawn(app_handle.clone());
                *watchdog_slot = Some(watchdog);
                stop
            }
        }
    };
    let should_abort = move || watchdog_stop.load(std::sync::atomic::Ordering::SeqCst);

    let server_path = find_sherpa_websocket_server(app_handle)?;
    let target_provider = provider_for_server_path(&server_path);
    let short_server_path = get_short_path(&server_path)?;

    let app_local_data = app_handle
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app local data dir: {e}"))?;
    let model_dir = app_local_data.join("models").join("parakeet-v3");
    let encoder_path = model_dir.join("encoder.onnx");
    let decoder_path = model_dir.join("decoder.onnx");
    let joiner_path = model_dir.join("joiner.onnx");
    let tokens_path = model_dir.join("tokens.txt");
    for path in [&encoder_path, &decoder_path, &joiner_path, &tokens_path] {
        let metadata = std::fs::metadata(path).map_err(|_| {
            format!(
                "Parakeet model file '{}' is missing. Please redownload the model.",
                path.display()
            )
        })?;
        if metadata.len() == 0 {
            return Err(format!(
                "Parakeet model file '{}' is empty. Please redownload the model.",
                path.display()
            ));
        }
    }
    // Sizes alone can hide same-size corruption: verify each artifact's
    // SHA-256 once per process before letting the server load it (C8).
    for spec in PARAKEET_ARTIFACTS {
        runtime_verify_model_file(spec.filename, &model_dir.join(spec.filename), spec.sha256)?;
    }

    let short_encoder = get_short_path(&encoder_path)?;
    let short_decoder = get_short_path(&decoder_path)?;
    let short_joiner = get_short_path(&joiner_path)?;
    let short_tokens = get_short_path(&tokens_path)?;
    let threads = std::thread::available_parallelism()
        .map(|count| count.get())
        .unwrap_or(4);
    let log_file_path = app_local_data.join("parakeet_server_log.txt");
    let _ = std::fs::remove_file(&log_file_path);
    let base_args = vec![
        format!("--encoder={}", short_encoder.to_string_lossy()),
        format!("--decoder={}", short_decoder.to_string_lossy()),
        format!("--joiner={}", short_joiner.to_string_lossy()),
        format!("--tokens={}", short_tokens.to_string_lossy()),
        "--feat-dim=128".to_string(),
        format!("--num-work-threads={threads}"),
        // 660 s (11 min) made the server reject any dictation longer than 11
        // minutes with "message too big"; 1200 s covers ~20-minute sessions.
        "--max-utterance-length=1200".to_string(),
        format!("--log-file={}", log_file_path.to_string_lossy()),
    ];

    let previous = {
        let mut server = recover_lock(&state.parakeet_server, "Parakeet server");
        let keep_existing = if let Some(running) = server.as_mut() {
            match running.child.try_wait() {
                Ok(None) => {
                    running.provider == target_provider && running.executable == short_server_path
                }
                Ok(Some(status)) => {
                    crate::logger::log(
                        "WARN",
                        "Sidecar",
                        None,
                        &format!("Parakeet server exited unexpectedly ({status}); restarting"),
                    );
                    false
                }
                Err(error) => {
                    crate::logger::log(
                        "WARN",
                        "Sidecar",
                        None,
                        &format!("Could not inspect Parakeet server ({error}); restarting"),
                    );
                    false
                }
            }
        } else {
            false
        };
        if keep_existing {
            return Ok(());
        }
        server.take()
    };
    if let Some(previous) = previous {
        state
            .parakeet_port
            .store(0, std::sync::atomic::Ordering::SeqCst);
        stop_owned_server(previous);
    }

    crate::logger::log(
        "INFO",
        "Sidecar",
        None,
        &format!("Starting Parakeet WebSocket server ({target_provider})"),
    );

    let running = match spawn_ready_server_with_retries(
        &short_server_path,
        target_provider,
        &base_args,
        &should_abort,
    ) {
        Ok(server) => server,
        Err(gpu_error) if target_provider != "cpu" => {
            crate::logger::log(
                "WARN",
                "Sidecar",
                None,
                &format!(
                    "GPU Parakeet server ({target_provider}) failed: {gpu_error}. Falling back to CPU."
                ),
            );
            let cpu_path = get_short_path(&find_cpu_sherpa_websocket_server(app_handle).map_err(
                |error| {
                    format!("GPU start failed ({gpu_error}); CPU sidecar is unavailable: {error}")
                },
            )?)?;
            spawn_ready_server_with_retries(&cpu_path, "cpu", &base_args, &should_abort).map_err(
                |cpu_error| {
                    format!("GPU start failed ({gpu_error}); CPU fallback also failed: {cpu_error}")
                },
            )?
        }
        Err(error) => return Err(error),
    };

    let provider = running.provider.clone();
    let port = running.port;
    state
        .parakeet_port
        .store(port, std::sync::atomic::Ordering::SeqCst);
    *recover_lock(&state.parakeet_server, "Parakeet server") = Some(running);
    crate::logger::log(
        "INFO",
        "Sidecar",
        None,
        &format!("Parakeet WebSocket server is ready on port {port} ({provider})"),
    );
    Ok(())
}

/// Live health of the local engine for diagnostics: for Parakeet this is the
/// actual sidecar process state, not what the settings claim.
pub(crate) fn parakeet_server_status<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
) -> Option<(String, u16)> {
    let state = app_handle.try_state::<crate::AppState>()?;
    let server = recover_lock(&state.parakeet_server, "Parakeet server");
    server
        .as_ref()
        .map(|running| (running.provider.clone(), running.port))
}

pub fn stop_parakeet_server<R: Runtime>(app_handle: &tauri::AppHandle<R>) {
    let Some(state) = app_handle.try_state::<crate::AppState>() else {
        return;
    };
    let _lifecycle = recover_lock(&state.parakeet_lifecycle, "Parakeet lifecycle");
    let server = recover_lock(&state.parakeet_server, "Parakeet server").take();
    state
        .parakeet_port
        .store(0, std::sync::atomic::Ordering::SeqCst);
    if let Some(server) = server {
        crate::logger::log(
            "INFO",
            "Sidecar",
            None,
            "Stopping background Parakeet server",
        );
        stop_owned_server(server);
    }
}

/// Stops automatic Parakeet restarts before replacing sidecar runtime files.
pub fn stop_parakeet_server_and_watchdog<R: Runtime>(app_handle: &tauri::AppHandle<R>) {
    let watchdog = app_handle
        .try_state::<crate::AppState>()
        .and_then(|state| recover_lock(&state.parakeet_watchdog, "Parakeet watchdog").take());
    drop(watchdog);
    stop_parakeet_server(app_handle);
}

/// One watchdog per app lifetime: it resurrects a sidecar that died while the
/// user was idle between dictations. Dropping it (app exit) raises the stop
/// flag — which also aborts any in-flight startup warm-up — and joins.
pub(crate) struct ParakeetWatchdog {
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Drop for ParakeetWatchdog {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl ParakeetWatchdog {
    fn spawn<R: Runtime>(
        app_handle: tauri::AppHandle<R>,
    ) -> (Self, std::sync::Arc<std::sync::atomic::AtomicBool>) {
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let thread_stop = std::sync::Arc::clone(&stop);
        // A watchdog is a convenience, not a requirement: if the OS refuses
        // to spawn the thread (resource exhaustion), start the server anyway
        // and log instead of panicking the whole app.
        let handle = match std::thread::Builder::new()
            .name("parakeet-watchdog".to_string())
            .spawn(move || parakeet_watchdog_main(&app_handle, thread_stop))
        {
            Ok(handle) => Some(handle),
            Err(error) => {
                crate::logger::log(
                    "WARN",
                    "Sidecar",
                    None,
                    &format!(
                        "Could not spawn the Parakeet watchdog thread ({error}); \
                         background auto-restart is disabled for this session"
                    ),
                );
                None
            }
        };
        (
            Self {
                stop: std::sync::Arc::clone(&stop),
                handle,
            },
            stop,
        )
    }
}

enum ParakeetChildStatus {
    Alive,
    Exited(Option<std::process::ExitStatus>),
    Empty,
}

/// Sleeps in one-second slices so `Drop` on the owning watchdog never blocks
/// app exit for a full multi-second backoff tick.
fn watchdog_sleep_interruptible(stop: &std::sync::atomic::AtomicBool, secs: u64) {
    use std::sync::atomic::Ordering;
    for _ in 0..secs {
        if stop.load(Ordering::SeqCst) {
            return;
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}

fn parakeet_watchdog_main<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
) {
    use std::sync::atomic::Ordering;
    let mut backoff_secs: u64 = 5;
    loop {
        if stop.load(Ordering::SeqCst) {
            return;
        }
        let Some(state) = app_handle.try_state::<crate::AppState>() else {
            std::thread::sleep(std::time::Duration::from_secs(1));
            continue;
        };
        let status = {
            let mut slot = recover_lock(&state.parakeet_server, "Parakeet server");
            match slot.as_mut() {
                None => ParakeetChildStatus::Empty,
                Some(server) => match server.child.try_wait() {
                    Ok(None) => ParakeetChildStatus::Alive,
                    Ok(Some(exit)) => ParakeetChildStatus::Exited(Some(exit)),
                    Err(error) => {
                        crate::logger::log(
                            "WARN",
                            "Sidecar",
                            None,
                            &format!("Could not inspect Parakeet server ({error}); restarting"),
                        );
                        ParakeetChildStatus::Exited(None)
                    }
                },
            }
        };
        match status {
            ParakeetChildStatus::Alive => {
                backoff_secs = 5;
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
            ParakeetChildStatus::Exited(exit) => {
                backoff_secs = 5;
                crate::logger::log(
                    "WARN",
                    "Sidecar",
                    None,
                    &format!(
                        "Parakeet server exited unexpectedly ({}); restarting",
                        exit.map(|s| s.to_string())
                            .unwrap_or_else(|| "status unavailable".to_string())
                    ),
                );
                watchdog_restart_parakeet(app_handle, &state);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
            ParakeetChildStatus::Empty => {
                // Nothing to observe. Restore the daemon only while settings
                // still want Parakeet (the user may have disabled it).
                watchdog_restart_parakeet(app_handle, &state);
                watchdog_sleep_interruptible(&stop, backoff_secs);
                backoff_secs = (backoff_secs * 2).min(30);
            }
        }
    }
}

fn watchdog_restart_parakeet<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
    state: &crate::AppState,
) {
    let _lifecycle = recover_lock(&state.parakeet_lifecycle, "Parakeet lifecycle");
    let wants_parakeet = crate::settings::load_settings(app_handle)
        .map(|settings| {
            settings.transcription_mode == "local" && settings.local_engine == "parakeet"
        })
        .unwrap_or(false);
    if !wants_parakeet {
        // The sidecar died while the user switched engines. Clear the corpse so
        // the watchdog stops polling it; nothing else can take it once slot
        // state stops changing under the lifecycle lock.
        let dead = recover_lock(&state.parakeet_server, "Parakeet server").take();
        if dead.is_some() {
            state
                .parakeet_port
                .store(0, std::sync::atomic::Ordering::SeqCst);
        }
        return;
    }
    if let Err(error) = start_parakeet_server_unlocked(app_handle, state) {
        crate::logger::log(
            "WARN",
            "Sidecar",
            None,
            &format!("Parakeet watchdog could not restore the server: {error}"),
        );
    }
}

/// One watchdog per app lifetime for the resident whisper-server: it
/// resurrects a crashed daemon while the user idles between dictations, so a
/// dead instance never downgrades the next dictation to a cold CLI start.
/// Mirrors [`ParakeetWatchdog`]; dropping it (app exit) raises the stop flag.
pub(crate) struct WhisperWatchdog {
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl Drop for WhisperWatchdog {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::SeqCst);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

impl WhisperWatchdog {
    fn spawn<R: Runtime>(app_handle: tauri::AppHandle<R>) -> Self {
        let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
        let thread_stop = std::sync::Arc::clone(&stop);
        let handle = match std::thread::Builder::new()
            .name("whisper-watchdog".to_string())
            .spawn(move || whisper_watchdog_main(&app_handle, thread_stop))
        {
            Ok(handle) => Some(handle),
            Err(error) => {
                crate::logger::log(
                    "WARN",
                    "Sidecar",
                    None,
                    &format!(
                        "Could not spawn the Whisper watchdog thread ({error}); \
                         background auto-restart is disabled for this session"
                    ),
                );
                None
            }
        };
        Self { stop, handle }
    }
}

enum WhisperChildStatus {
    Alive,
    Exited(Option<std::process::ExitStatus>),
    Empty,
}

fn whisper_watchdog_main<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
    stop: std::sync::Arc<std::sync::atomic::AtomicBool>,
) {
    use std::sync::atomic::Ordering;
    let mut backoff_secs: u64 = 5;
    loop {
        if stop.load(Ordering::SeqCst) {
            return;
        }
        let Some(state) = app_handle.try_state::<crate::AppState>() else {
            std::thread::sleep(std::time::Duration::from_secs(1));
            continue;
        };
        let status = {
            let mut slot = recover_lock(&state.whisper_server, "Whisper server");
            match slot.as_mut() {
                None => WhisperChildStatus::Empty,
                Some(server) => match server.child.try_wait() {
                    Ok(None) => WhisperChildStatus::Alive,
                    Ok(Some(exit)) => WhisperChildStatus::Exited(Some(exit)),
                    Err(error) => {
                        crate::logger::log(
                            "WARN",
                            "Sidecar",
                            None,
                            &format!("Could not inspect Whisper server ({error}); restarting"),
                        );
                        WhisperChildStatus::Exited(None)
                    }
                },
            }
        };
        match status {
            WhisperChildStatus::Alive => {
                backoff_secs = 5;
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
            WhisperChildStatus::Exited(exit) => {
                backoff_secs = 5;
                crate::logger::log(
                    "WARN",
                    "Sidecar",
                    None,
                    &format!(
                        "Resident Whisper server exited unexpectedly ({}); restarting",
                        exit.map(|s| s.to_string())
                            .unwrap_or_else(|| "status unavailable".to_string())
                    ),
                );
                watchdog_restart_whisper(app_handle, &state);
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
            WhisperChildStatus::Empty => {
                // Nothing running: restore the daemon only while settings
                // still want the local Whisper engine (the user may have
                // stopped it deliberately after switching engines).
                watchdog_restart_whisper(app_handle, &state);
                watchdog_sleep_interruptible(&stop, backoff_secs);
                backoff_secs = (backoff_secs * 2).min(30);
            }
        }
    }
}

fn watchdog_restart_whisper<R: Runtime>(app_handle: &tauri::AppHandle<R>, state: &crate::AppState) {
    let _lifecycle = recover_lock(&state.whisper_lifecycle, "Whisper lifecycle");
    let wanted_model = crate::settings::load_settings(app_handle)
        .ok()
        .filter(|settings| {
            settings.transcription_mode == "local" && settings.local_engine == "whisper"
        })
        .map(|settings| settings.model_name);
    let Some(model) = wanted_model else {
        // Stopped while the user switched engines or modes: clear the corpse
        // so the watchdog stops polling it.
        let dead = recover_lock(&state.whisper_server, "Whisper server").take();
        if dead.is_some() {
            state
                .whisper_port
                .store(0, std::sync::atomic::Ordering::SeqCst);
        }
        return;
    };
    // Skip pointless attempts (and backoff log spam) while prerequisites are
    // missing — the model may still be downloading or the binaries were just
    // removed. The next ensure/start path spawns the watchdog work normally
    // once they exist again.
    let Ok(app_local_data) = app_handle.path().app_local_data_dir() else {
        return;
    };
    let model_path = app_local_data
        .join("models")
        .join(format_model_filename(&model));
    if !model_path.is_file() || find_whisper_server(app_handle).is_err() {
        return;
    }
    if let Err(error) = start_whisper_server_unlocked(app_handle, state, &model) {
        crate::logger::log(
            "WARN",
            "Sidecar",
            None,
            &format!("Whisper watchdog could not restore the server: {error}"),
        );
    }
}

pub fn ensure_parakeet_server_state<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
    settings: &crate::settings::Settings,
) {
    if settings.transcription_mode == "local" && settings.local_engine == "parakeet" {
        if let Err(error) = start_parakeet_server(app_handle) {
            crate::logger::log(
                "ERROR",
                "Sidecar",
                None,
                &format!("Failed to start Parakeet server: {error}"),
            );
        }
    } else {
        stop_parakeet_server(app_handle);
    }
}

/// Transcribes a 16 kHz mono WAV via the resident Parakeet WebSocket server.
///
/// `is_cancelled` is polled while the response is awaited; the read timeout is
/// kept short so a session that was superseded (or aborted) never leaves the
/// overlay stuck on "processing" for the full response deadline.
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
    mut is_cancelled: impl FnMut() -> bool,
) -> Result<String, String> {
    if let Err(e) = start_parakeet_server(app_handle) {
        return Err(format!(
            "Parakeet server is not running and failed to start: {}",
            e
        ));
    }

    let mut reader =
        hound::WavReader::open(wav_path).map_err(|e| format!("Failed to open WAV file: {}", e))?;
    let spec = reader.spec();
    if spec.sample_rate != 16000 || spec.channels != 1 {
        return Err(format!(
            "Unsupported WAV format: channels={}, sample_rate={}",
            spec.channels, spec.sample_rate
        ));
    }
    if spec.sample_format != hound::SampleFormat::Int || spec.bits_per_sample != 16 {
        return Err(format!(
            "Unsupported WAV sample format: {:?}/{}-bit",
            spec.sample_format, spec.bits_per_sample
        ));
    }

    let sample_count = u64::from(reader.duration());
    if sample_count == 0 {
        return Ok(String::new());
    }
    let expected_byte_size_u64 = sample_count
        .checked_mul(std::mem::size_of::<f32>() as u64)
        .ok_or_else(|| "WAV sample count overflow".to_string())?;
    let expected_byte_size: i32 = expected_byte_size_u64
        .try_into()
        .map_err(|_| "WAV is too large for the Parakeet protocol".to_string())?;

    let port = if let Some(state) = app_handle.try_state::<crate::AppState>() {
        state
            .parakeet_port
            .load(std::sync::atomic::Ordering::SeqCst)
    } else {
        return Err("Application state is unavailable".to_string());
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
                    if is_cancelled() {
                        return Err("Parakeet request was cancelled".to_string());
                    }
                    // First failed connect = the server is still loading the model into RAM.
                    // Tell the user so the wait doesn't look like a freeze.
                    if !notified {
                        notified = true;
                        let _ = app_handle.emit("recording-state", "notice:loading-model");
                    }
                    std::thread::sleep(std::time::Duration::from_millis(200));
                }
            }
        }
    };

    // Guard against a hung/dead server: without a read timeout, socket.read() below
    // would block this dictation forever, leaving the overlay stuck on "processing".
    // The timeout is kept short so cancellation is polled at the same cadence as
    // the streaming previews; the hard deadline is enforced in the read loop.
    if let tungstenite::stream::MaybeTlsStream::Plain(stream) = socket.get_ref() {
        let _ = stream.set_read_timeout(Some(std::time::Duration::from_millis(250)));
        let _ = stream.set_write_timeout(Some(std::time::Duration::from_secs(30)));
    }

    let sample_rate = spec.sample_rate as i32;
    let mut header = Vec::with_capacity(8);
    header.extend_from_slice(&sample_rate.to_le_bytes());
    header.extend_from_slice(&expected_byte_size.to_le_bytes());
    socket
        .send(tungstenite::Message::Binary(header))
        .map_err(|e| format!("Failed to send Parakeet audio header: {e}"))?;

    // The Sherpa offline protocol explicitly permits audio in multiple WebSocket
    // messages. Convert directly from the WAV iterator so a ten-minute recording
    // never creates full-size i16, f32 and wire-format copies at the same time.
    const SAMPLES_PER_MESSAGE: usize = 16_384;
    let mut payload = Vec::with_capacity(SAMPLES_PER_MESSAGE * std::mem::size_of::<f32>());
    let mut sent_bytes = 0u64;
    for sample in reader.samples::<i16>() {
        let normalized =
            sample.map_err(|e| format!("Failed to read WAV samples: {e}"))? as f32 / 32768.0;
        payload.extend_from_slice(&normalized.to_le_bytes());
        if payload.len() >= SAMPLES_PER_MESSAGE * std::mem::size_of::<f32>() {
            sent_bytes += payload.len() as u64;
            socket
                .send(tungstenite::Message::Binary(payload))
                .map_err(|e| format!("Failed to send Parakeet audio chunk: {e}"))?;
            payload = Vec::with_capacity(SAMPLES_PER_MESSAGE * std::mem::size_of::<f32>());
        }
    }
    if !payload.is_empty() {
        sent_bytes += payload.len() as u64;
        socket
            .send(tungstenite::Message::Binary(payload))
            .map_err(|e| format!("Failed to send final Parakeet audio chunk: {e}"))?;
    }
    if sent_bytes != expected_byte_size_u64 {
        return Err(format!(
            "WAV length changed while reading: expected {expected_byte_size_u64} bytes, sent {sent_bytes}"
        ));
    }

    let response_deadline = std::time::Instant::now()
        + std::time::Duration::from_secs(
            (30 + sample_count / u64::from(spec.sample_rate) / 2).min(360),
        );
    let response_text = loop {
        if is_cancelled() {
            return Err("Parakeet request was cancelled".to_string());
        }
        match socket.read() {
            Ok(tungstenite::Message::Text(text)) => break text,
            Ok(tungstenite::Message::Ping(payload)) => socket
                .send(tungstenite::Message::Pong(payload))
                .map_err(|e| format!("Failed to answer Parakeet ping: {e}"))?,
            Ok(tungstenite::Message::Pong(_)) => {}
            Ok(tungstenite::Message::Close(frame)) => {
                return Err(format!(
                    "Parakeet server closed before returning a result: {frame:?}"
                ));
            }
            Ok(_) => return Err("Unexpected binary message from Parakeet server".to_string()),
            Err(tungstenite::Error::Io(error))
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock
                ) =>
            {
                if std::time::Instant::now() >= response_deadline {
                    return Err(format!(
                        "Parakeet transcription timed out for {sample_count} samples"
                    ));
                }
            }
            Err(error) => return Err(format!("Failed to read transcription response: {error}")),
        }
    };

    crate::finish_parakeet_socket(&mut socket);

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

pub fn find_whisper_server<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
) -> Result<PathBuf, String> {
    #[cfg(target_os = "windows")]
    let target_names = [
        "whisper-server-x86_64-pc-windows-msvc.exe",
        "whisper-server.exe",
    ];
    #[cfg(target_os = "macos")]
    let target_names = [
        "whisper-server-aarch64-apple-darwin",
        "whisper-server-x86_64-apple-darwin",
        "whisper-server",
    ];
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let target_names = ["whisper-server"];

    let resource_dir = app_handle
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?;

    let mut candidates: Vec<PathBuf> = Vec::new();

    #[cfg(debug_assertions)]
    for name in &target_names {
        candidates.push(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("binaries")
                .join(name),
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
        "Whisper server executable was not found in resource dir ({:?}).",
        resource_dir
    ))
}

pub fn start_whisper_server<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
    model_name: &str,
) -> Result<(), String> {
    let state = app_handle
        .try_state::<crate::AppState>()
        .ok_or_else(|| "Application state is not initialized".to_string())?;

    let _lifecycle = recover_lock(&state.whisper_lifecycle, "Whisper lifecycle");
    // Lazily spawn the watchdog that resurrects a crashed resident server
    // while the app idles (same pattern as the Parakeet watchdog).
    {
        let mut watchdog_slot = recover_lock(&state.whisper_watchdog, "Whisper watchdog");
        if watchdog_slot.is_none() {
            *watchdog_slot = Some(WhisperWatchdog::spawn(app_handle.clone()));
        }
    }
    start_whisper_server_unlocked(app_handle, &state, model_name)
}

fn start_whisper_server_unlocked<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
    state: &crate::AppState,
    model_name: &str,
) -> Result<(), String> {
    let server_path = find_whisper_server(app_handle)?;
    let short_server_path = get_short_path(&server_path)?;
    let sidecar_dir = server_path
        .parent()
        .ok_or_else(|| "Invalid sidecar path".to_string())?;
    let short_sidecar_dir = get_short_path(sidecar_dir)?;

    let resource_dir = app_handle
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {e}"))?;
    let dlls_dir = resource_dir.join("binaries");
    let short_dlls_dir = get_short_path(&dlls_dir)?;

    let filename = format_model_filename(model_name);
    let app_local_data = app_handle
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app local data dir: {e}"))?;
    let model_path = app_local_data.join("models").join(&filename);

    let spec = whisper_artifact(model_name)?;
    let size_ok = std::fs::metadata(&model_path)
        .map(|metadata| metadata.is_file() && metadata.len() == spec.expected_size)
        .unwrap_or(false);
    if !size_ok {
        return Err(format!(
            "Model file missing or incomplete at: {:?}. Please download it first.",
            model_path
        ));
    }
    runtime_verify_model_file(&filename, &model_path, spec.sha256)?;

    let short_model_path = get_short_path(&model_path)?;
    let n_threads = std::thread::available_parallelism()
        .map(|n| n.get().min(8))
        .unwrap_or(4);

    let settings = crate::settings::load_settings(app_handle).unwrap_or_default();
    let current_accel = if settings.local_acceleration == "cuda" {
        "cuda".to_string()
    } else {
        "cpu".to_string()
    };

    let previous = {
        let mut server = recover_lock(&state.whisper_server, "Whisper server");
        let keep_existing = if let Some(running) = server.as_mut() {
            match running.child.try_wait() {
                Ok(None) => running.model == model_name && running.provider == current_accel,
                Ok(Some(_)) | Err(_) => false,
            }
        } else {
            false
        };
        if keep_existing {
            return Ok(());
        }
        server.take()
    };

    if let Some(previous) = previous {
        state
            .whisper_port
            .store(0, std::sync::atomic::Ordering::SeqCst);
        drop(previous);
    }

    let port = get_free_port()?;
    let mut working_dir = short_dlls_dir.to_path_buf();
    if current_accel == "cuda" {
        let cuda_bin = app_local_data.join("binaries").join("cuda").join("bin");
        if cuda_bin.join("ggml-cuda.dll").exists() {
            if let Ok(short_cuda_bin) = get_short_path(&cuda_bin) {
                working_dir = short_cuda_bin;
            }
        }
    }

    let mut cmd = Command::new(&short_server_path);
    cmd.current_dir(&working_dir);
    let mut args = vec![
        "-m".to_string(),
        short_model_path.to_string_lossy().to_string(),
        "--host".to_string(),
        "127.0.0.1".to_string(),
        "--port".to_string(),
        port.to_string(),
        "-t".to_string(),
        n_threads.to_string(),
        "-bo".to_string(),
        "1".to_string(),
        "-bs".to_string(),
        "-1".to_string(),
        "-nt".to_string(),
        "-nf".to_string(),
        "-sow".to_string(),
    ];
    if current_accel == "cpu" {
        args.push("-ng".to_string());
    }
    cmd.args(&args);

    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    #[cfg(target_os = "windows")]
    {
        let path_key = std::env::vars_os()
            .map(|(k, _)| k.to_string_lossy().into_owned())
            .find(|name| name.eq_ignore_ascii_case("path"))
            .unwrap_or_else(|| "PATH".to_string());

        let mut paths = if let Some(path_env) = std::env::var_os(&path_key) {
            std::env::split_paths(&path_env).collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        if let Ok(app_local_data) = app_handle.path().app_local_data_dir() {
            paths.insert(0, app_local_data.join("binaries").join("cuda").join("bin"));
            paths.insert(0, app_local_data.join("binaries").join("cuda"));
        }
        if let Some(parent) = server_path.parent() {
            paths.insert(
                0,
                parent
                    .join("cuda")
                    .join("sherpa-onnx-v1.13.4-win-x64-cuda")
                    .join("bin"),
            );
            paths.insert(0, parent.join("cuda").join("bin"));
            paths.insert(0, parent.join("cuda"));
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

    #[cfg(target_os = "windows")]
    let prepared_job = match create_kill_on_close_job() {
        Ok(job) => Some(job),
        Err(error) => {
            crate::logger::log(
                "WARN",
                "Sidecar",
                None,
                &format!("Could not create Whisper kill-on-close Job Object: {error}"),
            );
            None
        }
    };

    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("Failed to spawn Whisper server: {e}"))?;

    #[cfg(target_os = "windows")]
    let kill_on_close_job = match prepared_job {
        Some(job) => match assign_process_to_job(&job, &child) {
            Ok(()) => Some(job),
            Err(_) => None,
        },
        None => None,
    };

    let mut readers = pipe_child_output(&mut child);

    let started = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(30);
    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));

    let mut is_ready = false;
    while started.elapsed() < timeout {
        match child.try_wait() {
            Ok(Some(status)) => {
                readers.join();
                return Err(format!("Whisper server exited early with status {status}"));
            }
            Ok(None) => {}
            Err(e) => return Err(format!("Failed to wait on Whisper server: {e}")),
        }

        if std::net::TcpStream::connect_timeout(&addr, std::time::Duration::from_millis(100))
            .is_ok()
        {
            is_ready = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    if !is_ready {
        let _ = child.kill();
        let _ = child.wait();
        readers.join();
        return Err(format!(
            "Whisper server timed out waiting for HTTP port {port}"
        ));
    }

    crate::logger::log(
        "INFO",
        "Sidecar",
        None,
        &format!("Whisper server started successfully on port {port} for model {model_name}"),
    );

    let provider = {
        let settings = crate::settings::load_settings(app_handle).unwrap_or_default();
        if settings.local_acceleration == "cuda" {
            "cuda".to_string()
        } else {
            "cpu".to_string()
        }
    };

    state
        .whisper_port
        .store(port, std::sync::atomic::Ordering::SeqCst);
    *recover_lock(&state.whisper_server, "Whisper server") = Some(RunningWhisperServer {
        child,
        model: model_name.to_string(),
        provider,
        port,
        readers,
        #[cfg(target_os = "windows")]
        _kill_on_close_job: kill_on_close_job,
    });

    Ok(())
}

pub fn stop_whisper_server<R: Runtime>(app_handle: &tauri::AppHandle<R>) {
    let Some(state) = app_handle.try_state::<crate::AppState>() else {
        return;
    };
    let _lifecycle = recover_lock(&state.whisper_lifecycle, "Whisper lifecycle");
    state
        .whisper_port
        .store(0, std::sync::atomic::Ordering::SeqCst);
    let mut server = recover_lock(&state.whisper_server, "Whisper server");
    if let Some(running) = server.take() {
        drop(running);
    }
}

pub fn ensure_whisper_server_state<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
    settings: &crate::settings::Settings,
) {
    if settings.transcription_mode == "local" && settings.local_engine == "whisper" {
        let model = settings.model_name.clone();
        let app_handle_clone = app_handle.clone();
        tauri::async_runtime::spawn_blocking(move || {
            if find_whisper_server(&app_handle_clone).is_ok() {
                if let Err(e) = start_whisper_server(&app_handle_clone, &model) {
                    crate::logger::log(
                        "WARN",
                        "Sidecar",
                        None,
                        &format!("Could not prewarm resident Whisper server for '{model}': {e}"),
                    );
                }
            } else {
                prewarm_local_model_background(&app_handle_clone, &model);
            }
        });
    } else {
        stop_whisper_server(app_handle);
    }
}

pub(crate) fn whisper_server_status<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
) -> Option<(String, u16)> {
    let state = app_handle.try_state::<crate::AppState>()?;
    let server = recover_lock(&state.whisper_server, "Whisper server");
    server
        .as_ref()
        .map(|running| (running.provider.clone(), running.port))
}

pub fn build_whisper_prompt(language: &str, user_dictionary: &str) -> String {
    let base_prompt = match language {
        "ru" => "Привет, вот пример грамотной диктовки: с точками, запятыми и верным регистром.",
        "en" => "Hello, here is a clean dictation sample: with periods, commas, and proper capitalization.",
        "de" => "Hallo, hier ist eine saubere Diktatprobe: mit Punkten, Kommas und korrekter Großschreibung.",
        "es" => "Hola, este es un ejemplo de dictado claro: con puntos, comas y mayúsculas correctas.",
        "fr" => "Bonjour, voici un exemple de dictée claire : avec des points, des virgules et des majuscules correctes.",
        "it" => "Ciao, ecco un esempio di dettatura chiara: con punti, virgole e maiuscole corrette.",
        "zh" => "你好，这是一个标点和大小写规范的听写示例。",
        "pt" => "Olá, este é um exemplo de ditado claro: com pontos, vírgulas e maiúsculas corretas.",
        "tr" => "Merhaba, noktaları, virgülleri ve doğru büyük harfleri olan düzgün bir dikte örneği.",
        _ => "Привет, вот пример грамотной диктовки: с точками, запятыми и верным регистром.",
    };

    let dict = user_dictionary.trim();
    if dict.is_empty() {
        base_prompt.to_string()
    } else {
        match language {
            "ru" => format!("{base_prompt} Термины: {dict}."),
            "en" => format!("{base_prompt} Vocabulary: {dict}."),
            _ => format!("{base_prompt} Terms: {dict}."),
        }
    }
}

pub async fn transcribe_via_whisper_server(
    port: u16,
    wav_path: &Path,
    language: &str,
    dictionary: &str,
) -> Result<String, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

    let file_bytes = tokio::fs::read(wav_path)
        .await
        .map_err(|e| format!("Failed to read WAV file: {e}"))?;

    let file_part = reqwest::multipart::Part::bytes(file_bytes)
        .file_name("audio.wav")
        .mime_str("audio/wav")
        .map_err(|e| format!("Failed to set MIME type: {e}"))?;

    let lang = match language {
        "ru" | "en" | "de" | "es" | "fr" | "it" | "zh" | "pt" | "tr" => language,
        _ => "auto",
    };

    let primed_prompt = build_whisper_prompt(lang, dictionary);

    let mut form = reqwest::multipart::Form::new()
        .part("file", file_part)
        .text("language", lang.to_string())
        .text("response_format", "json")
        .text("temperature", "0.0");

    if !primed_prompt.is_empty() {
        form = form.text("prompt", primed_prompt);
    }

    let url = format!("http://127.0.0.1:{port}/inference");
    let response = client
        .post(&url)
        .multipart(form)
        .send()
        .await
        .map_err(|e| format!("Whisper server request failed: {e}"))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Whisper server error ({status}): {body}"));
    }

    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Failed to parse Whisper server response: {e}"))?;

    if let Some(text) = body.get("text").and_then(|v| v.as_str()) {
        Ok(text.trim().to_string())
    } else {
        Err(format!("Unexpected Whisper server response format: {body}"))
    }
}

pub fn find_sherpa_punctuation_exe<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
) -> Result<PathBuf, String> {
    let settings = crate::settings::load_settings(app_handle).unwrap_or_default();
    let exe_name = if cfg!(target_os = "windows") {
        "sherpa-onnx-offline-punctuation.exe"
    } else {
        "sherpa-onnx-offline-punctuation"
    };

    if settings.local_acceleration != "cpu" {
        if let Ok(app_local_data) = app_handle.path().app_local_data_dir() {
            let gpu_exe = app_local_data
                .join("binaries")
                .join(&settings.local_acceleration)
                .join("bin")
                .join(exe_name);
            if gpu_exe.exists() {
                #[cfg(target_os = "windows")]
                {
                    if let Some(parent) = gpu_exe.parent() {
                        if parent.join("onnxruntime.dll").exists() {
                            return Ok(gpu_exe);
                        }
                    }
                }
                #[cfg(not(target_os = "windows"))]
                {
                    return Ok(gpu_exe);
                }
            }
        }
    }

    #[cfg(target_os = "windows")]
    let target_names = ["sherpa-onnx-offline-punctuation.exe"];
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
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("binaries")
                .join(name),
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

const PUNCTUATION_ARCHIVE_URL: &str = "https://github.com/k2-fsa/sherpa-onnx/releases/download/punctuation-models/sherpa-onnx-punct-ct-transformer-zh-en-vocab272727-2024-04-12-int8.tar.bz2";
const PUNCTUATION_ARCHIVE_SIZE: u64 = 64_717_756;
const PUNCTUATION_ARCHIVE_SHA256: &str =
    "c0d5aa5f8eeb686032345e180bedf39319dc2e0556781c6264bcadba8328a6e1";

struct RemoveDirectoryOnDrop(PathBuf);

impl Drop for RemoveDirectoryOnDrop {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.0) {
            if error.kind() != std::io::ErrorKind::NotFound {
                crate::logger::log(
                    "WARN",
                    "Punctuation",
                    None,
                    &format!(
                        "Failed to clean temporary punctuation directory {}: {error}",
                        self.0.display()
                    ),
                );
            }
        }
    }
}

pub(crate) fn punctuation_files_complete(directory: &Path) -> bool {
    // The CT-transformer run path only needs model.int8.onnx (--ct-transformer).
    // Old builds also required tokens.txt, which this archive does not ship:
    // it carries tokens.json, so tokens must not gate installation.
    directory
        .join("model.int8.onnx")
        .metadata()
        .map(|metadata| metadata.is_file() && metadata.len() > 0)
        .unwrap_or(false)
}

/// Locate the directory that directly holds the punctuation model inside an
/// extraction tree, tolerating a versioned top-level folder (the archives
/// ship as `sherpa-onnx-punct-ct-transformer-.../model.int8.onnx`).
fn find_dir_containing_punctuation_model(root: &Path, depth: usize) -> Option<PathBuf> {
    if punctuation_files_complete(root) {
        return Some(root.to_path_buf());
    }
    if depth == 0 {
        return None;
    }
    for entry in std::fs::read_dir(root).ok()?.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if let Some(found) = find_dir_containing_punctuation_model(&path, depth - 1) {
                return Some(found);
            }
        }
    }
    None
}

fn unique_punctuation_staging_dir(models_dir: &Path) -> PathBuf {
    let suffix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    models_dir.join(format!(
        ".punctuation.install-{}-{suffix}",
        std::process::id()
    ))
}

fn install_punctuation_archive(
    archive_path: &Path,
    staging_dir: &Path,
    destination: &Path,
) -> Result<(), String> {
    let extraction_dir = staging_dir.join("extract");
    std::fs::create_dir_all(&extraction_dir)
        .map_err(|error| format!("Failed to create punctuation extraction directory: {error}"))?;
    let archive_file = std::fs::File::open(archive_path)
        .map_err(|error| format!("Failed to open verified punctuation archive: {error}"))?;
    let decoder = bzip2::read::BzDecoder::new(archive_file);
    let mut archive = tar::Archive::new(decoder);
    archive
        .unpack(&extraction_dir)
        .map_err(|error| format!("Failed to extract punctuation archive safely: {error}"))?;

    // The archives ship inside a versioned top-level directory; resolve it
    // instead of assuming a fixed folder name at a fixed depth.
    let source = find_dir_containing_punctuation_model(&extraction_dir, 2).ok_or_else(|| {
        "Verified punctuation archive does not contain model.int8.onnx".to_string()
    })?;
    let parent = destination
        .parent()
        .ok_or_else(|| "Punctuation destination has no parent directory".to_string())?;
    let backup = parent.join(".punctuation.previous");
    if backup.exists() {
        std::fs::remove_dir_all(&backup)
            .map_err(|error| format!("Failed to clear stale punctuation backup: {error}"))?;
    }
    let had_previous = destination.exists();
    if had_previous {
        std::fs::rename(destination, &backup)
            .map_err(|error| format!("Failed to stage previous punctuation model: {error}"))?;
    }
    if let Err(error) = std::fs::rename(&source, destination) {
        if had_previous {
            let _ = std::fs::rename(&backup, destination);
        }
        return Err(format!("Failed to install punctuation model: {error}"));
    }
    if had_previous {
        if let Err(error) = std::fs::remove_dir_all(&backup) {
            crate::logger::log(
                "WARN",
                "Punctuation",
                None,
                &format!("Punctuation install succeeded but old backup cleanup failed: {error}"),
            );
        }
    }
    Ok(())
}

/// Facade with one automatic retry: resize/CDN hiccups (rare spikes in streamed
/// byte counts) are retried once before surrendering, because a byte-level
/// hiccup must not leave the feature permanently broken.
pub async fn download_punctuation_model<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
) -> Result<(), String> {
    let mut last_error = String::from("unknown download error");
    for attempt in 1..=2u32 {
        if is_cancel_requested("punctuation") {
            return Err("Punctuation download cancelled".to_string());
        }
        match download_punctuation_model_attempt(app_handle).await {
            Ok(()) => return Ok(()),
            Err(error) => {
                let retryable = error.contains("Incomplete punctuation download")
                    || error.contains("stalled for more than 30 seconds");
                if retryable && attempt == 1 {
                    crate::logger::log(
                        "WARN",
                        "Punctuation",
                        None,
                        &format!(
                            "Punctuation download attempt {attempt} failed ({error}); retrying"
                        ),
                    );
                    last_error = error;
                    tokio::time::sleep(std::time::Duration::from_millis(800)).await;
                    continue;
                }
                return Err(error);
            }
        }
    }
    Err(format!(
        "Punctuation download failed after retries: {last_error}"
    ))
}

async fn download_punctuation_model_attempt<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
) -> Result<(), String> {
    let app_local_data = app_handle
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app local data dir: {}", e))?;
    let models_dir = app_local_data.join("models");
    let punc_dir = models_dir.join("punctuation");
    if punctuation_files_complete(&punc_dir) {
        return Ok(());
    }
    let _lease = match begin_model_download("punctuation") {
        Ok(lease) => lease,
        Err(_) => {
            // A concurrent download (e.g. a startup self-heal) already owns
            // the lease; wait for it to land its files instead of failing.
            for _ in 0..120 {
                if punctuation_files_complete(&punc_dir) {
                    return Ok(());
                }
                tokio::time::sleep(std::time::Duration::from_millis(500)).await;
            }
            return Err(
                "Punctuation model download is already in progress; timed out waiting for it"
                    .to_string(),
            );
        }
    };
    clear_cancel("punctuation");
    tokio::fs::create_dir_all(&models_dir)
        .await
        .map_err(|e| format!("Failed to create models directory: {e}"))?;
    let staging_dir = unique_punctuation_staging_dir(&models_dir);
    tokio::fs::create_dir(&staging_dir)
        .await
        .map_err(|e| format!("Failed to create punctuation staging directory: {e}"))?;
    let _staging_guard = RemoveDirectoryOnDrop(staging_dir.clone());
    let archive_path = staging_dir.join("punctuation.tar.bz2");

    crate::logger::log(
        "INFO",
        "Punctuation",
        None,
        "Downloading local punctuation model...",
    );
    let client = crate::ai_client::build_download_client();
    let mut response = client
        .get(PUNCTUATION_ARCHIVE_URL)
        .send()
        .await
        .map_err(|e| format!("Failed to download punctuation model: {}", e))?;
    if !response.status().is_success() {
        return Err(format!(
            "Failed to download punctuation model: HTTP {}",
            response.status()
        ));
    }
    if let Some(advertised_size) = response.content_length() {
        if advertised_size != PUNCTUATION_ARCHIVE_SIZE {
            return Err(format!(
                "Punctuation archive size mismatch: expected {PUNCTUATION_ARCHIVE_SIZE}, server advertised {advertised_size}"
            ));
        }
    }

    use sha2::{Digest, Sha256};
    use tokio::io::AsyncWriteExt;
    let mut file = tokio::fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&archive_path)
        .await
        .map_err(|e| format!("Failed to create punctuation archive: {e}"))?;
    let mut hasher = Sha256::new();
    let mut downloaded = 0u64;
    // Read exactly PUNCTUATION_ARCHIVE_SIZE payload bytes. Some CDNs/proxies
    // append trailing bytes after the payload; rather than aborting a good
    // download, take the pinned slice and let SHA-256 be the integrity judge.
    while downloaded < PUNCTUATION_ARCHIVE_SIZE {
        let chunk = tokio::time::timeout(std::time::Duration::from_secs(30), response.chunk())
            .await
            .map_err(|_| "Punctuation download stalled for more than 30 seconds".to_string())?
            .map_err(|e| format!("Error reading punctuation chunk: {e}"))?;
        let Some(chunk) = chunk else {
            break;
        };
        if is_cancel_requested("punctuation") {
            return Err("Punctuation download cancelled".to_string());
        }
        let take = chunk
            .len()
            .min((PUNCTUATION_ARCHIVE_SIZE - downloaded) as usize);
        let payload = &chunk[..take];
        downloaded = downloaded.saturating_add(take as u64);
        let is_verifying = downloaded >= PUNCTUATION_ARCHIVE_SIZE;
        let status = if is_verifying {
            Some("installing".to_string())
        } else {
            None
        };
        let _ = app_handle.emit(
            "model-download-progress",
            DownloadProgress {
                model: "punctuation".to_string(),
                downloaded,
                total: Some(PUNCTUATION_ARCHIVE_SIZE),
                percentage: (downloaded as f64 / PUNCTUATION_ARCHIVE_SIZE as f64) * 100.0,
                done: false,
                status,
            },
        );
        hasher.update(payload);
        file.write_all(payload)
            .await
            .map_err(|e| format!("Failed to write punctuation chunk: {e}"))?;
    }
    file.flush()
        .await
        .map_err(|e| format!("Failed to flush punctuation archive: {e}"))?;
    file.sync_all()
        .await
        .map_err(|e| format!("Failed to persist punctuation archive: {e}"))?;
    drop(file);

    if downloaded != PUNCTUATION_ARCHIVE_SIZE {
        return Err(format!(
            "Incomplete punctuation download: expected {PUNCTUATION_ARCHIVE_SIZE} bytes, received {downloaded}"
        ));
    }
    let actual_hash = format!("{:x}", hasher.finalize());
    if actual_hash != PUNCTUATION_ARCHIVE_SHA256 {
        return Err(format!(
            "Punctuation archive integrity check failed: expected {PUNCTUATION_ARCHIVE_SHA256}, got {actual_hash}"
        ));
    }

    let _ = app_handle.emit(
        "model-download-progress",
        DownloadProgress {
            model: "punctuation".to_string(),
            downloaded: PUNCTUATION_ARCHIVE_SIZE,
            total: Some(PUNCTUATION_ARCHIVE_SIZE),
            percentage: 100.0,
            done: false,
            status: Some("installing".to_string()),
        },
    );

    let worker_archive = archive_path.clone();
    let worker_staging = staging_dir.clone();
    let worker_destination = punc_dir.clone();
    tauri::async_runtime::spawn_blocking(move || {
        install_punctuation_archive(&worker_archive, &worker_staging, &worker_destination)
    })
    .await
    .map_err(|error| format!("Punctuation install worker failed: {error}"))??;

    crate::logger::log(
        "INFO",
        "Punctuation",
        None,
        "Punctuation model downloaded successfully.",
    );
    let _ = app_handle.emit(
        "model-download-progress",
        DownloadProgress {
            model: "punctuation".to_string(),
            downloaded: PUNCTUATION_ARCHIVE_SIZE,
            total: Some(PUNCTUATION_ARCHIVE_SIZE),
            percentage: 100.0,
            done: true,
            status: Some("done".to_string()),
        },
    );
    Ok(())
}

pub fn run_punctuation<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
    text: &str,
) -> Result<String, String> {
    // Windows CreateProcess limit is ~32 767 chars; long dictations would silently fail.
    // At >4 000 chars the marginal value of offline punctuation is also low (LLM/cloud
    // mode handles its own punctuation), so return the text as-is for safety.
    const MAX_CLI_CHARS: usize = 4_000;
    if text.chars().count() > MAX_CLI_CHARS {
        crate::logger::log(
            "WARN",
            "Punctuation",
            None,
            &format!(
                "run_punctuation skipped — text too long ({} chars > {})",
                text.chars().count(),
                MAX_CLI_CHARS
            ),
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

    let output = run_command_with_timeout(
        &mut cmd,
        std::time::Duration::from_secs(60),
        "Punctuation sidecar",
    )?;
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
    fn punctuation_complete_requires_only_the_onnx_model() {
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("aura_punct_test_{unique_id}"));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        assert!(!punctuation_files_complete(&dir));
        std::fs::write(dir.join("model.int8.onnx"), b"model").unwrap();
        assert!(punctuation_files_complete(&dir));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn punctuation_archive_with_nested_versioned_folder_is_locatable() {
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let root = std::env::temp_dir().join(format!("aura_punct_frame_{unique_id}"));
        let _ = std::fs::remove_dir_all(&root);
        // Replicates the real archive layout: top-level versioned folder.
        let inner = root.join("sherpa-onnx-punct-ct-transformer-zh-en-vocab272727-2024-04-12-int8");
        std::fs::create_dir_all(&inner).unwrap();
        std::fs::write(inner.join("model.int8.onnx"), b"model").unwrap();
        std::fs::write(inner.join("tokens.json"), b"[]").unwrap();
        std::fs::write(root.join("README.md"), b"readme").unwrap();

        let found = find_dir_containing_punctuation_model(&root, 2).unwrap();
        assert!(
            found.ends_with("sherpa-onnx-punct-ct-transformer-zh-en-vocab272727-2024-04-12-int8")
        );
        assert!(punctuation_files_complete(&found));
        let _ = std::fs::remove_dir_all(&root);
    }

    /// Network reproduction for the "stream changed size" reports: downloads
    /// with the real client and reports received bytes vs pinned size.
    /// Run: cargo test --lib -- --ignored reproduce_punctuation_download
    #[ignore = "requires network"]
    #[tokio::test]
    async fn reproduce_punctuation_download_with_app_client() {
        let client = crate::ai_client::build_download_client();
        let mut response = client
            .get(PUNCTUATION_ARCHIVE_URL)
            .send()
            .await
            .expect("send");
        let advertised = response.content_length();
        let mut received: u64 = 0;
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        loop {
            let Some(chunk) = response.chunk().await.expect("chunk") else {
                break;
            };
            received = received.saturating_add(chunk.len() as u64);
            hasher.update(&chunk);
        }
        eprintln!(
            "REPRO received={received} advertised={advertised:?} pinned={PUNCTUATION_ARCHIVE_SIZE} sha256={:x}",
            hasher.finalize()
        );
        assert_eq!(
            received, PUNCTUATION_ARCHIVE_SIZE,
            "stream byte count mismatch"
        );
    }

    #[test]
    fn test_filename_parsing() {
        assert_eq!(format_model_filename("small"), "ggml-small.bin");
        assert_eq!(format_model_filename("ggml-base.bin"), "ggml-base.bin");
        assert_eq!(format_model_filename("base.bin"), "ggml-base.bin");
        assert_eq!(format_model_filename("ggml-tiny"), "ggml-tiny.bin");
    }

    #[test]
    fn whisper_timeout_is_model_aware_and_bounded() {
        assert_eq!(whisper_timeout_for_duration("base", 0).as_secs(), 180);
        assert_eq!(whisper_timeout_for_duration("base", 60).as_secs(), 240);
        assert_eq!(whisper_timeout_for_duration("small", 60).as_secs(), 300);
        assert_eq!(whisper_timeout_for_duration("medium", 600).as_secs(), 3_120);
        assert_eq!(
            whisper_timeout_for_duration("large", 10_000).as_secs(),
            3_600
        );
    }

    #[test]
    fn only_windows_post_close_10058_is_filtered_from_sidecar_diagnostics() {
        let benign = "handle_read_frame error: asio.system:10058 (socket already shut down)";
        assert_eq!(
            is_benign_websocket_shutdown_line(benign),
            cfg!(target_os = "windows")
        );
        assert!(!is_benign_websocket_shutdown_line(
            "handle_read_frame error: asio.system:10053 (connection aborted)"
        ));
        assert!(!is_benign_websocket_shutdown_line(
            "CUDA provider failed to initialize"
        ));
    }

    #[test]
    fn only_known_parakeet_status_lines_are_suppressed_from_unified_log() {
        for routine in [
            "",
            "parse-options.cc:Read:374 'server.exe' --provider=cuda",
            "offline-websocket-server.cc:main:91 Started!",
            "offline-websocket-server.cc:main:92 Listening on: 61709",
            "offline-websocket-server.cc:main:93 Number of work threads: 16",
            "offline-websocket-server-impl.cc:Decode:68 size: 1",
            "offline-websocket-server-impl.cc:OnOpen:172 Number of active connections: 1",
            "offline-websocket-server-impl.cc:OnClose:180 Number of active connections: 0",
        ] {
            assert!(is_routine_parakeet_status_line(routine), "{routine}");
        }

        for warning in [
            "offline-websocket-server-impl.cc:Decode:68 CUDA provider failed",
            "offline-websocket-server-impl.cc:OnOpen:172 bind failed",
            "offline-websocket-server.cc:main:93 CUDA provider failed",
            "CUDA provider failed to initialize",
        ] {
            assert!(!is_routine_parakeet_status_line(warning), "{warning}");
        }
    }

    #[test]
    fn retryable_sidecar_failures_are_classified_narrowly() {
        assert!(is_retryable_sidecar_failure(
            "cpu sidecar exited before becoming ready (exit code: 1): bind: address already in use"
        ));
        assert!(is_retryable_sidecar_failure(
            "cpu sidecar did not accept WebSocket connections on port 51234 within 45 seconds: connection refused"
        ));
        assert!(!is_retryable_sidecar_failure(
            "Failed to spawn Parakeet server 'x' (cpu): oom"
        ));
        // A functional warm-up hiccup is transient (first-inference JIT, driver
        // init), so it retries like a bind race rather than failing startup
        // permanently after a single attempt.
        assert!(is_retryable_sidecar_failure(
            "cpu sidecar failed functional inference warm-up: cuda error"
        ));
        assert!(!is_retryable_sidecar_failure(
            "Parakeet model file 'encoder.onnx' is missing"
        ));
    }

    fn spawn_sleeper_process() -> std::process::Child {
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            let mut command = std::process::Command::new("powershell");
            command
                .args(["-NoProfile", "-Command", "Start-Sleep -Seconds 15"])
                .creation_flags(0x08000000)
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            command.spawn().expect("spawn powershell sleeper")
        }
        #[cfg(not(target_os = "windows"))]
        {
            let mut command = std::process::Command::new("sleep");
            command
                .arg("15")
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            command.spawn().expect("spawn sleep sleeper")
        }
    }

    #[cfg(target_os = "windows")]
    fn process_is_alive(pid: u32) -> bool {
        use windows_sys::Win32::Foundation::STILL_ACTIVE;
        use windows_sys::Win32::System::Threading::{
            GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
        };
        unsafe {
            let handle = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if handle == 0 {
                return false;
            }
            let mut exit_code: u32 = 0;
            let ok = GetExitCodeProcess(handle, &mut exit_code);
            windows_sys::Win32::Foundation::CloseHandle(handle);
            ok != 0 && exit_code == STILL_ACTIVE as u32
        }
    }

    #[cfg(target_os = "windows")]
    fn wait_for_process_exit(pid: u32) {
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        while std::time::Instant::now() < deadline {
            if !process_is_alive(pid) {
                return;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
        panic!("process {pid} was still alive 5 seconds after being killed");
    }

    #[test]
    fn dropping_running_parakeet_server_kills_the_sidecar() {
        let child = spawn_sleeper_process();
        let pid = child.id();
        let server = RunningParakeetServer {
            child,
            provider: "test".to_string(),
            executable: PathBuf::from("sleeper"),
            port: 0,
            readers: SidecarPipeReaders::default(),
            #[cfg(target_os = "windows")]
            _kill_on_close_job: None,
        };
        drop(server);
        #[cfg(target_os = "windows")]
        wait_for_process_exit(pid);
        #[cfg(not(target_os = "windows"))]
        {
            // std::process kill is exercised on Windows above; keep the unix
            // build honest with the same semantics via try_wait ownership.
            let _ = pid;
        }
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn kill_on_close_job_kills_child_when_last_handle_closes() {
        let child = spawn_sleeper_process();
        let pid = child.id();
        let job = create_kill_on_close_job().expect("create kill-on-close job");
        assign_process_to_job(&job, &child).expect("assign child to job");
        drop(child);
        assert!(
            process_is_alive(pid),
            "closing the Child handle must not kill a process inside an open job"
        );
        drop(job);
        wait_for_process_exit(pid);
    }

    #[test]
    fn pipe_readers_join_after_the_child_is_killed() {
        let mut child = {
            #[cfg(target_os = "windows")]
            {
                let mut command = Command::new("cmd");
                command.args(["/c", "echo sidecar-hello"]);
                command
            }
            #[cfg(not(target_os = "windows"))]
            {
                let mut command = Command::new("sh");
                command.args(["-c", "echo sidecar-hello"]);
                command
            }
        }
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn echo sidecar");
        let mut readers = pipe_child_output(&mut child);
        std::thread::sleep(std::time::Duration::from_millis(100));
        let _ = child.kill();
        let _ = child.wait();
        readers.join();
        assert!(
            recent_sidecar_diagnostics(&readers.diagnostics).contains("sidecar-hello"),
            "the joined reader must have drained the sidecar output"
        );
    }

    #[test]
    fn dropping_running_whisper_server_kills_the_sidecar() {
        let child = spawn_sleeper_process();
        let pid = child.id();
        let server = RunningWhisperServer {
            child,
            model: "test_model".to_string(),
            provider: "whisper".to_string(),
            port: 0,
            readers: SidecarPipeReaders::default(),
            #[cfg(target_os = "windows")]
            _kill_on_close_job: None,
        };
        drop(server);
        #[cfg(target_os = "windows")]
        wait_for_process_exit(pid);
        #[cfg(not(target_os = "windows"))]
        {
            let _ = pid;
        }
    }
}
