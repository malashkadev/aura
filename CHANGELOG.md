# Changelog

All notable changes to Aura are documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/), versions follow [SemVer](https://semver.org/).

## [1.1.1] — 2026-09-15

### Added
- **Dutch (nl) Transcription Support**: Added Dutch (`nl` / Nederlands) as a fixed recognition language across Whisper local engine, cloud providers, and the system tray menu. Includes Windows Dutch keyboard layout detection (`0x0413` nl-NL, `0x0813` nl-BE) and native punctuation guidance.

### Fixed
- **Whisper Auto-Detect Prompt Poisoning (#5)**: Eliminated Cyrillic prompt leakage when `Auto-detect` is selected. In previous versions, the wildcard fallback in `build_whisper_prompt` defaulted to Russian text, which biased Whisper's decoder priors and frequently misclassified short non-Russian speech (such as Dutch) as Russian. Auto-detect now passes an empty base prompt (or user dictionary terms only).

## [1.1.0] — 2026-09-08

### Added
- **DirectX / Vulkan Gaming Mode**: Added automatic detection of fullscreen 3D games via Win32 `SHQueryUserNotificationState` and `MonitorFromWindow`. Automatically unloads resident Whisper/Parakeet processes to free 100% VRAM and pauses low-level keyboard hooks (`HOOK_PAUSED`) to ensure zero input latency. Includes dedicated orange tray icon (`32x32_gaming.png`) and manual override toggle.
- **Standby VRAM Unloader**: Automatic model unloading and VRAM release after configurable idle timeout (15, 30, 60 minutes or disabled), with instant background warm-up on next dictation hotkey press.
- **Dedicated Text Replacements & Dynamic Macros Tab**: Added 6th settings tab «Text» featuring user-defined replacement rules and live dynamic macros (`{date}`, `{date_ru}`, `{time}`, `{datetime}`). Text substitution uses atomic Win32 queue flushing to eliminate character dropouts across all applications.
- **300-Entry History Journal & Day Grouping**: Expanded history storage to 300 entries with date grouping («Сегодня», «Вчера», calendar dates), live search, and cloud/local source filters.
- **WASAPI Audio Device Hotplug**: Automatic recovery and seamless reconnection when USB or Bluetooth headsets/microphones are plugged in or disconnected during operation.
- **Custom High-DPI Tray Menu**: Custom drill-down tray menu with engine switching, Gaming Mode toggle, and 11 recognition languages, dynamically positioned via Win32 `MonitorFromPoint` and `GetMonitorInfoW` to eliminate clipping across 4K displays and mixed-DPI multi-monitor arrangements.

### Fixed
- Fixed tray menu positioning and coordinate clipping on secondary displays with differing Windows scaling factors.
- Cleaned up obsolete CSS classes and reinstated the strict Zero Shadows design invariant across all application components.
- Extended audio recording tail buffer to 450ms, eliminating trailing word truncation on natural speech pauses.

## [1.0.9] — 2026-08-29

### Added
- **Audio Input Device Selection**: Users can now select their physical microphone device directly in the Speech settings card. Added backend device enumeration with `cpal` in `src-tauri/src/audio_recorder.rs` and exposed `get_audio_input_devices` IPC command with DPAPI-backed persistence.
- **Context Editing AI Indicator (3 Sparkle Stars + «Редактирование»)**: When selection context editing is active, the overlay displays a crisp 3-star vector sparkle icon with localized «Редактирование» / «Edit» status in `src/overlay.html` and `src/overlay.js` across all 10 UI locales.
- **Google Gemini 3.6 Flash Multimodal Integration**: Upgraded Gemini cloud provider integration to the latest multimodal `gemini-3.6-flash` model with automatic version fallback cascade, enabling instant speech recognition and voice editing in a single network request.
- **Topmost Z-Order Guard**: Added `ensure_overlay_topmost` in `src-tauri/src/lib.rs` using Win32 `SetWindowPos(HWND_TOPMOST)` during recording session start to prevent full-screen or high-priority foreground windows from obscuring the overlay.

### Fixed
- **Post-Release Audio Grace Buffer (Tail Hold)**: Added a 160ms recording tail buffer upon hotkey release in `src-tauri/src/audio_recorder.rs` and widened the Silero VAD silence trimming margin to 350ms in `src-tauri/src/vad.rs`, completely eliminating trailing word clipping and truncated final syllables.
- **Robust Focus Guard & Safe Clipboard Handoff**: Enhanced focus tracking in `src-tauri/src/keyboard_simulator.rs` to verify matching window HWND, root top-level parent HWND (`GA_ROOT`), and target PID. If focus changes to an incompatible window mid-transcription, typing is safely aborted and the recognized text is placed on the clipboard with an overlay notification across all 10 UI locales.
- **Reliable Selection Capture**: Stabilized simulated `Ctrl+C` keystrokes in `src-tauri/src/keyboard_simulator.rs` with 30ms modifier delays, preventing clipboard race conditions in editors and web browsers.
- **Instant Overlay Dismissal Timing**: Eliminated the 1–2s visual delay after text paste by firing overlay hide and chime events immediately upon document insertion.
- **Provider-Aware Settings UI**: The selection editing card dynamically adapts to cloud provider capabilities, hiding for Hugging Face (which limits free tokens to ASR only) and enabling for Gemini, Groq, OpenAI, and Custom servers.

## [1.0.8] — 2026-08-16

### Added
- **GPU Acceleration Architecture (NVIDIA CUDA)**: Added hardware acceleration support for the local Parakeet engine in `src-tauri/src/whisper_runner.rs` and `src-tauri/src/lib.rs`. The runtime dynamically detects CUDA support, validates required system DLLs, provides on-demand background downloading with SHA-256 integrity verification, and automatically falls back to CPU execution if initialization fails or an NVIDIA GPU is unavailable.
- **Windows DPAPI Credential & History Protection**: Migrated API keys and transcription history storage to native Windows Data Protection API (`CryptProtectData` / `CryptUnprotectData`) with restricted DACLs in `src-tauri/src/secure_storage.rs`, `src-tauri/src/settings_secure.rs`, and `src-tauri/src/history_secure.rs`. File permissions are strictly locked to the current Windows user and SYSTEM, preventing unauthorized access by other processes.
- **Stable Prefix Locking for Live Streaming**: Implemented stable prefix locking and smart word-level diffing in `src-tauri/src/lib.rs`. Live streaming predictions now preserve committed tokens and incorporate Cyrillic `ё`/`е` tolerance and casing normalization during pauses, eliminating text jitter, typing desync, and duplicate words.
- **Unified Diagnostics & Anonymized Log Exporter**: Added an asynchronous rolling file logger and diagnostic exporter in `src-tauri/src/logger.rs` and `src-tauri/src/lib.rs`. It enables 1-click troubleshooting report generation with automated regex-based redaction of user speech, API keys, and usernames across all 9 UI locales.
- **Custom Accessible Dropdown Panels**: Replaced unstylable native browser select elements with animated sliding dropdown panels in `src/main.js` and `src/style.css`. Includes live page scroll tracking, full keyboard navigation (Arrow keys, Enter, Esc), and localized preview captions with engine health indicators.

### Fixed
- **Resident Sidecar Lifecycle Management & Watchdog**: Hardened the Parakeet WebSocket sidecar daemon in `src-tauri/src/whisper_runner.rs` using Windows Job Objects (`KillOnJobClose`), pipe reader cleanup routines, and an automatic restart watchdog under a re-entrant lifecycle lock to prevent orphaned background processes.
- **VAD Trailing Frame Zero-Padding**: Added zero-padding for trailing audio frames in `src-tauri/src/vad.rs` to satisfy Silero VAD's fixed 512-sample chunk requirement, ensuring that short final words or endings are never clipped during batch silence trimming.
- **Authoritative Audio WAV Fallback**: Implemented atomic `.partial` RAII WAV writing in `src-tauri/src/lib.rs`. If a streaming socket disconnects or sidecar becomes unresponsive, the full WAV recording serves as an authoritative fallback to guarantee zero transcription loss.
- **Dynamic Window-Height History Panel**: Removed the fixed 380px height cap on the history list in `src/style.css` and `src/main.js`. The panel now dynamically adapts to the full window height with smooth internal scrolling and enhanced engine/timing badges.

## [1.0.6] — 2026-07-11

### Fixed
- Added automatic whitespace and newline trimming (`.trim()`) to API keys in both frontend UI and backend before saving, preventing invalid HTTP header formatting from trailing characters copied from cloud consoles.
- Reverted the experimental secure `keyring` (system credential manager) integration. It caused credentials to disappear on some Windows configurations; keys are now safely stored back in the stable local JSON format.
- Added a robust RAII-cleanup guard (`DeleteOnDrop`) to ensure that failed, timed-out, or cancelled model downloads clean up their large temporary `.tmp` files, preventing disk space wastage.
- Fixed a bug where a connection timeout during model downloads was silently ignored, leaving corrupted files on disk instead of propagating the download error.
- Resolved a potential thread panic on Mutex lock poisoning in the Parakeet server lifecycle manager.
- Documented the Parakeet user dictionary limitation in both `README` files and source code: the model's NeMo transducer architecture only supports `greedy_search` decoding, making custom hotword biases incompatible at the engine level in `sherpa-onnx`.

## [1.0.5] — 2026-07-08

### Fixed
- The real fix for the updater manifest: `createUpdaterArtifacts: true` (added in 1.0.4) made Tauri produce signed `.sig` files, but a plain `tauri build` invocation never generates the `latest.json` manifest the updater endpoint needs, and the release workflow's asset glob patterns didn't even match the `.sig` files it did produce. Switched the release workflow from a manual `tauri build` + asset-upload step to the official `tauri-apps/tauri-action`, which builds, signs, uploads assets, and generates `latest.json` together. v1.0.4 was never published (its release draft was deleted; no user was on it) since I verified the artifacts before publishing and caught this.

## [1.0.4] — 2026-07-08 (superseded, never published)

- Added `createUpdaterArtifacts: true` — a necessary but insufficient fix; see 1.0.5.

## [1.0.3] — 2026-07-07

### Added
- Integrated **Silero VAD (Voice Activity Detection)** to trim silence and cut down silence hallucinations, replacing the old energy-threshold gate.
- Integrated **NVIDIA Parakeet** (TDT 0.6b v3) as a second local offline engine next to Whisper.cpp, running as a resident WebSocket server so the model loads once instead of per-utterance — recognition latency drops from ~12s to well under 1s.
- Offline English punctuation for local dictation via a small CT-Transformer model, auto-downloaded alongside Parakeet.
- Implemented **signed Tauri auto-updater**: releases are cryptographically signed in CI; the app checks for and can install updates in-app.

### Fixed
- Bundled the Parakeet websocket-server/punctuation sidecars as installer `resources` — they ran in `tauri dev` but were missing from the actual installer before this fix.
- `run_parakeet` now has a 30s socket read-timeout, so a hung/dead Parakeet server can no longer stall a dictation forever; a "loading model" overlay notice is shown on the first (slow) connection instead of a silent wait.

### Known limitations
- The custom dictionary is not applied when using the Parakeet engine (it runs as a resident daemon started with fixed arguments); it still works with Whisper and the cloud providers.

## [1.0.2] — 2026-07-07

### Added
- Local **Whisper Large v3 Turbo** models — full (~1.6 GB, best accuracy for RU/EN) and quantized Q5 (~550 MB, near-Turbo quality at half the size).
- Model downloads can now be **cancelled** with an (×) button on the progress bar.
- **Update indicator**: on launch the app checks GitHub for a newer release and shows a badge (with a dot on the About nav tab) linking to the release page.
- **Automatic cloud→local fallback**: if the cloud provider is unreachable (VPN block, region block, no network) the app retries the same recording with an already-downloaded local model and shows a brief notice, instead of just failing. Toggle: "Автопереключение при недоступности облака" (on by default).

### Changed
- Clearer error for HTTP 403: shows "VPN/proxy IP is blocked — turn off the VPN for Groq/OpenAI or switch server" instead of a misleading "No network".
- Cross-platform codebase: the macOS native port (CGEventTap hotkeys, CoreAudio capture) now lives in the main tree and compiles in CI.

### Fixed
- Local Whisper now uses all available CPU cores (`-t`) instead of a hardcoded 4 threads — several times faster transcription on modern many-core CPUs.
- Cancelling (or failing) a model download no longer leaves the card stuck; it can be retried immediately without restarting the app.

## [1.0.1] — 2026-07-06

### Changed
- Settings/overlay UI polish: refined modal exit animation, glassmorphic download confirmation, button sizing and accessibility tweaks.
- Bumped executable and UI version metadata to 1.0.1.

## [1.0.0] — 2026-07-05 (first public release)

### Added
- Global-hotkey dictation with push-to-talk and toggle (tap-to-latch) modes; `Esc` cancels a recording.
- Cloud engines: Gemini, OpenAI (Whisper + GPT), Groq (Whisper + Llama) with per-provider API keys.
- Local engine: whisper.cpp sidecar, model download manager (tiny/base/small/medium) with delete option.
- AI transcript cleanup: punctuation, grammar, filler-word removal; voice editing of selected text.
- Experimental live streaming mode with smart diff typing and adaptive chunk intervals.
- Transcription history (last 50 entries) with one-click copy.
- Custom dictionary hints, 9 fixed recognition languages + auto/keyboard-layout detection.
- Optional voice punctuation commands (Russian).
- Overlay with live waveform, recording timer, localized status/error messages and sound themes (zen / rhodes / sci-fi / classic) with volume control.
- Autostart with Windows, tray icon, single-instance guard.
- Focus guard: simulated typing/paste never lands in a window the user switched to mid-dictation.
- Windows system proxy support and network timeouts for cloud requests.

### Fixed
- Silence-hallucination filter no longer discards legitimate dictation containing marker words.
- Clipboard contents can no longer leak into cloud prompts when nothing is selected.
- Race between the streaming loop and final transcription (session generation counter).
- Blocking whisper.cpp/typing calls moved off the async runtime.
- Sidecar discovery works in dev and bundled builds; runtime DLLs ship with the installer.
- Model name validation prevents directory traversal.
- Alt-based hotkeys no longer steal focus from the active input field (e.g. in browsers) by disarming the menu-activating Alt release.
