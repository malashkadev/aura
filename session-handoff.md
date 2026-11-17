# Session Handoff — Aura 2.0

**Date:** 2026-11-18
**Baseline:** `main` ahead of `fadd23d` (P1–P4 audit/speedup + explicit API-key clear landed; `feat-009` in progress)
**Scope:** desktop program only; `website/` is out of scope and untouched.

## What changed

- **Earlier phases (`f8ba73a`..`a3d8f74`):** monochrome icon system, static audit gates, Phase 0–1 harness/docs, Phase 2–4 correctness/security/cleanup, and initial Phase 5 extractions (`history-view.js`, `model-cards.js`, `settings-autosave.js`, `settings-form.js`).
- **2026-11-18 P1 (Hot-path latency & idle resources):**
  - Persistent COM STA UIA worker (`aura-uia-caret`, `35 ms`) + complete skip of synthetic `Ctrl+C` (`simulate_copy`) when `ctx.has_authoritative_selection()` holds (`-260 ms`, no false line copy in IDEs or `^C` in terminals).
  - Non-destructive `GetClipboardSequenceNumber()` fallback selection probe (no `cb.clear()`, `10 ms × 20` polling) + `unrestored_clipboard` inheritance across overlapping sessions + `20 ms` non-blocking `(|)` caret arrow positioning.
  - Adaptive WASAPI tail hold (`90 ms` Toggle / `100–250 ms` Hold with early exit on `180 ms` silence), single-pass `StreamingVad` truncating trailing silence before WAV header finalization with 2-ACE DACL preserved, 16 kHz-only FIR `StreamingResampler` + recycled buffer pool, `stop_mic_meter()` thread join, event-driven `IMMNotificationClient` audio hotplug watcher, backgrounded startup audio sweep/device scan, and `cached_or_load_settings_without_secrets()` (`0` idle DPAPI calls).
- **2026-11-18 P2 (Correctness, IPC & sidecar resilience):**
  - AI selection-edit outputs bypass speech normalizer, replacements, smart-context, `(|)` caret extraction, and `is_silence_hallucination`; `ai_edit_mode == "local"` never leaks to cloud; UIPI elevation guard (`elevated-paste-blocked`) blocks pasting raw voice commands into admin windows; `edit_validate.rs` and `text_normalizer.rs` hardened.
  - Atomic `settings::update_settings` mutex; `tray-menu` narrowed from `allow-set-settings` to `set_transcription_mode` and `set_recognition_language`; dead `reprocess_history_text` removed; `open_url` HTTPS + host allowlist; `log_frontend_event` sanitized (`<= 512`, no newline/tag spoofing); `history.dpapi` corrupt-blob quarantine/recovery; HTTP/TCP timeouts + hung-server eviction on Whisper/Parakeet/Qwen3 resident servers; clean sidecar/watchdog shutdown on exit/relaunch; `WH_KEYBOARD_LL` Toggle suppression via `RECORDING_ACTIVE` (`ADR-035`) + `AURA_INJECT_TAG`.
- **2026-11-18 P3 (Frontend polish, typing & explicit API-key clear):**
  - Shared `src/sound-themes.js` with idle `AudioContext.suspend()`; zero idle `rAF`/`setInterval` in hidden `src/overlay.js`; incremental single-item prepend in `src/history-view.js`; isolated `#checkbox-gaming-mode` from 3D auto-detect; isolated uninstalled engine browsing from autosave; tray optimistic rollback + roving focus.
  - Protected saved DPAPI keys from empty-input debounce wipes and shipped quiet `.btn-clear-api-key` (`#btn-clear-key` & `#btn-clear-key-edit`) across Speech and Text tabs (9 UI locales, Visual QA verified).
  - Enabled `// @ts-check` + JSDoc across 12/13 `src/*.js` modules (`tsc -p tsconfig.json` 0 errors).
- **2026-11-18 P4 (Quality gates, behavioral tests & CI):**
  - All 10 `scripts/*.mjs` gates expanded to all `src/*.js` modules and all 3 windows, unified under single-process `scripts/run-all-audits.mjs` (`~140 ms` vs `~1225 ms`).
  - `tests/settings-autosave.test.mjs` migrated to `t.mock.timers`; added `tests/frontend-behavior.test.mjs` (`node:test` VM/DOM behavioral suite); hardened `.github/workflows/{ci,release}.yml` (`Swatinem/rust-cache`, post-test clean-tree check, SHA-256 prebuilt `cargo-audit` `v0.22.2` & `osv-scanner` `v2.3.8`, stale `RUSTSEC-2026-0258` removed).

## Verification

Run `./init.sh` (Git Bash) or `.\init.ps1` (PowerShell). Last recorded result:

- `npm run check` PASS (`lint` on 13 `src/*.js` + `tsc -p tsconfig.json` on 12/13 typed modules)
- `npm test` 75/75 PASS
- `npm run audit` 10/10 PASS (~140 ms via `scripts/run-all-audits.mjs`, 0 issues)
- `npm audit --audit-level=high` 0 vulnerabilities
- `npm run verify:manifest` 5/5 PASS
- `cargo fmt --check` & `cargo clippy --all-targets -- -D warnings` PASS (0 warnings)
- `cargo test --all-targets` 264 passed, 0 failed, 2 ignored
- `cargo audit --file src-tauri/Cargo.lock --ignore RUSTSEC-2024-0429` 0 CVE, 11 allowed warnings

## Next task

`feat-009` — form application & i18n modularization:

1. Extract settings-to-controls hydration from `loadSettings` into `src/settings-form.js` (keeping `// @ts-check` green).
2. Keep engine/audio/model/GPU orchestration in `main.js`, then extract `i18nDict` into its own module.
3. Run `npm run check`, `npm test`, `npm run audit`, `cargo test` after each extraction, then proceed to `feat-010` (`lib.rs` domain split) and `feat-012`.

## Constraints

- Do not push to GitHub.
- Do not touch `website/`.
- Keep the single-process static audit gate (`npm run audit`), `tsc -p tsconfig.json` (`npm run check`), and behavioral tests (`npm test`) green.

## Known gaps

- `cargo audit` keeps `--ignore RUSTSEC-2024-0429` for `glib 0.18.5` (pulled into `Cargo.lock` by Tauri v2's Linux-only `wry`/`webkit2gtk`/`gtk` target dependencies; never compiled on Windows) and reports 11 allowed warnings, 0 CVEs.
- Local `osv-scanner` (`v2.3.8`) flags new upstream `RUSTSEC-2026-0543` (`chrono 0.4.45` unmaintained) and `RUSTSEC-2026-0486` (`rustix 1.1.4` -> `1.1.6`) plus unused ignore `GHSA-qwgh-2vcv-g2f7` in `osv-scanner.toml`; bump `rustix` and refresh `osv-scanner.toml` before pushing to GitHub.
- `tests/icon-legacy-catalog.test.mjs` remains intentionally separate from `tests/icon-system.test.mjs`.
