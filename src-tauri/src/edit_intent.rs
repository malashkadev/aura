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
    /// Fixes grammar, punctuation, typos, and spelling without altering meaning.
    FixGrammar,
    /// Ordinary speech/dictation, not an edit command.
    NotACommand,
}

impl EditIntent {
    /// Returns the focused system instruction for the AI model tailored to this intent.
    pub fn system_prompt(&self, selected_text: &str) -> Option<String> {
        let is_cyrillic = is_primarily_cyrillic(selected_text);
        let lang_rule = if is_cyrillic {
            "CRITICAL LANGUAGE RULE: The input text is in Russian. Your output MUST be in Russian. Do NOT translate to English or any other language."
        } else {
            "CRITICAL LANGUAGE RULE: Your output MUST be in the exact same language as the input text. Do NOT translate to any other language."
        };

        match self {
            EditIntent::Translate { target_lang } => {
                let lang_name = lang_code_to_name(target_lang);
                Some(format!(
                    "You are an expert translator. Translate the user's text into {lang_name}. Output ONLY the direct translation. Do NOT wrap output in tags or delimiters. Never add explanations, greetings, quotes, or markdown commentary."
                ))
            }
            EditIntent::FixGrammar => {
                if is_cyrillic {
                    Some("Ты — профессиональный корректор. Исправь все орфографические, грамматические и пунктуационные ошибки и опечатки в исходном тексте. Сохраняй границы предложений и знаки препинания; каждое предложение должно оканчиваться точкой или другим финальным знаком. Сохраняй авторский стиль и смысл без лишнего перефразирования. ЯЗЫКОВОЕ ПРАВИЛО: Ответ пиши СТРОГО на русском языке. Ни в коем случае не переводи на английский. Не оборачивай ответ в теги, кавычки или маркеры. Выведи ТОЛЬКО исправленный русский текст без пояснений.".to_string())
                } else {
                    Some(format!(
                        "You are an expert proofreader. Correct all grammar, spelling, punctuation, capitalization, and typographical errors in the user's text. Preserve sentence boundaries and punctuation; ensure every sentence ends with proper terminal punctuation. Preserve the original voice, phrasing, and meaning without unnecessary rewriting. {lang_rule} Do NOT wrap your output in delimiter tags, markers, or brackets. Output ONLY the corrected text without commentary."
                    ))
                }
            }
            EditIntent::NotACommand => None,
        }
    }

    /// Returns the canonical command identifier for user toggles (e.g. "fix_grammar", "translate").
    pub fn command_id(&self) -> Option<&'static str> {
        match self {
            EditIntent::FixGrammar => Some("fix_grammar"),
            EditIntent::Translate { .. } => Some("translate"),
            _ => None,
        }
    }
}

/// Checks if an edit intent has been explicitly disabled by the user in settings.
pub fn is_command_disabled(intent: &EditIntent, disabled: &[String]) -> bool {
    if let Some(id) = intent.command_id() {
        disabled.iter().any(|d| d == id)
    } else {
        false
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

/// Normalizes an utterance by stripping leading/trailing punctuation and extra whitespace,
/// mapping Cyrillic 'ё' -> 'е' and folding common Latin diacritics to avoid ASR dialect mismatch.
fn normalize_utterance(utterance: &str) -> String {
    let trimmed = utterance.trim();
    let cleaned: String = trimmed
        .chars()
        .map(|c| {
            let lower = c.to_lowercase().next().unwrap_or(c);
            let normalized = match lower {
                'ё' => 'е',
                'á' | 'à' | 'â' | 'ä' | 'ã' | 'ā' => 'a',
                'é' | 'è' | 'ê' | 'ë' | 'ē' => 'e',
                'í' | 'ì' | 'î' | 'ï' | 'ī' | 'ı' => 'i',
                'ó' | 'ò' | 'ô' | 'ö' | 'õ' | 'ō' => 'o',
                'ú' | 'ù' | 'û' | 'ü' | 'ū' => 'u',
                'ý' | 'ÿ' => 'y',
                'č' | 'ç' => 'c',
                'š' | 'ş' => 's',
                'ž' => 'z',
                'ť' => 't',
                'ď' => 'd',
                'ň' => 'n',
                'ğ' => 'g',
                _ => lower,
            };
            if normalized.is_alphanumeric() || normalized.is_whitespace() || normalized == '-' {
                normalized
            } else {
                ' '
            }
        })
        .collect();
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Extracts target language code from a translation utterance across all supported languages.
fn detect_translation_target(normalized: &str) -> Option<&'static str> {
    if normalized.contains("английск")
        || normalized.contains("англ")
        || normalized.contains("english")
        || normalized.contains("anglais")
        || normalized.contains("englisch")
        || normalized.contains("ingles")
        || normalized.contains("inglese")
        || normalized.contains("inglês")
        || normalized.contains("engels")
        || normalized.contains("ingilizce")
        || normalized.contains("英语")
        || normalized.contains("英文")
    {
        Some("en")
    } else if normalized.contains("русск")
        || normalized.contains("russian")
        || normalized.contains("russe")
        || normalized.contains("russisch")
        || normalized.contains("ruso")
        || normalized.contains("russo")
        || normalized.contains("rusça")
        || normalized.contains("rusca")
        || normalized.contains("俄语")
        || normalized.contains("俄文")
    {
        Some("ru")
    } else if normalized.contains("немецк")
        || normalized.contains("нем")
        || normalized.contains("german")
        || normalized.contains("deutsch")
        || normalized.contains("allemand")
        || normalized.contains("alemán")
        || normalized.contains("aleman")
        || normalized.contains("tedesco")
        || normalized.contains("alemão")
        || normalized.contains("alemao")
        || normalized.contains("duits")
        || normalized.contains("almanca")
        || normalized.contains("德语")
        || normalized.contains("德文")
    {
        Some("de")
    } else if normalized.contains("испанск")
        || normalized.contains("spanish")
        || normalized.contains("español")
        || normalized.contains("espanol")
        || normalized.contains("espagnol")
        || normalized.contains("spanisch")
        || normalized.contains("spagnolo")
        || normalized.contains("espanhol")
        || normalized.contains("spaans")
        || normalized.contains("ispanyolca")
        || normalized.contains("西班牙语")
        || normalized.contains("西语")
    {
        Some("es")
    } else if normalized.contains("французск")
        || normalized.contains("франц")
        || normalized.contains("french")
        || normalized.contains("français")
        || normalized.contains("francais")
        || normalized.contains("französisch")
        || normalized.contains("franzosisch")
        || normalized.contains("francés")
        || normalized.contains("frances")
        || normalized.contains("francese")
        || normalized.contains("frans")
        || normalized.contains("fransızca")
        || normalized.contains("fransizca")
        || normalized.contains("法语")
        || normalized.contains("法文")
    {
        Some("fr")
    } else if normalized.contains("итальянск")
        || normalized.contains("italian")
        || normalized.contains("italiano")
        || normalized.contains("italien")
        || normalized.contains("italienisch")
        || normalized.contains("italiaans")
        || normalized.contains("italyanca")
        || normalized.contains("意大利语")
        || normalized.contains("意语")
    {
        Some("it")
    } else if normalized.contains("китайск")
        || normalized.contains("chinese")
        || normalized.contains("chinois")
        || normalized.contains("chinesisch")
        || normalized.contains("chino")
        || normalized.contains("chinês")
        || normalized.contains("chines")
        || normalized.contains("chinees")
        || normalized.contains("çince")
        || normalized.contains("cince")
        || normalized.contains("中文")
        || normalized.contains("汉语")
    {
        Some("zh")
    } else if normalized.contains("португальск")
        || normalized.contains("portuguese")
        || normalized.contains("portugais")
        || normalized.contains("portugiesisch")
        || normalized.contains("português")
        || normalized.contains("portugues")
        || normalized.contains("portoghese")
        || normalized.contains("portugees")
        || normalized.contains("portekizce")
        || normalized.contains("葡萄牙语")
        || normalized.contains("葡语")
    {
        Some("pt")
    } else if normalized.contains("турецк")
        || normalized.contains("turkish")
        || normalized.contains("turc")
        || normalized.contains("türkisch")
        || normalized.contains("turkisch")
        || normalized.contains("turco")
        || normalized.contains("turks")
        || normalized.contains("türkçe")
        || normalized.contains("turkce")
        || normalized.contains("土耳其语")
    {
        Some("tr")
    } else if normalized.contains("нидерландск")
        || normalized.contains("голландск")
        || normalized.contains("dutch")
        || normalized.contains("néerlandais")
        || normalized.contains("neerlandais")
        || normalized.contains("niederländisch")
        || normalized.contains("niederlandisch")
        || normalized.contains("holandés")
        || normalized.contains("holandes")
        || normalized.contains("olandese")
        || normalized.contains("holandês")
        || normalized.contains("nederlands")
        || normalized.contains("hollandaca")
        || normalized.contains("flemenkçe")
        || normalized.contains("flemenkce")
        || normalized.contains("荷兰语")
    {
        Some("nl")
    } else {
        None
    }
}

/// Strips conversational polite prefixes like "пожалуйста", "please", "bitte", "s'il vous plaît", "por favor".
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
        "bitte ",
        "kannst du ",
        "por favor ",
        "puedes ",
        "podrias ",
        "podrías ",
        "s il vous plait ",
        "s il vous plaît ",
        "s il te plait ",
        "s il te plaît ",
        "stp ",
        "svp ",
        "peux-tu ",
        "pourriez-vous ",
        "per favore ",
        "per piacere ",
        "puoi ",
        "pode ",
        "poderia ",
        "lütfen ",
        "lutfen ",
        "alstublieft ",
        "alsjeblieft ",
        "kun je ",
        "请 ",
        "请",
    ];
    let mut current = text;
    let mut stripped = true;
    while stripped {
        stripped = false;
        for p in prefixes {
            if let Some(rest) = current.strip_prefix(p) {
                if !rest.trim().is_empty() {
                    current = rest.trim_start();
                    stripped = true;
                    break;
                }
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
    let word_count = core.split_whitespace().count();

    // 0. EXCLUSIONS FOR COMMON CONVERSATIONAL IDIOMS
    let conversational_starters = [
        "короче говоря",
        "проще говоря",
        "суть в том",
        "суть дела",
        "главное чтобы",
        "главное что",
        "главное это",
    ];
    for starter in conversational_starters {
        if core == starter || core.starts_with(&format!("{starter} ")) {
            return EditIntent::NotACommand;
        }
    }

    // 1. TRANSLATE COMMANDS
    let is_translate_verb = core.starts_with("переведи")
        || core.starts_with("перевести")
        || core.starts_with("перевод")
        || core.starts_with("translate")
        || core.starts_with("perevedi")
        || core.starts_with("perevesti")
        || core.starts_with("transleit")
        || core.contains("übersetz")
        || core.contains("ubersetz")
        || core.starts_with("traduce")
        || core.starts_with("traducir")
        || core.starts_with("traduis")
        || core.starts_with("traduire")
        || core.starts_with("traduci")
        || core.starts_with("tradurre")
        || core.starts_with("traduza")
        || core.starts_with("traduzir")
        || core.contains("çevir")
        || core.contains("cevir")
        || core.contains("tercüme")
        || core.contains("tercume")
        || core.contains("vertaal")
        || core.contains("vertalen")
        || core.contains("翻译")
        || core.contains("翻成");

    if is_translate_verb {
        if let Some(target) = detect_translation_target(core) {
            if word_count <= 8 {
                return EditIntent::Translate {
                    target_lang: target.to_string(),
                };
            }
        } else if word_count <= 3 {
            // Bare "переведи" / "translate" without language
            let default_target = if default_target_lang.trim().is_empty() {
                "en"
            } else {
                default_target_lang
            };

            let target = if is_primarily_cyrillic(selected_text) {
                if default_target == "ru" {
                    if app_ui_lang != "ru" {
                        app_ui_lang.to_string()
                    } else {
                        "en".to_string()
                    }
                } else {
                    default_target.to_string()
                }
            } else if default_target != "en" {
                default_target.to_string()
            } else if app_ui_lang != "en" {
                app_ui_lang.to_string()
            } else {
                "ru".to_string()
            };

            return EditIntent::Translate {
                target_lang: target,
            };
        }
    }

    // 2. FIX GRAMMAR / SPELLING
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
        "исправить пунктуацию",
        "проверь пунктуацию",
        "проверить пунктуацию",
        "и справа шибки",
        "fix grammar",
        "fix errors",
        "fix spelling",
        "fix typos",
        "correct grammar",
        "correct mistakes",
        "proofread",
        "check grammar",
        "grammatik korrigieren",
        "korrigiere grammatik",
        "fehler korrigieren",
        "korrigiere fehler",
        "rechtschreibung prüfen",
        "rechtschreibung korrigieren",
        "korrekturlesen",
        "corregir gramática",
        "corregir gramatica",
        "corrige gramática",
        "corrige gramatica",
        "corrige errores",
        "corregir errores",
        "revisar texto",
        "corriger la grammaire",
        "corrige la grammaire",
        "corriger les erreurs",
        "corrige les erreurs",
        "relecture",
        "correggi grammatica",
        "correggi la grammatica",
        "correggi errori",
        "controlla grammatica",
        "corrigir gramática",
        "corrigir gramatica",
        "corrija a gramática",
        "corrija a gramatica",
        "corrija erros",
        "corrigir erros",
        "dilbilgisini düzelt",
        "dilbilgisini duzelt",
        "hataları düzelt",
        "hatalari duzelt",
        "düzelt",
        "duzelt",
        "grammatica controleren",
        "verbeter grammatica",
        "fouten verbeteren",
        "verbeter fouten",
        "corrigeer",
        "fehler beheben",
        "text prüfen",
        "text prufen",
        "revisa la gramática",
        "revisa la gramatica",
        "corrige el texto",
        "corrige les fautes",
        "vérifie le texte",
        "verifie le texte",
        "correggi il testo",
        "revise o texto",
        "metni kontrol et",
        "修改病句",
        "修正语法",
        "修改语法",
        "检查错误",
        "修改语病",
        "纠错",
        "纠正错别字",
    ];
    for kw in grammar_keywords {
        if core == kw || (word_count <= 8 && core.starts_with(&format!("{kw} "))) {
            return EditIntent::FixGrammar;
        }
    }

    // Default: Not a command, ordinary dictation!
    // All secondary commands (casual/formal styles, summarize, bulletize, expand, shorten, custom prefixes)
    // are intentionally pruned and safely pass through as ordinary speech dictation.
    EditIntent::NotACommand
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_translate_to_english() {
        let intent = parse_edit_intent("Переведи на английский.", "en", "Привет мир", "ru");
        assert_eq!(
            intent,
            EditIntent::Translate {
                target_lang: "en".to_string()
            }
        );

        let intent2 = parse_edit_intent(
            "пожалуйста переведи на немецкий язык",
            "en",
            "Привет мир",
            "ru",
        );
        assert_eq!(
            intent2,
            EditIntent::Translate {
                target_lang: "de".to_string()
            }
        );
    }

    #[test]
    fn parses_bare_translate_command() {
        // Cyrillic selection -> default target ("en")
        let intent = parse_edit_intent("переведи", "en", "Привет, как твои дела?", "ru");
        assert_eq!(
            intent,
            EditIntent::Translate {
                target_lang: "en".to_string()
            }
        );

        // English selection -> app UI lang ("ru")
        let intent2 = parse_edit_intent("translate", "en", "Hello, how are you doing today?", "ru");
        assert_eq!(
            intent2,
            EditIntent::Translate {
                target_lang: "ru".to_string()
            }
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
        assert_eq!(
            parse_edit_intent("проверь текст", "en", "Текст", "ru"),
            EditIntent::FixGrammar
        );
        assert_eq!(
            parse_edit_intent("исправь пунктуацию", "en", "Текст", "ru"),
            EditIntent::FixGrammar
        );
        assert_eq!(
            parse_edit_intent("Проверь пунктуацию", "en", "Текст", "ru"),
            EditIntent::FixGrammar
        );
        assert_eq!(
            parse_edit_intent("и справа шибки", "en", "Текст", "ru"),
            EditIntent::FixGrammar
        );
    }

    #[test]
    fn verifies_secondary_commands_are_safely_not_a_command() {
        // Previously secondary commands now safely pass through as NotACommand
        assert_eq!(
            parse_edit_intent("сократи", "en", "Текст", "ru"),
            EditIntent::NotACommand
        );
        assert_eq!(
            parse_edit_intent("сделай короче", "en", "Текст", "ru"),
            EditIntent::NotACommand
        );
        assert_eq!(
            parse_edit_intent("сделай проще", "en", "Текст", "ru"),
            EditIntent::NotACommand
        );
        assert_eq!(
            parse_edit_intent("непринужденный стиль", "en", "Текст", "ru"),
            EditIntent::NotACommand
        );
        assert_eq!(
            parse_edit_intent("деловой стиль", "en", "Текст", "ru"),
            EditIntent::NotACommand
        );
        assert_eq!(
            parse_edit_intent("выжимка и тезисы", "en", "Текст", "ru"),
            EditIntent::NotACommand
        );
        assert_eq!(
            parse_edit_intent("списком", "en", "Текст", "ru"),
            EditIntent::NotACommand
        );
        assert_eq!(
            parse_edit_intent("по пунктам", "en", "Текст", "ru"),
            EditIntent::NotACommand
        );
        assert_eq!(
            parse_edit_intent("развертывание текста", "en", "Текст", "ru"),
            EditIntent::NotACommand
        );
        assert_eq!(
            parse_edit_intent("распиши подробнее", "en", "Текст", "ru"),
            EditIntent::NotACommand
        );
    }

    #[test]
    fn rejects_ordinary_dictation_as_not_a_command() {
        assert_eq!(
            parse_edit_intent(
                "сегодня отличная погода я решил прогуляться",
                "en",
                "Текст",
                "ru"
            ),
            EditIntent::NotACommand
        );
        assert_eq!(
            parse_edit_intent(
                "мы должны сократить расходы в следующем квартале",
                "en",
                "Текст",
                "ru"
            ),
            EditIntent::NotACommand
        );
        assert_eq!(
            parse_edit_intent("hello this is voice dictation test", "en", "Text", "en"),
            EditIntent::NotACommand
        );
    }

    #[test]
    fn parses_multilingual_grammar_and_translate() {
        // German
        assert_eq!(
            parse_edit_intent("Bitte übersetze auf Englisch", "en", "Hallo Welt", "de"),
            EditIntent::Translate {
                target_lang: "en".to_string()
            }
        );
        assert_eq!(
            parse_edit_intent("korrigiere fehler", "en", "Fehlerhafter Text", "de"),
            EditIntent::FixGrammar
        );
        assert_eq!(
            parse_edit_intent("mach kürzer", "en", "Langer Text", "de"),
            EditIntent::NotACommand
        );

        // French
        assert_eq!(
            parse_edit_intent("S'il vous plaît traduis en allemand", "en", "Bonjour", "fr"),
            EditIntent::Translate {
                target_lang: "de".to_string()
            }
        );
        assert_eq!(
            parse_edit_intent("corrige la grammaire", "en", "Texte", "fr"),
            EditIntent::FixGrammar
        );
        assert_eq!(
            parse_edit_intent("fais plus court", "en", "Texte", "fr"),
            EditIntent::NotACommand
        );

        // Spanish
        assert_eq!(
            parse_edit_intent("por favor traduce al francés", "en", "Hola", "es"),
            EditIntent::Translate {
                target_lang: "fr".to_string()
            }
        );
        assert_eq!(
            parse_edit_intent("corrige errores", "en", "Texto", "es"),
            EditIntent::FixGrammar
        );
        assert_eq!(
            parse_edit_intent("hazlo más corto", "en", "Texto", "es"),
            EditIntent::NotACommand
        );

        // Italian
        assert_eq!(
            parse_edit_intent("traduci in inglese", "en", "Ciao", "it"),
            EditIntent::Translate {
                target_lang: "en".to_string()
            }
        );

        // Portuguese
        assert_eq!(
            parse_edit_intent("traduza para espanhol", "en", "Olá", "pt"),
            EditIntent::Translate {
                target_lang: "es".to_string()
            }
        );

        // Turkish
        assert_eq!(
            parse_edit_intent("lütfen türkçeye çevir", "en", "Hello", "tr"),
            EditIntent::Translate {
                target_lang: "tr".to_string()
            }
        );
        assert_eq!(
            parse_edit_intent("hataları düzelt", "en", "Metin", "tr"),
            EditIntent::FixGrammar
        );

        // Dutch
        assert_eq!(
            parse_edit_intent("vertaal naar het engels", "en", "Hallo", "nl"),
            EditIntent::Translate {
                target_lang: "en".to_string()
            }
        );
        assert_eq!(
            parse_edit_intent("verbeter fouten", "en", "Tekst", "nl"),
            EditIntent::FixGrammar
        );

        // Chinese
        assert_eq!(
            parse_edit_intent("请翻译成英语", "en", "你好世界", "zh"),
            EditIntent::Translate {
                target_lang: "en".to_string()
            }
        );
        assert_eq!(
            parse_edit_intent("修改语法", "en", "文本", "zh"),
            EditIntent::FixGrammar
        );
        assert_eq!(
            parse_edit_intent("请总结", "en", "长文本", "zh"),
            EditIntent::NotACommand
        );
    }

    #[test]
    fn rejects_conversational_idioms_and_long_sentences() {
        assert_eq!(
            parse_edit_intent(
                "короче говоря это было отличное решение",
                "en",
                "Текст",
                "ru"
            ),
            EditIntent::NotACommand
        );
        assert_eq!(
            parse_edit_intent("суть в том что нам нужно продолжать", "en", "Текст", "ru"),
            EditIntent::NotACommand
        );
        assert_eq!(
            parse_edit_intent("главное чтобы всё работало без сбоев", "en", "Текст", "ru"),
            EditIntent::NotACommand
        );
        assert_eq!(
            parse_edit_intent("главное что мы успели вовремя", "en", "Текст", "ru"),
            EditIntent::NotACommand
        );
        assert_eq!(
            parse_edit_intent("проще говоря это обычный текст", "en", "Текст", "ru"),
            EditIntent::NotACommand
        );

        // Long sentences starting with prefix (> 8 words) should not be captured as short edit commands
        assert_eq!(
            parse_edit_intent(
                "короче мы решили переписать этот весь проект на другой язык программирования и выпустить релиз",
                "en",
                "Текст",
                "ru"
            ),
            EditIntent::NotACommand
        );
        assert_eq!(
            parse_edit_intent(
                "главное мы должны закончить эту задачу как можно скорее чтобы успеть к дедлайну",
                "en",
                "Текст",
                "ru"
            ),
            EditIntent::NotACommand
        );
    }

    #[test]
    fn bare_translate_target_respects_ui_and_default_lang() {
        // Cyrillic text -> translates to default target (en)
        assert_eq!(
            parse_edit_intent("переведи", "en", "Привет мир", "ru"),
            EditIntent::Translate {
                target_lang: "en".to_string()
            }
        );

        // English text with German UI and default target "en" -> translates to German UI lang
        assert_eq!(
            parse_edit_intent("translate", "en", "Hello world", "de"),
            EditIntent::Translate {
                target_lang: "de".to_string()
            }
        );

        // English text with German UI and default target "es" -> translates to Spanish
        assert_eq!(
            parse_edit_intent("translate", "es", "Hello world", "de"),
            EditIntent::Translate {
                target_lang: "es".to_string()
            }
        );

        // English text with Russian UI -> translates to Russian
        assert_eq!(
            parse_edit_intent("переведи", "en", "Hello world", "ru"),
            EditIntent::Translate {
                target_lang: "ru".to_string()
            }
        );
    }

    #[test]
    fn test_commands_guide_chips_synchronization() {
        // Fix Grammar chips
        assert_eq!(
            parse_edit_intent("исправь ошибки", "en", "Текст", "ru"),
            EditIntent::FixGrammar
        );
        assert_eq!(
            parse_edit_intent("проверь текст", "en", "Текст", "ru"),
            EditIntent::FixGrammar
        );
        assert_eq!(
            parse_edit_intent("исправь пунктуацию", "en", "Текст", "ru"),
            EditIntent::FixGrammar
        );

        // Translate chips
        assert_eq!(
            parse_edit_intent("переведи", "en", "Текст", "ru"),
            EditIntent::Translate {
                target_lang: "en".into()
            }
        );
        assert_eq!(
            parse_edit_intent("переведи на английский", "en", "Текст", "ru"),
            EditIntent::Translate {
                target_lang: "en".into()
            }
        );
        assert_eq!(
            parse_edit_intent("переведи на немецкий", "en", "Текст", "ru"),
            EditIntent::Translate {
                target_lang: "de".into()
            }
        );
    }
}
