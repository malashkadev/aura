### 🎙️ Voice Selection Editing & AI Commands
* **In-Place Voice Editing**: Select any text in any Windows application (editor, browser, messenger, IDE, office suite), press and hold your global hotkey, and speak a command — Aura processes the selected text directly in-place.
* **Deterministic Core Commands**:
  * **Fix Grammar & Punctuation** (`FixGrammar` — e.g. *«исправь ошибки»*, *«проверь пунктуацию»*, *«fix grammar»*).
  * **Translate** (`Translate` — e.g. *«переведи на английский»*, *«переведи на русский»*, *«translate to Spanish»*).
* **Safe Dictation Fallback**: Any spoken phrase that is not recognized as a command safely passes through as normal dictation, eliminating stalls, erroneous rewrites, and command hallucination.
* **Supported Commands Guide**: Clean, balanced two-column interactive guide in Settings with syntax chips across 9 languages and a dedicated target language selector right under the translation command.
* **Diffuse Contour Glow**: When selection editing is active, the HUD overlay illuminates with an amber contour highlight, providing clear visual feedback that context editing is in progress.

### 🧠 Smart Context-Aware Text Insertion
* **Non-Invasive Caret-Context Inspection**: Aura inspects surrounding text before and after the caret via ultra-fast Win32 messages (`EM_GETSEL`, `WM_GETTEXT` on Edit, RichEdit, and Scintilla fields in ~0.05 ms) with a seamless fallback to Windows UI Automation (with a 20 ms fail-safe timeout). Password fields are automatically skipped.
* **Intelligent Sentence Adaptation**: Automatically lowercases initial words when inserting mid-sentence or continuing a thought, while strictly preserving uppercase acronyms (like `NASA`, `API`, `НДС`) and the English pronoun `"I"`.
* **Clean Boundaries**: Prevents double punctuation before commas or existing periods, normalizes boundary spaces, and includes a configurable toggle for trailing sentence periods on continuation.

### ⚡ Local-First Speech Recognition by Default
* **Autonomous & Private Out of the Box**: Aura now defaults to local speech recognition (`transcription_mode: "local"`), transcribing your voice entirely on your PC using resident Whisper.cpp or NVIDIA Parakeet TDT v3 background servers with CUDA acceleration. Audio never leaves your computer; cloud AI remains an opt-in provider.
* **First-Run Model Setup Card**: When no local model is installed, the General settings tab surfaces a streamlined recommendation card to download the balanced Base model (~145 MB) and test your microphone with a single click.

### 🎯 Smart Text Replacements & Caret Marker (`|`)
* **Cursor Marker in Snippets (`|`)**: Place a `|` marker inside any text replacement template (e.g. `Здравствуйте, |!`, `Hello, |!`) to set the exact final caret position after insertion.
* **Flow-Controlled Key Simulation**: Automatically sends batch left-arrow key events conforming to Windows Input Buffer Flow Control guidelines, preventing dropped keystrokes in deep applications.
* **Dynamic Time Tokens**: Insert live dates and times on the fly using `{date}`, `{date_ru}`, `{time}`, `{datetime}`.

### 🔮 Next-Generation HUD Overlay (Dual Beams & Morphing)
* **Dual Beams Conic Orbit**: Hardware-accelerated CSS Houdini `@property --beam-angle` with dual symmetric orange rays along the outer card perimeter (`border-box`), smoothly accelerating to 1.1s during processing.
* **Seamless 550ms Crossfade**: Smooth transition between recording and processing states without black flickers.
* **Kinetic Morphing Square**: Morphs into a compact 39×39px square upon completion with constant vector monogram scaling and zero kinetic distortion.

### 🧭 Interactive Spotlight Onboarding Tour
* **Guided 5-Step Walkthrough**: Highlights key capabilities (speech engines, language/dictionary/snippets, smart text editing, hotkeys & overlay, history & tray) with zero clutter.
* **Docked Footer Layout**: Fixed bottom control deck with zero popover collision or visual card obstruction.
* **Concentric Corner Geometry**: Mathematically computed radius matching inner card curvature ($R_{\text{cutout}} = R_{\text{target}} + \text{padding}$).
* **Dynamic Narrative Adaptation**: Automatically switches step narrative to text replacements and templates when the language card is hidden in Parakeet or cloud mode.

### 🔊 Procedural Web Audio Sound Themes & Volume Track
* **6 Synthesized Audio Themes**: Procedural Web Audio synthesis (Zen, Rhodes, Sci-Fi, Classic Bell, Bubble, Haptic Click) without external audio files.
* **Ergonomic Volume Slider**: 320px track with direct audio preview on adjustment and a vector mute toggle with memory restore.
* **WASAPI Microphone Hotplugging**: Dynamically detects USB/Bluetooth audio device connection/disconnection without restarting the application, backed by a 450ms tail hold buffer.

### 🛡️ Low-Level Input Protection & Switcher Harmony
* **Layout-Switcher Replay Suppression (Caramba Switcher / PowerToys)**: Neutralized phantom key repeats and letter flood when holding `Alt+V` by tagging synthetic injections (`dwExtraInfo = 0x41555241`), detecting Unicode packets (`VK_PACKET = 0xE7`), matching Latin ↔ Cyrillic keyboard rows, and enforcing a 1500 ms aftermath protection window while preserving real `Ctrl+V` pastes.
* **Session Lock / Unlock Hotkey Recovery**: Message-only window with `WTSRegisterSessionNotification` prevents sticky hotkey latches after locking Windows (`Win+L`).
* **Auto-Repeat Suppression**: Prevents held hotkeys from repeatedly starting and stopping recording sessions in both hold and toggle modes.
* **Whisper Silence Hallucination Filtering**: Strips end-of-speech silence artifacts ("Спасибо за просмотр.", "Thank you for watching.") and looped filler utterances (`ммммм...`, `hmmm...`) while strictly preserving real gratitude.

### 💾 Native Portable Settings Backup & Restore
* **System Dialog Integration (`rfd`)**: Export your settings, personal dictionary, and text replacements through the native Windows Save As dialog with custom file path selection.
* **Strict Security Isolation**: Exported backups never contain API keys. On import, files undergo structural JSON validation, 2 MB size limits, and sanitization before replacing settings.

### 🎨 Stealth Command Deck Design & Accessibility
* **The Stealth Command Deck**: 100% achromatic monochrome card and dialog surfaces with physical specular facets (`inset 0 1px 0 rgba(255, 255, 255, 0.08..0.12)`) and achromatic frosted plates.
* **Windows 11 Fluent About Rows**: Clean interactive rows in About with a dedicated application exit button.
* **Offline Typography**: Locally bundled WOFF2 `Onest` font with zero external network requests.
* **4-Tier Semantic Toast Capsule**: HUD capsule pill (`9999px`) with distinct icon semantics (`✓` for success, trash for deletion, `(i)` for in-progress operations, `!` for errors).
* **Keyboard Accessibility**: Arrow-key cyclic navigation on tabs, visible `:focus-visible` rings on interactive cards, proper ARIA semantics, and localized tray status tooltips across 9 languages.

### 🔒 Security, Stability & Installer Reliability
* **RustSec Vulnerability Resolution**: Upgraded `zip` to `9.0.5` (resolving RUSTSEC-2026-0367, 0368, 0369), `rustls-webpki` to `0.103.16` (RUSTSEC-2026-0382), `webpki-root-certs` to `1.0.10` (RUSTSEC-2026-0383), and pruned vulnerable updater dependencies.
* **De-Elevated Installer Auto-Launch**: Custom NSIS installer template uses `nsis_tauri_utils::RunAsUser` with process spawn verification and retry loop, ensuring Aura starts cleanly without administrator elevation after install.
* **Early Diagnostics & Safe Panic Hooks**: Pre-window crash logging in `%LOCALAPPDATA%\com.aura.app\logs\aura_diagnostics.log`.

### Package Manager (WinGet)
```cmd
winget install Malashka.Aura
```

---

**Full Changelog**: https://github.com/malashkadev/aura/compare/v1.1.1...v1.2.0

[![Download Aura on SourceForge](https://a.fsdn.com/con/app/sf-download-button)](https://sourceforge.net/projects/aura-voice-typing/files/latest/download)
