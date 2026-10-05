# Smart Context-Aware Dictation & Insertion Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Implement a zero-latency, context-aware dictation system on Windows that automatically detects whether spoken text starts a new sentence, continues an open sentence, inserts into the middle of text, or replaces a selection, adapting letter casing, trailing punctuation, and boundary whitespace accordingly.

**Architecture:** A two-layer hybrid architecture. Layer 1 uses native Windows UI Automation (`IUIAutomationTextPattern`) to capture caret context (`text_before`, `text_after`, `selected_text`) in a parallel background task (<=20ms timeout) during recording, feeding a deterministic Rust normalizer (<0.05ms) for 100% of standard dictation. Layer 2 enriches local/cloud LLM edit prompts with the surrounding caret context when a user issues explicit voice editing commands over a selection.

**Tech Stack:** Rust (Tauri v2, `windows` 0.54 UI Automation & COM), Vanilla JS / CSS, Segoe UI Variable, Windows Accessibility API.

**Spec:** [`docs/superpowers/specs/2026-10-05-smart-context-insertion-design.md`](file:///g:/Aura/2.0/docs/superpowers/specs/2026-10-05-smart-context-insertion-design.md)

## Global Constraints

- Installer footprint must remain ~20–25 MB (zero bundled heavy runtimes or LLMs).
- Perceptible typing latency must remain 0 ms (UIA extraction occurs concurrently in background while recording).
- Zero clipboard pollution: caret context and selection extraction via UIA must not mutate or clear the user's clipboard.
- Fail-safe timeout: UIA inspection must abort within 20 ms and fallback to standard dictation if the target window is unresponsive.
- Stealth Command Deck UI invariant: Tactical Orange (`#ff4200`) <= 10%, quiet `.btn.btn-secondary`, no solid orange hover fills.
- Strict multi-language coverage across all 10 supported UI languages (`ru`, `en`, `de`, `es`, `fr`, `it`, `zh`, `pt`, `tr`, `nl`).

## Review Focus

1. **Acronym Preservation:** Words like `API`, `HTML`, `США`, `GPU` spoken mid-sentence must NOT be lowercased to `api`, `html`, `сша`, `gpu`.
2. **Mid-Sentence Trailing Period Stripping:** Spoken text inserted before an existing word must unconditionally strip trailing periods (e.g. `«Мы должны [курсор] встретиться»` + `«как можно скорее.»` $\rightarrow$ `«Мы должны как можно скорее встретиться»`).
3. **Punctuation Collision Avoidance:** If `text_after` begins with a comma or period, the inserted text must not duplicate punctuation or create invalid punctuation clusters (`«слово, ,»`).
4. **Boundary Space Insertion:** If the caret is positioned immediately after a word without a trailing space (`«купил|»`), a leading space must be prepended; if inserting into the middle, both leading and trailing spaces must be normalized.
5. **Unresponsive Window Immunity:** Target applications running elevated (UIPI) or hung GUI threads must not cause Aura to block, freeze, or drop dictation output.

---

## Tasks

### Task 1: Smart Insertion Data Models & Mode Classifier

**Files:**
- Create: `2.0/src-tauri/src/smart_insertion.rs`
- Modify: `2.0/src-tauri/src/lib.rs` (register module)
- Test: `2.0/src-tauri/src/smart_insertion.rs` (unit tests inline)

**Interfaces:**
- Produces: `InsertionMode`, `CaretContext`, `CaretContextSource`, `classify_insertion_mode(ctx: &CaretContext) -> InsertionMode`.

- [ ] **Step 1: Write the failing unit tests for `classify_insertion_mode`**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_classify_new_sentence_empty() {
        let ctx = CaretContext::default();
        assert_eq!(classify_insertion_mode(&ctx), InsertionMode::NewSentence);
    }

    #[test]
    fn test_classify_new_sentence_after_terminal_punctuation() {
        let ctx = CaretContext {
            text_before: "Привет.".to_string(),
            ..Default::default()
        };
        assert_eq!(classify_insertion_mode(&ctx), InsertionMode::NewSentence);

        let ctx_newline = CaretContext {
            text_before: "Строка\n".to_string(),
            ..Default::default()
        };
        assert_eq!(classify_insertion_mode(&ctx_newline), InsertionMode::NewSentence);
    }

    #[test]
    fn test_classify_mid_sentence_insertion() {
        let ctx = CaretContext {
            text_before: "Мы должны ".to_string(),
            text_after: " встретиться завтра.".to_string(),
            ..Default::default()
        };
        assert_eq!(classify_insertion_mode(&ctx), InsertionMode::MidSentenceInsertion);
    }

    #[test]
    fn test_classify_sentence_continuation() {
        let ctx = CaretContext {
            text_before: "Я пошел в магазин и купил ".to_string(),
            text_after: "".to_string(),
            ..Default::default()
        };
        assert_eq!(classify_insertion_mode(&ctx), InsertionMode::SentenceContinuation);
    }

    #[test]
    fn test_classify_selection_replacement() {
        let ctx = CaretContext {
            text_before: "Мы должны ".to_string(),
            selected_text: "быстро".to_string(),
            text_after: " встретиться.".to_string(),
            ..Default::default()
        };
        assert_eq!(classify_insertion_mode(&ctx), InsertionMode::SelectionReplacement);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path 2.0/src-tauri/Cargo.toml test_classify_`
Expected: FAIL (module or types not found)

- [ ] **Step 3: Implement `CaretContext`, `InsertionMode`, and `classify_insertion_mode` in `smart_insertion.rs`**

```rust
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CaretContext {
    pub text_before: String,
    pub selected_text: String,
    pub text_after: String,
    pub source: CaretContextSource,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CaretContextSource {
    #[default]
    None,
    UiaTextPattern,
    UiaValuePattern,
    Win32Edit,
    ClipboardFallback,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InsertionMode {
    NewSentence,
    MidSentenceInsertion,
    SentenceContinuation,
    SelectionReplacement,
}

pub fn classify_insertion_mode(ctx: &CaretContext) -> InsertionMode {
    if !ctx.selected_text.trim().is_empty() {
        return InsertionMode::SelectionReplacement;
    }
    let before_trimmed = ctx.text_before.trim_end();
    if before_trimmed.is_empty() {
        return InsertionMode::NewSentence;
    }
    let last_char = before_trimmed.chars().last().unwrap_or('\0');
    let is_terminal = matches!(last_char, '.' | '!' | '?' | '\n' | '\r' | '…');
    if is_terminal {
        InsertionMode::NewSentence
    } else {
        let after_trimmed = ctx.text_after.trim_start();
        if after_trimmed.is_empty() {
            InsertionMode::SentenceContinuation
        } else {
            InsertionMode::MidSentenceInsertion
        }
    }
}
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path 2.0/src-tauri/Cargo.toml test_classify_`
Expected: PASS (all 5 tests pass)

- [ ] **Step 5: Commit**

```bash
git add 2.0/src-tauri/src/smart_insertion.rs 2.0/src-tauri/src/lib.rs
git commit -m "feat(smart-insertion): add CaretContext and insertion mode classifier"
```

---

### Task 2: Smart Text Transformation Engine (Casing, Trailing Punctuation, Whitespace)

**Files:**
- Modify: `2.0/src-tauri/src/smart_insertion.rs`
- Test: `2.0/src-tauri/src/smart_insertion.rs`

**Interfaces:**
- Consumes: `CaretContext`, `InsertionMode`, `classify_insertion_mode`.
- Produces: `adapt_transcription_to_context(raw_text: &str, ctx: &CaretContext, trailing_period_on_continuation: bool) -> String`.

- [ ] **Step 1: Write failing unit tests for `adapt_transcription_to_context`**

```rust
#[test]
fn test_adapt_mid_sentence_lowercases_and_strips_period() {
    let ctx = CaretContext {
        text_before: "Мы должны ".to_string(),
        text_after: " закончить проект.".to_string(),
        ..Default::default()
    };
    let adapted = adapt_transcription_to_context("Как можно скорее.", &ctx, true);
    assert_eq!(adapted, "как можно скорее ");
}

#[test]
fn test_adapt_preserves_acronyms_mid_sentence() {
    let ctx = CaretContext {
        text_before: "Используйте наш ".to_string(),
        text_after: " для интеграции.".to_string(),
        ..Default::default()
    };
    let adapted = adapt_transcription_to_context("REST API.", &ctx, true);
    assert_eq!(adapted, "REST API ");
}

#[test]
fn test_adapt_continuation_respects_trailing_period_setting() {
    let ctx = CaretContext {
        text_before: "Я купил ".to_string(),
        text_after: "".to_string(),
        ..Default::default()
    };
    let with_period = adapt_transcription_to_context("Пачку чая.", &ctx, true);
    assert_eq!(with_period, "пачку чая.");

    let without_period = adapt_transcription_to_context("Пачку чая.", &ctx, false);
    assert_eq!(without_period, "пачку чая");
}

#[test]
fn test_adapt_boundary_spaces_prepends_if_missing() {
    let ctx = CaretContext {
        text_before: "Я купил".to_string(), // no space after купил
        text_after: "".to_string(),
        ..Default::default()
    };
    let adapted = adapt_transcription_to_context("пачку чая.", &ctx, true);
    assert_eq!(adapted, " пачку чая.");
}

#[test]
fn test_adapt_no_punctuation_duplication_before_comma() {
    let ctx = CaretContext {
        text_before: "Мы должны ".to_string(),
        text_after: ", конечно, успеть.".to_string(),
        ..Default::default()
    };
    let adapted = adapt_transcription_to_context("постараться.", &ctx, true);
    assert_eq!(adapted, "постараться");
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path 2.0/src-tauri/Cargo.toml test_adapt_`
Expected: FAIL (`adapt_transcription_to_context` not found)

- [ ] **Step 3: Implement `adapt_transcription_to_context`**

Implement the deterministic transformation algorithm in `smart_insertion.rs`:
- Acronym detection: check if first word is >=2 uppercase chars without lowercase.
- Lowercase first char when in `MidSentenceInsertion` or `SentenceContinuation` (unless acronym).
- Strip trailing period `.` when in `MidSentenceInsertion` or when `trailing_period_on_continuation == false` in `SentenceContinuation`.
- Strip trailing period if `text_after` begins with punctuation (avoiding `..` or `.,`).
- Ensure boundary whitespace: prepend space if `text_before` lacks trailing whitespace; append space if `MidSentenceInsertion` and `text_after` lacks leading whitespace and does not begin with punctuation.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path 2.0/src-tauri/Cargo.toml test_adapt_`
Expected: PASS (all 5 tests pass)

- [ ] **Step 5: Commit**

```bash
git add 2.0/src-tauri/src/smart_insertion.rs
git commit -m "feat(smart-insertion): implement casing, trailing punctuation, and boundary space rules"
```

---

### Task 3: Windows Caret Context Extractor (`caret_context.rs`)

**Files:**
- Create: `2.0/src-tauri/src/caret_context.rs`
- Modify: `2.0/src-tauri/Cargo.toml` (add `Win32_UI_Accessibility` & `Win32_System_Com` to `windows` 0.54)
- Modify: `2.0/src-tauri/src/lib.rs` (register module)
- Test: `2.0/src-tauri/src/caret_context.rs`

**Interfaces:**
- Produces: `pub fn capture_caret_context(timeout_ms: u64) -> CaretContext`.

- [ ] **Step 1: Write mock and safety tests for `capture_caret_context`**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_timeout_safety() {
        let start = std::time::Instant::now();
        let _ctx = capture_caret_context(20);
        let elapsed = start.elapsed();
        // Must complete or abort within safety window (<100ms)
        assert!(elapsed.as_millis() < 100);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path 2.0/src-tauri/Cargo.toml test_capture_`
Expected: FAIL (`caret_context` not found)

- [ ] **Step 3: Implement `capture_caret_context` using Windows UI Automation**

In `2.0/src-tauri/src/caret_context.rs`:
- Use `windows::Win32::System::Com::{CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED}`.
- Use `windows::Win32::UI::Accessibility::{CUIAutomation, IUIAutomation, IUIAutomationTextPattern, TextPatternRangeEndpoint_Start, TextPatternRangeEndpoint_End, TextUnit_Character}`.
- Query `GetFocusedElement`, retrieve `TextPattern`, call `GetSelection`.
- Extract `text_before` (100 chars backward), `text_after` (100 chars forward), and `selected_text`.
- Wrap execution in a channel with `timeout_ms` (20 ms default). If COM stalls or fails, return `CaretContext::default()` immediately.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path 2.0/src-tauri/Cargo.toml test_capture_`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add 2.0/src-tauri/Cargo.toml 2.0/src-tauri/src/caret_context.rs 2.0/src-tauri/src/lib.rs
git commit -m "feat(caret-context): implement Windows UI Automation context extractor with 20ms fail-safe"
```

---

### Task 4: Core Pipeline Integration in `lib.rs` & `llm_runner.rs`

**Files:**
- Modify: `2.0/src-tauri/src/lib.rs:2840-2940` (spawn parallel background context capture on hotkey)
- Modify: `2.0/src-tauri/src/lib.rs:3960-4050` (apply smart context adaptation before typing/pasting)
- Modify: `2.0/src-tauri/src/llm_runner.rs` (pass surrounding context into LLM prompt for edit commands)
- Test: `2.0/src-tauri/src/lib.rs` (pipeline unit tests)

**Interfaces:**
- Consumes: `caret_context::capture_caret_context`, `smart_insertion::adapt_transcription_to_context`.
- Stores: `state.caret_context: Mutex<Option<CaretContext>>`.

- [ ] **Step 1: Write unit test verifying pipeline integration with mock context**

```rust
#[test]
fn test_pipeline_smart_context_flow() {
    let raw = "Пачку чая.";
    let ctx = CaretContext {
        text_before: "Я пошел и купил ".to_string(),
        text_after: "".to_string(),
        ..Default::default()
    };
    let adapted = smart_insertion::adapt_transcription_to_context(raw, &ctx, true);
    assert_eq!(adapted, "пачку чая.");
}
```

- [ ] **Step 2: Run test to verify it passes**

Run: `cargo test --manifest-path 2.0/src-tauri/Cargo.toml test_pipeline_smart_context_flow`
Expected: PASS

- [ ] **Step 3: Integrate context capture and adaptation into `lib.rs`**

- In `AppState`: add `pub caret_context: Mutex<Option<CaretContext>>`.
- In `start_recording_session`: spawn parallel thread calling `caret_context::capture_caret_context(20)` and storing result in `state.caret_context`.
- If selection is detected via UIA without clipboard, populate `state.selected_text` without needing `Ctrl+C`.
- In `stop_recording_session` / transcription result:
  Retrieve `state.caret_context`. If `settings.smart_context_enabled`, apply `adapt_transcription_to_context`.
- If LLM edit pass is active, provide `text_before` and `text_after` as surrounding sentence context in system instructions.

- [ ] **Step 4: Run full Rust test suite**

Run: `cargo test --manifest-path 2.0/src-tauri/Cargo.toml`
Expected: All tests PASS

- [ ] **Step 5: Commit**

```bash
git add 2.0/src-tauri/src/lib.rs 2.0/src-tauri/src/llm_runner.rs
git commit -m "feat(pipeline): wire caret context capture and smart adaptation into recording pipeline"
```

---

### Task 5: Settings Persistence in `settings_secure.rs`

**Files:**
- Modify: `2.0/src-tauri/src/settings_secure.rs`
- Test: `2.0/src-tauri/src/settings_secure.rs`

**Interfaces:**
- Produces: `settings.smart_context_enabled: bool`, `settings.smart_context_trailing_period: bool`.

- [ ] **Step 1: Write test for new settings serialization and defaults**

```rust
#[test]
fn test_smart_context_settings_defaults() {
    let settings = Settings::default();
    assert!(settings.smart_context_enabled);
    assert!(settings.smart_context_trailing_period);

    let json = serde_json::json!({});
    let loaded: Settings = serde_json::from_value(json).unwrap();
    assert!(loaded.smart_context_enabled);
    assert!(loaded.smart_context_trailing_period);
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --manifest-path 2.0/src-tauri/Cargo.toml test_smart_context_settings_defaults`
Expected: FAIL (fields not found)

- [ ] **Step 3: Add `smart_context_enabled` and `smart_context_trailing_period` to `Settings`**

In `2.0/src-tauri/src/settings_secure.rs`:
```rust
#[serde(default = "default_true")]
pub smart_context_enabled: bool,

#[serde(default = "default_true")]
pub smart_context_trailing_period: bool,
```

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test --manifest-path 2.0/src-tauri/Cargo.toml test_smart_context_settings_defaults`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add 2.0/src-tauri/src/settings_secure.rs
git commit -m "feat(settings): add smart_context_enabled and smart_context_trailing_period settings"
```

---

### Task 6: Frontend UI Controls & 10-Language Localization

**Files:**
- Modify: `2.0/src/index.html` (add smart context card in «Текст» tab)
- Modify: `2.0/src/main.js` (add bindings, load/save synchronization, and translations for 10 languages)
- Modify: `2.0/src/style.css` (styling conforming to Stealth Command Deck invariants)
- Test: `2.0/tests/frontend-static.test.mjs`

**Interfaces:**
- Elements: `#toggle-smart-context`, `#group-smart-context-trailing-period`, `#toggle-smart-context-trailing-period`.

- [ ] **Step 1: Write frontend test asserting settings controls and translations exist**

In `2.0/tests/frontend-static.test.mjs`:
Assert that all 10 language dictionaries contain:
- `smart_context_label`
- `smart_context_sub`
- `smart_context_trailing_period_label`
- `smart_context_trailing_period_sub`

- [ ] **Step 2: Run test to verify it fails**

Run: `npm test`
Expected: FAIL (missing locale keys)

- [ ] **Step 3: Implement HTML markup, CSS, JS event bindings, and all 10 locale translations**

- `index.html`: add clean switch for `#toggle-smart-context` and dependent sub-toggle `#toggle-smart-context-trailing-period` in `#card-selection-edit`.
- `main.js`: bind `change` events, sync with Tauri IPC `set_settings`, populate all 10 locales (`ru`, `en`, `de`, `es`, `fr`, `it`, `zh`, `pt`, `tr`, `nl`).
- `style.css`: clean layout, padding `12px/16px`, zero solid orange hovers, Tactical Orange <= 10%.

- [ ] **Step 4: Run frontend tests to verify they pass**

Run: `npm test` and `npm run check`
Expected: PASS (all tests pass, 0 errors)

- [ ] **Step 5: Commit**

```bash
git add 2.0/src/index.html 2.0/src/main.js 2.0/src/style.css 2.0/tests/frontend-static.test.mjs
git commit -m "feat(ui): add smart context-aware insertion controls with 10-language localization"
```

---

### Task 7: Full System Verification & Quality Gates

**Files:**
- Run existing check scripts: `npm test`, `cargo test`, `check-ipc.mjs`, `check-css-invariants.mjs`, `check-aria.mjs`, `audit-all-elements.mjs`.
- Visual QA Pre-flight: CDP headless screenshot of the «Текст» tab.

- [ ] **Step 1: Run full Rust test suite**

Run: `cargo test --manifest-path 2.0/src-tauri/Cargo.toml`
Expected: >= 175 tests PASS, 0 FAIL

- [ ] **Step 2: Run Rust Clippy**

Run: `cargo clippy --manifest-path 2.0/src-tauri/Cargo.toml --all-targets -- -D warnings`
Expected: Clean (0 warnings, 0 errors)

- [ ] **Step 3: Run full Frontend test suite & Static Integrity scripts**

Run: `npm test && npm run check && node scripts/check-ipc.mjs && node scripts/check-css-invariants.mjs && node scripts/check-aria.mjs`
Expected: Clean (all exit with 0)

- [ ] **Step 4: Visual QA Pre-flight check via Headless Edge/Chrome**

Capture screenshot of the «Текст» tab with the new controls enabled and disabled. Inspect typography, alignment, contrast, and spacing.

- [ ] **Step 5: Update project tracking artifacts and commit**

Update `feature_list.json`, `progress.md`, and Obsidian Vault (`Aura.md`).
```bash
git add -A
git commit -m "chore: record smart context insertion completion and quality evidence"
```
