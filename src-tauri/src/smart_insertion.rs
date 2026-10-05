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
    let trimmed_horizontal_spaces = ctx.text_before.trim_end_matches(|c: char| c == ' ' || c == '\t');
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
