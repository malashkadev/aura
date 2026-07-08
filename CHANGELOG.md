# Changelog

All notable changes to Aura are documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/), versions follow [SemVer](https://semver.org/).

## [1.0.4] — 2026-07-08

### Fixed
- The v1.0.3 release built and published successfully but was missing `createUpdaterArtifacts: true` in `tauri.conf.json` — Tauri silently skipped generating the signed updater bundle (`.zip`, `.sig`, `latest.json`) even though the signing key was present. In-app one-click updates therefore couldn't find an update yet; the app fell back to opening the release page in the browser, so nothing was broken for users, just not fully wired. Fixed by adding the flag; this release is the first with working in-app updates.

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
