# Session Handoff — Aura 2.0

**Date:** 2026-11-15
**Baseline:** `main` after the Phase 0–1 commit; parent `793579f`
**Scope:** desktop program only; `website/` is out of scope.

## What changed

- Icon system rolled out and committed (`f8ba73a`).
- Static audit scripts turned into real gates; all test files wired (`793579f`).
- Phase 0 harness: `init.sh`, `init.ps1`, `progress.md`, `session-handoff.md`, `feature_list.json`, `ROADMAP.md`.
- Phase 1 docs: `PRODUCT.md` rewritten, `PRODUCT.md` and `DESIGN.md` tracked.

## Verification

Run `./init.sh` (Git Bash) or `.\init.ps1` (PowerShell). Last recorded result:

- `npm test` 49/49
- `npm run audit` 10/10 PASS
- `cargo test --all-targets` 239 passed, 3 ignored
- `npm run verify:manifest` 5/5
- clean tree

## Next task

`feat-005` — user-facing correctness:

1. Custom select fallback before JS / on settings failure.
2. Localized errors for microphone, GPU, diagnostics, and backup.
3. Honest autostart failure status.
4. Engine-health path without DPAPI decryption every 10 seconds.

Acceptance: `npm test`, `npm run audit`, `cargo test`; manual scenarios for missing model, missing microphone, blocked autostart.

## Constraints

- Do not push to GitHub.
- Do not touch `website/`.
- Do not rewrite `main.js` or `lib.rs` before Phases 2–4 are green.
- Keep the static audit gate green: new hardcoded text, raw titles, non-canonical shadows, or IPC mismatches will fail `npm run audit`.

## Known gaps

- OSV-Scanner not run locally (no Go).
- Icon-catalog tests target the legacy registry; production registry is covered by frontend-static tests (`feat-008`).
- `cargo audit` has 12 allowed warnings, no CVE.
