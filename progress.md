# Aura 2.0 — Progress

**Updated:** 2026-11-15
**Baseline:** `main` at `cac731f` (Phase 4 complete)
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
- `19eb873` — removed the unreachable punctuation pipeline, its 12 MB sidecar, diagnostics fields, and UI branches.
- `ff6e1ac` — production icon-registry tests added; the legacy catalog suite keeps its own file.
- `cac731f` — 2026 audits, handoffs, and E2E checklist moved to `docs/history/`; New Design assets moved under `docs/design/`.

## Quality gates (last run)

| Gate | Result |
|---|---|
| `npm ci` | 0 vulnerabilities |
| `npm run check` | pass |
| `npm test` | 65/65 pass |
| `npm run audit` | 10/10 checks PASS |
| `npm run verify:manifest` | 5/5 surfaces |
| `npm audit --audit-level=high` | 0 vulnerabilities |
| `cargo fmt --check` | pass |
| `cargo clippy -D warnings` | pass |
| `cargo test --all-targets` | 237 passed, 0 failed, 2 ignored |
| `cargo audit` | 0 CVE, 10 allowed warnings |
| Clean tree | `git diff` 0, `git diff --cached` 0, untracked 0 |

## Decisions

1. **Punctuation pipeline** — remove. `run_punctuation` and the punctuation-model download are unreachable, and the pinned model is Chinese/English only.
2. **PRODUCT.md / DESIGN.md** — track in git and maintain. `AGENTS.md` already treats them as canonical.
3. **Refactoring** — incremental, one module per commit, after Phases 0–4. No big-bang rewrite.
4. **No push** — all work is local until the owner decides otherwise.
5. **Website** — out of scope while the owner is redesigning it.

## Next

Start with `feat-009` (structural refactor), first module:

1. Extract history rendering, search, and filters from `main.js` into a focused module.
2. Keep `main.js` as the orchestrator; no behavior changes.
3. After each extraction run `npm test`, `npm run audit`, `cargo test`.

Then `models`, `settings-form`, `i18n`; then `lib.rs` commands by domain (`feat-010`); finish with `feat-011` polish and `feat-012` verification.

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
