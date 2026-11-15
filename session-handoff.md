# Session Handoff — Aura 2.0

**Date:** 2026-11-15
**Baseline:** `main` at `8580b3e` (Phase 3 complete)
**Scope:** desktop program only; `website/` is out of scope.

## What changed

- Icon system rolled out and committed (`f8ba73a`).
- Static audit scripts turned into real gates; all test files wired (`793579f`).
- Phase 0 harness: `init.sh`, `init.ps1`, `progress.md`, `session-handoff.md`, `feature_list.json`, `ROADMAP.md` (`fa02b6a`).
- Phase 1 docs: `PRODUCT.md` rewritten, `PRODUCT.md` and `DESIGN.md` tracked (`fa02b6a`).
- Phase 2: select fallback (`1d0f204`), localized backend errors (`f04c0c2`), honest autostart (`1fd16fc`), health without DPAPI (`404363d`).
- Phase 3: HTTPS custom-endpoint guard (`8e842ab`), stale-audio sweep and ACL (`7978a47`), yanked dependency updates (`8580b3e`).

## Verification

Run `./init.sh` (Git Bash) or `.\init.ps1` (PowerShell). Last recorded result:

- `npm test` 54/54
- `npm run audit` 10/10 PASS
- `cargo test --all-targets` 241 passed, 3 ignored
- `cargo audit` 0 CVE, 10 allowed warnings
- `npm run verify:manifest` 5/5
- clean tree

## Next task

`feat-007` — dead code and test honesty:

1. Remove `run_punctuation`, `download_punctuation_model`, the punctuation model download, and the UI branches. The pinned model is Chinese/English only and nothing calls it.
2. Move the 20 icon-catalog tests onto the production `src/icon-system.js` registry; keep the legacy catalog as a comparison test.
3. Move root-level historical artifacts into `docs/` or `design-lab/`.

Acceptance: no unreachable public pipeline, production icon registry has direct tests, root stays clean.

## Constraints

- Do not push to GitHub.
- Do not touch `website/`.
- Do not rewrite `main.js` or `lib.rs` before Phases 2–4 are green.
- Keep the static audit gate green: new hardcoded text, raw titles, non-canonical shadows, or IPC mismatches will fail `npm run audit`.

## Known gaps

- OSV-Scanner not run locally (no Go).
- Icon-catalog tests target the legacy registry; production registry is covered by frontend-static tests (`feat-008`).
- `cargo audit` has 10 allowed warnings, no CVE.
