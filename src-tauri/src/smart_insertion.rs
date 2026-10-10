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

    // Check if cursor is at the start of text or following a newline/paragraph break
    let trimmed_horizontal_spaces = ctx.text_before.trim_end_matches([' ', '\t']);
    if trimmed_horizontal_spaces.is_empty()
        || trimmed_horizontal_spaces.ends_with('\n')
        || trimmed_horizontal_spaces.ends_with('\r')
    {
        return InsertionMode::NewSentence;
    }

    let before_trimmed = ctx.text_before.trim_end();
    // Strip trailing closing quotes and closing brackets before inspecting terminal punctuation
    let punct_candidate = before_trimmed.trim_end_matches([
        '"', '\'', '»', '”', '’', ')', ']', '}', '›', '“', '‘',
    ]);
    let last_char = punct_candidate.chars().last().unwrap_or('\0');
    let is_terminal = matches!(last_char, '.' | '!' | '?' | '…');

    if is_terminal {
        InsertionMode::NewSentence
    } else {
        let after_on_current_line = ctx.text_after.trim_start_matches([' ', '\t']);
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
        "т.д.", "т.п.", "т.е.", "т.к.", "др.", "пр.", "руб.", "коп.", "г.", "ул.",
        "etc.", "e.g.", "i.e.", "vs.", "al.", "approx.", "dept.", "fig.", "inc.",
        "ltd.", "prof.", "vol.",
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
            let before_trimmed = ctx.text_before.trim_end();
            let punct_candidate = before_trimmed.trim_end_matches([
                '"', '\'', '»', '”', '’', ')', ']', '}', '›', '“', '‘',
            ]);
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
    let after_trimmed = ctx.text_after.trim_start();
    let after_starts_with_dot = after_trimmed.starts_with('.');
    let after_starts_with_other_punct = after_trimmed.starts_with(|c: char| {
        matches!(c, ',' | '!' | '?' | ':' | ';' | '-' | '—' | ')' | ']' | '}' | '»' | '"')
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
    if !ctx.text_before.is_empty() {
        let last_before = ctx.text_before.chars().last().unwrap_or('\0');
        let before_has_whitespace = matches!(last_before, ' ' | '\t' | '\n' | '\r');
        let before_is_open_bracket = matches!(last_before, '(' | '[' | '{' | '«' | '"');
        if !before_has_whitespace && !before_is_open_bracket {
            result = format!(" {}", result);
        }
    }

    // Trailing space check
    if mode == InsertionMode::MidSentenceInsertion && !ctx.text_after.is_empty() {
        let first_after = ctx.text_after.chars().next().unwrap_or('\0');
        let after_has_whitespace = matches!(first_after, ' ' | '\t' | '\n' | '\r');
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
        let adapted_clean = adapt_transcription_to_context("Как можно скорее.", &ctx_has_space, true);
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
        assert_eq!(classify_insertion_mode(&ctx_quotes), InsertionMode::NewSentence);

        let ctx_guillemets = CaretContext {
            text_before: "«Это важно!» ".to_string(),
            ..Default::default()
        };
        assert_eq!(classify_insertion_mode(&ctx_guillemets), InsertionMode::NewSentence);

        let ctx_brackets = CaretContext {
            text_before: "(См. главу 1.) ".to_string(),
            ..Default::default()
        };
        assert_eq!(classify_insertion_mode(&ctx_brackets), InsertionMode::NewSentence);
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
}
