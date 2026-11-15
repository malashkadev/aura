# Session Handoff — Aura 2.0

**Date:** 2026-11-15
**Baseline:** `main` at `404363d` (Phase 2 complete)
**Scope:** desktop program only; `website/` is out of scope.

## What changed

- Icon system rolled out and committed (`f8ba73a`).
- Static audit scripts turned into real gates; all test files wired (`793579f`).
- Phase 0 harness: `init.sh`, `init.ps1`, `progress.md`, `session-handoff.md`, `feature_list.json`, `ROADMAP.md` (`fa02b6a`).
- Phase 1 docs: `PRODUCT.md` rewritten, `PRODUCT.md` and `DESIGN.md` tracked (`fa02b6a`).
- Phase 2: select fallback (`1d0f204`), localized backend errors (`f04c0c2`), honest autostart (`1fd16fc`), health without DPAPI (`404363d`).

## Verification

Run `./init.sh` (Git Bash) or `.\init.ps1` (PowerShell). Last recorded result:

- `npm test` 53/53
- `npm run audit` 10/10 PASS
- `cargo test --all-targets` 239 passed, 3 ignored
- `npm run verify:manifest` 5/5
- clean tree

## Next task

`feat-006` — security hygiene:

1. Require HTTPS for non-local custom API endpoints; allow `http` only for loopback.
2. Sweep `aura-rec-*.wav` / `aura-chunk-*.wav` at startup and keep them in a restricted directory.
3. Update yanked `clipboard-win` and verify `async-executor`.
4. Revisit `withGlobalTauri` exposure for the overlay and tray windows.

Acceptance: `cargo audit`, `cargo test`, `npm audit`, no residual temp audio after an abnormal exit.

## Constraints

- Do not push to GitHub.
- Do not touch `website/`.
- Do not rewrite `main.js` or `lib.rs` before Phases 2–4 are green.
- Keep the static audit gate green: new hardcoded text, raw titles, non-canonical shadows, or IPC mismatches will fail `npm run audit`.

## Known gaps

- OSV-Scanner not run locally (no Go).
- Icon-catalog tests target the legacy registry; production registry is covered by frontend-static tests (`feat-008`).
- `cargo audit` has 12 allowed warnings, no CVE.
