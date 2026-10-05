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
    let last_char = before_trimmed.chars().last().unwrap_or('\0');
    let is_terminal = matches!(last_char, '.' | '!' | '?' | '…');

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

fn is_acronym(word: &str) -> bool {
    let clean = word.trim_matches(|c: char| !c.is_alphabetic());
    if clean.chars().count() < 2 {
        return false;
    }
    clean.chars().all(|c| c.is_uppercase())
}

fn lowercase_first_char(text: &str) -> String {
    let mut words = text.split_whitespace();
    if let Some(first_word) = words.next() {
        if is_acronym(first_word) {
            return text.to_string();
        }
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
            let is_terminal = before_trimmed.is_empty()
                || matches!(before_trimmed.chars().last(), Some('.' | '!' | '?' | '…'));
            if is_terminal {
                uppercase_first_char(trimmed)
            } else {
                lowercase_first_char(trimmed)
            }
        }
    };

    // 2. Trailing punctuation adjustment
    let after_trimmed = ctx.text_after.trim_start();
    let after_starts_with_punct = after_trimmed.starts_with(|c: char| {
        matches!(c, '.' | ',' | '!' | '?' | ':' | ';' | '-' | '—' | ')' | ']' | '}' | '»' | '"')
    });

    if after_starts_with_punct {
        // Prevent punctuation collision (e.g. "текст., " or "текст..")
        while text.ends_with('.') {
            text.pop();
        }
    } else {
        match mode {
            InsertionMode::MidSentenceInsertion => {
                // Mid-sentence insertions must never inject terminal periods
                while text.ends_with('.') {
                    text.pop();
                }
            }
            InsertionMode::SentenceContinuation => {
                if !trailing_period_on_continuation {
                    while text.ends_with('.') {
                        text.pop();
                    }
                }
            }
            InsertionMode::SelectionReplacement => {
                if !after_trimmed.is_empty() {
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
}
