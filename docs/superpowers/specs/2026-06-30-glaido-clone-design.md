# Design Specification: Glaido Clone (Voice Layer)

A premium, lightweight, and fast voice dictation application for Windows, built on Tauri (Rust + HTML/CSS/JS). The application lives in the system tray, listens for a global hotkey (`Alt+N`), records audio, transcribes it (using cloud Gemini/OpenAI API or offline local Whisper.cpp), and inserts the cleaned-up text directly into the active window.

---

## Technical Stack
- **Framework**: Tauri v2 (Rust Backend + Web Frontend)
- **Frontend**: Vanilla HTML5, CSS3 (glassmorphic styling, HSL colors), and Vanilla TypeScript/JavaScript
- **Audio Capture**: Rust crate `cpal` + `hound` (WAV encoder)
- **Global Hotkey (Hold-to-Talk)**: Native Windows Low-Level Keyboard Hook (`SetWindowsHookEx` with `WH_KEYBOARD_LL` via `windows-sys`)
- **Local Speech-to-Text**: Pre-compiled `whisper.cpp` (`main.exe` renamed to `whisper-sidecar.exe`) bundled as a Tauri sidecar
- **Cloud Speech-to-Text & Processing**: Gemini API (Flash 1.5/2.0) with support for direct audio uploads, or OpenAI Whisper API

---

## Core Features

### 1. System Tray Integration
- The application starts minimized in the Windows system tray.
- Double-clicking the tray icon opens the **Settings Window**.
- Right-clicking the tray icon opens a context menu:
  - Settings
  - Pause Hotkeys
  - Exit

### 2. Settings Window (UI/UX)
- Designed with premium dark-mode aesthetics (deep grays, accent lime/green, smooth transitions, glassmorphic card effects).
- **Tabs**:
  - **General Settings**:
    - Transcription Mode: Toggle between **Cloud AI** and **Local AI (Offline)**.
    - Global Hotkey: Change default hotkey (default is `Alt+N`).
  - **Cloud AI Settings**:
    - Select API Provider: **Gemini API** or **OpenAI API**.
    - API Key input field.
  - **Local AI Settings**:
    - Model Downloader: Select model size (Base ~150MB, Small ~480MB). Shows download progress.
  - **About**: Version info and status.

### 3. Recording Indicator (Overlay)
- A borderless, click-through, floating overlay window appears in the bottom-middle of the screen (or near the cursor) when recording is active.
- Displays a pulsing microphone icon and a "Listening..." indicator.
- Transitions to "Processing..." upon key release, then disappears.

### 4. Global Hotkey Engine (Alt + N)
- Implemented as a background thread in Rust using low-level Win32 hooks to detect key hold and release events:
  - **On Alt+N Down**:
    1. Temporarily capture the currently selected text by simulating `Ctrl+C` keystrokes and reading the clipboard.
    2. Instantly trigger the floating overlay window.
    3. Start recording audio from the default input device (microphone).
  - **On Alt or N Up**:
    1. Stop recording.
    2. Change overlay state to "Processing...".
    3. Run transcription (Cloud or Local).
    4. Format and rewrite the transcript using the selected AI engine (applying prompt instructions to clean filler words and match selected text context).
    5. Write the resulting text to the clipboard, simulate `Ctrl+V` (paste) to insert the text, and restore the original clipboard content.
    6. Close the overlay window.

---

## Data Flow & Processing

### Local Mode (Offline)
1. Audio is saved as a 16kHz Mono WAV file in `AppData/Local/Temp/glaido-temp.wav`.
2. The Tauri app invokes the `whisper-sidecar.exe` process with the path to the selected model (e.g. `ggml-small.bin`) and the WAV file.
3. The resulting text is parsed and inserted directly.

### Cloud Mode
1. Audio is captured in WAV format.
2. If **Gemini API** is selected:
   - Upload the audio WAV file directly to the Gemini API (using file API or inline part).
   - Prompt:
     ```text
     You are an elite dictation and editing assistant.
     Task: Transcribe the audio.
     Context: The user had this text selected: [SELECTED_TEXT]
     Instructions: Clean up the dictation by removing filler words (um, uh, like), correcting grammar, and adding proper punctuation.
     If the dictation contains a command to edit the selected text (e.g., "make this formal", "refactor to async"), perform the edit on the selected text.
     Return ONLY the final transcribed/edited text. Do not add explanations.
     ```
   - Receive the text and paste it.
3. If **OpenAI API** is selected:
   - Call OpenAI Whisper API to get the raw transcript.
   - Call OpenAI Chat Completion API (GPT-4o-mini) to clean it up using the same prompt.

---

## Verification Plan

### Automated Tests
- Integration tests for the hotkey hook listener in Rust.
- Unit tests for WAV encoder validation.
- Mock API requests to verify Gemini and OpenAI integrations.

### Manual Verification
- Test hold-to-talk recording duration (under 5s, under 30s, under 60s).
- Verify the context editing feature: select some text, hold `Alt+N`, say "make this email more polite", release, and check if the selected text gets replaced by a polite version.
- Validate offline mode transcription accuracy.
- Verify low CPU and memory footprint during background idle state.
