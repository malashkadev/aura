# Aura v1.1.0

### 🎮 DirectX & Vulkan Gaming Mode
* **Automatic 3D Game Detection**: Recognizes fullscreen DirectX/Vulkan games via `SHQueryUserNotificationState` and `MonitorFromWindow`.
* **Zero Input Lag**: Automatically pauses the global low-level keyboard hook (`HOOK_PAUSED`) while gaming.
* **100% VRAM Release**: Unloads resident local speech models (Whisper.cpp / Sherpa-ONNX) from memory to dedicate all GPU resources to the game.
* **Dedicated Tray Indicator**: Dynamic orange tray icon (`32x32_gaming.png`) and manual toggle in the tray menu and settings.

### 💤 VRAM Standby Unloader (Sleep Mode)
* **Automatic Idle Offloading**: Unloads heavy local neural network models from GPU VRAM and RAM after configurable inactivity (15, 30, 60 minutes, or disabled).
* **Instant Wake-up**: Seamlessly re-initializes on the next dictation hotkey press without restarting the app.

### 🖥️ Custom High-DPI System Tray Menu
* **Per-Monitor DPI Geometry**: Dynamically calculated popup position using Win32 API (`MonitorFromPoint` + `GetMonitorInfoW`), eliminating clipping and misplacement on 4K displays and mixed-DPI setups.
* **Stealth Command Deck**: Fast engine switching (Whisper / Parakeet / Cloud), gaming mode toggle, and drill-down language selection covering all 11 recognition languages.

### ✍️ Text Replacements (Dedicated "Text" Tab)
* **Custom Text Expansion**: Offline substitution rules to instantly replace spoken triggers with full text snippets, email addresses, and technical jargon.
* **Win32 Input Queue Flushing**: Atomic text replacement with buffered backspacing and typing delays to prevent dropped characters across fast editors.

### 📜 300-Item Encrypted History
* **Expanded Capacity**: History depth increased to 300 entries with Windows DPAPI encryption (`CryptProtectData`).
* **Day Grouping & Search**: Categorized by day ("Today", "Yesterday", and calendar dates) with real-time text filtering and source tags.

### 🎙️ Audio Engine & Hardware Stability
* **WASAPI Audio Hotplug**: Real-time headset and microphone connection/disconnection monitoring without application restarts.
* **450ms Tail Hold Buffer**: Extended recording grace margin combined with non-destructive Silero VAD prevents clipped word and sentence endings.
* **6 Procedural Audio Themes**: Synthesized Web Audio feedback themes (Zen, Rhodes, Sci-Fi, Classic Bell, Water Drop, Haptic Switch).

### Package Manager (WinGet)
```cmd
winget install Malashka.Aura
```

---

**Full Changelog**: https://github.com/malashkadev/aura/compare/v1.0.10...v1.1.0

[![Download Aura on SourceForge](https://a.fsdn.com/con/app/sf-download-button)](https://sourceforge.net/projects/aura-voice-typing/files/latest/download)
