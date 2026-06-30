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

/// Locate the whisper sidecar executable under the resources directory or fallback paths
pub fn find_sidecar<R: Runtime>(app_handle: &tauri::AppHandle<R>) -> Result<PathBuf, String> {
    let target_name = "whisper-sidecar-x86_64-pc-windows-msvc.exe";

    let resource_dir = app_handle
        .path()
        .resource_dir()
        .map_err(|e| format!("Failed to get resource dir: {}", e))?;

    // Try direct paths first
    let direct_paths = [
        resource_dir.join("binaries").join(target_name),
        resource_dir.join("_up_").join("binaries").join(target_name),
        resource_dir.join(target_name),
    ];

    for path in &direct_paths {
        if path.exists() {
            return Ok(path.clone());
        }
    }

    // Fallback: search recursively
    if let Some(path) = find_file_recursive(&resource_dir, target_name) {
        return Ok(path);
    }

    // Additional fallback for dev mode running from workspace root
    let dev_path = PathBuf::from("binaries").join(target_name);
    if dev_path.exists() {
        return Ok(dev_path);
    }

    let dev_src_path = PathBuf::from("src-tauri").join("binaries").join(target_name);
    if dev_src_path.exists() {
        return Ok(dev_src_path);
    }

    Err(format!(
        "Could not find sidecar executable '{}' in resource dir ({:?}) or current working directory.",
        target_name, resource_dir
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

    let client = reqwest::Client::new();
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

/// Run transcription using the local Whisper sidecar binary and return the result
pub fn run_local_whisper<R: Runtime>(
    app_handle: &tauri::AppHandle<R>,
    model_name: &str,
    wav_path: &str,
    language: &str,
) -> Result<String, String> {
    // 1. Locate sidecar binary
    let sidecar_path = find_sidecar(app_handle)?;
    let sidecar_dir = sidecar_path
        .parent()
        .ok_or_else(|| "Invalid sidecar path".to_string())?;

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

    let output = Command::new(&sidecar_path)
        .current_dir(sidecar_dir)
        .args([
            "-m",
            model_path.to_str().ok_or("Invalid model path encoding")?,
            "-f",
            wav_path,
            "-l",
            language,
            "-nt",
            "-np",
        ])
        .output()
        .map_err(|e| format!("Failed to run sidecar command: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("Whisper sidecar exited with error: {}", stderr));
    }

    let text = String::from_utf8_lossy(&output.stdout);
    Ok(text.trim().to_string())
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
