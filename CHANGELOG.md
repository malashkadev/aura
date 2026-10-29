# Changelog

All notable changes to Aura are documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/), versions follow [SemVer](https://semver.org/).

## [1.2.0] — 2026-10-30

_First major public release since v1.1.1, consolidating all performance, voice selection editing, context awareness, and design system milestones._

### Added
- **Voice Selection Editing & AI Commands**: Select any text in any Windows application (editor, browser, messenger, IDE, office suite), hold the global hotkey (`Alt + V`), and speak a command — Aura processes the selected text directly in-place:
  - **Deterministic Intent Parsing (`edit_intent.rs`)**: Canonical commands for `FixGrammar` (*"fix grammar"*, *"check punctuation"*, *"correct typos"* across all supported languages) and `Translate` (*"translate to English"*, *"translate to Spanish"*, *"translate to German"* across all supported languages), with clean passthrough for regular dictation without LLM hallucinations or delay.
  - **Output Quality & Echo Gate (`edit_validate.rs`)**: Validates editing results by target alphabet match and length invariants, rejecting echo leaks before touching user document.
  - **Supported Commands Guide**: Balanced two-column interactive guide in Settings with syntax chips across 9 languages and a dedicated target language selector right under the translation command.
  - **Diffuse Contour Glow**: Warm amber contour glow around the HUD overlay signaling active selection edit mode.
- **Smart Context-Aware Text Insertion (`smart_insertion.rs` & `caret_context.rs`)**: Two-layer hybrid text insertion adapting letter casing (lowercasing mid-sentence while strictly preserving uppercase acronyms like NASA, API, HTTP, UUID and English pronoun "I"), trailing punctuation, and boundary spacing based on surrounding caret context (<0.05ms) via native Win32 messages (`EM_GETSEL`/`WM_GETTEXT` on Edit, RichEdit, Scintilla) with Windows UI Automation fallback. Includes master toggle and trailing period continuation toggle in the Text settings tab across all 9 UI locales.
- **Cursor Marker in Text Replacements (`|`)**: a `|` inside a replacement or snippet sets the final caret position after insertion (e.g. `Hello, |!`); the first marker wins, later pipes stay literal. Implemented in the shared normalizer (`text_normalizer.rs`), applied to live dictation, history reprocessing and paste/streaming paths, with batch left-arrow key simulation conforming to Windows Input Buffer Flow Control and a dedicated hint in all 9 UI locales.
- **Next-Generation HUD Overlay**: Hardware-accelerated CSS Houdini `@property --beam-angle` with dual symmetric orange rays along the outer card perimeter (`border-box`), smoothly accelerating to 1.1s during processing, seamless 550ms crossfade between recording and processing, and kinetic morphing into a compact 39×39px square with constant vector monogram scaling.
- **Interactive Spotlight Onboarding Tour**: 5-step guided walkthrough highlighting key capabilities (speech engines, language/dictionary/snippets, smart text editing, hotkeys & overlay, history & tray) with a non-overlapping Docked Footer, concentric corner geometry ($R_{\text{cutout}} = R_{\text{target}} + \text{padding}$), and dynamic narrative fallback for Parakeet/cloud modes. Replayable from the About tab.
- **Procedural Web Audio Themes & Tactical Volume Track**: 6 synthesized sound themes (Zen, Rhodes, Sci-Fi, Classic Bell, Bubble, Haptic Click) generated in code without external audio assets, ergonomic 320px volume track with live preview and vector mute button, plus WASAPI microphone hotplugging and 450ms tail hold buffer.
- **Windows Session Lock/Unlock Hotkey Recovery (`keyboard_hook.rs`)**: a message-only window with `WTSRegisterSessionNotification` clears stale hotkey latches after lock/unlock or hook pause, so the hotkey cannot stay "stuck" after a locked session (`Win+L`).
- **Early Startup Logging & Panic Hook (`logger.rs`)**: diagnostics logging now starts before the Tauri builder (pid/cwd/args) and a panic hook records panics that happen before window creation.
- **Engine Health "Starting" State (`lib.rs`, `main.js`)**: `get_engine_health` reports a `starting` flag and the first local dictation briefly waits for the resident Whisper prewarm instead of failing.
- **Portable Settings Backup & Restore**: the About tab can export a secret-free JSON copy of settings, dictionary, and text replacements through the native Windows Save As dialog (`rfd::FileDialog`), and restore it on another installation; format/version and completeness checks reject corrupt or foreign files before anything is saved.
- **First-Run Setup Card**: when local mode has no speech model installed, the General tab now surfaces a one-click Base-model download and microphone test instead of a silent first-run failure.
- **Windows Update Native Toast Notifications**: Integrated Windows desktop toast notifications (`tauri-plugin-notification` v2.4.0) alerting users when a new version is detected by the 4-hour background checker or on startup, suppressed automatically during Gaming Mode.
- **Redesigned Text Replacements UI**: Two-block chip cards (`.replacement-trigger` badge, arrow `→`, `.replacement-value`, `.replacement-delete` `✕`, and `.replacement-add-row` grid) replacing stretched rows, aligned with *The Stealth Command Deck* achromatic palette.

### Changed
- **Local speech recognition is now the out-of-the-box default**: `transcription_mode` defaults to `local` with the local engine radio preselected (`settings_secure.rs`); cloud AI remains an opt-in.
- **The Stealth Command Deck Design System**: Achromatic monochrome card and dialog surfaces with physical specular facets (`inset 0 1px 0 rgba(255, 255, 255, 0.08..0.12)`), achromatic frosted plates, and offline WOFF2 `Onest` typography with zero external network requests.
- **Windows 11 Fluent About Rows**: Clean interactive rows in About tab with a dedicated application exit button.
- **4-Tier Semantic Toast Capsule**: HUD capsule pill (`9999px`) with distinct icon semantics (`✓` for success, trash for deletion, `(i)` for in-progress operations, `!` for errors).
- **Accessibility pass**: custom selects expose `aria-controls`, `aria-activedescendant`, labelled listboxes, and per-option ids/selection state; the spotlight tour makes the settings content inert, traps Tab focus, and restores focus on close; full keyboard arrow navigation across tabs with visible `:focus-visible` focus rings.
- **Whisper prewarm parity**: removed the unreachable `whisper_keep_in_memory` setting; Whisper now prewarms unconditionally like Parakeet, eliminating a dead configuration field and settings drift.
- **Release quality gates**: CI and the release workflow run `npm audit` alongside lint, typecheck, frontend tests, and static audits; a new test guards the frontend `Settings` round-trip.

### Security
- **Diagnostic report path redaction**: exported diagnostics replace the current Windows profile prefix (both slash styles) with `%USERPROFILE%` and keep unrelated account names intact.
- **Backup secret isolation**: exported settings never include API keys; imported files have API-key fields stripped and are size/completeness validated before `set_settings`.
- **RustSec Dependency Vulnerability Upgrades**: upgraded `zip` 2.4.2 to `9.0.5` (resolving RUSTSEC-2026-0367, 0368, 0369), `rustls-webpki` 0.103.14 to `0.103.16` (RUSTSEC-2026-0382), `webpki-root-certs` 1.0.9 to `1.0.10` (RUSTSEC-2026-0383), and reconfigured `tauri-plugin-updater` with `default-features = false, features = ["rustls-tls"]` to prune vulnerable transitive zip 4.6.1.
- **Use-After-Free in Windows Security DACL (`secure_storage.rs`)**: Eliminated dangling pointer UAF (`SEC-01`) by copying the security identifier (SID) before dropping the token information buffer.

### Fixed
- **Layout-Switcher Key Replay Suppression (Caramba Switcher / PowerToys)**: third-party keyboard remappers and layout switchers replaying buffered hotkey characters after `Alt+V` are neutralized via `is_unicode_injection` (`VK_PACKET = 0xE7`), a 1500 ms `INJECTED_GUARD_UNTIL_MS` window, and Latin ↔ Cyrillic row counterpart mapping, while allowing real `Ctrl+V` clipboard pastes.
- **Win32 Alt Key Sticking & Ctrl+V Hook Isolation**: Fixed inverted `GetAsyncKeyState` check in `restore_modifiers()` in `keyboard_simulator.rs` and eliminated synthetic `KeyDown(VK_MENU)`. Added strict exact modifier mask in `keyboard_hook.rs` ensuring hotkey `Alt+V` cannot falsely trigger on standard `Ctrl+V`.
- **Hotkey Auto-Repeat Leak (`keyboard_hook.rs`)**: session finalization is deferred until the target key is released (a `KEY_SUPPRESSED` latch covers keys swallowed by the hook), so held-key auto-repeat can no longer start or stop a session spuriously in toggle or hold mode.
- **Filler-Loop & Trailing Silence Hallucinations**: repeated filler loops (e.g. `hmmmmm...`, `ummmmm...`) are classified as silence (with 2-letter genuine words preserved), and trailing YouTube closure artifacts ("Thank you for watching.", "Thanks for watching.") are cleanly stripped while strictly preserving stand-alone gratitude.
- **Speech Pause Hesitation Ellipsis Cleanup**: Integrated `clean_speech_pause_ellipses()` in `text_normalizer.rs` to automatically transform hesitation ellipses (`...` / `…`) generated by Whisper during natural pauses into commas before conjunctions, spaces mid-sentence, or terminal periods.
- **UTF-8 Slicing Panic in Normalizer (`text_normalizer.rs`)**: Fixed byte boundary slicing panic (`FAULT-01`) when speech hallucination prefixes contain 3-byte UTF-8 em-dashes (`—`). Made subtitle prefix case-insensitive search fully Unicode-safe.
- **Latent Multi-byte Ellipsis Slicing Panic (`text_normalizer.rs`)**: Fixed byte boundary slicing panic (`FAULT-02`) on multi-byte horizontal ellipsis `…` in sentence repetition pruning (`prune_trailing_repeated_sentences`).
- **Gaming Mode Active Recording Cancellation (`lib.rs`)**: Automatically and cleanly cancels active speech recording sessions when Gaming Mode is engaged (`CONC-01`), preventing orphaned audio captures and blocked input hooks.
- **Non-blocking Audio Device Initialization (`audio_recorder.rs`)**: Scoped `self.state` mutex to release lock before the 10-second WASAPI driver startup wait (`CONC-03`), preventing deadlocks during stop or cancel operations.
- **Whisper Server Process Termination Isolation (`whisper_runner.rs`)**: Process handle dropped outside mutex lock during server shutdown (`CONC-04`), avoiding mutex contention across threads.
- **Cloud Finalize Transcription Timeout (`lib.rs`)**: Expanded final cloud transcription timeout from 45 seconds to 600 seconds (`FAULT-03`), ensuring long audio dictations complete reliably.
- **Guarded Clipboard Injection (`lib.rs`)**: Added strict verification of clipboard write success prior to sending `Ctrl+V` keystrokes (`FAULT-04`), preventing stale clipboard data leakage if the Windows clipboard is locked.
- **AI Context Editing Error Reporting (`ai_client.rs`)**: Restored explicit error propagation on Hugging Face authorization failures and Custom API errors (`FAULT-05`) instead of silently pasting raw voice prompts over selected text.
- **Tour Spotlight Fallback (`main.js`)**: the onboarding cutout no longer jumps to the top-left corner and now prefers the step-declared alternate card (`#card-replacements`) instead of the first visible card when the primary target is hidden.
- **Engine Health Chip Synchronization (`lib.rs`, `main.js`)**: the chip is driven by the backend `starting` flag instead of a frontend timer, eliminating a "starting…" ghost state and matching the resident server boot.
- **Installer Launch After Installation (`nsis/installer.nsi`)**: finish-page launch and `/R` restart now use `nsis_tauri_utils::RunAsUser` (never elevated) with a one-shot `FindProcessCurrentUser` check and retry, fixing "Aura did not start after installation" reports.
- **Settings Persistence Drift (`main.js`, `settings_secure.rs`)**: a frontend/backend test now compares every non-secret native `Settings` field with the save payload, so a new option cannot silently reset or disappear.
- **Settings Export Destination (`main.js`, `lib.rs`)**: export now opens the native Windows Save As dialog and writes only to the path the user chooses, instead of silently downloading to the default folder.
- **Confirmation and First-Run Styling (`style.css`, `index.html`)**: the primary confirmation action now uses the Tactical Orange accent, and the first-run recommendation card uses the standard card surface, aligned 36px actions, and a compact accent icon instead of an orange outline.

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
