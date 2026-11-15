# Session Handoff — Aura 2.0

**Date:** 2026-11-15
**Baseline:** `main` at `cac731f` (Phase 4 complete)
**Scope:** desktop program only; `website/` is out of scope.

## What changed

- Icon system rolled out and committed (`f8ba73a`).
- Static audit scripts turned into real gates; all test files wired (`793579f`).
- Phase 0 harness: `init.sh`, `init.ps1`, `progress.md`, `session-handoff.md`, `feature_list.json`, `ROADMAP.md` (`fa02b6a`).
- Phase 1 docs: `PRODUCT.md` rewritten, `PRODUCT.md` and `DESIGN.md` tracked (`fa02b6a`).
- Phase 2: select fallback (`1d0f204`), localized backend errors (`f04c0c2`), honest autostart (`1fd16fc`), health without DPAPI (`404363d`).
- Phase 3: HTTPS custom-endpoint guard (`8e842ab`), stale-audio sweep and ACL (`7978a47`), yanked dependency updates (`8580b3e`).
- Phase 4: punctuation pipeline removed (`19eb873`), production icon tests (`ff6e1ac`), history docs moved (`cac731f`).

## Verification

Run `./init.sh` (Git Bash) or `.\init.ps1` (PowerShell). Last recorded result:

- `npm test` 65/65
- `npm run audit` 10/10 PASS
- `cargo test --all-targets` 237 passed, 2 ignored
- `cargo audit` 0 CVE, 10 allowed warnings
- `npm run verify:manifest` 5/5
- clean tree

## Next task

`feat-009` — structural refactor, first module:

1. Extract history rendering, search, and filters from `main.js` into a focused module.
2. Keep `main.js` as the orchestrator; no behavior changes.
3. Run `npm test`, `npm run audit`, `cargo test` after each extraction.

Then `models`, `settings-form`, `i18n`; then `lib.rs` commands by domain (`feat-010`); finish with `feat-011` polish and `feat-012` verification.

## Constraints

- Do not push to GitHub.
- Do not touch `website/`.
- Do not rewrite `main.js` or `lib.rs` before Phases 2–4 are green.
- Keep the static audit gate green: new hardcoded text, raw titles, non-canonical shadows, or IPC mismatches will fail `npm run audit`.

## Known gaps

- OSV-Scanner not run locally (no Go).
- The legacy icon-catalog suite remains intentionally separate from the production-registry suite.
- `cargo audit` has 10 allowed warnings, no CVE.
