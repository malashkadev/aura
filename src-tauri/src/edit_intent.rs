//! Deterministic intent parser for voice commands spoken over selected text.
//!
//! Separating command instruction from document data guarantees that:
//! 1. Small models (Qwen3 0.6B - 4B) receive ONLY the document data to manipulate,
//!    preventing them from translating the command words ("переведи на английский")
//!    themselves or echoing them back.
//! 2. Plain dictations spoken while a selection happens to be active are immediately
//!    recognized as `NotACommand` and pass through without any LLM latency or corruption.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditIntent {
    /// Translates selected text to `target_lang` (ISO 639-1 code e.g. "en", "ru", "de", "es", "fr", "it", "zh", "pt", "tr", "nl").
    Translate { target_lang: String },
    /// Shortens or condenses selected text.
    Shorten,
    /// Expands or elaborates selected text with more details.
    Expand,
    /// Fixes grammar, punctuation, typos, and spelling without altering meaning.
    FixGrammar,
    /// Rewrites selected text into a formal/professional tone.
    RewriteFormal,
    /// Rewrites selected text into a casual/friendly tone.
    RewriteCasual,
    /// Summarizes the essence or key points of the selected text.
    Summarize,
    /// Formats the selected text as a bulleted list.
    Bulletize,
    /// Custom recognized instruction.
    Custom(String),
    /// Ordinary speech/dictation, not an edit command.
    NotACommand,
}

impl EditIntent {
    /// Returns the focused system instruction for the AI model tailored to this intent.
    pub fn system_prompt(&self) -> Option<String> {
        match self {
            EditIntent::Translate { target_lang } => {
                let lang_name = lang_code_to_name(target_lang);
                Some(format!(
                    "You are an expert translator. Translate the user's text into {lang_name}. Output ONLY the direct translation. Never add explanations, greetings, quotes, or markdown commentary."
                ))
            }
            EditIntent::Shorten => Some(
                "You are an expert editor. Condense and shorten the user's text while preserving its core meaning and key facts. Output MUST be significantly shorter than input. Output ONLY the shortened text without commentary."
                    .to_string(),
            ),
            EditIntent::Expand => Some(
                "You are an expert writer. Expand and elaborate on the user's text with helpful details and clarity while preserving the original intent. Output ONLY the expanded text without commentary."
                    .to_string(),
            ),
            EditIntent::FixGrammar => Some(
                "You are an expert proofreader. Correct all grammar, spelling, punctuation, and typographical errors in the user's text. Preserve the original voice, meaning, and language. Output ONLY the corrected text without commentary."
                    .to_string(),
            ),
            EditIntent::RewriteFormal => Some(
                "You are a professional editor. Rewrite the user's text in a polished, formal, and professional tone suitable for business communication. Output ONLY the rewritten text without commentary."
                    .to_string(),
            ),
            EditIntent::RewriteCasual => Some(
                "You are a creative editor. Rewrite the user's text in a natural, friendly, and casual tone. Output ONLY the rewritten text without commentary."
                    .to_string(),
            ),
            EditIntent::Summarize => Some(
                "You are an expert analyst. Provide a concise summary capturing the key points and essential takeaways of the user's text. Output ONLY the summary without commentary."
                    .to_string(),
            ),
            EditIntent::Bulletize => Some(
                "You are an organizational assistant. Format the user's text into clear, readable bullet points (- item). Output ONLY the bulleted list without commentary."
                    .to_string(),
            ),
            EditIntent::Custom(instruction) => Some(format!(
                "You are an expert text editor. Apply this instruction to the user's text: {instruction}. Output ONLY the resulting text without explanations or conversational filler."
            )),
            EditIntent::NotACommand => None,
        }
    }
}

pub fn lang_code_to_name(code: &str) -> &'static str {
    match code {
        "ru" => "Russian",
        "en" => "English",
        "de" => "German",
        "es" => "Spanish",
        "fr" => "French",
        "it" => "Italian",
        "zh" => "Chinese",
        "pt" => "Portuguese",
        "tr" => "Turkish",
        "nl" => "Dutch",
        _ => "English",
    }
}

/// Returns true if text contains primarily Cyrillic characters.
pub fn is_primarily_cyrillic(text: &str) -> bool {
    let mut cyrillic = 0usize;
    let mut total_alpha = 0usize;
    for ch in text.chars() {
        if ch.is_alphabetic() {
            total_alpha += 1;
            if ('\u{0400}'..='\u{04FF}').contains(&ch) || ('\u{0500}'..='\u{052F}').contains(&ch) {
                cyrillic += 1;
            }
        }
    }
    if total_alpha == 0 {
        return false;
    }
    cyrillic * 2 >= total_alpha
}

/// Normalizes an utterance by stripping leading/trailing punctuation and extra whitespace.
fn normalize_utterance(utterance: &str) -> String {
    let trimmed = utterance.trim();
    let cleaned: String = trimmed
        .chars()
        .map(|c| if c.is_alphanumeric() || c.is_whitespace() || c == '-' { c } else { ' ' })
        .collect();
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
}

/// Extracts target language code from a translation utterance.
fn detect_translation_target(normalized: &str) -> Option<&'static str> {
    if normalized.contains("английск") || normalized.contains("англ") || normalized.contains("english") {
        Some("en")
    } else if normalized.contains("русск") || normalized.contains("russian") {
        Some("ru")
    } else if normalized.contains("немецк") || normalized.contains("нем") || normalized.contains("german") || normalized.contains("deutsch") {
        Some("de")
    } else if normalized.contains("испанск") || normalized.contains("spanish") || normalized.contains("espanol") {
        Some("es")
    } else if normalized.contains("французск") || normalized.contains("франц") || normalized.contains("french") {
        Some("fr")
    } else if normalized.contains("итальянск") || normalized.contains("italian") {
        Some("it")
    } else if normalized.contains("китайск") || normalized.contains("chinese") {
        Some("zh")
    } else if normalized.contains("португальск") || normalized.contains("portuguese") {
        Some("pt")
    } else if normalized.contains("турецк") || normalized.contains("turkish") {
        Some("tr")
    } else if normalized.contains("нидерландск") || normalized.contains("голландск") || normalized.contains("dutch") {
        Some("nl")
    } else {
        None
    }
}

/// Strips conversational polite prefixes like "пожалуйста", "please", "аура".
fn strip_polite_prefixes(text: &str) -> &str {
    let prefixes = [
        "пожалуйста ",
        "аура пожалуйста ",
        "аура ",
        "прошу ",
        "можешь ",
        "please ",
        "aura please ",
        "aura ",
        "can you ",
        "could you ",
    ];
    let mut current = text;
    let mut stripped = true;
    while stripped {
        stripped = false;
        for p in prefixes {
            if let Some(rest) = current.strip_prefix(p) {
                current = rest.trim_start();
                stripped = true;
                break;
            }
        }
    }
    current
}

/// Parses a spoken utterance into an `EditIntent`.
///
/// Uses the selected text and user settings to resolve default translation directions:
/// - If command says "переведи" without language:
///   - If selected text is Cyrillic -> translates to `default_target_lang` (default "en").
///   - If selected text is already Latin and `default_target_lang` is "en" -> translates to `app_ui_lang` (or "ru").
pub fn parse_edit_intent(
    utterance: &str,
    default_target_lang: &str,
    selected_text: &str,
    app_ui_lang: &str,
) -> EditIntent {
    let normalized = normalize_utterance(utterance);
    if normalized.is_empty() {
        return EditIntent::NotACommand;
    }

    let core = strip_polite_prefixes(&normalized);

    // 1. TRANSLATE COMMANDS
    let is_translate_verb = core.starts_with("переведи")
        || core.starts_with("перевести")
        || core.starts_with("перевод")
        || core.starts_with("translate");

    if is_translate_verb {
        if let Some(target) = detect_translation_target(core) {
            return EditIntent::Translate {
                target_lang: target.to_string(),
            };
        }

        // Bare "переведи" / "translate" without language
        let default_target = if default_target_lang.trim().is_empty() {
            "en"
        } else {
            default_target_lang
        };

        let target = if is_primarily_cyrillic(selected_text) {
            default_target.to_string()
        } else if default_target == "en" {
            if app_ui_lang == "en" {
                "ru".to_string()
            } else {
                app_ui_lang.to_string()
            }
        } else {
            default_target.to_string()
        };

        return EditIntent::Translate { target_lang: target };
    }

    // 2. SHORTEN
    let shorten_keywords = [
        "сократи",
        "сократить",
        "укороти",
        "сделай короче",
        "покороче",
        "короче",
        "shorten",
        "make shorter",
        "make it shorter",
        "cut down",
    ];
    for kw in shorten_keywords {
        if core == kw || core.starts_with(&format!("{kw} ")) {
            return EditIntent::Shorten;
        }
    }

    // 3. EXPAND
    let expand_keywords = [
        "разверни",
        "распиши",
        "распиши подробнее",
        "подробнее",
        "дополни",
        "сделай длиннее",
        "expand",
        "elaborate",
        "make longer",
        "add details",
    ];
    for kw in expand_keywords {
        if core == kw || core.starts_with(&format!("{kw} ")) {
            return EditIntent::Expand;
        }
    }

    // 4. FIX GRAMMAR / SPELLING
    let grammar_keywords = [
        "исправь грамматику",
        "исправь ошибки",
        "исправь опечатки",
        "исправь текст",
        "исправить грамматику",
        "исправить ошибки",
        "проверь текст",
        "проверь грамматику",
        "исправление ошибок",
        "исправь пунктуацию",
        "fix grammar",
        "fix errors",
        "fix spelling",
        "fix typos",
        "correct grammar",
        "correct mistakes",
        "proofread",
        "check grammar",
    ];
    for kw in grammar_keywords {
        if core == kw || core.starts_with(&format!("{kw} ")) {
            return EditIntent::FixGrammar;
        }
    }

    // 5. REWRITE FORMAL
    let formal_keywords = [
        "формальнее",
        "сделай формальнее",
        "официальнее",
        "деловой стиль",
        "сделай официально",
        "make formal",
        "make it formal",
        "formal tone",
        "professional tone",
        "more formal",
    ];
    for kw in formal_keywords {
        if core == kw || core.starts_with(&format!("{kw} ")) {
            return EditIntent::RewriteFormal;
        }
    }

    // 6. REWRITE CASUAL
    let casual_keywords = [
        "проще",
        "сделай проще",
        "дружелюбнее",
        "разговорнее",
        "разговорный стиль",
        "make casual",
        "make it casual",
        "casual tone",
        "friendly tone",
        "simpler",
    ];
    for kw in casual_keywords {
        if core == kw || core.starts_with(&format!("{kw} ")) {
            return EditIntent::RewriteCasual;
        }
    }

    // 7. SUMMARIZE
    let summarize_keywords = [
        "краткое содержание",
        "главное",
        "выжимка",
        "суть",
        "в двух словах",
        "резюмируй",
        "главные мысли",
        "summarize",
        "summary",
        "tldr",
        "key takeaways",
        "key points",
    ];
    for kw in summarize_keywords {
        if core == kw || core.starts_with(&format!("{kw} ")) {
            return EditIntent::Summarize;
        }
    }

    // 8. BULLETIZE / LIST
    let list_keywords = [
        "списком",
        "по пунктам",
        "сделай списком",
        "оформи списком",
        "маркированный список",
        "в виде списка",
        "bullet points",
        "bullet list",
        "as a list",
        "format as list",
        "make a list",
    ];
    for kw in list_keywords {
        if core == kw || core.starts_with(&format!("{kw} ")) {
            return EditIntent::Bulletize;
        }
    }

    // 9. CUSTOM COMMANDS (must be imperative and concise: under 12 words)
    let word_count = core.split_whitespace().count();
    if word_count <= 12 {
        let custom_prefixes = [
            "сделай ",
            "перепиши ",
            "преобразуй ",
            "оформи ",
            "rewrite ",
            "rephrase ",
            "convert to ",
            "format as ",
        ];
        for cp in custom_prefixes {
            if core.starts_with(cp) {
                return EditIntent::Custom(core.to_string());
            }
        }
    }

    // Default: Not a command, ordinary dictation!
    EditIntent::NotACommand
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_translate_to_english() {
        let intent = parse_edit_intent(
            "Переведи на английский.",
            "en",
            "Привет мир",
            "ru",
        );
        assert_eq!(intent, EditIntent::Translate { target_lang: "en".to_string() });

        let intent2 = parse_edit_intent(
            "пожалуйста переведи на немецкий язык",
            "en",
            "Привет мир",
            "ru",
        );
        assert_eq!(intent2, EditIntent::Translate { target_lang: "de".to_string() });
    }

    #[test]
    fn parses_bare_translate_command() {
        // Cyrillic selection -> default target ("en")
        let intent = parse_edit_intent(
            "переведи",
            "en",
            "Привет, как твои дела?",
            "ru",
        );
        assert_eq!(intent, EditIntent::Translate { target_lang: "en".to_string() });

        // English selection -> app UI lang ("ru")
        let intent2 = parse_edit_intent(
            "translate",
            "en",
            "Hello, how are you doing today?",
            "ru",
        );
        assert_eq!(intent2, EditIntent::Translate { target_lang: "ru".to_string() });
    }

    #[test]
    fn parses_shorten_commands() {
        assert_eq!(
            parse_edit_intent("сократи", "en", "Текст", "ru"),
            EditIntent::Shorten
        );
        assert_eq!(
            parse_edit_intent("Сократи пожалуйста этот фрагмент.", "en", "Текст", "ru"),
            EditIntent::Shorten
        );
        assert_eq!(
            parse_edit_intent("make it shorter", "en", "Text", "en"),
            EditIntent::Shorten
        );
    }

    #[test]
    fn parses_fix_grammar_commands() {
        assert_eq!(
            parse_edit_intent("исправь грамматику", "en", "Текст", "ru"),
            EditIntent::FixGrammar
        );
        assert_eq!(
            parse_edit_intent("Исправь ошибки и опечатки", "en", "Текст", "ru"),
            EditIntent::FixGrammar
        );
        assert_eq!(
            parse_edit_intent("fix grammar please", "en", "Text", "en"),
            EditIntent::FixGrammar
        );
    }

    #[test]
    fn parses_tone_and_formatting_commands() {
        assert_eq!(
            parse_edit_intent("сделай формальнее", "en", "Текст", "ru"),
            EditIntent::RewriteFormal
        );
        assert_eq!(
            parse_edit_intent("сделай проще", "en", "Текст", "ru"),
            EditIntent::RewriteCasual
        );
        assert_eq!(
            parse_edit_intent("списком", "en", "Текст", "ru"),
            EditIntent::Bulletize
        );
        assert_eq!(
            parse_edit_intent("выжимка", "en", "Текст", "ru"),
            EditIntent::Summarize
        );
    }

    #[test]
    fn rejects_ordinary_dictation_as_not_a_command() {
        assert_eq!(
            parse_edit_intent("сегодня отличная погода я решил прогуляться", "en", "Текст", "ru"),
            EditIntent::NotACommand
        );
        assert_eq!(
            parse_edit_intent("мы должны сократить расходы в следующем квартале", "en", "Текст", "ru"),
            EditIntent::NotACommand
        );
        assert_eq!(
            parse_edit_intent("hello this is voice dictation test", "en", "Text", "en"),
            EditIntent::NotACommand
        );
    }
}
