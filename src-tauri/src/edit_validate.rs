//! Validation gate for AI text editing results.
//!
//! Guarantees that:
//! 1. The result is never an empty response or an unedited echo of the input.
//! 2. For translations, the output script aligns with the target language.
//! 3. For shortening, the result is actually shorter than the original text.
//! 4. For grammar fixes, the text does not hallucinate new content or collapse.

use crate::edit_intent::{is_primarily_cyrillic, EditIntent};

#[derive(Debug, PartialEq, Eq)]
pub enum ValidationError {
    EmptyResult,
    EchoUnmodified,
    EchoSpokenCommand,
    RepetitionLoop,
    NotShortened,
    WrongScript,
    LengthOutOfBounds,
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::EmptyResult => write!(f, "Model returned an empty response"),
            ValidationError::EchoUnmodified => {
                write!(f, "Model returned the original text without edits")
            }
            ValidationError::EchoSpokenCommand => write!(
                f,
                "Model answered with the command itself instead of applying it"
            ),
            ValidationError::RepetitionLoop => write!(f, "Model fell into a repetition loop"),
            ValidationError::NotShortened => write!(f, "Model did not shorten the text"),
            ValidationError::WrongScript => write!(
                f,
                "Translation output script did not match the target language"
            ),
            ValidationError::LengthOutOfBounds => write!(
                f,
                "Edited text length deviated excessively from expected bounds"
            ),
        }
    }
}

/// Normalizes text for comparison by collapsing whitespace and stripping punctuation.
fn normalize_for_comparison(text: &str) -> String {
    text.chars()
        .map(|c| if c.is_alphanumeric() { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Validates an edited result against the intended action and input parameters.
pub fn validate_edit_result(
    intent: &EditIntent,
    original_selection: &str,
    spoken_command: &str,
    output: &str,
) -> Result<(), ValidationError> {
    let output_trimmed = output.trim();
    if output_trimmed.is_empty() {
        return Err(ValidationError::EmptyResult);
    }

    let norm_output = normalize_for_comparison(output_trimmed);
    let norm_orig = normalize_for_comparison(original_selection);
    let norm_cmd = normalize_for_comparison(spoken_command);

    // 1. Output must not be an echo of the command phrase itself (e.g. saying "переведи" and getting "переведи")
    if !norm_cmd.is_empty() && norm_output == norm_cmd {
        return Err(ValidationError::EchoSpokenCommand);
    }

    match intent {
        EditIntent::Translate { target_lang } => {
            if target_lang == "ru" {
                // Should have Cyrillic letters
                if !is_primarily_cyrillic(output_trimmed) && original_selection.chars().count() > 8
                {
                    return Err(ValidationError::WrongScript);
                }
            } else if ["en", "de", "es", "fr", "it", "pt", "nl", "tr"]
                .contains(&target_lang.as_str())
            {
                // Should NOT be primarily Cyrillic if translating to Latin-based European languages
                if is_primarily_cyrillic(output_trimmed) && original_selection.chars().count() > 8 {
                    return Err(ValidationError::WrongScript);
                }
            } else if target_lang == "zh" {
                let has_cjk = output_trimmed
                    .chars()
                    .any(|c| ('\u{4E00}'..='\u{9FFF}').contains(&c));
                if !has_cjk && original_selection.chars().count() > 8 {
                    return Err(ValidationError::WrongScript);
                }
            }
        }
        EditIntent::Shorten => {
            let orig_len = original_selection.chars().count();
            let out_len = output_trimmed.chars().count();
            // If original had more than 20 characters, shortened text MUST be shorter
            if orig_len > 20 && out_len >= orig_len {
                return Err(ValidationError::NotShortened);
            }
        }
        EditIntent::FixGrammar => {
            let orig_len = original_selection.chars().count();
            let out_len = output_trimmed.chars().count();
            // A grammar fix should never be less than 40% or more than 250% of original
            if orig_len > 25 && (out_len < orig_len * 4 / 10 || out_len > orig_len * 5 / 2) {
                return Err(ValidationError::LengthOutOfBounds);
            }
        }
        _ => {}
    }

    // 2. Output script must match original selection for all non-translation intents
    let is_translation_custom = if let EditIntent::Custom(instruction) = intent {
        let ins_lower = instruction.to_lowercase();
        ins_lower.contains("перевод")
            || ins_lower.contains("переведи")
            || ins_lower.contains("перевести")
            || ins_lower.contains("translate")
            || ins_lower.contains("traduc")
            || ins_lower.contains("übersetz")
            || ins_lower.contains("ubersetz")
            || ins_lower.contains("vertal")
            || ins_lower.contains("çevir")
            || ins_lower.contains("cevir")
            || ins_lower.contains("tercüm")
            || ins_lower.contains("tercum")
            || ins_lower.contains("翻译")
    } else {
        false
    };

    if !matches!(intent, EditIntent::Translate { .. }) && !is_translation_custom {
        let orig_alpha = original_selection
            .chars()
            .filter(|c| c.is_alphabetic())
            .count();
        let out_alpha = output_trimmed.chars().filter(|c| c.is_alphabetic()).count();
        if orig_alpha >= 8 && out_alpha >= 8 {
            let orig_cyrillic = is_primarily_cyrillic(original_selection);
            let out_cyrillic = is_primarily_cyrillic(output_trimmed);
            if orig_cyrillic != out_cyrillic {
                return Err(ValidationError::WrongScript);
            }
        }
    }

    // 3. Output must not be identical to original selection (unless intent was FixGrammar on perfectly correct short text)
    if norm_output == norm_orig && intent != &EditIntent::FixGrammar {
        return Err(ValidationError::EchoUnmodified);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_successful_translation() {
        let res = validate_edit_result(
            &EditIntent::Translate {
                target_lang: "en".to_string(),
            },
            "Привет, как дела?",
            "переведи на английский",
            "Hello, how are you?",
        );
        assert!(res.is_ok());
    }

    #[test]
    fn rejects_command_self_translation_echo() {
        let res = validate_edit_result(
            &EditIntent::Translate {
                target_lang: "en".to_string(),
            },
            "Привет, мир!",
            "translate to english",
            "translate to english",
        );
        assert_eq!(res, Err(ValidationError::EchoSpokenCommand));
    }

    #[test]
    fn rejects_unshortened_text() {
        let res = validate_edit_result(
            &EditIntent::Shorten,
            "Это очень длинный текст, который нужно обязательно сократить для краткости.",
            "сократи",
            "Это очень длинный текст, который нужно обязательно сократить для краткости и полноты понимания темы.",
        );
        assert_eq!(res, Err(ValidationError::NotShortened));
    }

    #[test]
    fn rejects_wrong_alphabet_for_translation() {
        let res = validate_edit_result(
            &EditIntent::Translate {
                target_lang: "en".to_string(),
            },
            "Это важный рабочий документ для всей команды.",
            "переведи на английский",
            "Это важный рабочий документ для всей команды.",
        );
        assert_eq!(res, Err(ValidationError::WrongScript));
    }

    #[test]
    fn rejects_translation_during_shortening() {
        let res = validate_edit_result(
            &EditIntent::Shorten,
            "Найден публичный ключ, но нет приватного ключа. Убедитесь, что переменная установлена.",
            "сократи",
            "Public key found, but no private key. Ensure variable is set.",
        );
        assert_eq!(res, Err(ValidationError::WrongScript));
    }

    #[test]
    fn allows_script_change_for_custom_translation_instruction() {
        let res = validate_edit_result(
            &EditIntent::Custom("сделай перевод на английский".to_string()),
            "Это важный рабочий документ для всей команды.",
            "сделай перевод на английский",
            "This is an important working document for the entire team.",
        );
        assert!(res.is_ok());
    }
}
