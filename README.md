# Aura — Voice Dictation for Windows

[![CI](https://github.com/malashkadev/aura/actions/workflows/ci.yml/badge.svg)](https://github.com/malashkadev/aura/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-orange.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-Windows%2010%2F11-blue)](#)

Hold a hotkey, speak, release — Aura transcribes your speech, cleans it up (punctuation, filler words) and types it into **any** Windows application. Works with cloud AI (Gemini / OpenAI / Groq) or fully offline with local Whisper.

> 🇷🇺 [Документация на русском](README.ru.md)

<!-- TODO: add a screenshot/GIF here before publishing -->
<!-- ![Aura demo](docs/demo.gif) -->

## Features

- **Global hotkey dictation** — hold to talk, or short-tap to latch recording (toggle mode); `Esc` cancels.
- **Two engines**:
  - **Cloud** — Gemini, OpenAI (Whisper + GPT) or Groq (Whisper + Llama), with per-provider API keys.
  - **Local** — whisper.cpp sidecar, 100% offline and private; models downloaded from the settings UI.
- **AI cleanup** — removes filler words, fixes punctuation and grammar, never answers your questions — only transcribes them.
- **Edit selected text by voice** — select text, dictate a command (“make this formal”, “translate to English”).
- **Live streaming mode** (experimental) — text appears as you speak and is replaced by the final version.
- **Transcription history** — the last 50 dictations with one-click copy.
- **Custom dictionary** — bias recognition towards your names and terms.
- **11 language options** — auto-detect, keyboard-layout detection, or a fixed language (ru, en, de, es, fr, it, zh, pt, tr).
- **Voice punctuation commands** (optional) — “запятая”, “новая строка” → `,`, newline.
- **Polished overlay** — live waveform, recording timer, error states and optional sound themes (zen / rhodes / sci-fi / classic).
- **Quality of life** — autostart with Windows, tray icon, single-instance guard, focus guard (never types into the wrong window).

## Installation

Download the latest installer from [Releases](https://github.com/malashkadev/aura/releases) and run it.

For cloud mode you will need an API key — the free [Groq](https://console.groq.com/) tier works great. For local mode just download a Whisper model from the settings (base is a good start).

## Usage

| Action | Default |
|---|---|
| Start recording | hold `Alt+V` |
| Finish and paste | release the hotkey |
| Latch recording (toggle mode, optional) | short tap `Alt+V` |
| Cancel recording | `Esc` |

The hotkey, language, engine and everything else is configurable from the settings window (tray icon → «Открыть настройки»).

## Building from source

Prerequisites: [Rust](https://rustup.rs/) (stable), [Node.js](https://nodejs.org/) 18+, WebView2 (preinstalled on Windows 11).

```bash
git clone https://github.com/malashkadev/aura.git
cd aura
npm install
npm run dev     # development
npm run build   # NSIS/MSI installer in src-tauri/target/release/bundle/
```

The whisper.cpp sidecar binaries ship in `src-tauri/binaries/`. To update them to a newer whisper.cpp release, run `python install_whisper.py`.

Run the test suite:

```bash
cd src-tauri
cargo test
```

## Privacy

- **Local mode** never sends anything anywhere — audio is processed on your machine.
- **Cloud mode** sends the recorded audio (and the selected text, when you use voice editing) to the provider you chose. Nothing else is collected; there is no telemetry.
- Settings (including API keys) are stored locally in `%APPDATA%/com.aura.app/settings.json`; history in `%LOCALAPPDATA%/com.aura.app/history.json`. Known limitation: API keys are stored in plain text — see [issue tracker](https://github.com/malashkadev/aura/issues) for the Credential Manager migration plan.

## License

[MIT](LICENSE)
