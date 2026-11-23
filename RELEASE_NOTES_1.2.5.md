### ⚡ Hot-Path Latency Speedup (`-450..850 ms`)
* **Instant Selection Capture**: Persistent COM STA UIA worker (`aura-uia-caret`, `35 ms`) skips synthetic `Ctrl+C` whenever authoritative Win32 or UIA selection is available (`-260 ms`), eliminating phantom line copies in IDEs and `^C` interrupts in terminals.
* **Non-Destructive Clipboard Probe**: Uses `GetClipboardSequenceNumber()` fallback polling without clearing the clipboard, preserving Explorer files, Excel cells, and rich formats.
* **Adaptive Audio Capture & Zero-Idle Polling**: Adaptive WASAPI tail hold (`90 ms` Toggle / `100–250 ms` Hold with early silence exit), single-pass `StreamingVad`, 16 kHz FIR `StreamingResampler` with recycled buffer pool, event-driven `IMMNotificationClient` audio hotplugging, and `O(1)` cached non-secret settings reads (`0` idle DPAPI calls).

### 🧠 Overhauled Voice Selection Editing & AI Accuracy
* **Single-Pass `FixGrammar` & 32 Translation Languages**: Expanded `edit_intent.rs` to 32 target languages, added `UnsupportedTranslateTarget` safe rejection, removed conflicting prompt tails, and preserved non-Russian Cyrillic (`uk`, `be`, `sr`, `kk`, etc.), Latin, and CJK languages during proofreading.
* **Dedicated Custom Edit Model**: Split `custom_edit_model_name` from speech `custom_model_name` across Settings and cloud providers, and strictly isolated `ai_edit_mode == "local"` from unintended cloud fallback.
* **Safe Dictation Isolation & Context Adaptation**: `PassThroughDictation` preserves normal dictation over selected text, `adapt_edit_result_to_selection_context` restores whitespace boundaries cleanly, and UIPI elevation guards protect admin windows.

### 🎨 6 HUD Overlay Visualizers & Sound Themes
* **6 Animated HUD Styles (`#card-overlay`)**: Choose between `Wave`, `Matrix`, `Scale`, `Tape`, `Beam`, and `Ribbon` recording visualizers directly in Hotkeys settings alongside the recording timer toggle.
* **Refreshed Procedural Sound Themes**: Replaced `Rhodes` and `Sci-Fi` with procedural Web Audio `Xylophone` and `Pebble` themes, suspended cleanly on idle (`AudioContext.suspend()`).
* **Monochrome SVG Icon System**: Unified monochrome SVG iconography across all settings tabs and the High-DPI tray menu.

### 🔒 Privacy, Reliability & Security Hardening
* **Explicit DPAPI Key Controls & Backup Compatibility**: Quiet `.btn-clear-api-key` buttons across Speech and Text tabs protect saved keys from accidental debounce wipes; importing `v1.2.0` settings backups seamlessly fills defaults and migrates legacy sound themes.
* **Transient Clipboard Privacy & Model Guards**: Dictation inserts are marked transient to avoid Win+V and cloud clipboard history; broken/missing models are rejected before saving, active models cannot be deleted, toasts are dismissible, and Whisper/OpenAI prompt echoes on silence are discarded.
* **Log ACL Self-Healing & Security Audits**: Added `CONTAINER_INHERIT_ACE | OBJECT_INHERIT_ACE` directory ACL inheritance, 2-step log self-healing, `llm_runner` lock-free process teardown, HTTPS-only custom URLs, and upgraded Rust dependencies (`cookie 0.18.3`, `url 2.5.9`, `rustix 1.1.6`).

### Package Manager (WinGet)
```cmd
winget install Malashka.Aura
```

---

**Full Changelog**: https://github.com/malashkadev/aura/compare/v1.2.0...v1.2.5

[![Download Aura on SourceForge](https://a.fsdn.com/con/app/sf-download-button)](https://sourceforge.net/projects/aura-voice-typing/files/latest/download)
