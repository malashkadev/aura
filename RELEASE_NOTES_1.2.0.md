### ⚡ Local-First Speech Recognition by Default
* **Autonomous & Private Out of the Box**: Aura now defaults to local speech recognition (`transcription_mode: "local"`), transcribing your voice entirely on your PC using resident Whisper or NVIDIA Parakeet servers with CUDA acceleration. Cloud AI remains an opt-in provider.
* **First-Run Model Setup Card**: When no local model is installed, the General settings tab surfaces a streamlined recommendation card to download the balanced Base model (~145 MB) and test your microphone with a single click.

### 💾 Native Portable Settings Backup & Restore
* **System Dialog Integration (`rfd`)**: Export your settings, personal dictionary, and text replacements through the native Windows Save As dialog with custom file path selection.
* **Strict Security Isolation**: Exported backups never contain API keys. On import, files undergo structural JSON validation, 2 MB size limits, and sanitization before replacing settings.

### 🛡️ Low-Level Input Protection & Switcher Harmony
* **Layout-Switcher Replay Suppression (Caramba Switcher / PowerToys)**: Neutralized phantom key repeats and letter flood when holding `Alt+V` by tagging synthetic injections (`dwExtraInfo`), detecting Unicode packets (`VK_PACKET = 0xE7`), matching Latin ↔ Cyrillic keyboard rows, and enforcing a 1500 ms aftermath window while preserving real `Ctrl+V` pastes.
* **Session Lock / Unlock Hotkey Recovery**: Message-only window with `WTSRegisterSessionNotification` prevents sticky hotkey latches after locking Windows (`Win+L`).

### 🎯 Smart Text Replacements & Caret Context
* **Cursor Marker in Snippets (`|`)**: Place a `|` marker inside any text replacement template (e.g. `Здравствуйте, |!`) to set the exact final caret position after insertion.
* **Native Win32 Caret-Context Fast Path**: Direct high-speed reading of classic `Edit`, RichEdit, and Scintilla fields via Win32 messages with fallback to UI Automation.

### 🔒 Security & Dependency Hardening
* **RustSec Vulnerability Resolution**: Upgraded `zip` to `9.0.5` (resolving RUSTSEC-2026-0367, 0368, 0369), `rustls-webpki` to `0.103.16` (RUSTSEC-2026-0382), `webpki-root-certs` to `1.0.10` (RUSTSEC-2026-0383), and pruned vulnerable updater dependencies.

### 🎨 Stealth Command Deck Design & Accessibility
* **Refined Dark Surfaces**: Achromatic monochrome card and dialog surfaces with physical specular facets (`inset 0 1px 0 rgba(255, 255, 255, 0.08..0.12)`).
* **Keyboard Navigation & Accessibility**: Complete keyboard arrow cycling on tabs, visible `:focus-visible` rings on interactive cards, proper ARIA semantics, and localized tray status tooltips across 9 languages.

### Package Manager (WinGet)
```cmd
winget install Malashka.Aura
```

---

**Full Changelog**: https://github.com/malashkadev/aura/compare/v1.1.2...v1.2.0

[![Download Aura on SourceForge](https://a.fsdn.com/con/app/sf-download-button)](https://sourceforge.net/projects/aura-voice-typing/files/latest/download)
