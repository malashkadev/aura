use std::env;
use std::fs;
use std::path::PathBuf;

const APP_COMMANDS: &[&str] = &[
    "get_settings",
    "set_provider_key",
    "set_settings",
    "download_model_command",
    "cancel_model_download",
    "delete_model_command",
    "get_downloaded_models",
    "get_history",
    "clear_history",
    "copy_to_clipboard",
    "check_for_app_update",
    "install_app_update",
    "open_url",
    "relaunch_app",
    "minimize_window",
    "close_window",
    "start_dragging_command",
    "hide_overlay_window",
    "download_gpu_binaries",
    "delete_gpu_binaries",
    "cancel_gpu_download",
    "check_gpu_downloaded",
    "get_diagnostic_report",
    "get_engine_health",
    "log_frontend_event",
    "start_mic_meter",
    "stop_mic_meter",
    "reprocess_history_text",
    "get_audio_input_devices",
    "toggle_gaming_mode",
    "get_gaming_mode_state",
    "show_settings_window",
    "exit_app",
];

fn is_cuda_dll(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    lower.starts_with("cublas")
        || lower.starts_with("cuda")
        || lower.starts_with("cuinj")
        || lower.starts_with("nvrtc")
        || lower == "ggml-cuda.dll"
}

fn copy_runtime_dlls() {
    let Some(out_dir) = env::var_os("OUT_DIR") else {
        return;
    };
    let mut profile_dir = PathBuf::from(out_dir);
    while profile_dir.file_name().and_then(|name| name.to_str()) != Some("build") {
        if !profile_dir.pop() {
            return;
        }
    }
    if !profile_dir.pop() {
        return;
    }

    let Some(manifest_dir) = env::var_os("CARGO_MANIFEST_DIR") else {
        return;
    };
    let binaries_dir = PathBuf::from(manifest_dir).join("binaries");
    let Ok(entries) = fs::read_dir(binaries_dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("dll") {
            continue;
        }
        let Some(file_name) = path.file_name() else {
            continue;
        };
        if let Some(name_str) = file_name.to_str() {
            if is_cuda_dll(name_str) {
                continue;
            }
        }
        if let Err(error) = fs::copy(&path, profile_dir.join(file_name)) {
            println!(
                "cargo:warning=Could not copy runtime DLL {}: {error}",
                path.display()
            );
        }
    }
}

fn main() {
    let app_manifest = tauri_build::AppManifest::new().commands(APP_COMMANDS);
    let attributes = tauri_build::Attributes::new().app_manifest(app_manifest);
    if let Err(error) = tauri_build::try_build(attributes) {
        panic!("failed to run Tauri build script: {error}");
    }
    copy_runtime_dlls();
}
