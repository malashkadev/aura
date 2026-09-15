<p align="center">
  <img src="docs/logo.svg?v=5" width="120" height="120" alt="Aura Logo" />
</p>

<h1 align="center">Aura — Voice Typing for Windows</h1>

<p align="center">
  <b>Instant offline & cloud voice-to-text dictation directly under your cursor.</b><br />
  <a href="https://aura-beryl-five.vercel.app/">🌐 Official Website & Live Demo</a>
</p>

<p align="center">
  <a href="https://github.com/malashkadev/aura/actions/workflows/ci.yml"><img src="https://github.com/malashkadev/aura/actions/workflows/ci.yml/badge.svg" alt="CI" /></a>
  <a href="https://github.com/malashkadev/aura/releases"><img src="https://img.shields.io/github/v/release/malashkadev/aura?color=10b981&label=release" alt="Release" /></a>
  <a href="https://sourceforge.net/projects/aura-voice-typing/files/latest/download"><img src="https://img.shields.io/sourceforge/dt/aura-voice-typing.svg" alt="Download on SourceForge" /></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-AGPL_v3-blue.svg" alt="License: AGPL v3" /></a>
  <img src="https://img.shields.io/badge/platform-Windows%2010%2F11-blue" alt="Platform" />
</p>

> 🇷🇺 [Документация на русском](README.ru.md)

Press and hold the hotkey, speak, and release — Aura instantly transcribes your speech and types the text right under your cursor in **any** Windows application. Runs 100% offline via local models (Whisper, NVIDIA Parakeet with CUDA GPU acceleration) or connects to cloud providers (Gemini, Groq, OpenAI, Hugging Face, Custom API).

**100% Free & Open Source (AGPL-3.0)** — no subscriptions, paywalls, telemetry, or advertisements.

<p align="center">
  <img src="docs/Aura.gif" width="49%" alt="Aura Dictation Demo" />
  <img src="docs/Settings.gif" width="49%" alt="Aura Settings Demo" />
</p>

## Features

- **Global hotkey dictation** — hold to talk (`Alt+V`), or short-tap to latch recording (toggle mode); `Esc` cancels.
- **Two recognition modes**:
  - **Local (100% offline & private)** — whisper.cpp or NVIDIA Parakeet TDT v3 (sherpa-onnx) on CPU or NVIDIA CUDA GPU. Audio never leaves your computer; download models in one click directly from settings.
  - **Cloud** — Google Gemini, Groq, OpenAI, Hugging Face, or your custom OpenAI-compatible server.
- **NVIDIA CUDA GPU acceleration** — hardware GPU acceleration for Whisper and Parakeet models with 1-click on-demand runtime downloading and automatic CPU fallback.
- **Gaming Mode** — automatic detection of 3D games (DirectX / Vulkan) with keyboard hook suspension and model memory unloading for maximum FPS, plus manual tray toggle.
- **Idle VRAM Standby (Sleep)** — configurable standby timer (15, 30, 60 minutes) to free GPU memory without closing Aura.
- **Real-time streaming input** — smooth word-level streaming into active fields without flickering or text duplicates (NVIDIA Parakeet).
- **Text Replacements & Dynamic Macros** — local rules for instant text expansion into snippets, email addresses, or live date/time tokens (`{date}`, `{date_ru}`, `{time}`, `{datetime}`).
- **Focus Guard & Context Editing** — ensures transcribed text is never typed into the wrong window if focus shifts (with safe clipboard handoff), and provides 1-click text editing on selected text with a visual AI sparkle indicator.
- **Audio device selection, WASAPI Hotplug & Tail Hold** — choose physical microphone input devices with dynamic device hotplugging, 450ms tail grace buffer, and non-destructive Silero VAD silence trimming.
- **Transcription history with day grouping** — up to 300 encrypted dictations (DPAPI) with day grouping (Today / Yesterday / calendar dates), live search, and source filters.
- **Custom dictionary** — bias recognition towards your names, brands, and technical terms.
- **12 language options** — auto-detect, keyboard-layout detection, or fixed selection (ru, en, de, es, fr, it, zh, pt, tr, nl).
- **Polished overlay with 6 sound themes** — microphone VU meter, recording timer, 6 audio themes (Zen, Rhodes, Sci-Fi, Classic Bell, Bubble, Haptic), and display customization.
- **Native High-DPI Tray Menu** — modern tray menu with per-monitor DPI scaling, instant gaming mode toggle, and settings access.
- **System integration & security** — autostart with Windows, Windows DPAPI credential encryption, and one-click diagnostic reports.

## How Aura compares

Aura combines the speed and privacy of local neural networks with the flexibility of cloud AI providers.

| Feature | **Aura** | **Handy** | **Wispr Flow** |
|---|---|---|---|
| **Price** | **Free (AGPL-3.0)** | Free (Open Source) | Paid (subscription) |
| **Platforms** | Windows *(macOS in progress)* | Windows / macOS / Linux | Windows / macOS |
| **Local Models (Offline)** | **✅ Whisper & Parakeet (CUDA GPU / CPU)** | ✅ Whisper / Parakeet | ❌ Cloud only |
| **Cloud Providers** | **✅ Gemini, Groq, OpenAI, HF, Custom** | ❌ | ✅ Proprietary cloud |
| **Real-time Streaming Input** | **✅** | ❌ | ✅ |
| **Gaming Mode (D3D/Vulkan)** | **✅ Auto + Manual** | ❌ | ❌ |
| **Text Replacements & Macros** | **✅** | ❌ | ➖ |
| **Focus Guard Protection** | **✅** | ❌ | ➖ |
| **Data Encryption (DPAPI)** | **✅** | ❌ | ➖ |
| **Custom Term Dictionary** | **✅** | ❌ | ✅ |
| **Transcription History** | **✅ Up to 300 items (DPAPI)** | ❌ | ✅ |

> Official builds target Windows 10 & 11 (macOS port compiles in CI). Handy is a mature cross-platform choice for offline dictation. Wispr Flow offers a polished UI, but requires a paid subscription and proprietary cloud.

## Installation

### Via Windows Package Manager (WinGet)

The fastest way to install Aura on Windows 10/11:

```powershell
winget install Malashka.Aura
```

### Standalone Installer

Download the installer from [GitHub Releases](https://github.com/malashkadev/aura/releases) or via [SourceForge Mirror](https://sourceforge.net/projects/aura-voice-typing/files/latest/download) and run it. You can explore the interactive settings mockup and live demo on our [Official Website](https://aura-beryl-five.vercel.app/).

[![Download Aura on SourceForge](https://a.fsdn.com/con/app/sf-download-button)](https://sourceforge.net/projects/aura-voice-typing/files/latest/download)

For local mode, download a model from the "Speech" tab in Settings (`Base` model is a great starting point for fast CPU inference, `Large v3 Turbo (Q5)` for NVIDIA CUDA). For cloud mode, provide an API key for your chosen provider (Groq and Google Gemini offer free tiers).

> **First launch — "Windows protected your PC"?** The installer is not yet signed with an Authenticode certificate, so Windows SmartScreen displays a standard warning for new open-source binaries. Click **More info → Run anyway**. The entire codebase is open-source for independent audit and self-compilation.

## Usage

| Action | Default |
|---|---|
| Start recording | hold `Alt + V` |
| Finish and type text | release the hotkey |
| Latch recording (toggle mode) | short tap `Alt + V` |
| Cancel recording | `Esc` |

The hotkey, language, recognition engine, and overlay settings are configurable via Settings (tray icon → "Open Settings").

## Building from source

Prerequisites: [Rust](https://rustup.rs/) (stable), [Node.js](https://nodejs.org/) 18+, WebView2 (preinstalled on Windows 10/11).

```bash
git clone https://github.com/malashkadev/aura.git
cd aura
npm install
npm run dev     # development mode
npm run build   # NSIS/MSI installer in src-tauri/target/release/bundle/
```

Whisper.cpp sidecar binaries live in `src-tauri/binaries/`. To update them to a newer release, run `python install_whisper.py`.

Run test suite:

```bash
cd src-tauri
cargo test
```

## Privacy & Security

- **Local mode** runs entirely offline on your machine (CPU / GPU) and never transmits audio over the network.
- **Cloud mode** sends audio recordings, generated transcripts, selected text (only when selection editing is active), and custom dictionary terms directly to your chosen provider over TLS with zero telemetry.
- **Secure local storage** — API keys and transcription history are encrypted using **Windows DPAPI** (`CryptProtectData`) with file access restricted to the current user and SYSTEM via ACL.
- **Update checks** run only on demand or when explicitly enabled in settings.

## What's new in v1.1.0

- **Gaming Mode** — automatic detection of 3D games and graphics APIs (DirectX / Vulkan) with safe suspension of the low-level keyboard hook and model memory unloading to maintain 100% FPS in games, plus manual tray toggle.
- **Idle VRAM Standby (Sleep)** — automatic model unloading from dedicated GPU memory when idle for 15, 30, or 60 minutes, with instant reload on next activation.
- **Local Text Replacements & Dynamic Macros** — custom expansion rules to replace spoken triggers with text snippets, email addresses, or live date/time tokens (`{date}`, `{date_ru}`, `{time}`, `{datetime}`).
- **Native High-DPI Tray Menu** — custom WebView2 tray window with per-monitor DPI awareness and multi-monitor cursor placement, featuring instant gaming mode toggle.
- **Dynamic Microphone Hotplug (WASAPI)** — real-time detection and graceful reconnection when headsets or USB microphones are plugged or unplugged.
- **300-Item Encrypted History** — day grouping (Today / Yesterday), live search, source filtering, and DPAPI encryption.
- **450ms Grace Buffer (Tail Hold)** — expanded post-release tail hold buffer and non-destructive Silero VAD to eliminate clipped word and sentence endings.
- **New Sound Themes** — added "Bubble" (water droplet) and "Haptic" (mechanical tactile switch) sound themes alongside Zen, Rhodes, Sci-Fi, and Classic Bell.
- **Windows Package Manager Support** — install and update via one command: `winget install Malashka.Aura`.

## Roadmap

- **macOS support** — native port (global hotkeys via `CGEventTap`, CoreAudio capture) is in the codebase and **compiles in CI**. Remaining items: macOS whisper binary, `.app` bundle, Accessibility-permission flow, and hardware testing.

## License

[AGPL-3.0](LICENSE)
