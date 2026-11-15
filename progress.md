# Aura 2.0 — Progress

**Updated:** 2026-11-15
**Baseline:** `main` at `8580b3e` (Phase 3 complete)
**Status:** green — working tree clean, every local gate passes.

## Current state

The program is at version 1.2.0 with the icon-system rollout, the static-audit gates, and the Phase 2 correctness fixes landed. The desktop app is in scope; `website/` is intentionally out of scope.

Completed in this cycle:

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

## Quality gates (last run)

| Gate | Result |
|---|---|
| `npm ci` | 0 vulnerabilities |
| `npm run check` | pass |
| `npm test` | 54/54 pass |
| `npm run audit` | 10/10 checks PASS |
| `npm run verify:manifest` | 5/5 surfaces |
| `npm audit --audit-level=high` | 0 vulnerabilities |
| `cargo fmt --check` | pass |
| `cargo clippy -D warnings` | pass |
| `cargo test --all-targets` | 241 passed, 0 failed, 3 ignored |
| `cargo audit` | 0 CVE, 10 allowed warnings |
| Clean tree | `git diff` 0, `git diff --cached` 0, untracked 0 |

## Decisions

1. **Punctuation pipeline** — remove. `run_punctuation` and the punctuation-model download are unreachable, and the pinned model is Chinese/English only.
2. **PRODUCT.md / DESIGN.md** — track in git and maintain. `AGENTS.md` already treats them as canonical.
3. **Refactoring** — incremental, one module per commit, after Phases 0–4. No big-bang rewrite.
4. **No push** — all work is local until the owner decides otherwise.
5. **Website** — out of scope while the owner is redesigning it.

## Next

Start with `feat-007` (dead code and test honesty):

1. Remove the unused punctuation pipeline, model download, and UI branches.
2. Move the 20 icon-catalog tests onto the production `src/icon-system.js` registry.
3. Move root-level historical artifacts into `docs/` or `design-lab/`.

Then `feat-008`, `feat-011`, and the structural refactor.

## How to resume

```bash
./init.sh          # Git Bash
.\init.ps1         # Windows PowerShell
```

Then read `feature_list.json`, pick the first `planned` feature whose dependencies are `done`, and read `session-handoff.md`.

## Known caveats

- OSV-Scanner did not run locally because Go is not installed; GitHub Actions remains the only place it executes.
- The 20 icon-catalog tests still exercise the legacy registry in `design-lab/icon-system/legacy/`; production `src/icon-system.js` is covered by the frontend-static tests. `feat-008` closes this gap.
- The item-count audit (`P2-6`) still reports 12 allowed `cargo audit` warnings; no CVE.
