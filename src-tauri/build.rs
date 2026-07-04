use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    tauri_build::build();

    // Copy DLLs from binaries/ to target/debug or target/release
    if let Ok(out_dir) = env::var("OUT_DIR") {
        let mut profile_dir = PathBuf::from(out_dir);
        while profile_dir.file_name().and_then(|s| s.to_str()) != Some("build") {
            if !profile_dir.pop() {
                break;
            }
        }
        if profile_dir.pop() {
            // profile_dir is now target/debug or target/release
            let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap_or_default();
            let binaries_dir = PathBuf::from(manifest_dir).join("binaries");

            if binaries_dir.exists() {
                if let Ok(entries) = fs::read_dir(binaries_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("dll") {
                            let dest = profile_dir.join(path.file_name().unwrap());
                            let _ = fs::copy(&path, &dest);
                        }
                    }
                }
            }
        }
    }
}
