# Progress Ledger - Glaido Clone Development

## Task Status
- [x] Task 1: Project Scaffolding
- [x] Task 2: Global Win32 Keyboard Hook
- [x] Task 3: Audio Recorder Component
- [x] Task 4: AI API clients (Gemini & OpenAI)
- [x] Task 5: Local Whisper.cpp Sidecar Integration
- [x] Task 6: UI & Overlay Windows

## Activity Log
- **2026-06-30**: Implemented global Win32 low-level keyboard hook (WH_KEYBOARD_LL) to capture Alt+N pressed/released states, suppressing the N key to prevent text input in active windows during recording.
- **2026-06-30**: Implemented `AudioRecorder` using `cpal` to capture microphone stream, downmixing channels to mono, resampling to 16kHz via linear interpolation, and writing to a WAV file using `hound`.
- **2026-06-30**: Implemented `ai_client` module with `transcribe_and_clean` supporting both Gemini (with `inlineData` Base64 audio upload) and OpenAI (Whisper transcription and Chat Completions `gpt-4o-mini` formatting). Added request/response serialization/deserialization unit tests.
- **2026-06-30**: Integrated `whisper.cpp` Windows x64 binary as a Tauri externalBin sidecar. Implemented model downloader from Hugging Face with real-time download progress events.
- **2026-06-30**: Created glassmorphic Settings dashboard and borderless transparent recording Overlay windows. Glued hook workflow with context capture (Ctrl+C), recording, transcription, and paste simulation (Ctrl+V).
