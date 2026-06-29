### Task 5: Local Whisper.cpp Sidecar Integration

**Files:**
- Create: `d:/Загрузки/1/src-tauri/src/whisper_runner.rs`

- [ ] **Step 1: Integrate whisper-sidecar.exe**
  Download static build of `whisper.cpp` (`main.exe`) and place it in Tauri sidecar bin directory as `whisper-sidecar.exe`.
  
- [ ] **Step 2: Implement Rust Execution Command**
  In `whisper_runner.rs`, write code to execute the sidecar process passing model path and WAV file path, then read output text.

---

