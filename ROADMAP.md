# Aura 2.0 — Roadmap

Program-only roadmap. The marketing website is tracked separately and is out of scope here.

**Baseline:** `main` at the commit that landed the icon system and audit gates, version 1.2.0.
**Rule:** one phase at a time; every phase ends with the full `./init.sh` gate and updated `progress.md` / `feature_list.json`.

## Phase 0 — Harness and entrypoint

**Status:** done.

- `init.sh` / `init.ps1` as the single startup and verification path.
- `progress.md`, `session-handoff.md`, `feature_list.json`.
- Remove stale `build-storybook` references from `AGENTS.md`.

**Exit:** `./init.sh` passes from a clean checkout.

## Phase 1 — Product and design records

**Status:** done.

- Track `PRODUCT.md` and `DESIGN.md` in git.
- Rewrite `PRODUCT.md` around the current positioning: voice typing, local/cloud engines, selection editing, honest capability limits.
- Keep `DESIGN.md` as the desktop design canon.

**Exit:** a fresh clone contains every document `AGENTS.md` tells an agent to read.

## Phase 2 — User-facing correctness

**Status:** done.

- Empty custom selects render a readable fallback before JS or when settings fail.
- Backend errors are localized; raw Rust text stays in the console log.
- Autostart registration failures are surfaced as a non-fatal warning.
- Engine-health polling reads non-secret settings, so DPAPI is not decrypted every 10 seconds.

**Evidence:** `npm test` 53/53, `npm run audit` 10/10, `cargo test` 239 passed, `init.ps1` exit 0.

## Phase 3 — Security and hygiene

**Status:** planned.

- Require HTTPS for non-local custom API endpoints; allow `http` only for loopback.
- Sweep `aura-rec-*.wav` / `aura-chunk-*.wav` at startup and keep them in a restricted directory.
- Update yanked `clipboard-win` and verify `async-executor`.
- Revisit `withGlobalTauri` exposure for the overlay and tray windows.

**Exit:** `cargo audit`, `cargo test`, `npm audit`, no residual temp audio after an abnormal exit.

## Phase 4 — Dead code and test honesty

**Status:** planned; requires owner decision.

- Remove the unused punctuation pipeline (`run_punctuation`, punctuation model download, UI branches). The pinned model is Chinese/English only and nothing calls it.
- Move the 20 icon-catalog tests onto the production `src/icon-system.js` registry; keep the legacy catalog as a separate comparison test.
- Move root-level historical artifacts into `docs/` or `design-lab/`.

**Exit:** no unreachable public pipeline, production icon registry has direct tests, root stays clean.

## Phase 5 — Structural refactor

**Status:** planned; starts only after Phases 0–4 are green.

`main.js` extraction order:

1. `i18n` dictionaries and `getTranslation`.
2. History rendering, search, and filters.
3. Model cards, download/delete, progress.
4. Settings form and autosave.
5. `main.js` remains the orchestrator.

`lib.rs` extraction order:

1. Settings commands.
2. Model commands.
3. History commands.
4. Audio and microphone commands.
5. Update and diagnostics commands.

**Rules:** one module per commit, no behavior change, the same tests before and after, each commit independently revertible.

**Exit:** smaller `main.js` / `lib.rs`, unchanged public behavior, all gates green.

## Phase 6 — Polish and final verification

**Status:** planned.

- `defer` for `icon-system.js`, a real `<h1>`, remove `aria-expanded` from the native select, remove the badge hover lift, initialize `<html lang>` from the system locale, drop the audio-device `console.log`.
- Full local CI replica, including `npm ci`, `npm run audit`, `cargo audit`, and OSV-Scanner if Go is available.
- Update `progress.md`, `feature_list.json`, and `session-handoff.md`; record accepted residual risks.

**Exit:** no open P0/P1, every gate green, the next session starts with `./init.sh` and nothing else.
