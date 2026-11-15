# Aura 2.0 — Progress

**Updated:** 2026-11-15
**Baseline:** `main` after the Phase 0–1 commit (parent `793579f`)
**Status:** green — working tree clean, every local gate passes.

## Current state

The program is at version 1.2.0 with the icon-system rollout and the static-audit gates landed. The desktop app is in scope; `website/` is intentionally out of scope.

Completed in this cycle:

- `f8ba73a` — monochrome icon system, settings polish, design-lab, legacy icon registry moved out of `src/`.
- `793579f` — every static audit script fails on violations; `npm test` runs all `tests/*.test.mjs` (49 tests).
- Phase 0 — `init.sh`, `init.ps1`, `progress.md`, `session-handoff.md`, `feature_list.json`, `ROADMAP.md`; AGENTS.md cleaned up.
- Phase 1 — `PRODUCT.md` rewritten and tracked; `DESIGN.md` tracked as the desktop design canon.

## Quality gates (last run)

| Gate | Result |
|---|---|
| `npm ci` | 0 vulnerabilities |
| `npm run check` | pass |
| `npm test` | 49/49 pass |
| `npm run audit` | 10/10 checks PASS |
| `npm run verify:manifest` | 5/5 surfaces |
| `npm audit --audit-level=high` | 0 vulnerabilities |
| `cargo fmt --check` | pass |
| `cargo clippy -D warnings` | pass |
| `cargo test --all-targets` | 239 passed, 0 failed, 3 ignored |
| `cargo audit` | 0 CVE, 12 allowed warnings |
| Clean tree | `git diff` 0, `git diff --cached` 0, untracked 0 |

## Decisions

1. **Punctuation pipeline** — remove. `run_punctuation` and the punctuation-model download are unreachable, and the pinned model is Chinese/English only.
2. **PRODUCT.md / DESIGN.md** — track in git and maintain. `AGENTS.md` already treats them as canonical.
3. **Refactoring** — incremental, one module per commit, after Phases 0–4. No big-bang rewrite.
4. **No push** — all work is local until the owner decides otherwise.
5. **Website** — out of scope while the owner is redesigning it.

## Next

Start with `feat-005` (user-facing correctness):

1. Add a readable fallback to every custom select before JS or when `get_settings` fails.
2. Extend localized backend errors for microphone, GPU, diagnostics, and backup.
3. Surface autostart registration failures.
4. Stop loading/decrypting DPAPI secrets on every engine-health poll.

Then `feat-006` (security hygiene), `feat-007` (punctuation removal), `feat-008` (icon tests on production registry), and only then the structural refactor.

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
