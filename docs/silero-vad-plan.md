# Integration plan: Silero VAD (voice activity detection)

Status: **proposal / hand-off spec** (not implemented). Self-contained brief for an AI agent
or manual implementation.

## Goal

Replace Aura's crude energy-threshold silence gate with a real **Silero VAD** — a tiny
(~2 MB) neural voice-activity detector. This makes the app skip true silence (not just quiet
audio), trim dead air before recognition, and stop wasting cloud API calls on empty chunks.

## Current state

Aura decides "is there speech?" with a plain RMS energy threshold:

- [`lib.rs`](../src-tauri/src/lib.rs): `const SILENCE_RMS_THRESHOLD: f32 = 0.005;` and the
  `rms()` helper. In the streaming loop it does `rms(&samples[new_start..]) > THRESHOLD` to
  decide whether to send a chunk.
- Problem: RMS fires on **any** sound — fans, keyboard, hum, music — so silent-but-noisy rooms
  trigger it, and genuinely quiet speech can be missed. It's the main source of "silence
  hallucinations" (handled today by the `is_silence_hallucination` blocklist, a workaround).

## Why Silero

- Distinguishes **speech** from noise, not just loud from quiet.
- Tiny ONNX model (~1.7 MB) — **bundle it directly**, no download needed (unlike Whisper models).
- Works on exactly the audio Aura already produces: 16 kHz mono f32 (see
  `process_and_write_wav` in [`audio_recorder.rs`](../src-tauri/src/audio_recorder.rs)).
- Processes 32 ms frames (512 samples @ 16 kHz), outputs a speech probability per frame.

## Where it helps (three wins)

1. **Streaming loop** — replace `rms() > SILENCE_RMS_THRESHOLD` with `vad::has_speech(new_audio)`.
   Fewer wasted API calls, better latency, respects Groq rate limits.
2. **Final transcription** — trim leading/trailing silence before sending to Whisper/cloud →
   faster, far fewer hallucinations on silent tails. Lets you shrink or drop the
   `is_silence_hallucination` blocklist over time.
3. **Overlay UX (optional)** — could drive a "listening / silence" hint.

## Approach A — in-process Rust module (recommended, do this first)

Add a `vad` module; keep it engine-agnostic so it benefits **whisper AND cloud** immediately.

- **Crate:** `voice_activity_detector` (Silero v5 via `ort`/onnxruntime) or `vad-rs` (the one
  Handy uses). Pick whichever builds cleanly on Windows + macOS in CI.
- **Model:** ship `silero_vad.onnx` (~1.7 MB) in `src-tauri/resources/` and load via
  `resource_dir()` (mirror how `find_sidecar` resolves paths). No network download.
- **API to add** (`src-tauri/src/vad.rs`):
  ```rust
  /// True if any 16 kHz-mono frame in `samples` exceeds the speech-probability threshold.
  pub fn has_speech(samples: &[f32]) -> bool;
  /// Returns samples with leading/trailing non-speech trimmed (keeps a small margin).
  pub fn trim_silence(samples: &[f32]) -> Vec<f32>;
  ```
  Run these on a blocking thread (`spawn_blocking`) like the whisper call, since ONNX inference
  is CPU work.
- **Hook points in `lib.rs`:**
  - Streaming loop: swap the `rms(...) > SILENCE_RMS_THRESHOLD` check for `vad::has_speech(...)`.
  - `finalize_recording`: run `vad::trim_silence` on the recorded samples before
    `process_and_write_wav` / before sending to the cloud.
  - Keep `rms()` only for the overlay's live volume bars (that's a fine use of RMS).
- **Settings (optional):** `pub vad_enabled: bool` (default true) + a sensitivity slider mapped
  to the probability threshold, following the existing `#[serde(default)]` pattern in
  `settings.rs`.

Effort: small. No second process, ~2 MB model, immediate benefit to every engine.

## Approach B — via sherpa-onnx (comes bundled with Parakeet)

`sherpa-onnx` (the runtime proposed for Parakeet in
[`parakeet-integration-plan.md`](parakeet-integration-plan.md)) **ships Silero VAD** and has a
combined **VAD + ASR** offline mode: it segments audio internally and only transcribes speech.

## Can it be combined with Parakeet? — Yes, and here's the relationship

- **They're complementary, not either/or.** Silero VAD is a *pre-filter*; Parakeet is a
  *recognizer*.
- If you add Parakeet through sherpa-onnx, you get its built-in VAD **for free on the Parakeet
  path** — no extra work there.
- **But** that VAD only covers the Parakeet path. The **cloud engines and whisper.cpp** still
  benefit from a standalone VAD. So Approach A is still worth doing even if Parakeet lands.
- **Recommended sequence:**
  1. **Silero VAD (Approach A) first** — small, independent, helps the current whisper + cloud
     pipeline right now, and is not blocked by anything.
  2. **Parakeet later** — when you wire sherpa-onnx, let its combined VAD+ASR handle the
     Parakeet path; the Approach-A module keeps serving whisper/cloud.
  This way one shared crate/runtime philosophy covers everything and there's no wasted work.

## Testing

- Unit: feed a synthetic silent buffer and a synthetic tone/speech sample to `has_speech` /
  `trim_silence` and assert the expected boolean/trim (pure functions, same style as the
  existing `test_rms` and `test_process_and_write_wav`).
- CI: if using an `ort`-based crate, make sure onnxruntime links on both Windows and
  macos-latest (the `build-macos` job will catch link errors).
- Manual: dictate with a noisy fan running — old RMS gate would fire on the fan; VAD should not.

## Gotchas

- **onnxruntime linkage** — `ort` may need a bundled or system onnxruntime; pin the feature that
  downloads/links a static lib so CI stays reproducible on both platforms.
- **Frame size** — Silero expects fixed frame sizes (256/512 samples at 8/16 kHz). Aura is
  already 16 kHz mono, so feed 512-sample frames.
- **Threshold tuning** — start at 0.5 speech probability; expose as a sensitivity setting.
- **Don't over-trim** — keep ~200 ms of margin around detected speech so word onsets aren't clipped.
