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

**Status:** done.

- Custom endpoints require HTTPS; plain HTTP is allowed only for loopback, and embedded credentials are rejected.
- Leftover `aura-rec-*` / `aura-chunk-*` files are swept at startup; pending and finalized WAV files receive the current-user/SYSTEM ACL.
- `clipboard-win` 5.4.3 and `async-executor` 1.14.1 replace the yanked versions.
- `withGlobalTauri` stays enabled deliberately: the frontend is unbundled and needs the global bridge, while per-window capabilities remain the security boundary and are covered by the exact-permission test.

**Evidence:** `npm test` 54/54, `cargo test` 241 passed, `cargo audit` 0 CVE / 10 allowed warnings, `init.ps1` exit 0.

## Phase 4 — Dead code and test honesty

**Status:** done.

- Removed the unused punctuation pipeline, its model download, diagnostics fields, UI branches, provenance entry, and the 12 MB sidecar binary.
- Added a direct production-registry test suite for `src/icon-system.js`; the legacy registry tests remain as the catalog comparison suite.
- Moved the 2026 audits, handoffs, and E2E checklist into `docs/history/`; the New Design source assets now live in `docs/design/new-design/`.

**Evidence:** `npm test` 65/65, `cargo test` 237 passed, `npm run audit` 10/10, `init.ps1` exit 0.

## Phase 5 — Structural refactor

**Status:** in progress. History view (`src/history-view.js`) and model cards (`src/model-cards.js`) are extracted; the settings form is next.

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
