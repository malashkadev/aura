# SourceForge Publication Kit — Aura v1.1.0

Этот документ содержит все готовые тексты, метаданные и инструкции для публикации релиза **Aura v1.1.0** на платформе [SourceForge](https://sourceforge.net/projects/aura-voice-typing/).

---

## 1. Quick Metadata (Основная информация о проекте)

- **Project Name:** Aura — Voice Typing for Windows
- **Unix Name:** `aura-voice-typing`
- **Project URL:** https://sourceforge.net/projects/aura-voice-typing/
- **License:** GNU Affero General Public License v3 (AGPL-3.0)
- **Programming Language:** Rust, JavaScript / HTML / CSS (Tauri v2)
- **Operating System:** Windows 10, Windows 11 (64-bit x86_64)
- **Intended Audience:** End Users / Desktop, Developers, Writers, Healthcare & Legal Professionals

---

## 2. Short Summary / Tagline (Краткое описание — до 250 символов)

### English (Primary for SourceForge)
> Fast, private offline & cloud voice typing for Windows 10/11. Types directly under the cursor into any app using local Whisper & Parakeet (CPU & CUDA) or Cloud AI. Zero telemetry, 100% Free & Open Source (AGPL-3.0).

*(Length: 226 characters)*

### Russian
> Быстрый приватный голосовой ввод для Windows 10/11. Печатает прямо под курсором в любое приложение через локальные Whisper и Parakeet (CPU и CUDA) или облачный ИИ. Без телеметрии, 100% бесплатно и с открытым кодом.

*(Length: 224 символа)*

---

## 3. Project Description (Полное описание проекта для страницы Overview)

### English Markdown (Copy & Paste to SourceForge Description)

```markdown
**Aura** is a fast, accurate, and completely private voice typing utility for Windows 10 and 11. Press and hold your global hotkey (`Alt + V`), speak naturally, and release — Aura instantly transcribes your speech and types the text right under your cursor into **any** active Windows application (editors, browsers, messengers, terminals, IDEs, and office suites).

Unlike proprietary dictation software that locks you into paid subscriptions and transmits your voice to remote corporate servers, Aura is **100% Free & Open Source (AGPL-3.0)**, contains **zero telemetry**, and processes audio entirely offline on your local computer by default.

---

### Key Capabilities

- **Local & Offline Speech Recognition**:
  - **Whisper.cpp**: Resident background server keeping GGML models (`Base` to `Large v3 Turbo Q5`) resident in memory for sub-300ms transcription on CPU or NVIDIA CUDA GPU.
  - **NVIDIA Parakeet TDT v3** (0.6B int8 via `sherpa-onnx`): High-accuracy real-time streaming speech recognition with conjunction smoothing and prefix stabilization.
  - **1-Click NVIDIA CUDA GPU Acceleration**: Automatically downloads required CUDA runtime libraries on demand, enabling lightning-fast GPU transcription with automatic CPU fallback.

- **Optional Cloud AI Providers**:
  - Connect your own API keys to Google Gemini (multimodal 3.8 & 3.6 Flash), Groq (Whisper Large v3), OpenAI (Whisper-1), Hugging Face, or any custom OpenAI-compatible server URL.
  - Secondary contextual text editing and rewriting on selected text with a visual AI sparkle badge.

- **Gaming Mode (DirectX & Vulkan)**:
  - Automatically detects fullscreen 3D games, pauses the low-level keyboard hook (`HOOK_PAUSED`) to ensure zero input lag, and unloads resident speech models to free 100% of GPU VRAM for maximum gaming FPS.
  - Includes a dedicated orange tray status indicator (`32x32_gaming.png`) and instant manual override toggle.

- **Standby VRAM Unloader (Sleep Mode)**:
  - Automatically unloads heavy neural network models from GPU memory after a configurable period of inactivity (15, 30, or 60 minutes), instantly re-warming on your next dictation hotkey press.

- **Text Replacements & Dynamic Macros**:
  - Define custom offline expansion rules to turn spoken abbreviations into full snippets, email addresses, or formatted technical jargon.
  - Dynamic macros: insert live date and time stamps (`{date}`, `{date_ru}`, `{time}`, `{datetime}`).
  - Atomic Win32 queue flushing prevents dropped characters during rapid replacement.

- **Audio Engine & Stability**:
  - **WASAPI Audio Hotplug**: Dynamically detects USB/Bluetooth headset or microphone connections and disconnections without application restart.
  - **450ms Tail Hold Buffer**: Extended recording grace margin combined with non-destructive Silero VAD prevents clipped word and sentence endings.
  - **Focus Guard**: Prevents typing into unintended windows if focus changes mid-dictation, safely copying text to the Windows clipboard with an overlay notification.

- **Polished User Experience**:
  - **6-Tab Modern Settings**: General, Speech, Text, Hotkeys, History, and About.
  - **High-DPI Custom Tray Menu**: Per-monitor DPI scaling, drill-down navigation, engine switching, and 11 recognition languages.
  - **300-Item Encrypted History**: Encrypted locally via Windows DPAPI (`CryptProtectData`), organized by day («Today», «Yesterday», calendar dates) with instant search and source filtering.
  - **6 Procedural Audio Themes**: Web Audio synthesized feedback sounds (Zen, Rhodes, Sci-Fi, Classic Bell, Water Drop, Haptic Click) with adjustable volume.
  - **Full 9-Language Localization**: English, Russian, German, Spanish, French, Italian, Chinese, Portuguese, and Turkish.

---

### Requirements

- **Operating System**: Windows 10 (1809+) or Windows 11 (64-bit x86_64).
- **RAM**: Minimum 4 GB (8 GB recommended for local models).
- **GPU (Optional)**: NVIDIA GPU with CUDA support for accelerated local inference.
- **WebView2**: Built into Windows 10/11.

---

### Official Links

- **Website & Interactive Demo**: [https://aura-beryl-five.vercel.app/](https://aura-beryl-five.vercel.app/)
- **GitHub Repository**: [https://github.com/malashkadev/aura](https://github.com/malashkadev/aura)
- **WinGet Command**: `winget install Malashka.Aura`
```

---

## 4. Features List (Список функций для поля "Features" на SourceForge)

Каждый пункт вводится отдельной строкой:

1. Direct voice-to-text typing right under the cursor in any active Windows application via global hotkey (`Alt + V`).
2. 100% private offline speech recognition using resident Whisper.cpp and NVIDIA Parakeet TDT v3 neural networks.
3. 1-click on-demand NVIDIA CUDA GPU hardware acceleration with automatic CPU fallback.
4. Optional Cloud AI providers: Google Gemini (3.8/3.6 Flash), Groq, OpenAI, Hugging Face, and Custom OpenAI-compatible URLs.
5. Gaming Mode: automatic 3D game detection (DirectX/Vulkan), keyboard hook pause (`HOOK_PAUSED`), and 100% VRAM release.
6. Standby VRAM Unloader: automatic GPU memory release on idle (15, 30, 60 minutes) with instant background pre-warming.
7. Real-time streaming voice input with prefix stabilization and conjunction smoothing.
8. Text expansion rules with dynamic macros for live dates and times (`{date}`, `{date_ru}`, `{time}`, `{datetime}`).
9. Safe Focus Guard with automatic clipboard fallback if window focus shifts during transcription.
10. Dynamic microphone selection with WASAPI device hotplugging and 450ms tail hold buffer.
11. 300-entry transcription history with day grouping and native Windows DPAPI (`CryptProtectData`) encryption.
12. Minimalist High-DPI system tray menu with drill-down language and engine selectors.
13. 6 procedural Web Audio sound themes (Zen, Rhodes, Sci-Fi, Bell, Water Drop, Haptic Switch).
14. Full 9-language UI localization (English, Russian, German, Spanish, French, Italian, Chinese, Portuguese, Turkish).
15. 100% Free and Open Source under AGPL-3.0 with zero telemetry, zero analytics, and zero subscriptions.

---

## 5. Categories, Tags & Keywords (Категории и теги)

### Categories:
- `Communications :: Speech`
- `Office/Business :: Office Suites`
- `Text Editors`
- `Utilities`

### Tags / Topics:
`voice-typing`, `speech-to-text`, `dictation`, `whisper`, `whisper-cpp`, `parakeet`, `nvidia-cuda`, `cuda`, `stt`, `ai`, `offline-ai`, `local-ai`, `windows-11`, `windows-10`, `rust`, `tauri`, `agpl-3`

---

## 6. Release File Details (Параметры файла для загрузки в раздел Files)

- **Folder Path:** `/Aura 1.1.0/` (или `/v1.1.0/`)
- **File Name:** `Aura_1.1.0_x64-setup.exe`
- **File Size:** `23 449 392` bytes (22.36 MB)
- **SHA-256 Checksum:** `40400126D30931E552C8E30CE050C6DEF52BFE7EF3123A82AFDDFDA66F8F3534`
- **Default Download Setting:** После загрузки файла нажать на иконку `(i)` рядом с файлом и выбрать чекбокс **"Default download for: Windows"**.
