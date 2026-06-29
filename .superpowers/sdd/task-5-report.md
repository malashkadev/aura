# Task 5 Report: Local Whisper.cpp Sidecar Integration

## Status: DONE

## Files Created/Modified
- **Modified**: [src-tauri/tauri.conf.json](file:///d:/Загрузки/1/src-tauri/tauri.conf.json)
- **Modified**: [src-tauri/src/lib.rs](file:///d:/Загрузки/1/src-tauri/src/lib.rs)
- **Created**: [src-tauri/src/whisper_runner.rs](file:///d:/Загрузки/1/src-tauri/src/whisper_runner.rs)
- **Added**: [src-tauri/binaries/](file:///d:/Загрузки/1/src-tauri/binaries/) (Whisper sidecar and required DLLs)
- **Created**: [task-5-report.md](file:///d:/Загрузки/1/.superpowers/sdd/task-5-report.md) (this report)

## Implementation Details
1. **Whisper Sidecar Registration**:
   - Registered `binaries/whisper-sidecar` under `bundle.externalBin` in `tauri.conf.json`.
2. **Model Downloader (`download_model`)**:
   - Resolves the target folder to `AppData/Local/com.glaido.app/models/` using `tauri::path::BaseDirectory::AppLocalData`.
   - Downloads model binary files from the Hugging Face `ggerganov/whisper.cpp` repository.
   - Writes the download stream to a `.tmp` file and performs an atomic rename (`fs::rename`) once fully downloaded.
   - Emits progress events (`model-download-progress`) with fields `model`, `downloaded`, `total`, `percentage`, and `done` using Tauri's `Emitter` trait, allowing settings UI integration.
3. **Robust Filename Formatting (`format_model_filename`)**:
   - Correctly formats various user-specified model names (e.g., `"small"`, `"base.bin"`, `"ggml-tiny"`, `"ggml-small.bin"`) to `ggml-<size>.bin` without double prefixing or duplicating extensions.
4. **Sidecar Resolution (`find_sidecar`)**:
   - Searches for the target-specific sidecar binary (`whisper-sidecar-x86_64-pc-windows-msvc.exe`) within direct paths under `resource_dir`, falling back to recursive directory search, and dev fallback directories.
   - Ensures compatibility across both Tauri packaging and local development runs.
5. **Transcription Runner (`run_local_whisper`)**:
   - Spawns the sidecar process with arguments `-m <model_path> -f <wav_path> -l ru -nt -np`.
   - Sets the process's working directory (`current_dir`) to the sidecar's directory, ensuring DLLs like `SDL2.dll` and `ggml-base.dll` are located and loaded successfully by the OS.
   - Reads the transcription from standard output, trims it, and returns the result.

## Verification & Testing
- **Command**: `cargo test`
- **Result**: Successfully compiled and passed unit tests:
  - `whisper_runner::tests::test_filename_parsing`
  - `ai_client::tests::test_gemini_request_serialization`
  - `ai_client::tests::test_openai_chat_deserialization`
  - `ai_client::tests::test_gemini_response_deserialization`
  - `audio_recorder::tests::test_process_and_write_wav`
- **Test Output**:
  ```
  running 5 tests
  test ai_client::tests::test_gemini_request_serialization ... ok
  test ai_client::tests::test_gemini_response_deserialization ... ok
  test ai_client::tests::test_openai_chat_deserialization ... ok
  test whisper_runner::tests::test_filename_parsing ... ok
  test audio_recorder::tests::test_process_and_write_wav ... ok

  test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
  ```

## Commits Created
- **SHA**: `ef4d282`
- **Subject**: `feat: implement local whisper.cpp sidecar integration and model downloader`
