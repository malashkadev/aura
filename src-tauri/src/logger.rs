use crate::settings::Settings;
use crate::whisper_runner::SidecarRuntimeStatus;
use std::fmt::Write as FmtWrite;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, SyncSender, TrySendError};
use std::sync::Mutex;
use std::thread;
use std::time::SystemTime;
use tauri::Manager;

pub const MAX_LOG_SIZE: u64 = 10 * 1024 * 1024; // 10 MB
pub const LOG_FILE_NAME: &str = "aura_diagnostics.log";
const LOGGER_QUEUE_CAPACITY: usize = 2_048;
const EARLY_LOG_IDENTIFIER: &str = "com.aura.app";

static LOGGER_TX: Mutex<Option<SyncSender<String>>> = Mutex::new(None);
static DROPPED_LOG_LINES: AtomicU64 = AtomicU64::new(0);
static LOGGER_DIR: Mutex<Option<PathBuf>> = Mutex::new(None);
static INITIALIZED: AtomicBool = AtomicBool::new(false);

pub fn anonymize_speech(text: &str, log_speech_text: bool) -> String {
    if log_speech_text {
        text.to_string()
    } else {
        let word_count = text.split_whitespace().count();
        let char_count = text.chars().count();
        format!("[REDACTED: {} words, {} chars]", word_count, char_count)
    }
}

pub fn format_session_tag(session_id: u64) -> String {
    format!("#session-{}", session_id)
}

pub fn init(log_dir: PathBuf) {
    if INITIALIZED.swap(true, Ordering::SeqCst) {
        return;
    }

    if let Err(e) = fs::create_dir_all(&log_dir) {
        INITIALIZED.store(false, Ordering::SeqCst);
        eprintln!("[LOGGER ERROR] Failed to create log directory: {}", e);
        return;
    }

    let (tx, rx) = mpsc::sync_channel::<String>(LOGGER_QUEUE_CAPACITY);
    let dir = log_dir.clone();

    thread::spawn(move || {
        while let Ok(msg) = rx.recv() {
            if let Err(e) = rotate_logs_if_needed(&dir, MAX_LOG_SIZE) {
                eprintln!("[LOGGER ERROR] Failed log rotation: {}", e);
            }
            let log_path = dir.join(LOG_FILE_NAME);
            if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(&log_path) {
                let _ = writeln!(file, "{}", msg);
            }
        }
    });

    if let Ok(mut lock) = LOGGER_TX.lock() {
        *lock = Some(tx);
    }
    if let Ok(mut lock) = LOGGER_DIR.lock() {
        *lock = Some(log_dir);
    }
}

fn early_log_dir() -> Option<PathBuf> {
    std::env::var_os("LOCALAPPDATA")
        .map(|base| PathBuf::from(base).join(EARLY_LOG_IDENTIFIER).join("logs"))
}

pub fn init_early() {
    if let Some(log_dir) = early_log_dir() {
        init(log_dir);
    }
}

pub fn log_startup() {
    let args: Vec<String> = std::env::args().collect();
    let cwd = std::env::current_dir()
        .map(|path| path.display().to_string())
        .unwrap_or_default();
    log(
        "INFO",
        "App",
        None,
        &format!(
            "Aura process starting (pid={}, cwd={}, args={:?})",
            std::process::id(),
            cwd,
            args
        ),
    );
}

pub fn install_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        log_panic_sync(&info.to_string());
    }));
}

pub fn log_panic_sync(message: &str) {
    let line = format!(
        "{} [ERROR] [Panic] {}",
        format_timestamp(SystemTime::now()),
        message
    );
    eprintln!("{}", line);

    let log_dir = LOGGER_DIR
        .try_lock()
        .ok()
        .and_then(|dir| dir.as_ref().cloned())
        .or_else(early_log_dir);
    let Some(log_dir) = log_dir else {
        return;
    };

    let _ = fs::create_dir_all(&log_dir);
    if let Ok(mut file) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_dir.join(LOG_FILE_NAME))
    {
        let _ = writeln!(file, "{}", line);
    }
}

pub fn log(level: &str, tag: &str, session: Option<&str>, message: &str) {
    let timestamp = format_timestamp(SystemTime::now());
    let level_str = level.to_uppercase();
    let session_str = match session {
        Some(s) if s.starts_with('[') => format!(" {}", s),
        Some(s) => format!(" [{}]", s),
        None => String::new(),
    };

    let formatted_line = format!(
        "{} [{}] [{}]{} {}",
        timestamp, level_str, tag, session_str, message
    );
    eprintln!("{}", formatted_line);

    let lock = match LOGGER_TX.lock() {
        Ok(lock) => lock,
        Err(poisoned) => poisoned.into_inner(),
    };
    if let Some(tx) = lock.as_ref() {
        match tx.try_send(formatted_line) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                let dropped = DROPPED_LOG_LINES.fetch_add(1, Ordering::Relaxed) + 1;
                if dropped == 1 || dropped.is_multiple_of(1_000) {
                    eprintln!(
                        "[LOGGER WARN] Dropped {dropped} log lines because the disk writer is behind"
                    );
                }
            }
            Err(TrySendError::Disconnected(_)) => {
                eprintln!("[LOGGER ERROR] Background log writer is unavailable");
            }
        }
    }
}

pub fn rotate_logs_if_needed(log_dir: &Path, max_bytes: u64) -> std::io::Result<()> {
    let log_path = log_dir.join(LOG_FILE_NAME);
    if log_path.exists() {
        if let Ok(meta) = fs::metadata(&log_path) {
            if meta.len() >= max_bytes {
                let log_1_path = log_dir.join(format!("{}.1", LOG_FILE_NAME));
                let log_2_path = log_dir.join(format!("{}.2", LOG_FILE_NAME));

                if log_2_path.exists() {
                    let _ = fs::remove_file(&log_2_path);
                }
                if log_1_path.exists() {
                    let _ = fs::rename(&log_1_path, &log_2_path);
                }
                let _ = fs::rename(&log_path, &log_1_path);
                File::create(&log_path)?;
            }
        }
    }
    Ok(())
}

pub fn get_recent_logs(max_lines: usize) -> Vec<String> {
    let log_dir = match LOGGER_DIR.lock() {
        Ok(lock) => match lock.as_ref() {
            Some(dir) => dir.clone(),
            None => return Vec::new(),
        },
        Err(_) => return Vec::new(),
    };

    // Read oldest-to-newest so a crash from just before the last rotation is
    // still visible in the diagnostic report.
    let candidates = [
        log_dir.join(format!("{LOG_FILE_NAME}.2")),
        log_dir.join(format!("{LOG_FILE_NAME}.1")),
        log_dir.join(LOG_FILE_NAME),
    ];
    let mut lines = Vec::new();
    for path in candidates {
        let Ok(file) = File::open(path) else {
            continue;
        };
        let reader = BufReader::new(file);
        lines.extend(reader.lines().map_while(Result::ok));
    }

    if lines.len() > max_lines {
        lines.split_off(lines.len() - max_lines)
    } else {
        lines
    }
}

fn format_timestamp(now: SystemTime) -> String {
    let duration = now
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();

    let days = (secs / 86400) as i64;
    let rem_secs = (secs % 86400) as u32;
    let hours = rem_secs / 3600;
    let mins = (rem_secs % 3600) / 60;
    let seconds = rem_secs % 60;

    let z = days + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };

    format!(
        "{:04}-{:02}-{:02} {:02}:{:02}:{:02}.{:03}Z",
        year, m, d, hours, mins, seconds, millis
    )
}

const DIAGNOSTIC_LOG_SAMPLE_LINES: usize = 1_200;
const RECENT_ISSUE_LIMIT: usize = 40;
const EDIT_TRACE_LIMIT: usize = 60;
const MAX_TROUBLESHOOTING_HINTS: usize = 10;

#[derive(Clone, Debug, PartialEq, Eq)]
enum FileState {
    Missing,
    Present(u64),
    NotRegularFile,
    Unreadable(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct FileCheck {
    label: String,
    expected_size: Option<u64>,
    state: FileState,
}

impl FileCheck {
    fn inspect(label: impl Into<String>, path: &Path, expected_size: Option<u64>) -> Self {
        let state = match fs::metadata(path) {
            Ok(metadata) if metadata.is_file() => FileState::Present(metadata.len()),
            Ok(_) => FileState::NotRegularFile,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => FileState::Missing,
            Err(error) => FileState::Unreadable(error.to_string()),
        };
        Self {
            label: label.into(),
            expected_size,
            state,
        }
    }

    fn is_present(&self) -> bool {
        matches!(self.state, FileState::Present(size) if size > 0)
    }

    fn is_expected(&self) -> bool {
        matches!(
            (&self.state, self.expected_size),
            (FileState::Present(size), Some(expected)) if *size == expected
        )
    }

    fn summary(&self) -> String {
        match &self.state {
            FileState::Missing => "Missing".to_string(),
            FileState::NotRegularFile => "Not a regular file".to_string(),
            FileState::Unreadable(error) => {
                format!("Unreadable ({})", sanitize_error_for_report(error))
            }
            FileState::Present(size) => match self.expected_size {
                Some(expected) if *size == expected => {
                    format!("Present ({})", human_bytes(*size))
                }
                Some(expected) => format!(
                    "Wrong size: {} (expected {})",
                    human_bytes(*size),
                    human_bytes(expected)
                ),
                None => format!("Present ({})", human_bytes(*size)),
            },
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct LogSignals {
    elevated_paste_blocked: bool,
    port_conflict: bool,
    cuda_failure: bool,
    panic: bool,
    integrity_failure: bool,
    audio_failure: bool,
}

#[derive(Debug)]
struct DiagnosticSnapshot {
    version: String,
    os: &'static str,
    arch: &'static str,
    pid: u32,
    elevated: Option<bool>,
    executable: Option<PathBuf>,
    app_data_dir: PathBuf,
    settings_path: Option<PathBuf>,
    settings_error: Option<String>,
    settings: Option<Settings>,
    speech_text_included: bool,
    whisper_status: SidecarRuntimeStatus,
    parakeet_status: SidecarRuntimeStatus,
    recording_active: Option<bool>,
    hook_paused: bool,
    gaming_mode_active: Option<bool>,
    detected_audio_devices: Vec<String>,
    parakeet_files: Vec<FileCheck>,
    punctuation_file: FileCheck,
    cuda_runtime_source: &'static str,
    cuda_files: Vec<FileCheck>,
    whisper_models: Vec<FileCheck>,
    selected_model_file: Option<FileCheck>,
    partial_downloads: Vec<FileCheck>,
    recent_issues: Vec<String>,
    logs: Vec<String>,
    edit_trace: Vec<String>,
    signals: LogSignals,
}

fn speech_text_may_be_reported(settings: Option<&Settings>) -> bool {
    // Fail closed: if settings could not be loaded, exported speech text must
    // stay redacted even if the on-disk setting was previously enabled.
    settings
        .map(|settings| settings.log_speech_text)
        .unwrap_or(false)
}

fn human_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    let value = bytes as f64;
    if value >= GB {
        format!("{:.2} GB", value / GB)
    } else if value >= MB {
        format!("{:.1} MB", value / MB)
    } else if value >= KB {
        format!("{:.1} KB", value / KB)
    } else {
        format!("{bytes} bytes")
    }
}

fn single_line_text(value: &str) -> String {
    let mut cleaned = String::new();
    let mut last_was_space = false;
    for character in value.chars() {
        let character = if character.is_control() {
            ' '
        } else {
            character
        };
        if character.is_whitespace() {
            if !last_was_space && !cleaned.is_empty() {
                cleaned.push(' ');
            }
            last_was_space = true;
        } else {
            cleaned.push(character);
            last_was_space = false;
        }
    }
    cleaned.trim().to_string()
}

fn sanitize_error_for_report(error: &str) -> String {
    const MAX_ERROR_CHARS: usize = 600;
    let cleaned = single_line_text(error);
    if cleaned.is_empty() {
        return "Unknown error".to_string();
    }
    if cleaned.chars().count() > MAX_ERROR_CHARS {
        let mut truncated: String = cleaned.chars().take(MAX_ERROR_CHARS).collect();
        truncated.push_str("...");
        truncated
    } else {
        cleaned
    }
}

fn analyze_log_signals(logs: &[String]) -> LogSignals {
    let mut signals = LogSignals::default();
    for line in logs {
        let line = line.to_ascii_lowercase();
        signals.elevated_paste_blocked |= line.contains("elevated-paste-blocked");
        signals.port_conflict |= line.contains("address already in use")
            || line.contains("os error 10048")
            || line.contains("only one usage of each socket address");
        signals.cuda_failure |= line.contains("cuda provider failed")
            || line.contains("failed to initialize cuda")
            || line.contains("cuda error");
        signals.panic |= line.contains("[panic]") || line.contains("panicked at");
        signals.integrity_failure |= line.contains("sha-256 integrity")
            || line.contains("integrity check failed")
            || line.contains("failed sha-256");
        signals.audio_failure |= (line.contains("microphone") || line.contains("audio device"))
            && (line.contains("failed") || line.contains("unavailable") || line.contains("error"));
    }
    signals
}

fn collect_partial_downloads(app_local_data: &Path) -> Vec<FileCheck> {
    let candidate_directories = [
        app_local_data.join("models"),
        app_local_data.join("models").join("parakeet-v3"),
        app_local_data.join("models").join("punctuation-archive"),
        app_local_data.join("binaries").join("cuda-archives"),
    ];
    let mut partials = Vec::new();
    for directory in candidate_directories {
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let Some(filename) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if !path.is_file() || !filename.ends_with(".part") {
                continue;
            }
            let label = path
                .strip_prefix(app_local_data)
                .unwrap_or(&path)
                .display()
                .to_string();
            partials.push(FileCheck::inspect(label, &path, None));
        }
    }
    partials.sort_by(|left, right| left.label.cmp(&right.label));
    partials
}

#[cfg(target_os = "windows")]
fn process_is_elevated() -> Option<bool> {
    use windows_sys::Win32::Foundation::{CloseHandle, HANDLE};
    use windows_sys::Win32::Security::{
        GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
    };
    use windows_sys::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    let mut token: HANDLE = 0;
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token) } == 0 {
        return None;
    }
    let mut elevation = TOKEN_ELEVATION { TokenIsElevated: 0 };
    let mut returned_size = 0u32;
    let result = unsafe {
        GetTokenInformation(
            token,
            TokenElevation,
            &mut elevation as *mut _ as *mut _,
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut returned_size,
        )
    };
    unsafe {
        CloseHandle(token);
    }
    if result == 0 {
        None
    } else {
        Some(elevation.TokenIsElevated != 0)
    }
}

#[cfg(not(target_os = "windows"))]
fn process_is_elevated() -> Option<bool> {
    None
}

fn format_sidecar_status(status: &SidecarRuntimeStatus) -> String {
    match status {
        SidecarRuntimeStatus::NotRunning => "Not running".to_string(),
        SidecarRuntimeStatus::Starting => "Starting".to_string(),
        SidecarRuntimeStatus::Ready {
            provider,
            port,
            pid,
        } => format!("Ready (provider: {provider}, port: {port}, pid: {pid})"),
        SidecarRuntimeStatus::Exited { code: Some(code) } => {
            format!("Exited (code: {code})")
        }
        SidecarRuntimeStatus::Exited { code: None } => "Exited (exit code unavailable)".to_string(),
        SidecarRuntimeStatus::InspectionError(error) => {
            format!("Inspection error: {}", sanitize_error_for_report(error))
        }
    }
}

fn settings_text(settings: Option<&Settings>, field: impl FnOnce(&Settings) -> String) -> String {
    settings
        .map(field)
        .unwrap_or_else(|| "Unavailable".to_string())
}

fn settings_bool(settings: Option<&Settings>, field: impl FnOnce(&Settings) -> bool) -> String {
    settings_bool_value(settings.map(field))
}

fn settings_bool_value(value: Option<bool>) -> String {
    match value {
        Some(true) => "true".to_string(),
        Some(false) => "false".to_string(),
        None => "Unavailable".to_string(),
    }
}

fn format_audio_input(settings: Option<&Settings>, detected: &[String]) -> String {
    let detected_count = detected.len();
    let Some(settings) = settings else {
        return format!("Unavailable ({detected_count} input device(s) detected)");
    };
    let configured = settings.audio_input_device.trim();
    if configured.is_empty() || configured == "default" {
        return format!("System default ({detected_count} input device(s) detected)");
    }
    let configured = single_line_text(configured);
    if detected.iter().any(|device| device == &configured) {
        format!("\"{configured}\" (detected; {detected_count} input device(s) total)")
    } else {
        format!("\"{configured}\" (NOT detected; {detected_count} input device(s) found)")
    }
}

fn append_file_checks(report: &mut String, files: &[FileCheck], indent: &str) {
    for file in files {
        let _ = writeln!(report, "{indent}- `{}`: {}", file.label, file.summary());
    }
}

fn troubleshooting_hints(snapshot: &DiagnosticSnapshot) -> Vec<String> {
    let mut hints = Vec::new();
    let mut push_hint = |hint: String| {
        if !hints.contains(&hint) {
            hints.push(hint);
        }
    };

    if snapshot.settings_error.is_some() {
        push_hint(
            "Settings could not be loaded. Back up the settings file, then repair or reset Aura settings; configured engine and cloud-key details in this report are unknown."
                .to_string(),
        );
    }

    let parakeet_installed = snapshot.parakeet_files.iter().all(FileCheck::is_expected);
    if let Some(settings) = snapshot.settings.as_ref() {
        if settings.transcription_mode == "cloud" && settings.active_api_key().trim().is_empty() {
            push_hint(
                "Cloud transcription is selected, but the active API key is empty. Add a key in Settings -> Cloud or switch to Local transcription."
                    .to_string(),
            );
        }
        if settings.transcription_mode == "local" {
            match settings.local_engine.as_str() {
                "parakeet" if !parakeet_installed => push_hint(
                    "Parakeet V3 is selected, but its model files are missing or have the wrong size. Re-download Parakeet V3 in Settings -> Voice."
                        .to_string(),
                ),
                "whisper" => {
                    if let Some(file) = snapshot.selected_model_file.as_ref() {
                        if !file.is_expected() {
                            push_hint(format!(
                                "The selected Whisper model '{}' is missing or incomplete. Re-download it in Settings -> Voice.",
                                file.label
                            ));
                        }
                    }
                    if !snapshot.punctuation_file.is_present() {
                        push_hint(
                            "The optional punctuation model is not installed. Re-download it in Settings -> Voice if Whisper output lacks punctuation."
                                .to_string(),
                        );
                    }
                }
                _ => {}
            }
            if settings.local_acceleration == "cuda"
                && snapshot.cuda_runtime_source == "Incomplete/Missing"
            {
                push_hint(
                    "CUDA acceleration is selected, but the Aura-managed CUDA runtime is incomplete. Reinstall the CUDA runtime in Settings -> Voice, or switch acceleration to CPU."
                        .to_string(),
                );
            }
        }
    }

    if !snapshot.partial_downloads.is_empty() {
        push_hint(
            "Unfinished download files are present. Retry the affected model or runtime download (Aura resumes from the .part file); delete a listed file only if retries keep failing."
                .to_string(),
        );
    }

    for (label, status) in [
        ("Whisper", &snapshot.whisper_status),
        ("Parakeet", &snapshot.parakeet_status),
    ] {
        match status {
            SidecarRuntimeStatus::Exited { code } => push_hint(format!(
                "The {label} sidecar exited{}. Review the recent Sidecar/CUDA errors above, then start a new dictation or restart Aura.",
                code.map(|code| format!(" (code {code})"))
                    .unwrap_or_default()
            )),
            SidecarRuntimeStatus::InspectionError(_) => push_hint(format!(
                "Aura could not inspect the {label} sidecar process. Restart Aura and generate a fresh report."
            )),
            _ => {}
        }
    }

    if snapshot.detected_audio_devices.is_empty() {
        push_hint(
            "No audio input devices were detected. Connect a microphone, allow microphone access in Windows Settings -> Privacy, then restart Aura."
                .to_string(),
        );
    } else if let Some(settings) = snapshot.settings.as_ref() {
        let configured = settings.audio_input_device.trim();
        if !configured.is_empty()
            && configured != "default"
            && !snapshot
                .detected_audio_devices
                .iter()
                .any(|device| device == configured)
        {
            push_hint(format!(
                "The configured microphone '{}' was not detected. Re-select the input device in Settings -> Voice.",
                single_line_text(configured)
            ));
        }
    }

    if snapshot.hook_paused {
        push_hint(
            "The global hotkey hook is paused (for example by Gaming Mode or standby). Toggle Gaming Mode off or restart Aura before testing the hotkey."
                .to_string(),
        );
    }
    if snapshot.signals.elevated_paste_blocked {
        push_hint(
            "A target application runs elevated, so Windows blocked simulated paste. Run Aura as administrator or paste with Ctrl+V from the clipboard."
                .to_string(),
        );
    }
    if snapshot.signals.port_conflict {
        push_hint(
            "A local sidecar could not bind its port because the address is already in use. Restart Aura; if it persists, close other local speech servers or reboot."
                .to_string(),
        );
    }
    if snapshot.signals.cuda_failure {
        push_hint(
            "CUDA initialization failed in the logs. Reinstall the CUDA runtime or switch acceleration to CPU, then retry."
                .to_string(),
        );
    }
    if snapshot.signals.integrity_failure {
        push_hint(
            "A model failed integrity verification. Re-download the affected model in Settings -> Voice; the verified archive is not reused after a hash mismatch."
                .to_string(),
        );
    }
    if snapshot.signals.audio_failure {
        push_hint(
            "Audio capture reported an error. Re-select the input device in Settings, check Windows microphone permissions, and test the microphone in Settings."
                .to_string(),
        );
    }
    if snapshot.signals.panic {
        push_hint(
            "Aura logged a panic/crash. Restart Aura and attach this report to a bug report; the panic line is included under recent warnings and errors."
                .to_string(),
        );
    }

    if hints.is_empty() {
        hints.push(
            "No obvious configuration problem was detected. If the issue persists, include the recent warnings and edit trace above in a bug report."
                .to_string(),
        );
    }
    hints.truncate(MAX_TROUBLESHOOTING_HINTS);
    hints
}

pub fn generate_diagnostic_report<R: tauri::Runtime>(
    app_handle: &tauri::AppHandle<R>,
) -> Result<String, String> {
    let version = app_handle.package_info().version.to_string();
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    let settings_result = crate::settings::load_settings(app_handle);
    let settings_error = settings_result
        .as_ref()
        .err()
        .map(|error| sanitize_error_for_report(error));
    let settings = settings_result.ok();
    let speech_text_included = speech_text_may_be_reported(settings.as_ref());

    let app_local_data = app_handle
        .path()
        .app_local_data_dir()
        .map_err(|e| format!("Failed to get app local data dir: {}", e))?;
    let models_dir = app_local_data.join("models");
    let binaries_dir = app_local_data.join("binaries");

    let parakeet_dir = models_dir.join("parakeet-v3");
    let parakeet_files = crate::whisper_runner::parakeet_artifact_sizes()
        .into_iter()
        .map(|(filename, expected_size)| {
            FileCheck::inspect(filename, &parakeet_dir.join(filename), Some(expected_size))
        })
        .collect();

    let punc_dir = models_dir.join("punctuation");
    let punctuation_file =
        FileCheck::inspect("model.int8.onnx", &punc_dir.join("model.int8.onnx"), None);

    let cuda_bin_dir = binaries_dir.join("cuda").join("bin");
    let cuda_files = [
        "onnxruntime_providers_cuda.dll",
        "onnxruntime.dll",
        "ggml-cuda.dll",
        "sherpa-onnx-offline-websocket-server.exe",
    ]
    .into_iter()
    .map(|filename| FileCheck::inspect(filename, &cuda_bin_dir.join(filename), None))
    .collect();
    let cuda_runtime_source = crate::cuda_runtime_source_label(&cuda_bin_dir);

    let mut whisper_models = Vec::new();
    if let Ok(entries) = fs::read_dir(&models_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }
            let Some(filename) = path.file_name().and_then(|name| name.to_str()) else {
                continue;
            };
            if !filename.starts_with("ggml-") || !filename.ends_with(".bin") {
                continue;
            }
            let name = &filename[5..filename.len() - 4];
            whisper_models.push(FileCheck::inspect(
                name,
                &path,
                crate::whisper_runner::whisper_model_expected_size(name),
            ));
        }
    }
    whisper_models.sort_by(|left, right| left.label.cmp(&right.label));

    let selected_model_file = settings.as_ref().and_then(|settings| {
        if settings.transcription_mode != "local" || settings.local_engine != "whisper" {
            return None;
        }
        let filename = crate::whisper_runner::whisper_model_filename(&settings.model_name).ok()?;
        Some(FileCheck::inspect(
            &settings.model_name,
            &models_dir.join(filename),
            crate::whisper_runner::whisper_model_expected_size(&settings.model_name),
        ))
    });

    let partial_downloads = collect_partial_downloads(&app_local_data);

    let runtime_state = app_handle.try_state::<crate::AppState>();
    let recording_active = runtime_state
        .as_ref()
        .map(|state| state.is_recording.load(Ordering::SeqCst));
    let gaming_mode_active = runtime_state
        .as_ref()
        .map(|state| state.gaming_mode_active.load(Ordering::Acquire));
    let hook_paused = crate::keyboard_hook::is_hook_paused();
    let detected_audio_devices = crate::audio_recorder::list_audio_input_devices();

    let whisper_status = crate::whisper_runner::whisper_runtime_status(app_handle);
    let parakeet_status = crate::whisper_runner::parakeet_runtime_status(app_handle);

    let raw_logs = get_recent_logs(DIAGNOSTIC_LOG_SAMPLE_LINES);
    let signals = analyze_log_signals(&raw_logs);

    let mut logs: Vec<String> = raw_logs
        .iter()
        .skip(raw_logs.len().saturating_sub(50))
        .cloned()
        .collect();
    logs = sanitize_logs_for_report(&logs, speech_text_included);

    let mut recent_issues: Vec<String> = raw_logs
        .iter()
        .filter(|line| {
            line.contains("[WARN]")
                || line.contains("[ERROR]")
                || line.contains("[Panic]")
                || line.contains("panicked at")
        })
        .cloned()
        .collect();
    if recent_issues.len() > RECENT_ISSUE_LIMIT {
        recent_issues = recent_issues.split_off(recent_issues.len() - RECENT_ISSUE_LIMIT);
    }
    let recent_issues = sanitize_logs_for_report(&recent_issues, speech_text_included);

    // The unified tail is dominated by sidecar noise, which pushes edit
    // activity out of the window. Keep only the lines that describe a command
    // on selected text: what was captured, what route it took, how long each
    // step took and why it failed.
    let mut edit_trace: Vec<String> = raw_logs
        .iter()
        .filter(|line| {
            line.contains("[LLM]") || line.contains("[Context]") || line.contains("Final result: ")
        })
        .cloned()
        .collect();
    if edit_trace.len() > EDIT_TRACE_LIMIT {
        edit_trace = edit_trace.split_off(edit_trace.len() - EDIT_TRACE_LIMIT);
    }
    let edit_trace = sanitize_logs_for_report(&edit_trace, speech_text_included);

    let settings_path = crate::settings::get_settings_path(app_handle).ok();

    let snapshot = DiagnosticSnapshot {
        version,
        os,
        arch,
        pid: std::process::id(),
        elevated: process_is_elevated(),
        executable: std::env::current_exe().ok(),
        app_data_dir: app_local_data,
        settings_path,
        settings_error,
        settings,
        speech_text_included,
        whisper_status,
        parakeet_status,
        recording_active,
        hook_paused,
        gaming_mode_active,
        detected_audio_devices,
        parakeet_files,
        punctuation_file,
        cuda_runtime_source,
        cuda_files,
        whisper_models,
        selected_model_file,
        partial_downloads,
        recent_issues,
        logs,
        edit_trace,
        signals,
    };

    let report = format_diagnostic_report(&snapshot);
    Ok(match user_profile_dir() {
        Some(profile) => redact_profile_paths(&report, &profile),
        None => report,
    })
}

fn user_profile_dir() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        std::env::var("USERPROFILE").ok()
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var("HOME").ok()
    }
}

/// Replaces the real home-directory prefix with a portable placeholder so a
/// diagnostic report can be shared publicly without leaking the account name.
fn redact_profile_paths(report: &str, profile_dir: &str) -> String {
    let profile = profile_dir.trim().trim_end_matches(['\\', '/']);
    if profile.is_empty() || profile.ends_with(':') {
        // An empty value or a bare drive root (C:\) cannot identify a user.
        return report.to_string();
    }

    let mut redacted = report.replace(profile, "%USERPROFILE%");
    let forward_slash_profile = profile.replace('\\', "/");
    if forward_slash_profile != profile {
        redacted = redacted.replace(&forward_slash_profile, "%USERPROFILE%");
    }
    redacted
}

pub fn sanitize_logs_for_report(logs: &[String], log_speech_text: bool) -> Vec<String> {
    if log_speech_text {
        return logs.to_vec();
    }

    logs.iter()
        .map(|line| {
            if let Some(pos) = line.find("Final result: ") {
                let prefix = &line[..pos + "Final result: ".len()];
                let speech = &line[pos + "Final result: ".len()..];
                if !speech.starts_with("[REDACTED:") {
                    let redacted = anonymize_speech(speech, false);
                    format!("{}{}", prefix, redacted)
                } else {
                    line.clone()
                }
            } else {
                line.clone()
            }
        })
        .collect()
}

fn format_diagnostic_report(snapshot: &DiagnosticSnapshot) -> String {
    let settings = snapshot.settings.as_ref();
    let mut report = String::new();

    let _ = writeln!(report, "# Aura Diagnostic Report\n");

    let _ = writeln!(report, "## System & Environment");
    let _ = writeln!(report, "- **App Version**: {}", snapshot.version);
    let _ = writeln!(report, "- **OS**: {}", snapshot.os);
    let _ = writeln!(report, "- **Architecture**: {}", snapshot.arch);
    let _ = writeln!(report, "- **Process ID**: {}", snapshot.pid);
    let _ = writeln!(
        report,
        "- **Elevated**: {}",
        match snapshot.elevated {
            Some(true) => "Yes (administrator)",
            Some(false) => "No",
            None => "Unavailable",
        }
    );
    let _ = writeln!(
        report,
        "- **Executable**: {}",
        snapshot
            .executable
            .as_ref()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|| "Unavailable".to_string())
    );
    let _ = writeln!(
        report,
        "- **App Data**: {}",
        snapshot.app_data_dir.display()
    );
    let _ = writeln!(
        report,
        "- **Report Redaction**: {}",
        if snapshot.speech_text_included {
            "Speech text included (log_speech_text is enabled)"
        } else {
            "Speech text redacted (log_speech_text disabled or settings unavailable)"
        }
    );

    let _ = writeln!(report, "\n## Settings Load");
    if let Some(error) = snapshot.settings_error.as_deref() {
        let _ = writeln!(report, "- **Status**: FAILED");
        let _ = writeln!(report, "- **Error**: {}", sanitize_error_for_report(error));
    } else {
        let _ = writeln!(report, "- **Status**: OK");
        let _ = writeln!(report, "- **Error**: None");
    }
    let settings_file = match snapshot.settings_path.as_ref() {
        Some(path) => format!(
            "{} ({})",
            path.display(),
            if path.exists() { "exists" } else { "missing" }
        ),
        None => "Unavailable".to_string(),
    };
    let _ = writeln!(report, "- **Settings File**: {settings_file}");

    let _ = writeln!(report, "\n## Configured State");
    let _ = writeln!(
        report,
        "- **Transcription Mode**: {}",
        settings_text(settings, |settings| settings.transcription_mode.clone())
    );
    let _ = writeln!(
        report,
        "- **Local Engine**: {}",
        settings_text(settings, |settings| settings.local_engine.clone())
    );
    let _ = writeln!(
        report,
        "- **Selected Model**: {}",
        settings_text(settings, |settings| settings.model_name.clone())
    );
    let _ = writeln!(
        report,
        "- **Acceleration**: {}",
        settings_text(settings, |settings| settings.local_acceleration.clone())
    );
    let _ = writeln!(
        report,
        "- **Real-time Streaming**: {}",
        settings_bool(settings, |settings| settings.streaming_enabled)
    );
    let _ = writeln!(
        report,
        "- **Cloud Fallback**: {}",
        settings_bool(settings, |settings| settings.cloud_fallback_enabled)
    );
    let _ = writeln!(
        report,
        "- **AI Edit Mode**: {}",
        settings_text(settings, |settings| settings.ai_edit_mode.clone())
    );
    let _ = writeln!(
        report,
        "- **Local Edit Model**: {}",
        settings_text(settings, |settings| settings.llm_model.clone())
    );
    let _ = writeln!(
        report,
        "- **Cloud Provider**: {}",
        settings_text(settings, |settings| settings.api_provider.clone())
    );
    let _ = writeln!(
        report,
        "- **Active API Key**: {}",
        settings_text(settings, |settings| {
            if settings.active_api_key().trim().is_empty() {
                "Not configured".to_string()
            } else {
                "Configured".to_string()
            }
        })
    );
    let _ = writeln!(
        report,
        "- **Hotkey**: {}",
        settings_text(settings, |settings| format!(
            "\"{}\"",
            single_line_text(&settings.hotkey)
        ))
    );
    let _ = writeln!(
        report,
        "- **Toggle Mode**: {}",
        settings_bool(settings, |settings| settings.toggle_enabled)
    );
    let _ = writeln!(
        report,
        "- **Log Speech Text**: {}",
        settings_bool(settings, |settings| settings.log_speech_text)
    );
    let _ = writeln!(
        report,
        "- **Gaming Mode Enabled**: {}",
        settings_bool(settings, |settings| settings.gaming_mode_enabled)
    );

    let _ = writeln!(report, "\n## Runtime Engines");
    let _ = writeln!(
        report,
        "- **Whisper Sidecar**: {}",
        format_sidecar_status(&snapshot.whisper_status)
    );
    let _ = writeln!(
        report,
        "- **Parakeet Sidecar**: {}",
        format_sidecar_status(&snapshot.parakeet_status)
    );
    let _ = writeln!(
        report,
        "- **Recording Active**: {}",
        settings_bool_value(snapshot.recording_active)
    );
    let _ = writeln!(report, "- **Hotkey Hook Paused**: {}", snapshot.hook_paused);
    let _ = writeln!(
        report,
        "- **Gaming Mode Active**: {}",
        settings_bool_value(snapshot.gaming_mode_active)
    );
    let _ = writeln!(
        report,
        "- **Audio Input**: {}",
        format_audio_input(settings, &snapshot.detected_audio_devices)
    );

    let _ = writeln!(
        report,
        "\n## Component Presence & Size (not integrity-verified)"
    );
    let _ = writeln!(
        report,
        "Checks below only inspect file existence and size. SHA-256 is verified during download or model load, not by this report."
    );

    let parakeet_installed = snapshot.parakeet_files.iter().all(FileCheck::is_expected);
    let parakeet_present = snapshot.parakeet_files.iter().all(FileCheck::is_present);
    let _ = writeln!(
        report,
        "\n- **Parakeet V3 Model**: {}",
        if parakeet_installed {
            "Installed (all expected files match size)"
        } else if parakeet_present {
            "Present, but at least one file has an unexpected size"
        } else {
            "Incomplete or missing"
        }
    );
    append_file_checks(&mut report, &snapshot.parakeet_files, "  ");

    let _ = writeln!(
        report,
        "- **Punctuation Model**: {}",
        if snapshot.punctuation_file.is_present() {
            "Installed"
        } else {
            "Missing or incomplete"
        }
    );
    let _ = writeln!(
        report,
        "  - `model.int8.onnx`: {}",
        snapshot.punctuation_file.summary()
    );

    let _ = writeln!(
        report,
        "- **CUDA Runtime Source**: {}",
        snapshot.cuda_runtime_source
    );
    let _ = writeln!(report, "- **CUDA Runtime Files**:");
    append_file_checks(&mut report, &snapshot.cuda_files, "  ");

    let _ = writeln!(
        report,
        "- **Whisper GGML Models**: {}",
        if snapshot.whisper_models.is_empty() {
            "None installed".to_string()
        } else {
            format!("{} model(s) found", snapshot.whisper_models.len())
        }
    );
    append_file_checks(&mut report, &snapshot.whisper_models, "  ");

    let selected_model = match settings {
        None => "Unavailable".to_string(),
        Some(settings) if settings.transcription_mode == "cloud" => {
            "Not required (cloud transcription)".to_string()
        }
        Some(settings) if settings.local_engine == "parakeet" => {
            "Not applicable (Parakeet V3 selected)".to_string()
        }
        Some(_) => snapshot
            .selected_model_file
            .as_ref()
            .map(|file| format!("`{}`: {}", file.label, file.summary()))
            .unwrap_or_else(|| "Unavailable".to_string()),
    };
    let _ = writeln!(report, "- **Selected Local Model File**: {selected_model}");

    let _ = writeln!(report, "\n## Interrupted Downloads");
    if snapshot.partial_downloads.is_empty() {
        let _ = writeln!(report, "- None");
    } else {
        append_file_checks(&mut report, &snapshot.partial_downloads, "");
    }

    let recent_issues = if snapshot.recent_issues.is_empty() {
        "(No WARN/ERROR/Panic lines in the sampled log)".to_string()
    } else {
        snapshot.recent_issues.join("\n")
    };
    let _ = writeln!(
        report,
        "\n## Recent Warnings, Errors & Panics (last {RECENT_ISSUE_LIMIT} from up to {DIAGNOSTIC_LOG_SAMPLE_LINES} log lines)"
    );
    let _ = writeln!(report, "```\n{recent_issues}\n```");

    let hints = troubleshooting_hints(snapshot);
    let _ = writeln!(report, "\n## Troubleshooting Hints");
    for (index, hint) in hints.iter().enumerate() {
        let _ = writeln!(report, "{}. {}", index + 1, hint);
    }

    let trace_content = if snapshot.edit_trace.is_empty() {
        "(No edit commands recorded yet. Run a command over selected text and copy the report again.)"
            .to_string()
    } else {
        snapshot.edit_trace.join("\n")
    };
    let _ = writeln!(
        report,
        "\n## Edit Command Trace (last {EDIT_TRACE_LIMIT} relevant lines)"
    );
    let _ = writeln!(report, "```\n{trace_content}\n```");

    let log_content = if snapshot.logs.is_empty() {
        "(No logs recorded)".to_string()
    } else {
        snapshot.logs.join("\n")
    };
    let _ = writeln!(report, "\n## Unified Log (Last 50 Lines)");
    let _ = writeln!(report, "```\n{log_content}\n```");

    report
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn early_log_identifier_matches_tauri_config() {
        let config = include_str!("../tauri.conf.json");
        let expected = format!("\"identifier\": \"{}\"", EARLY_LOG_IDENTIFIER);
        assert!(
            config.contains(&expected),
            "EARLY_LOG_IDENTIFIER must stay in sync with tauri.conf.json ({expected})"
        );
    }

    #[test]
    fn test_anonymize_speech() {
        assert_eq!(
            anonymize_speech("hello world from aura", true),
            "hello world from aura"
        );
        assert_eq!(
            anonymize_speech("hello world from aura", false),
            "[REDACTED: 4 words, 21 chars]"
        );
        assert_eq!(anonymize_speech("", false), "[REDACTED: 0 words, 0 chars]");
    }

    #[test]
    fn test_format_session_tag() {
        assert_eq!(format_session_tag(123456), "#session-123456");
        assert_eq!(format_session_tag(0), "#session-0");
    }

    #[test]
    fn test_log_rotation() {
        let unique_id = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let temp_dir = std::env::temp_dir().join(format!("aura_log_test_{}", unique_id));
        fs::create_dir_all(&temp_dir).unwrap();

        let log_file = temp_dir.join(LOG_FILE_NAME);
        fs::write(&log_file, "A".repeat(100)).unwrap();

        // 1st rotation: log file size 100 >= threshold 50 -> rotated to log.1
        rotate_logs_if_needed(&temp_dir, 50).unwrap();
        assert!(temp_dir.join(LOG_FILE_NAME).exists());
        assert!(temp_dir.join(format!("{}.1", LOG_FILE_NAME)).exists());
        assert_eq!(
            fs::read_to_string(temp_dir.join(format!("{}.1", LOG_FILE_NAME))).unwrap(),
            "A".repeat(100)
        );

        // Fill current log file again
        fs::write(&log_file, "B".repeat(100)).unwrap();

        // 2nd rotation: log file size 100 >= threshold 50 -> log.1 becomes log.2, current becomes log.1
        rotate_logs_if_needed(&temp_dir, 50).unwrap();
        assert!(temp_dir.join(LOG_FILE_NAME).exists());
        assert!(temp_dir.join(format!("{}.1", LOG_FILE_NAME)).exists());
        assert!(temp_dir.join(format!("{}.2", LOG_FILE_NAME)).exists());
        assert_eq!(
            fs::read_to_string(temp_dir.join(format!("{}.1", LOG_FILE_NAME))).unwrap(),
            "B".repeat(100)
        );
        assert_eq!(
            fs::read_to_string(temp_dir.join(format!("{}.2", LOG_FILE_NAME))).unwrap(),
            "A".repeat(100)
        );

        let _ = fs::remove_dir_all(&temp_dir);
    }

    fn present_check(label: &str, size: u64, expected: Option<u64>) -> FileCheck {
        FileCheck {
            label: label.to_string(),
            expected_size: expected,
            state: FileState::Present(size),
        }
    }

    fn test_snapshot() -> DiagnosticSnapshot {
        let mut settings = Settings {
            ui_language: "en".to_string(),
            transcription_mode: "local".to_string(),
            local_engine: "parakeet".to_string(),
            model_name: "parakeet-v3".to_string(),
            local_acceleration: "cuda".to_string(),
            streaming_enabled: true,
            cloud_fallback_enabled: true,
            hotkey: "Alt+V".to_string(),
            audio_input_device: "default".to_string(),
            ai_edit_mode: "auto".to_string(),
            llm_model: "qwen3-1.7b".to_string(),
            api_provider: "gemini".to_string(),
            api_key_gemini: "very-secret-key".to_string(),
            ..Settings::default()
        };
        settings.sync_active_api_key();

        let parakeet_files = crate::whisper_runner::parakeet_artifact_sizes()
            .into_iter()
            .map(|(filename, size)| present_check(filename, size, Some(size)))
            .collect();

        DiagnosticSnapshot {
            version: "1.2.0".to_string(),
            os: "windows",
            arch: "x86_64",
            pid: 4242,
            elevated: Some(false),
            executable: Some(PathBuf::from("C:\\Program Files\\Aura\\aura.exe")),
            app_data_dir: PathBuf::from("C:\\Users\\Test\\AppData\\Local\\com.aura.app"),
            settings_path: Some(PathBuf::from("C:\\Users\\Test\\settings.json")),
            settings_error: None,
            settings: Some(settings),
            speech_text_included: false,
            whisper_status: SidecarRuntimeStatus::Ready {
                provider: "cpu".to_string(),
                port: 3033,
                pid: 4243,
            },
            parakeet_status: SidecarRuntimeStatus::NotRunning,
            recording_active: Some(false),
            hook_paused: false,
            gaming_mode_active: Some(false),
            detected_audio_devices: vec!["Microphone (USB)".to_string()],
            parakeet_files,
            punctuation_file: present_check("model.int8.onnx", 123_456, None),
            cuda_runtime_source: "System NVIDIA runtime (PATH)",
            cuda_files: vec![
                present_check("onnxruntime_providers_cuda.dll", 1_000, None),
                present_check("sherpa-onnx-offline-websocket-server.exe", 2_000, None),
            ],
            whisper_models: vec![present_check("base", 147_951_465, Some(147_951_465))],
            selected_model_file: None,
            partial_downloads: Vec::new(),
            recent_issues: vec![
                "2026-07-20 00:00:01.000Z [WARN] [Audio] Test log line 2".to_string(),
            ],
            logs: vec![
                "2026-07-20 00:00:00.000Z [INFO] [System] Test log line 1".to_string(),
                "2026-07-20 00:00:01.000Z [WARN] [Audio] Test log line 2".to_string(),
            ],
            edit_trace: vec![
                "2026-07-20 00:00:02.000Z [INFO] [LLM] Edit request: model=qwen3-1.7b, language=ru, spoken=8 chars, selection=120 chars".to_string(),
                "2026-07-20 00:00:03.000Z [INFO] [LLM] Local model replied in 900 ms (128 -> 74 chars, model=qwen3-1.7b)".to_string(),
            ],
            signals: LogSignals::default(),
        }
    }

    #[test]
    fn test_format_diagnostic_report() {
        let report = format_diagnostic_report(&test_snapshot());

        assert!(report.contains("# Aura Diagnostic Report"));
        assert!(report.contains("## System & Environment"));
        assert!(report.contains("## Settings Load"));
        assert!(report.contains("## Configured State"));
        assert!(report.contains("## Runtime Engines"));
        assert!(report.contains("## Component Presence & Size (not integrity-verified)"));
        assert!(report.contains("## Interrupted Downloads"));
        assert!(report.contains("## Troubleshooting Hints"));
        assert!(report.contains("## Edit Command Trace (last 60 relevant lines)"));
        assert!(report.contains("## Unified Log (Last 50 Lines)"));
        assert!(report.contains("Edit request: model=qwen3-1.7b"));
        assert!(report.contains("- **App Version**: 1.2.0"));
        assert!(report.contains("- **Status**: OK"));
        assert!(report.contains("- **Transcription Mode**: local"));
        assert!(
            report.contains("- **Whisper Sidecar**: Ready (provider: cpu, port: 3033, pid: 4243)")
        );
        assert!(report.contains("- **Parakeet Sidecar**: Not running"));
        assert!(report.contains("- **Parakeet V3 Model**: Installed"));
        assert!(report
            .contains("- **Selected Local Model File**: Not applicable (Parakeet V3 selected)"));
        assert!(report.contains("- **CUDA Runtime Source**: System NVIDIA runtime (PATH)"));
        assert!(report.contains("- **Real-time Streaming**: true"));
        assert!(report.contains("Test log line 1"));
        assert!(report.contains("No obvious configuration problem was detected"));
        assert!(!report.contains("very-secret-key"));
    }

    #[test]
    fn test_redact_profile_paths_replaces_windows_and_forward_slash_forms() {
        let report = concat!(
            "exe=C:\\Users\\Ivan\\AppData\\Local\\Aura\\aura-app.exe\n",
            "settings=C:/Users/Ivan/AppData/Roaming/aura/settings.enc\n",
            "other=C:\\Users\\SomeoneElse\\file.txt"
        );

        let redacted = redact_profile_paths(report, r"C:\Users\Ivan\");

        assert!(redacted.contains(r"exe=%USERPROFILE%\AppData\Local\Aura\aura-app.exe"));
        assert!(redacted.contains("settings=%USERPROFILE%/AppData/Roaming/aura/settings.enc"));
        assert!(!redacted.contains(r"C:\Users\Ivan"));
        assert!(!redacted.contains("C:/Users/Ivan"));
        // Only the current profile is replaced; unrelated account names stay intact.
        assert!(redacted.contains(r"C:\Users\SomeoneElse\file.txt"));
    }

    #[test]
    fn test_redact_profile_paths_handles_empty_prefix() {
        let report = "nothing to redact";
        assert_eq!(redact_profile_paths(report, ""), report);
        assert_eq!(redact_profile_paths(report, r"C:\"), report);
    }

    #[test]
    fn test_format_diagnostic_report_with_failed_settings_load() {
        let mut snapshot = test_snapshot();
        snapshot.settings = None;
        snapshot.settings_error =
            Some("could not parse\n\u{7}settings for this profile".to_string());
        snapshot.speech_text_included = false;
        snapshot.selected_model_file = None;

        let report = format_diagnostic_report(&snapshot);

        assert!(report.contains("- **Status**: FAILED"));
        assert!(report.contains("- **Error**: could not parse settings for this profile"));
        assert!(!report.contains('\u{7}'));
        assert!(!report.contains("could not parse\n"));
        assert!(report.contains("- **Transcription Mode**: Unavailable"));
        assert!(report.contains("- **Selected Model**: Unavailable"));
        assert!(report.contains("- **Selected Local Model File**: Unavailable"));
        assert!(report.contains("Speech text redacted"));
        assert!(report.contains("Settings could not be loaded."));
    }

    #[test]
    fn test_sidecar_status_summary_distinguishes_runtime_states() {
        assert_eq!(
            format_sidecar_status(&SidecarRuntimeStatus::NotRunning),
            "Not running"
        );
        assert_eq!(
            format_sidecar_status(&SidecarRuntimeStatus::Starting),
            "Starting"
        );
        assert_eq!(
            format_sidecar_status(&SidecarRuntimeStatus::Ready {
                provider: "cuda".to_string(),
                port: 1234,
                pid: 99,
            }),
            "Ready (provider: cuda, port: 1234, pid: 99)"
        );
        assert_eq!(
            format_sidecar_status(&SidecarRuntimeStatus::Exited { code: Some(3) }),
            "Exited (code: 3)"
        );
        assert_eq!(
            format_sidecar_status(&SidecarRuntimeStatus::Exited { code: None }),
            "Exited (exit code unavailable)"
        );
        assert_eq!(
            format_sidecar_status(&SidecarRuntimeStatus::InspectionError(
                "try_wait\nfailed".to_string()
            )),
            "Inspection error: try_wait failed"
        );
    }

    #[test]
    fn test_error_sanitization_removes_control_characters() {
        assert_eq!(
            sanitize_error_for_report("line one\nline two\t\u{7}done"),
            "line one line two done"
        );
        assert_eq!(sanitize_error_for_report(" \n\t "), "Unknown error");
    }

    #[test]
    fn test_speech_redaction_fails_closed_without_settings() {
        assert!(!speech_text_may_be_reported(None));
        let settings = Settings::default();
        assert!(!speech_text_may_be_reported(Some(&settings)));
        let settings = Settings {
            log_speech_text: true,
            ..Settings::default()
        };
        assert!(speech_text_may_be_reported(Some(&settings)));
    }

    #[test]
    fn test_log_signals_detect_actionable_failures() {
        let logs = vec![
            "2026-07-20 00:00:00.000Z [WARN] [Input] elevated-paste-blocked".to_string(),
            "2026-07-20 00:00:01.000Z [ERROR] [Sidecar] bind: address already in use".to_string(),
            "2026-07-20 00:00:02.000Z [ERROR] [Sidecar] CUDA provider failed to initialize"
                .to_string(),
            "2026-07-20 00:00:03.000Z [ERROR] [Panic] panicked at src/main.rs".to_string(),
            "2026-07-20 00:00:04.000Z [ERROR] [Model] SHA-256 integrity failed".to_string(),
            "2026-07-20 00:00:05.000Z [ERROR] [Audio] microphone device failed".to_string(),
        ];
        let signals = analyze_log_signals(&logs);
        assert!(signals.elevated_paste_blocked);
        assert!(signals.port_conflict);
        assert!(signals.cuda_failure);
        assert!(signals.panic);
        assert!(signals.integrity_failure);
        assert!(signals.audio_failure);
    }

    #[test]
    fn test_sanitize_logs_for_report() {
        let logs = vec![
            "2026-07-20 00:58:11.597Z [INFO] [ASR] [#session-1] Final result: Hello world test speech".to_string(),
            "2026-07-20 00:58:35.784Z [INFO] [ASR] [#session-2] Final result: [REDACTED: 3 words, 12 chars]".to_string(),
        ];

        let sanitized_off = sanitize_logs_for_report(&logs, false);
        assert_eq!(sanitized_off[0], "2026-07-20 00:58:11.597Z [INFO] [ASR] [#session-1] Final result: [REDACTED: 4 words, 23 chars]");
        assert_eq!(sanitized_off[1], "2026-07-20 00:58:35.784Z [INFO] [ASR] [#session-2] Final result: [REDACTED: 3 words, 12 chars]");

        let sanitized_on = sanitize_logs_for_report(&logs, true);
        assert_eq!(sanitized_on[0], logs[0]);
        assert_eq!(sanitized_on[1], logs[1]);
    }
}
