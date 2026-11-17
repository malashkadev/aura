# Aura 2.0 — Progress

**Updated:** 2026-11-18
**Baseline:** `main` ahead of `fadd23d` (P1–P4 full-stack audit & speedup + explicit API-key clear landed; Phase 5 `feat-009` in progress)
**Status:** green — desktop working tree verified across all frontend, static-audit, security, and Rust gates (`website/` untouched).

## Current state

The program is at version 1.2.0 with the monochrome icon system, hardened single-process static-audit gates, Phase 2–4 correctness/security fixes, and the 2026-11-18 full-stack P1–P4 audit & latency speedup plus explicit DPAPI API-key clear controls landed. The desktop app is in scope; `website/` is intentionally out of scope.

Completed in earlier commits:

- `f8ba73a` — monochrome icon system, settings polish, design-lab, legacy icon registry moved out of `src/`.
- `793579f` — every static audit script fails on violations; `npm test` runs all `tests/*.test.mjs`.
- `fa02b6a` — Phase 0–1: `init.sh`/`init.ps1`, roadmap, feature tracker, progress, handoff, tracked and rewritten product docs.
- `1d0f204` — custom selects stay readable before enhancement and when settings fail.
- `f04c0c2` — raw backend errors replaced with translated messages; detail stays in the console log.
- `1fd16fc` — autostart registration failures are surfaced as a non-fatal warning instead of a false success.
- `404363d` — engine-health polling reads non-secret settings, so DPAPI is no longer decrypted every 10 seconds.
- `8e842ab` — custom API endpoints require HTTPS outside loopback; embedded credentials are rejected.
- `7978a47` — stale `aura-rec-*`/`aura-chunk-*` files are swept at startup; WAV files get the current-user/SYSTEM ACL.
- `8580b3e` — `clipboard-win` 5.4.3 and `async-executor` 1.14.1 replace the yanked versions.
- `19eb873` — removed the unreachable punctuation pipeline, its 12 MB sidecar, diagnostics fields, and UI branches.
- `ff6e1ac` — production icon-registry tests added; the legacy catalog suite keeps its own file.
- `cac731f` — 2026 audits, handoffs, and E2E checklist moved to `docs/history/`; New Design assets moved under `docs/design/`.
- `31a82ae`, `2e05c6e`, `2ede29f`, `8dc69e0`, `a3d8f74` — extracted `history-view.js`, `model-cards.js`, `settings-autosave.js`, and `settings-form.js`; moved view factories inside `DOMContentLoaded`.

Completed in the 2026-11-18 P1–P4 audit, speedup & UX cycle:

- **P1 — Hot-path latency & zero-idle overhead:**
  - Persistent COM STA UIA thread (`aura-uia-caret`) in `caret_context.rs` (`35 ms` warm capture) and complete skip of synthetic `Ctrl+C` (`simulate_copy`) whenever `ctx.has_authoritative_selection()` holds (`-260 ms`, zero false line copies in IDEs or `^C` in terminals).
  - Non-destructive fallback clipboard probe using `GetClipboardSequenceNumber()` + `simulate_copy_checked()` (`10 ms × 20` polling without `cb.clear()`, preserving `ClipboardBackup::Preserve` Explorer files/Excel/RTF) and `unrestored_clipboard` inheritance across overlapping sessions in the `400 ms` settle window.
  - Adaptive WASAPI tail hold (`90 ms` in Toggle mode; `100 ms` min / early exit on `180 ms` silence / `250 ms` cap in Hold mode), single-pass incremental `StreamingVad` in `recorder_worker` truncating trailing silence before WAV header finalization while preserving the restrictive 2-ACE DACL, 63-tap FIR `StreamingResampler` evaluated only at 16 kHz output instants with recycled WASAPI buffer pool (`0` steady-state callback allocations), and `stop_mic_meter()` thread join.
  - Early `(|)` caret left-arrow positioning (`20 ms` post-paste in `spawn_blocking` guarded by `session_gen` and `FocusSignature`), instant overlay display parallel to WASAPI open, `~30 Hz` targeted `emit_to("overlay", "volume-level")`, event-driven Windows `IMMNotificationClient` audio hotplug watcher, backgrounded startup `sweep_stale_audio`/device scan, and `cached_or_load_settings_without_secrets()` (`0` idle DPAPI/`lsass.exe` calls).
- **P2 — Correctness, IPC least privilege, sidecar resilience & security:**
  - AI selection-edit outputs (`executed_edit_target.is_some()`) bypass speech normalizer, text replacements, smart-context adaptation, `(|)` caret extraction, and ASR `is_silence_hallucination` filter; `ai_edit_mode == "local"` strictly forbids cloud fallback (`edit-failed`); UIPI elevation guard (`elevated-paste-blocked`) prevents pasting raw voice commands into admin windows; `edit_validate.rs` blocks prompt-echo and stripped `<code>` blocks; `text_normalizer.rs` restricts trailing polite-signoff stripping to pure short utterances and expands number normalization.
  - Serialized `settings::update_settings` mutex across all settings writers; narrowed `capabilities/tray-menu.json` from `allow-set-settings` to atomic `set_transcription_mode` and `set_recognition_language`; removed dead `reprocess_history_text` and orphaned permissions.
  - Enforced HTTPS + trusted host allowlist in `open_url`, newline/bracket/length (`<= 512`) sanitization in `log_frontend_event` (`logger.rs`, `lib.rs`, lock-free hook logging), `install_app_update` single-flight `AtomicBool`, `cancel_model_download("llm-runtime-*")`, `cuda-archives`/`*.part` cleanup, `history.dpapi` corrupt-blob quarantine/recovery, `TempWavGuard` RAII, restrictive DACL on `last_transcription_error.txt`, HTTP/TCP socket timeouts + hung-server eviction on Whisper/Parakeet/Qwen3 resident servers, clean sidecar/watchdog exit on `RunEvent::Exit`/`exit_app`/`relaunch_app`, and `WH_KEYBOARD_LL` Toggle suppression via `RECORDING_ACTIVE` (`ADR-035`) + `AURA_INJECT_TAG`.
- **P3 — Frontend polish, shared audio synth, JSDoc typing & explicit API-key clear:**
  - Extracted `src/sound-themes.js` with idle `AudioContext.suspend()`; eliminated idle `60 FPS` `requestAnimationFrame` and `500 ms` `setInterval` in hidden `src/overlay.js`; paused hidden-window polling/audio on `visibilitychange`; added incremental single-entry DOM prepend in `src/history-view.js`.
  - Isolated manual `#checkbox-gaming-mode` from 3D-game auto-detection, isolated `committedLocalEngine`/`committedModelName` when browsing uninstalled engines, and added optimistic rollback + keyboard roving focus in `src/tray-menu.js`.
  - Protected saved DPAPI provider keys from accidental empty-input debounce wipes and added quiet `.btn-clear-api-key` buttons (`#btn-clear-key` and `#btn-clear-key-edit`) across both Speech and Text tabs (localized in all 9 UI locales, synced across tabs, Visual QA verified).
  - Enabled `// @ts-check` + JSDoc annotations across 12/13 `src/*.js` modules checked by `tsc -p tsconfig.json` (all modular files outside the `main.js` orchestrator).
- **P4 — Quality gates, fast single-process audit, behavioral tests & CI hardening:**
  - Upgraded all 10 `scripts/*.mjs` gates to dynamically scan all `src/*.js` modules and all 3 desktop windows (`index.html`, `overlay.html`, `tray-menu.html`), full CSS `@media`/inline-style rules, dynamic i18n bounds, and `.emit`/`.emit_to` events; unified `npm run audit` into single-process `scripts/run-all-audits.mjs` (`~140 ms` vs `~1225 ms`).
  - Converted `tests/settings-autosave.test.mjs` to deterministic `t.mock.timers` and added `tests/frontend-behavior.test.mjs` (`node:test` VM/DOM behavioral suite covering Gaming Mode isolation, tray rollback, uninstalled engine browsing, DPAPI key debounce protection + explicit `#btn-clear-key`/`#btn-clear-key-edit` sync, and hidden `overlay.js` zero idle `rAF`/timers).
  - Hardened `.github/workflows/{ci,release}.yml`: moved clean-workspace verification after `cargo test`, added `Swatinem/rust-cache` (`v2.9.3`), switched `cargo-audit` (`v0.22.2`) and `osv-scanner` (`v2.3.8`) to SHA-256-verified prebuilt binaries in `$RUNNER_TEMP`, and pruned stale `--ignore RUSTSEC-2026-0258` while documenting `RUSTSEC-2024-0429` (`glib 0.18.5` via Linux-only Tauri `wry`/`webkit2gtk`/`gtk`).

## Quality gates (last run)

| Gate | Result |
|---|---|
| `npm ci` | 0 vulnerabilities |
| `npm run check` | pass (`node --check` on 13 `src/*.js` + `tsc -p tsconfig.json` on 12/13 typed modules) |
| `npm test` | 75/75 pass (deterministic `t.mock.timers` + VM/DOM behavioral suite) |
| `npm run audit` | 10/10 checks PASS in ~140 ms (`scripts/run-all-audits.mjs`, 0 issues) |
| `npm run verify:manifest` | 5/5 surfaces |
| `npm audit --audit-level=high` | 0 vulnerabilities |
| `cargo fmt --check` | pass |
| `cargo clippy --all-targets -- -D warnings` | pass (0 warnings) |
| `cargo test --all-targets` | 264 passed, 0 failed, 2 ignored |
| `cargo audit --file src-tauri/Cargo.lock --ignore RUSTSEC-2024-0429` | 0 CVE, 11 allowed warnings (`RUSTSEC-2026-0258` resolved and removed) |
| Clean desktop tree | `website/` quarantined untouched; 0 untracked/temp artifacts in desktop scope |

## Decisions

1. **Punctuation pipeline** — remove. `run_punctuation` and the punctuation-model download are unreachable, and the pinned model is Chinese/English only.
2. **PRODUCT.md / DESIGN.md** — track in git and maintain. `AGENTS.md` already treats them as canonical.
3. **Refactoring** — incremental, one module per commit, after Phases 0–4. No big-bang rewrite.
4. **No push** — all work is local until the owner decides otherwise.
5. **Website** — out of scope while the owner is redesigning it.
6. **DPAPI API-key lifecycle** — empty input on debounce never wipes a stored DPAPI key; deletion requires an explicit click on `.btn-clear-api-key` (`#btn-clear-key` / `#btn-clear-key-edit`).

## Next

Continue `feat-009` with the form application:

1. Extract the settings-to-controls application from `loadSettings` into `src/settings-form.js`.
2. Keep engine/audio/model/GPU orchestration in `main.js`.
3. Run `npm run check`, `npm test`, `npm run audit`, `cargo test` after the extraction.

Then `i18n` extraction from `main.js`; then `lib.rs` commands by domain (`feat-010`); finish with `feat-012` final release verification.

## How to resume

```bash
./init.sh          # Git Bash
.\init.ps1         # Windows PowerShell
```

Then read `feature_list.json`, pick the first `in_progress` or `planned` feature whose dependencies are `done`, and read `session-handoff.md`.

## Known caveats

- `cargo audit` reports 0 vulnerabilities and 11 allowed unmaintained/unsound warnings; `--ignore RUSTSEC-2024-0429` remains solely for `glib 0.18.5` pulled into `Cargo.lock` by Tauri v2's Linux-only `wry`/`webkit2gtk`/`gtk` target dependencies (never compiled on Windows).
- Prebuilt `osv-scanner` (`v2.3.8`) flags new upstream advisories `RUSTSEC-2026-0543` (`chrono 0.4.45` unmaintained) and `RUSTSEC-2026-0486` (`rustix 1.1.4` -> `1.1.6`) plus a now-unused `GHSA-qwgh-2vcv-g2f7` ignore in `osv-scanner.toml`; update `rustix` / `osv-scanner.toml` before pushing to GitHub Actions.
- The 20 icon-catalog tests in `tests/icon-legacy-catalog.test.mjs` remain intentionally separate from the production `tests/icon-system.test.mjs` suite (`feat-008`).
