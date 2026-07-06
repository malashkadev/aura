# Integration plan: local NVIDIA Parakeet (via sherpa-onnx)

Status: **proposal / hand-off spec** (not implemented). This document is a self-contained
brief you can hand to an AI coding agent or implement yourself.

## Goal

Add a second **local** speech-to-text engine — NVIDIA **Parakeet TDT 0.6b v3** — alongside
the existing whisper.cpp engine. Parakeet v3 is small (0.6B params), fast on CPU, ships
built-in punctuation & capitalization, and supports 25 European languages **including
Russian and English**. It tops the open ASR leaderboard (~6.3% avg WER).

The user picks the local engine in Settings; everything downstream (recording, overlay,
LLM cleanup, history) stays the same.

## Why sherpa-onnx (chosen approach)

Aura already runs whisper.cpp as an **external sidecar process** (see
[`whisper_runner.rs`](../src-tauri/src/whisper_runner.rs): `find_sidecar`, `run_local_whisper`,
`externalBin` in `tauri.conf.json`). The cleanest path is to reuse that exact pattern with a
second sidecar: [**sherpa-onnx**](https://github.com/k2-fsa/sherpa-onnx) (Apache-2.0).

- Prebuilt CPU binaries for Windows/macOS/Linux, no CUDA required.
- Runs Parakeet transducer ONNX models offline.
- Ships a Silero VAD too (bonus — see "Related" below).
- Out-of-process = an ML runtime crash can't take down the Tauri app.

**Alternative (Option B):** embed the [`transcribe-rs`](https://crates.io/crates/transcribe-rs)
crate (the one Handy uses) directly in Rust — no second process, but heavier build/linking
and a bigger binary. Prefer sherpa-onnx unless you specifically want in-process.

## Deliverables / files to touch

1. **`src-tauri/binaries/`** — add the sherpa-onnx offline-recognizer executable per platform
   (same naming convention Tauri expects for `externalBin`, e.g.
   `sherpa-onnx-offline-x86_64-pc-windows-msvc.exe`). Register it in `tauri.conf.json`
   → `bundle.externalBin` next to `whisper-sidecar`. Its runtime `.dll`/`.dylib`s go in
   `bundle.resources` like the whisper DLLs already do.

2. **Model download** — Parakeet ships as a *folder* of ONNX files, not a single `.bin`:
   `encoder.onnx`, `decoder.onnx`, `joiner.onnx`, `tokens.txt` (int8-quantized package
   ≈ 600 MB). Grab the **v3** package from the sherpa-onnx model releases
   (https://github.com/k2-fsa/sherpa-onnx/releases — look for
   `sherpa-onnx-nemo-parakeet-tdt-0.6b-v3` / int8). Extend the download manager to fetch and
   unzip an archive into `%LOCALAPPDATA%/com.aura.app/models/parakeet-v3/` (the current
   `download_model` in `whisper_runner.rs` downloads a single file — add a sibling that
   downloads + extracts a zip, and reports progress the same way via the
   `model-download-progress` event).

3. **`settings.rs`** — add a field, e.g. `pub local_engine: String` (`"whisper"` | `"parakeet"`),
   default `"whisper"`. Follow the existing `#[serde(default)]` + `Default` pattern so old
   settings files keep loading.

4. **New runner** — add `run_parakeet(app_handle, wav_path, language, dictionary)` (mirror
   `run_local_whisper`'s signature and `spawn_blocking` usage from
   [`lib.rs`](../src-tauri/src/lib.rs): `run_local_whisper_async`). It shells out to the
   sherpa-onnx sidecar. Approximate CLI shape (verify flags against the current sherpa-onnx
   docs — they evolve):

   ```
   sherpa-onnx-offline \
     --encoder=<dir>/encoder.onnx \
     --decoder=<dir>/decoder.onnx \
     --joiner=<dir>/joiner.onnx \
     --tokens=<dir>/tokens.txt \
     --model-type=nemo_transducer \
     --num-threads=4 \
     <wav_path>
   ```

   Parse recognized text from stdout (sherpa prints structured lines — extract the
   transcript field). Reuse the Windows `CREATE_NO_WINDOW` flag and `get_short_path`
   handling already in `whisper_runner.rs`.

5. **Dispatch** — in `lib.rs` where the code currently branches on
   `settings.transcription_mode == "local"` and calls `run_local_whisper_async(...)`
   (there are **two** call sites: the streaming loop and `finalize_recording`), branch again
   on `settings.local_engine`: `"parakeet"` → `run_parakeet_async`, else the existing whisper
   path. Language: Parakeet auto-detects; still pass the resolved `effective_language` if you
   later want to force it. Note Parakeet already returns punctuation, so consider skipping the
   voice-punctuation post-process for it.

6. **Frontend** — in the "Local model" card ([`src/index.html`](../src/index.html)) add an
   engine toggle (Whisper ⇄ Parakeet) above the model cards, and show the Parakeet
   download/size card when Parakeet is selected. Wire it exactly like the existing
   `.model-card` handlers in [`src/main.js`](../src/main.js) (they're generic over
   `data-model`) and add a `settings.local_engine` load/save line next to `model_name`.
   Add i18n keys for the new labels to all 9 language dicts.

7. **`get_downloaded_models`** — currently scans for `ggml-*.bin`. Add detection for the
   Parakeet folder (presence of `parakeet-v3/encoder.onnx`) so the UI shows it as installed.

## Testing

- Unit: add a stdout-parsing test for the sherpa-onnx output format (pure string function,
  no binary needed — same style as the existing tests).
- CI: the macОС/Windows compile jobs need the new `externalBin` to *exist* at build-script
  time — extend the placeholder step in `.github/workflows/ci.yml` to also `touch` the
  sherpa-onnx sidecar names (mirrors what we already do for the whisper sidecar).
- Manual: download the model from Settings, dictate RU and EN, compare latency/accuracy vs
  `whisper-large-v3-turbo`.

## Gotchas

- **Model is a folder, not one file** — the whole download/"is it installed?"/delete flow
  assumes a single `.bin` today. This is the biggest change.
- **Sidecar size** — sherpa-onnx + onnxruntime add ~30–60 MB to the installer; the model is a
  separate ~600 MB download (fine — same as large models).
- **Exact CLI flags & model package name** drift between sherpa-onnx releases — pin a version
  and verify against its docs before wiring the args.
- **License** — sherpa-onnx is Apache-2.0 and the Parakeet weights are CC-BY-4.0; both are
  compatible with Aura's AGPL-3.0 (add attribution in NOTICE/README).

## Related quick win (do this first, it's cheaper)

sherpa-onnx also bundles **Silero VAD**. Even before Parakeet, swapping Aura's current
energy-threshold silence gate (`SILENCE_RMS_THRESHOLD` in `lib.rs`) for a real VAD would trim
pauses, cut hallucinations, and reduce cloud API calls — smaller effort, immediate payoff.
