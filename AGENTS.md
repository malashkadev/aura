# Aura — Agent Guide

<!-- skills-project:start — генерируется skills-project-init.mjs, руками не править -->
## Skills routing (Aura)

Проектный навык побеждает глобальный роутер. Раздел «Core Architectural Invariants» в этом файле важнее любого совета из глобальных навыков.

### Проектные навыки

| Навык | Где | Что делает |
|---|---|---|
| `aura-product-sync` | `.agents/skills` | Enforces Aura's core product invariants, anti-slop copywriting standards, settings mockup architecture (6 tabs v1.1.0+), multi-… |
| `defuddle` | `.agents/skills` | Extract clean markdown content from web pages using Defuddle CLI, removing clutter and navigation to save tokens. |
| `excalidraw-diagram` | `.agents/skills` | Generate Excalidraw diagrams from text content. |
| `interactive-demo-previews` | `.agents/skills` | Превью окна, модалки или компонента перед правкой проекта Аура. |
| `json-canvas` | `.agents/skills` | Create and edit JSON Canvas files (.canvas) with nodes, edges, groups, and connections. |
| `mermaid-visualizer` | `.agents/skills` | Transform text content into professional Mermaid diagrams for presentations and documentation. |
| `obsidian-bases` | `.agents/skills` | Create and edit Obsidian Bases (.base files) with views, filters, formulas, and summaries. |
| `obsidian-cli` | `.agents/skills` | Interact with Obsidian vaults using the Obsidian CLI to read, create, search, and manage notes, tasks, properties, and more. |
| `obsidian-markdown` | `.agents/skills` | Create and edit Obsidian Flavored Markdown with wikilinks, embeds, callouts, properties, and other Obsidian-specific syntax. |
| `tauri-v2` | `.agents/skills` | Tauri v2+ cross-platform app development with Rust backend. |

### Куда идти с задачей

| Задача | Сначала проектное | Потом глобальное |
|---|---|---|
| UI десктопного приложения | `aura-product-sync` | `ui-design-craft → ui-interface-craft` |
| Rust / Tauri / IPC / хуки Windows | `tauri-v2` | `software-engineering → core-app-architecture`, `software-engineering → win32-system-hooks` |
| Лендинг или website/ | `aura-product-sync` | `ui-design-craft → landing-page-system`, `text-copywriting-craft → unslop-engine` |
| Тексты о продукте и тексты интерфейса | `aura-product-sync` | `text-copywriting-craft → ux-copy / redaktura`, `unslop-engine` |
| Схема, флоу, дашборд, «покажи, как будет» | — | `ui-design-craft → generative-ui` |
| Прототип конкретного окна 1:1 по коду | `interactive-demo-previews` | `ui-design-craft → interactive-demo-previews` |
| Заметки, ADR, диаграммы Obsidian | `obsidian-cli`, `obsidian-markdown`, `obsidian-bases`, `mermaid-visualizer`, `json-canvas`, `excalidraw-diagram` | `knowledge-memory-vault` |
| Извлечь контент со страницы | `defuddle` | — |
| Баг, сбой сборки, падающий тест | — | `software-engineering → systematic-debugging` |
| Структура кода, кто кого вызывает | — | `MCP codebase-memory, проект G-Aura-2.0` |

### Глобальные советы, которые здесь не действуют

- `ui-interface-craft` — допускает синие и индиго-подтоны, тени и внешние шрифты — в десктопном UI Ауры это запрещено разделом Core Architectural Invariants.
- `landing-page-system` — его шрифты и палитра относятся только к website/; перенос в src/ запрещён правилом Zero Cross-Contamination.
- `stageshow, creative-ui-fx` — декоративные эффекты не применяются к рабочему UI без явной просьбы владельца.

### Артефакты и проверка

- Артефакты: `.opencode/artifacts/`.
- Релевантные глобальные роутеры: `software-engineering`, `ui-design-craft`, `knowledge-memory-vault`, `text-copywriting-craft`.
- Индекс глобальных навыков: `C:/Users/Ваня/.gemini/config/skills/ROUTES.md`.
<!-- skills-project:end -->

## Where design decisions live

Two stores. They answer different questions — do not confuse them.

**1. Current record (canonical). Plain files, read them directly:**

`C:\Users\Ваня\Vaults\obsidian`

- `work/active/Aura.md` — live project dashboard: status, open tasks, milestones, wins.
- `brain/Key Decisions.md` — chronological ADR register (ADR-001…ADR-052).
- `brain/decisions/` — one file per decision, including the rejected alternatives.
- `perf/Brag Doc.md` — shipped work with evidence.

Read these before changing anything whose rationale was recorded: the product surface (PRODUCT.md, ROADMAP.md), storage formats, and public interfaces. If the record and this repo disagree, the record holds the *why* — reconcile before changing behavior.

**2. Archive of the August 2026 sprints. Only through the `om` MCP server:**

`G:\Aura\my-vault` — read-only, do not write new project data into it.

Its `memories/` hold nine durable lessons from the first audit rounds; `work/active/` holds the old sprint notes. `om` is pointed at it through `OM_VAULT_PATH` in `opencode.json`.

- **`recall`** returns those durable lessons. An empty result means this archive has nothing — it is *not* evidence that the current record is missing.
- **`search`** and **`expand`** reach the archived notes and their links; `expand` is cheaper than searching again for a note's neighbourhood.
- **`reason`** answers questions that need judgement across several archived notes.
- **`remember`** and **`record_work`** write new lessons into the archive vault. Set `confidence` (`verified` / `inferred` / `unverified`) honestly.
- **`health`** when something that should be there cannot be found. Every failure in this layer looks identical from outside (no results); it tells them apart.

`om` does **not** see the canonical vault: that directory has no `vault-manifest.json` and no `memories/`, so pointing the server at it returns zero results. Current decisions come from the files in list 1, never from `recall`.

**Say what came back in whatever you write before implementing:** a plan, a design note, an issue comment. Name the recorded decisions the work rests on, anything found that argues *against* the approach, and an explicit "nothing recorded on this" when the record is empty.

If the `om` server is missing (no tools `mcp__om__*` available), the wiring in `opencode.json` needs fixing — flag it rather than skipping. Its absence does not block reading the canonical record.

## Code graph (codebase-memory MCP)

The index for this repository is the project **`G-Aura-2.0`**, root `G:\Aura\2.0`.

`G-Aura` (root `G:\Aura`) is a legacy index from August 2026 that the current MCP build will not
delete. Never query it — it describes three-month-old code. If a tool offers it, use `G-Aura-2.0`.

The index does not refresh itself. Re-indexing takes about four seconds and should follow any
substantial change in `src/` or `src-tauri/`:

```
index_repository(repo_path="G:/Aura/2.0", name="G-Aura-2.0")
```

## Recording what you learn

Test for which tool: would it help someone working on a *different* project?

- **`remember`** — durable lesson. `scope: "project"` for something specific to this repo (`projects: ["aura"]`), `"platform"` for dependency quirks, `"general"` only with an explicit `generality` reason.
- **`record_work`** — what happened *here*: changes, decisions and the alternatives rejected, what was learned, what is still open, how it was verified.

A dependency limitation that would bite any project is a `remember`. "Landed the watch engine and here is what it cost" is a `record_work`. Do both when both are true.

## Core Architectural Invariants (DO NOT VIOLATE)

1. **Both Whisper.cpp AND NVIDIA Parakeet support BOTH CPU and NVIDIA CUDA hardware modes as resident background servers**:
   - **CPU Mode (Default out-of-the-box)**:
     - Whisper runs via resident `whisper-server` (with `-ng` CPU flag, multi-threaded up to 8 threads), keeping models resident in RAM.
     - Parakeet runs via bundled CPU WebSocket server.
     - Works out of the box on any Windows PC without discrete GPU.
   - **NVIDIA CUDA Mode (High-Performance Acceleration)**:
     - Whisper runs via resident `whisper-server` linked with `ggml-cuda.dll`, keeping models loaded directly in VRAM (`CUDA0`), delivering instant transcription without cold-start delay.
     - Parakeet runs via resident `sherpa-onnx-offline-websocket-server` on CUDA.
   - **NEVER** claim or assume that Whisper cannot run as a resident server, cannot use CPU, or cannot use GPU/CUDA. Both modes operate seamlessly in Aura.
   - The base app installer stays lightweight (~20–25 MB). CUDA runtime binaries (`sherpa-onnx` CUDA + `whisper-cublas` `ggml-cuda.dll`) are downloaded on-demand in 1 click into `%LOCALAPPDATA%\com.aura.app\binaries\cuda\bin\`.

2. **Desktop App UI vs. Website Design System (Strict Quarantine & Separation)**:
   - **Desktop App UI (`src/`, `src-tauri/`)**:
     - **North Star**: "The Stealth Command Deck" — a quiet, high-precision background Windows utility.
     - **Achromatic Palette Invariant (Strict)**: All background surfaces, cards, modals, popovers, and dialogs MUST be 100% achromatic monochrome (`hsl(0, 0%, ...)` / `#000000`, `#090909`, `#0d0d0d`, `#0f0f0f`, `#121212`). **BANNED**: blue-tinted, navy, or indigo-tinted dark hex colors (`#030305`, `#0a0a0c`, `#0e0e12`, `#111114`, `#131317`, `#121214`, `rgba(20, 20, 26)`).
     - **The Rarity Rule (Strict)**: Tactical Orange (`#ff4200`) must never exceed 10% of any screen layout. It is strictly reserved for active recording indicators, active navigation tab, active toggles, and critical status.
     - **Local Typography Invariant**: Desktop app uses locally bundled WOFF2 `Onest` (`src/assets/fonts/`) with system fallbacks (`Segoe UI Variable Display`). **NEVER** link external Google Fonts / Gstatic in the desktop app.
     - **Zero Shadows & Specular Facet Rule (Strict)**: Outer drop shadows (`box-shadow`) and outer glows (`--accent-glow`, `filter`) are banned across all app surfaces. Use only physical top-edge specular facet lines: `inset 0 1px 0 rgba(255, 255, 255, 0.08)` for cards, `rgba(255, 255, 255, 0.12)` for modals.
     - **Direct Feedback & Zero Redundancy**: Controls that produce feedback (like sound themes and volume) must play feedback directly on adjustment. **NEVER** inject redundant "Test" / "Preview" buttons when the control itself already previews.
     - **Quiet Shared Credentials**: Multi-tab credentials (like API keys) synchronize silently in real-time backed by Windows DPAPI. **NEVER** clutter form labels with internal technical badges (e.g. "Saved in Windows DPAPI").
     - **Native Desktop Interaction**: `user-select: none` is globally enforced on `*, *::before, *::after` and all dialogs/modals. Only `<input>`, `<textarea>`, and `.history-text` permit text selection.
     - **Button Invariants in Desktop App**:
       - Standard & utility action buttons (e.g. "Add", "Reset", "Clear", "Test", secondary dialog actions) MUST use `.btn.btn-secondary` or standard quiet desktop styling.
       - **Rest (Static) State**: Background is transparent/dark, border is `1px solid var(--border-color)` (`#29292980`), text is `var(--text-secondary)`. **NEVER use bright orange borders or orange tinted backgrounds in static state.**
       - **Hover State**: Subtle border highlight (`border-color: var(--accent-color)`) or subtle white glow (`rgba(255, 255, 255, 0.05)`). **NEVER fill buttons with solid orange (`#ff4200`) on hover.**
       - **Component Reuse First**: NEVER create ad-hoc button classes (like `.btn-action-sm`). Always reuse existing classes (`.btn.btn-secondary`, `.custom-input`, `.form-group`).
     - **Form Layout & Spacing**:
       - Use `.form-group` structure and standard application padding (`12px`/`16px`). Do not inject ad-hoc small rows with arbitrary non-standard heights.
   - **Marketing Website (`website/`)**:
     - High-contrast landing page with conversion CTA buttons, full pills (`9999px`), 3D assets, and marketing showcase mockups. Uses Google Font `Onest` and deep space obsidian with velvet blue undertones (`#030305`, `--surface-card: rgba(20, 20, 26, 0.65)`).
     - **Zero Cross-Contamination Rule**: NEVER leak classes, button styles, gradient fills, font families, or color tokens between `website/` and `src/`. Automated quarantine is enforced by `tests/frontend-static.test.mjs`.


## Startup Workflow

Before writing code:
1. **Confirm working directory** with `pwd`
2. **Read this file** completely
3. **Read project docs**: `PRODUCT.md`, `ROADMAP.md`, `docs/`
4. **Run `./init.sh`** to verify environment is healthy
5. **Read `feature_list.json`** to see current feature state and dependencies
6. **Review `progress.md`** for active tasks and continuity

If baseline verification is failing, repair that first before adding new scope.

## Working Rules

- **One feature at a time**: Pick exactly one unfinished feature from `feature_list.json`
- **Stay in scope**: Do not modify files unrelated to the active feature
- **Verification required**: Don't claim done without running verification commands
- **Update artifacts**: Before ending session, update `progress.md` and `feature_list.json`
- **Leave clean state**: Next session must be able to run `./init.sh` immediately

## Required Artifacts

- `feature_list.json` — Feature state tracker (source of truth)
- `progress.md` — Session continuity and progress log
- `session-handoff.md` — Handoff between sessions
- `init.sh` — Standard startup and verification path

## Definition of Done

A feature is done only when ALL of the following are true:
- [ ] Target behavior is implemented
- [ ] Required verification actually ran (tests / lint / build / type checks)
- [ ] Evidence recorded in `feature_list.json` or `progress.md`
- [ ] Repository remains restartable and clean from standard startup path

## End of Session

Before ending a session:
1. Update `progress.md` with current state
2. Update `feature_list.json` with new feature status
3. Record verification evidence
4. Record any unresolved risks or blockers
5. Leave repo clean enough for next session to run `./init.sh` immediately

## Verification Commands

```bash
# Full verification entrypoint
./init.sh
```

Required checks:
- Build check: `npm run build-storybook`
- Testing: `npm test` or `cargo test` in `2.0/src-tauri`
