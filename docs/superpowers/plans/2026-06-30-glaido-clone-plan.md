# Glaido Clone Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a voice dictation Windows application (Glaido clone) using Tauri, Rust, and HTML/CSS/JS.

**Architecture:** A tray-minimized Tauri application that intercept-listens for global `Alt+N` hold/release events, captures audio, transcribes it via Gemini API or local Whisper.cpp sidecar, and simulates typing/pasting to insert it into the active window.

**Tech Stack:** Tauri v2, Rust, cpal, hound, windows-sys, HTML5/CSS3 (vanilla glassmorphic styling), JavaScript.

## Global Constraints
- Target platform: Windows.
- Running commands: PowerShell script execution is disabled, so run npm/npx commands using `cmd /c`.

---

### Task 1: Project Scaffolding

**Files:**
- Create: `d:/Загрузки/1/package.json`
- Create: `d:/Загрузки/1/src-tauri/Cargo.toml`
- Create: `d:/Загрузки/1/src-tauri/tauri.conf.json`

- [ ] **Step 1: Run create-tauri-app**
  Run: `cmd /c "npx -y create-tauri-app@latest ./ -m npm -t vanilla --tauri-version 2 -y -f"`
  Expected: Success, folders `src-tauri` and `src` created.

- [ ] **Step 2: Install dependencies**
  Run: `cmd /c "npm install"`
  Expected: npm packages installed successfully.

- [ ] **Step 3: Add Rust dependencies**
  Add crates to `src-tauri/Cargo.toml` under `[dependencies]`:
  ```toml
  windows-sys = { version = "0.52", features = ["Win32_UI_WindowsAndMessaging", "Win32_Foundation", "Win32_System_LibraryLoader", "Win32_UI_Input_KeyboardAndMouse"] }
  cpal = "0.15"
  hound = "3.5"
  reqwest = { version = "0.11", features = ["json", "multipart"] }
  serde = { version = "1.0", features = ["derive"] }
  serde_json = "1.0"
  tokio = { version = "1", features = ["full"] }
  ```

---

### Task 2: Global Win32 Keyboard Hook

**Files:**
- Create: `d:/Загрузки/1/src-tauri/src/keyboard_hook.rs`
- Modify: `d:/Загрузки/1/src-tauri/src/main.rs`

- [ ] **Step 1: Implement Win32 Low-Level Keyboard Hook**
  Create `keyboard_hook.rs` containing hook logic using `SetWindowsHookExW` to capture down/up transitions of `N` while `Alt` is held. Define callback interfaces to notify Tauri core.

- [ ] **Step 2: Integrate hook in main.rs**
  Start the hook thread on Tauri app initialization and forward events to Tauri event bus.

---

### Task 3: Audio Recorder Component

**Files:**
- Create: `d:/Загрузки/1/src-tauri/src/audio_recorder.rs`

- [ ] **Step 1: Implement Audio Capture**
  Create `audio_recorder.rs` with `AudioRecorder` struct. Use `cpal` to capture input stream from the default microphone, sample rate 16kHz, mono, and record buffer to a WAV file using `hound`.

---

### Task 4: AI API clients (Gemini & OpenAI)

**Files:**
- Create: `d:/Загрузки/1/src-tauri/src/ai_client.rs`

- [ ] **Step 1: Implement Gemini API Audio Upload**
  Create `ai_client.rs`. Add logic to send WAV files to Gemini API using multipart upload and prompt instructions to correct/format transcription.
  
- [ ] **Step 2: Implement OpenAI Whisper Client**
  Add alternative client for OpenAI Whisper API + GPT-4o-mini formatting.

---

### Task 5: Local Whisper.cpp Sidecar Integration

**Files:**
- Create: `d:/Загрузки/1/src-tauri/src/whisper_runner.rs`

- [ ] **Step 1: Integrate whisper-sidecar.exe**
  Download static build of `whisper.cpp` (`main.exe`) and place it in Tauri sidecar bin directory as `whisper-sidecar.exe`.
  
- [ ] **Step 2: Implement Rust Execution Command**
  In `whisper_runner.rs`, write code to execute the sidecar process passing model path and WAV file path, then read output text.

---

### Task 6: UI & Overlay Windows

**Files:**
- Create: `d:/Загрузки/1/src/index.html`
- Create: `d:/Загрузки/1/src/style.css`
- Create: `d:/Загрузки/1/src/main.js`

- [ ] **Step 1: Settings Dashboard**
  Design premium glassmorphic dark-theme configuration UI for hotkey config, API keys, and model downloads.

- [ ] **Step 2: Floating Overlay Widget**
  Create a borderless, small, semi-transparent window that Tauri displays during active voice recording.
