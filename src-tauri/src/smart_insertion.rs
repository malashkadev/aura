#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CaretContext {
    pub text_before: String,
    pub selected_text: String,
    pub text_after: String,
    pub source: CaretContextSource,
}

impl CaretContext {
    /// Returns `true` when the selection range was read directly from a text
    /// API (`Win32Edit` or `UiaTextPattern`) so an empty `selected_text`
    /// authoritatively proves no text is selected, allowing callers to skip
    /// synthesizing `Ctrl+C`.
    pub fn has_authoritative_selection(&self) -> bool {
        matches!(
            self.source,
            CaretContextSource::Win32Edit | CaretContextSource::UiaTextPattern
        )
    }
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

pub fn is_whitespace_or_invisible(c: char) -> bool {
    c.is_whitespace()
        || matches!(
            c,
            '\u{200B}'..='\u{200D}' | '\u{FEFF}' | '\u{00AD}' | '\u{FFFC}' | '\u{FFFD}' | '\0'
        )
}

pub fn is_horizontal_whitespace_or_invisible(c: char) -> bool {
    c != '\n' && c != '\r' && is_whitespace_or_invisible(c)
}

pub fn classify_insertion_mode(ctx: &CaretContext) -> InsertionMode {
    if !ctx
        .selected_text
        .trim_matches(is_whitespace_or_invisible)
        .is_empty()
    {
        return InsertionMode::SelectionReplacement;
    }

    // Check if cursor is at the start of text or following a newline/paragraph break
    let trimmed_horizontal_spaces = ctx
        .text_before
        .trim_end_matches(is_horizontal_whitespace_or_invisible);
    if trimmed_horizontal_spaces.is_empty()
        || trimmed_horizontal_spaces.ends_with('\n')
        || trimmed_horizontal_spaces.ends_with('\r')
    {
        return InsertionMode::NewSentence;
    }

    let before_trimmed = ctx.text_before.trim_end_matches(is_whitespace_or_invisible);
    // Strip trailing closing quotes and closing brackets before inspecting terminal punctuation
    let punct_candidate =
        before_trimmed.trim_end_matches(['"', '\'', '»', '”', '’', ')', ']', '}', '›', '“', '‘']);
    let last_char = punct_candidate.chars().last().unwrap_or('\0');
    let is_terminal = matches!(last_char, '.' | '!' | '?' | '…');

    if is_terminal {
        InsertionMode::NewSentence
    } else {
        let after_on_current_line = ctx
            .text_after
            .trim_start_matches(is_horizontal_whitespace_or_invisible);
        if after_on_current_line.is_empty()
            || after_on_current_line.starts_with('\n')
            || after_on_current_line.starts_with('\r')
        {
            InsertionMode::SentenceContinuation
        } else {
            InsertionMode::MidSentenceInsertion
        }
    }
}

fn is_acronym(word: &str) -> bool {
    let clean = word.trim_matches(|c: char| !c.is_alphabetic());
    if clean.chars().count() < 2 {
        return false;
    }
    clean.chars().all(|c| c.is_uppercase())
}

fn preserves_leading_capital(text: &str) -> bool {
    let mut words = text.split_whitespace();
    if let Some(first_word) = words.next() {
        if is_acronym(first_word) {
            return true;
        }
        let clean = first_word.trim_matches(|c: char| !c.is_alphabetic());
        if clean == "I" || first_word.starts_with("I'") || first_word.starts_with("I’") {
            return true;
        }
    }
    false
}

fn ends_with_known_abbreviation(text: &str) -> bool {
    let lower = text.to_lowercase();
    const ABBREVIATIONS: &[&str] = &[
        "т.д.", "т.п.", "т.е.", "т.к.", "др.", "пр.", "руб.", "коп.", "г.", "ул.", "etc.", "e.g.",
        "i.e.", "vs.", "al.", "approx.", "dept.", "fig.", "inc.", "ltd.", "prof.", "vol.",
    ];
    for &abbr in ABBREVIATIONS {
        if lower.ends_with(abbr) {
            return true;
        }
    }
    false
}

fn lowercase_first_char(text: &str) -> String {
    if preserves_leading_capital(text) {
        return text.to_string();
    }
    let mut chars = text.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_lowercase().collect::<String>() + chars.as_str(),
    }
}

fn uppercase_first_char(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

pub fn adapt_transcription_to_context(
    raw_text: &str,
    ctx: &CaretContext,
    trailing_period_on_continuation: bool,
) -> String {
    let trimmed = raw_text.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let mode = classify_insertion_mode(ctx);

    // 1. Casing adjustment
    let mut text = match mode {
        InsertionMode::NewSentence => uppercase_first_char(trimmed),
        InsertionMode::MidSentenceInsertion | InsertionMode::SentenceContinuation => {
            lowercase_first_char(trimmed)
        }
        InsertionMode::SelectionReplacement => {
            let before_trimmed = ctx.text_before.trim_end_matches(is_whitespace_or_invisible);
            let punct_candidate = before_trimmed
                .trim_end_matches(['"', '\'', '»', '”', '’', ')', ']', '}', '›', '“', '‘']);
            let is_terminal = punct_candidate.is_empty()
                || matches!(punct_candidate.chars().last(), Some('.' | '!' | '?' | '…'));
            if is_terminal {
                uppercase_first_char(trimmed)
            } else {
                lowercase_first_char(trimmed)
            }
        }
    };

    // 2. Trailing punctuation adjustment
    let after_trimmed = ctx
        .text_after
        .trim_start_matches(is_whitespace_or_invisible);
    let after_starts_with_dot = after_trimmed.starts_with('.');
    let after_starts_with_other_punct = after_trimmed.starts_with(|c: char| {
        matches!(
            c,
            ',' | '!' | '?' | ':' | ';' | '-' | '—' | ')' | ']' | '}' | '»' | '"'
        )
    });

    if after_starts_with_dot {
        // Prevent duplicate dots ("..")
        while text.ends_with('.') {
            text.pop();
        }
    } else if after_starts_with_other_punct {
        // Prevent punctuation collision unless preserving an established abbreviation dot (e.g. "и т.д., ")
        if !ends_with_known_abbreviation(&text) {
            while text.ends_with('.') {
                text.pop();
            }
        }
    } else {
        match mode {
            InsertionMode::MidSentenceInsertion => {
                if !ends_with_known_abbreviation(&text) {
                    while text.ends_with('.') {
                        text.pop();
                    }
                }
            }
            InsertionMode::SentenceContinuation => {
                if trailing_period_on_continuation {
                    while text.ends_with([',', ';', ':', '-', '—']) {
                        text.pop();
                    }
                    if !text.ends_with(['.', '!', '?', '…']) {
                        text.push('.');
                    }
                } else if !ends_with_known_abbreviation(&text) {
                    while text.ends_with('.') {
                        text.pop();
                    }
                }
            }
            InsertionMode::SelectionReplacement => {
                if !after_trimmed.is_empty() && !ends_with_known_abbreviation(&text) {
                    while text.ends_with('.') {
                        text.pop();
                    }
                }
            }
            InsertionMode::NewSentence => {}
        }
    }

    // 3. Boundary whitespace adjustment
    let mut result = text;

    // Leading space check
    let clean_before = ctx.text_before.trim_end_matches(|c: char| {
        matches!(
            c,
            '\u{200B}'..='\u{200D}' | '\u{FEFF}' | '\u{00AD}' | '\u{FFFC}' | '\u{FFFD}' | '\0'
        )
    });
    if !clean_before.is_empty() {
        let last_before = clean_before.chars().last().unwrap_or('\0');
        let before_has_whitespace = is_whitespace_or_invisible(last_before);
        let before_is_open_bracket = matches!(last_before, '(' | '[' | '{' | '«' | '"');
        if !before_has_whitespace && !before_is_open_bracket {
            result = format!(" {}", result);
        }
    }

    // Trailing space check
    let clean_after = ctx.text_after.trim_start_matches(|c: char| {
        matches!(
            c,
            '\u{200B}'..='\u{200D}' | '\u{FEFF}' | '\u{00AD}' | '\u{FFFC}' | '\u{FFFD}' | '\0'
        )
    });
    if mode == InsertionMode::MidSentenceInsertion && !clean_after.is_empty() {
        let first_after = clean_after.chars().next().unwrap_or('\0');
        let after_has_whitespace = is_whitespace_or_invisible(first_after);
        let after_is_punct = matches!(
            first_after,
            '.' | ',' | '!' | '?' | ':' | ';' | '-' | '—' | ')' | ']' | '}' | '»' | '"'
        );
        if !after_has_whitespace && !after_is_punct {
            result.push(' ');
        }
    }

    result
}

/// Preserves exact leading and trailing whitespace (e.g., a trailing space from
/// double-clicking a word or trailing `\r\n`/`\n` from triple-clicking a paragraph)
/// from `original_selection` around the trimmed LLM output.
pub fn restore_selection_edge_whitespace(original_selection: &str, edited: &str) -> String {
    let trimmed = edited.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    let lead_len = original_selection
        .len()
        .saturating_sub(original_selection.trim_start().len());
    let trail_len = original_selection
        .len()
        .saturating_sub(original_selection.trim_end().len());
    let leading = &original_selection[..lead_len];
    let trailing = if lead_len + trail_len <= original_selection.len() {
        &original_selection[original_selection.len() - trail_len..]
    } else {
        ""
    };
    format!("{leading}{trimmed}{trailing}")
}

/// Deterministically reconciles LLM output (`FixGrammar` / `Translate`) with the
/// surrounding sentence context (`CaretContext`) and exact edge whitespace of the
/// selection.
///
/// - Standalone / end-of-line / multi-line selections keep the model's capitalization
///   and terminal punctuation untouched.
/// - Single-line inline selections inside an existing sentence (e.g. fixing `"севодня"`
///   inside `"Я сделаю это [севодня] вечером."`) avoid unwanted capitalization or a
///   stray period in the middle of the sentence, and never create duplicate `".."`.
pub fn adapt_edit_result_to_selection_context(
    original_selection: &str,
    edited: &str,
    caret_ctx: Option<&CaretContext>,
) -> String {
    let edited_trimmed = edited.trim();
    if edited_trimmed.is_empty() {
        return String::new();
    }

    let orig_trimmed = original_selection.trim();
    let is_single_line = !orig_trimmed.contains('\n')
        && !orig_trimmed.contains('\r')
        && !edited_trimmed.contains('\n')
        && !edited_trimmed.contains('\r');

    let mut adjusted = edited_trimmed.to_string();

    if is_single_line {
        if let Some(ctx) = caret_ctx {
            let has_context = !ctx.text_before.is_empty() || !ctx.text_after.is_empty();
            if has_context {
                // Check if `text_before` continues a sentence on the same line.
                let before_h = ctx
                    .text_before
                    .trim_end_matches(is_horizontal_whitespace_or_invisible);
                let before_continues_sentence = if before_h.is_empty()
                    || before_h.ends_with('\n')
                    || before_h.ends_with('\r')
                {
                    false
                } else {
                    let punct_candidate = before_h
                        .trim_end_matches(['"', '\'', '»', '”', '’', ')', ']', '}', '›', '“', '‘']);
                    !matches!(
                        punct_candidate.chars().last(),
                        None | Some('.' | '!' | '?' | '…' | '。' | '！' | '？' | ':')
                    )
                };

                // If the selection sat mid-sentence and started lowercase, do not let
                // sentence-mode proofreading capitalize the first word (acronyms & "I" stay safe).
                let orig_starts_lowercase = orig_trimmed
                    .chars()
                    .find(|c| c.is_alphabetic())
                    .is_some_and(|c| c.is_lowercase());
                if before_continues_sentence && orig_starts_lowercase {
                    adjusted = lowercase_first_char(&adjusted);
                }

                // Check if `text_after` continues the same sentence on the same line.
                let orig_ended_terminal = matches!(
                    orig_trimmed.chars().last(),
                    Some('.' | '!' | '?' | '…' | '。' | '！' | '？')
                );
                if !orig_ended_terminal {
                    let after_on_line = ctx
                        .text_after
                        .trim_start_matches(is_horizontal_whitespace_or_invisible);
                    if !after_on_line.is_empty()
                        && !after_on_line.starts_with('\n')
                        && !after_on_line.starts_with('\r')
                    {
                        let first_after = after_on_line.chars().next().unwrap_or('\0');
                        let after_is_dot = matches!(first_after, '.' | '。');
                        let after_is_continuation_punct = matches!(
                            first_after,
                            ',' | ';' | ':' | '!' | '?' | ')' | ']' | '}' | '»' | '"' | '-' | '—'
                        );
                        let after_starts_lowercase = first_after.is_lowercase();

                        let should_strip_added_terminal = after_is_dot
                            || after_is_continuation_punct
                            || after_starts_lowercase
                            || before_continues_sentence;

                        if should_strip_added_terminal
                            && !ends_with_known_abbreviation(&adjusted)
                            && !adjusted.ends_with("...")
                            && matches!(
                                adjusted.chars().last(),
                                Some('.' | '?' | '!' | '。' | '？' | '！')
                            )
                        {
                            adjusted.pop();
                        }
                    }
                }
            }
        }
    }

    restore_selection_edge_whitespace(original_selection, &adjusted)
}

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
        assert_eq!(
            classify_insertion_mode(&ctx_newline),
            InsertionMode::NewSentence
        );
    }

    #[test]
    fn test_classify_mid_sentence_insertion() {
        let ctx = CaretContext {
            text_before: "Мы должны ".to_string(),
            text_after: " встретиться завтра.".to_string(),
            ..Default::default()
        };
        assert_eq!(
            classify_insertion_mode(&ctx),
            InsertionMode::MidSentenceInsertion
        );
    }

    #[test]
    fn test_classify_sentence_continuation() {
        let ctx = CaretContext {
            text_before: "Я пошел в магазин и купил ".to_string(),
            text_after: "".to_string(),
            ..Default::default()
        };
        assert_eq!(
            classify_insertion_mode(&ctx),
            InsertionMode::SentenceContinuation
        );
    }

    #[test]
    fn test_classify_selection_replacement() {
        let ctx = CaretContext {
            text_before: "Мы должны ".to_string(),
            selected_text: "быстро".to_string(),
            text_after: " встретиться.".to_string(),
            ..Default::default()
        };
        assert_eq!(
            classify_insertion_mode(&ctx),
            InsertionMode::SelectionReplacement
        );
    }

    #[test]
    fn test_adapt_mid_sentence_lowercases_and_strips_period() {
        // Case A: text_after has no leading space -> adds trailing space
        let ctx_no_space = CaretContext {
            text_before: "Мы должны ".to_string(),
            text_after: "закончить проект.".to_string(),
            ..Default::default()
        };
        let adapted = adapt_transcription_to_context("Как можно скорее.", &ctx_no_space, true);
        assert_eq!(adapted, "как можно скорее ");

        // Case B: text_after already has leading space -> avoids double space
        let ctx_has_space = CaretContext {
            text_before: "Мы должны ".to_string(),
            text_after: " закончить проект.".to_string(),
            ..Default::default()
        };
        let adapted_clean =
            adapt_transcription_to_context("Как можно скорее.", &ctx_has_space, true);
        assert_eq!(adapted_clean, "как можно скорее");
    }

    #[test]
    fn test_adapt_preserves_acronyms_mid_sentence() {
        let ctx = CaretContext {
            text_before: "Используйте наш ".to_string(),
            text_after: "для интеграции.".to_string(),
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

    #[test]
    fn test_adapt_preserves_english_i_pronoun() {
        let ctx = CaretContext {
            text_before: "As ".to_string(),
            text_after: " said earlier.".to_string(),
            ..Default::default()
        };
        let adapted = adapt_transcription_to_context("I think so.", &ctx, true);
        assert_eq!(adapted, "I think so");
    }

    #[test]
    fn test_classify_new_sentence_after_closing_quotes_and_brackets() {
        let ctx_quotes = CaretContext {
            text_before: "Он сказал: \"Пора.\" ".to_string(),
            ..Default::default()
        };
        assert_eq!(
            classify_insertion_mode(&ctx_quotes),
            InsertionMode::NewSentence
        );

        let ctx_guillemets = CaretContext {
            text_before: "«Это важно!» ".to_string(),
            ..Default::default()
        };
        assert_eq!(
            classify_insertion_mode(&ctx_guillemets),
            InsertionMode::NewSentence
        );

        let ctx_brackets = CaretContext {
            text_before: "(См. главу 1.) ".to_string(),
            ..Default::default()
        };
        assert_eq!(
            classify_insertion_mode(&ctx_brackets),
            InsertionMode::NewSentence
        );
    }

    #[test]
    fn test_adapt_preserves_abbreviation_dots() {
        let ctx = CaretContext {
            text_before: "Купи фрукты, овощи ".to_string(),
            text_after: "для праздника.".to_string(),
            ..Default::default()
        };
        let adapted_ru = adapt_transcription_to_context("и т.д.", &ctx, true);
        assert_eq!(adapted_ru, "и т.д. ");

        let adapted_en = adapt_transcription_to_context("etc.", &ctx, true);
        assert_eq!(adapted_en, "etc. ");
    }

    #[test]
    fn test_adapt_continuation_enforces_period_when_raw_text_lacks_dot() {
        let ctx = CaretContext {
            text_before: "Я купил ".to_string(),
            text_after: "".to_string(),
            ..Default::default()
        };
        // Raw text without any punctuation should have a period pushed
        let adapted = adapt_transcription_to_context("пачку чая", &ctx, true);
        assert_eq!(adapted, "пачку чая.");

        // Trailing comma/colon replaced by period
        let adapted_comma = adapt_transcription_to_context("пачку чая,", &ctx, true);
        assert_eq!(adapted_comma, "пачку чая.");

        // Exclamation / question marks preserved
        let adapted_excl = adapt_transcription_to_context("пачку чая!", &ctx, true);
        assert_eq!(adapted_excl, "пачку чая!");
    }

    #[test]
    fn test_classify_sentence_continuation_before_newline() {
        let ctx_newline = CaretContext {
            text_before: "Мы обсудили это ".to_string(),
            text_after: "\nСледующий раздел документа".to_string(),
            ..Default::default()
        };
        assert_eq!(
            classify_insertion_mode(&ctx_newline),
            InsertionMode::SentenceContinuation
        );

        let adapted = adapt_transcription_to_context("и приняли решение", &ctx_newline, true);
        assert_eq!(adapted, "и приняли решение.");
    }

    #[test]
    fn test_classify_new_sentence_with_invisible_and_unicode_spaces() {
        // Zero-width space
        let ctx_zws = CaretContext {
            text_before: "\u{200B}".to_string(),
            ..Default::default()
        };
        assert_eq!(
            classify_insertion_mode(&ctx_zws),
            InsertionMode::NewSentence
        );

        // Non-breaking space
        let ctx_nbsp = CaretContext {
            text_before: "\u{00A0}".to_string(),
            ..Default::default()
        };
        assert_eq!(
            classify_insertion_mode(&ctx_nbsp),
            InsertionMode::NewSentence
        );

        // Object replacement character (UIA embedded object like '+' button in Gemini)
        let ctx_obj = CaretContext {
            text_before: "\u{FFFC}".to_string(),
            ..Default::default()
        };
        assert_eq!(
            classify_insertion_mode(&ctx_obj),
            InsertionMode::NewSentence
        );

        // Mixed invisible and whitespace
        let ctx_mixed = CaretContext {
            text_before: " \u{200B}\u{00A0}\t\u{FEFF} ".to_string(),
            ..Default::default()
        };
        assert_eq!(
            classify_insertion_mode(&ctx_mixed),
            InsertionMode::NewSentence
        );
    }

    #[test]
    fn test_adapt_preserves_capitalization_and_avoids_space_in_empty_field_with_invisibles() {
        let ctx_zws = CaretContext {
            text_before: "\u{200B}".to_string(),
            ..Default::default()
        };
        let adapted = adapt_transcription_to_context("Привет, как твои дела?", &ctx_zws, true);
        assert_eq!(adapted, "Привет, как твои дела?");

        let ctx_obj = CaretContext {
            text_before: "\u{FFFC}".to_string(),
            ..Default::default()
        };
        let adapted_obj = adapt_transcription_to_context("Привет, как твои дела?", &ctx_obj, true);
        assert_eq!(adapted_obj, "Привет, как твои дела?");

        let ctx_nbsp = CaretContext {
            text_before: "\u{00A0}".to_string(),
            ..Default::default()
        };
        let adapted_nbsp =
            adapt_transcription_to_context("Привет, как твои дела?", &ctx_nbsp, true);
        assert_eq!(adapted_nbsp, "Привет, как твои дела?");
    }

    #[test]
    fn test_caret_context_authoritative_selection() {
        for (source, expected) in [
            (CaretContextSource::Win32Edit, true),
            (CaretContextSource::UiaTextPattern, true),
            (CaretContextSource::UiaValuePattern, false),
            (CaretContextSource::ClipboardFallback, false),
            (CaretContextSource::None, false),
        ] {
            let ctx = CaretContext {
                source,
                ..Default::default()
            };
            assert_eq!(ctx.has_authoritative_selection(), expected, "{source:?}");
        }
    }

    #[test]
    fn test_restore_selection_edge_whitespace_preserves_double_and_triple_click_bounds() {
        assert_eq!(
            restore_selection_edge_whitespace("превет ", "Привет"),
            "Привет "
        );
        assert_eq!(
            restore_selection_edge_whitespace("  hello  ", "Hello."),
            "  Hello.  "
        );
        assert_eq!(
            restore_selection_edge_whitespace("Строка без точки\r\n", "Строка без точки."),
            "Строка без точки.\r\n"
        );
    }

    #[test]
    fn test_adapt_edit_result_inline_vs_standalone_sentence() {
        // Standalone sentence at end of document: keeps capital and added final period.
        let standalone = CaretContext {
            text_before: String::new(),
            selected_text: "Я скоро буду дома".to_string(),
            text_after: String::new(),
            source: CaretContextSource::Win32Edit,
        };
        assert_eq!(
            adapt_edit_result_to_selection_context(
                "Я скоро буду дома",
                "Я скоро буду дома.",
                Some(&standalone)
            ),
            "Я скоро буду дома."
        );

        // First sentence missing period before a new capitalized sentence: keeps added period!
        let before_next_sentence = CaretContext {
            text_before: String::new(),
            selected_text: "Я скоро буду дома".to_string(),
            text_after: " Завтра уеду.".to_string(),
            source: CaretContextSource::Win32Edit,
        };
        assert_eq!(
            adapt_edit_result_to_selection_context(
                "Я скоро буду дома",
                "Я скоро буду дома.",
                Some(&before_next_sentence)
            ),
            "Я скоро буду дома."
        );

        // Single typo word mid-sentence (with double-click trailing space): lowercase + no period + space kept.
        let mid_word = CaretContext {
            text_before: "Я сделаю это ".to_string(),
            selected_text: "севодня ".to_string(),
            text_after: "вечером.".to_string(),
            source: CaretContextSource::Win32Edit,
        };
        assert_eq!(
            adapt_edit_result_to_selection_context("севодня ", "Сегодня.", Some(&mid_word)),
            "сегодня "
        );

        // Fragment right before existing period: prevents double dot "..".
        let before_dot = CaretContext {
            text_before: "Он сказал, ".to_string(),
            selected_text: "што придет".to_string(),
            text_after: ".".to_string(),
            source: CaretContextSource::Win32Edit,
        };
        assert_eq!(
            adapt_edit_result_to_selection_context("што придет", "Что придёт.", Some(&before_dot)),
            "что придёт"
        );

        // Multi-line text is never mutated by inline rules, only edge newlines preserved.
        let multi_orig = "Строка 1\nСтрока 2\n";
        let multi_edited = "Строка 1.\nСтрока 2.";
        assert_eq!(
            adapt_edit_result_to_selection_context(multi_orig, multi_edited, Some(&mid_word)),
            "Строка 1.\nСтрока 2.\n"
        );
    }
}
