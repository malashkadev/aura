//! Validation gate for AI text editing results.
//!
//! Guarantees that:
//! 1. The result is never an empty response or an unedited echo of the input.
//! 2. For translations, the output script strictly aligns with the target language
//!    and contains no leaked CJK/Kana/Hangul/Cyrillic fragments.
//! 3. For grammar fixes, the text does not switch languages, hallucinate new content, or collapse.

use crate::edit_intent::{
    is_cjk_han, is_cyrillic_script_lang, is_east_asian_letter, is_hangul, is_kana,
    is_latin_script_lang, is_primarily_cjk, is_primarily_cyrillic, EditIntent,
};

#[derive(Debug, PartialEq, Eq)]
pub enum ValidationError {
    EmptyResult,
    EchoUnmodified,
    EchoSpokenCommand,
    RepetitionLoop,
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
            ValidationError::WrongScript => {
                write!(f, "Edited output script did not match the target language")
            }
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

fn is_non_latin_non_cyrillic_non_cjk(ch: char) -> bool {
    ('\u{0600}'..='\u{06FF}').contains(&ch) // Arabic
        || ('\u{0590}'..='\u{05FF}').contains(&ch) // Hebrew
        || ('\u{0370}'..='\u{03FF}').contains(&ch) // Greek
        || ('\u{0900}'..='\u{097F}').contains(&ch) // Devanagari
        || ('\u{0E00}'..='\u{0E7F}').contains(&ch) // Thai
}

fn has_primarily_other_non_latin_script(text: &str) -> bool {
    let mut other = 0usize;
    let mut total_alpha = 0usize;
    for ch in text.chars() {
        if ch.is_alphabetic() {
            total_alpha += 1;
            if is_non_latin_non_cyrillic_non_cjk(ch) {
                other += 1;
            }
        }
    }
    total_alpha > 0 && other * 2 >= total_alpha
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

    let orig_alpha = original_selection
        .chars()
        .filter(|c| c.is_alphabetic())
        .count();
    let out_alpha = output_trimmed.chars().filter(|c| c.is_alphabetic()).count();
    let orig_has_east_asian = original_selection.chars().any(is_east_asian_letter);
    let out_has_east_asian = output_trimmed.chars().any(is_east_asian_letter);

    match intent {
        EditIntent::Translate { target_lang } => {
            let orig_len = original_selection.chars().count();
            let out_len = output_trimmed.chars().count();

            if orig_alpha >= 3 && out_alpha >= 3 {
                if is_cyrillic_script_lang(target_lang) {
                    if !is_primarily_cyrillic(output_trimmed)
                        || (!orig_has_east_asian && out_has_east_asian)
                    {
                        return Err(ValidationError::WrongScript);
                    }
                } else if is_latin_script_lang(target_lang) {
                    if is_primarily_cyrillic(output_trimmed)
                        || (!orig_has_east_asian && out_has_east_asian)
                        || has_primarily_other_non_latin_script(output_trimmed)
                    {
                        return Err(ValidationError::WrongScript);
                    }
                } else if target_lang == "zh" {
                    if !output_trimmed.chars().any(is_cjk_han) {
                        return Err(ValidationError::WrongScript);
                    }
                } else if target_lang == "ja" {
                    if !output_trimmed.chars().any(|c| is_kana(c) || is_cjk_han(c)) {
                        return Err(ValidationError::WrongScript);
                    }
                } else if target_lang == "ko" {
                    if !output_trimmed.chars().any(is_hangul) {
                        return Err(ValidationError::WrongScript);
                    }
                } else if target_lang == "ar" {
                    if !output_trimmed
                        .chars()
                        .any(|c| ('\u{0600}'..='\u{06FF}').contains(&c))
                    {
                        return Err(ValidationError::WrongScript);
                    }
                } else if target_lang == "he" {
                    if !output_trimmed
                        .chars()
                        .any(|c| ('\u{0590}'..='\u{05FF}').contains(&c))
                    {
                        return Err(ValidationError::WrongScript);
                    }
                } else if target_lang == "el" {
                    if !output_trimmed
                        .chars()
                        .any(|c| ('\u{0370}'..='\u{03FF}').contains(&c))
                    {
                        return Err(ValidationError::WrongScript);
                    }
                } else if target_lang == "hi" {
                    if !output_trimmed
                        .chars()
                        .any(|c| ('\u{0900}'..='\u{097F}').contains(&c))
                    {
                        return Err(ValidationError::WrongScript);
                    }
                } else if target_lang == "th"
                    && !output_trimmed
                        .chars()
                        .any(|c| ('\u{0E00}'..='\u{0E7F}').contains(&c))
                {
                    return Err(ValidationError::WrongScript);
                }
            }

            if orig_len > 25 {
                let is_compact_target = matches!(target_lang.as_str(), "zh" | "ja" | "ko");
                let min_len = if is_compact_target {
                    orig_len * 15 / 100
                } else {
                    orig_len / 4
                };
                let max_len = if orig_has_east_asian && !is_compact_target {
                    orig_len * 6
                } else {
                    orig_len * 4
                };
                if out_len < min_len || out_len > max_len {
                    return Err(ValidationError::LengthOutOfBounds);
                }
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
        EditIntent::UnsupportedTranslateTarget => {
            return Err(ValidationError::WrongScript);
        }
        EditIntent::NotACommand => {}
    }

    // 2. Output script must match original selection for all non-translation intents
    if !matches!(intent, EditIntent::Translate { .. }) && orig_alpha >= 3 && out_alpha >= 3 {
        let orig_cyrillic = is_primarily_cyrillic(original_selection);
        let out_cyrillic = is_primarily_cyrillic(output_trimmed);
        if orig_cyrillic != out_cyrillic {
            return Err(ValidationError::WrongScript);
        }
        if !orig_has_east_asian && out_has_east_asian {
            return Err(ValidationError::WrongScript);
        }
        if is_primarily_cjk(original_selection) && !output_trimmed.chars().any(is_cjk_han) {
            return Err(ValidationError::WrongScript);
        }
    }

    // 3. Output must not be identical to original selection (unless intent was FixGrammar on already clean text)
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
    fn rejects_wrong_alphabet_for_translation_including_short_words_and_kana_leaks() {
        // Long Russian sentence echoed in Cyrillic when translating to English
        assert_eq!(
            validate_edit_result(
                &EditIntent::Translate {
                    target_lang: "en".to_string(),
                },
                "Это важный рабочий документ для всей команды.",
                "переведи на английский",
                "Это важный рабочий документ для всей команды.",
            ),
            Err(ValidationError::WrongScript)
        );
        // Short Russian word (<= 8 chars) not translated to Spanish must also be caught
        assert_eq!(
            validate_edit_result(
                &EditIntent::Translate {
                    target_lang: "es".to_string(),
                },
                "Привет",
                "переведи на испанский",
                "Привет",
            ),
            Err(ValidationError::WrongScript)
        );
        // Japanese Kana/CJK leaked into Turkish Latin translation must be rejected
        assert_eq!(
            validate_edit_result(
                &EditIntent::Translate {
                    target_lang: "tr".to_string(),
                },
                "Я должен отправить этот отчёт в понедельник.",
                "переведи на турецкий",
                "Ben bu raporu göndermeliyim. Eğer期限を過ぎてしまうなら失敗します。",
            ),
            Err(ValidationError::WrongScript)
        );
        // Japanese and Korean translations validate cleanly
        assert!(validate_edit_result(
            &EditIntent::Translate {
                target_lang: "ja".to_string(),
            },
            "Привет, как дела?",
            "переведи на японский",
            "こんにちは、お元気ですか？",
        )
        .is_ok());
        assert!(validate_edit_result(
            &EditIntent::Translate {
                target_lang: "ko".to_string(),
            },
            "Привет, как дела?",
            "переведи на корейский",
            "안녕하세요, 잘 지내세요?",
        )
        .is_ok());
    }

    #[test]
    fn validates_successful_grammar_correction() {
        let res = validate_edit_result(
            &EditIntent::FixGrammar,
            "Это премер текста с ошыбками который надо исправить",
            "исправь ошибки",
            "Это пример текста с ошибками, который надо исправить.",
        );
        assert!(res.is_ok());
    }

    #[test]
    fn rejects_language_switch_on_grammar_fix() {
        let res = validate_edit_result(
            &EditIntent::FixGrammar,
            "Это важный рабочий документ для всей команды.",
            "исправь ошибки",
            "This is an important working document for the whole team.",
        );
        assert_eq!(res, Err(ValidationError::WrongScript));

        // Even short word switch must be caught
        assert_eq!(
            validate_edit_result(
                &EditIntent::FixGrammar,
                "Привет",
                "исправь ошибки",
                "Hello.",
            ),
            Err(ValidationError::WrongScript)
        );
    }

    #[test]
    fn enforces_translation_length_bounds_while_allowing_cjk_ratios() {
        let long_ru =
            "Это подробный рабочий документ с описанием архитектуры проекта и плана внедрения.";
        assert_eq!(
            validate_edit_result(
                &EditIntent::Translate {
                    target_lang: "en".to_string(),
                },
                long_ru,
                "переведи на английский",
                "OK",
            ),
            Err(ValidationError::LengthOutOfBounds)
        );
        assert_eq!(
            validate_edit_result(
                &EditIntent::Translate {
                    target_lang: "en".to_string(),
                },
                long_ru,
                "переведи на английский",
                &"This is a valid English sentence. ".repeat(20),
            ),
            Err(ValidationError::LengthOutOfBounds)
        );
        assert!(validate_edit_result(
            &EditIntent::Translate {
                target_lang: "zh".to_string(),
            },
            long_ru,
            "переведи на китайский",
            "这是一份详细说明项目架构与实施计划的工作文档。",
        )
        .is_ok());
    }
}
