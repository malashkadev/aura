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
            EditIntent::Shorten => Some(format!(
                "You are an expert editor. Condense and shorten the user's text while preserving its core meaning and key facts. Output MUST be significantly shorter than input. {lang_rule} Do NOT wrap output in tags or delimiters. Output ONLY the shortened text without commentary."
            )),
            EditIntent::Expand => Some(format!(
                "You are an expert writer. Expand and elaborate on the user's text with helpful details and clarity while preserving the original intent. {lang_rule} Do NOT wrap output in tags or delimiters. Output ONLY the expanded text without commentary."
            )),
            EditIntent::FixGrammar => Some(format!(
                "You are an expert proofreader. Correct all grammar, spelling, punctuation, capitalization, and typographical errors in the user's text. Preserve sentence boundaries and punctuation; ensure every sentence ends with proper terminal punctuation. Preserve the original voice, phrasing, and meaning without unnecessary rewriting. {lang_rule} Do NOT wrap your output in delimiter tags, markers, or brackets. Output ONLY the corrected text without commentary."
            )),
            EditIntent::RewriteFormal => Some(format!(
                "You are a professional editor. Rewrite the user's text in a polished, formal, and professional tone suitable for business communication. {lang_rule} Do NOT wrap output in tags or delimiters. Output ONLY the rewritten text without commentary."
            )),
            EditIntent::RewriteCasual => Some(format!(
                "You are a creative editor. Rewrite the user's text in a natural, friendly, and casual tone. {lang_rule} Do NOT wrap output in tags or delimiters. Output ONLY the rewritten text without commentary."
            )),
            EditIntent::Summarize => Some(format!(
                "You are an expert analyst. Provide a concise summary capturing the key points and essential takeaways of the user's text. {lang_rule} Do NOT wrap output in tags or delimiters. Output ONLY the summary without commentary."
            )),
            EditIntent::Bulletize => Some(format!(
                "You are an organizational assistant. Format the user's text into clear, readable bullet points (- item). {lang_rule} Do NOT wrap output in tags or delimiters. Output ONLY the bulleted list without commentary."
            )),
            EditIntent::Custom(instruction) => {
                let custom_rule = if is_cyrillic {
                    "CRITICAL: Keep output in Russian unless the instruction explicitly specifies translation."
                } else {
                    "CRITICAL: Maintain the exact same language as the input text unless the instruction explicitly specifies translation."
                };
                Some(format!(
                    "You are an expert text editor. Apply this instruction to the user's text: {instruction}. {custom_rule} Do NOT wrap output in tags or delimiters. Output ONLY the resulting text without explanations or conversational filler."
                ))
            }
            EditIntent::NotACommand => None,
        }
    }

    /// Returns the canonical command identifier for user toggles (e.g. "fix_grammar", "translate").
    pub fn command_id(&self) -> Option<&'static str> {
        match self {
            EditIntent::FixGrammar => Some("fix_grammar"),
            EditIntent::Translate { .. } => Some("translate"),
            EditIntent::Shorten => Some("shorten"),
            EditIntent::Expand => Some("expand"),
            EditIntent::Bulletize => Some("bulletize"),
            EditIntent::Summarize => Some("summarize"),
            EditIntent::RewriteFormal => Some("rewrite_formal"),
            EditIntent::RewriteCasual => Some("rewrite_casual"),
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

/// Normalizes an utterance by stripping leading/trailing punctuation and extra whitespace.
fn normalize_utterance(utterance: &str) -> String {
    let trimmed = utterance.trim();
    let cleaned: String = trimmed
        .chars()
        .map(|c| if c.is_alphanumeric() || c.is_whitespace() || c == '-' { c } else { ' ' })
        .collect();
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
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

            return EditIntent::Translate { target_lang: target };
        }
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
        "condense",
        "sekreti",
        "sekretit",
        "sokrati",
        "sokratit",
        "kürzen",
        "kurzen",
        "kürze",
        "kurze",
        "kürzer",
        "kurzer",
        "mach kürzer",
        "mach kurzer",
        "acortar",
        "acorta",
        "hazlo más corto",
        "hazlo mas corto",
        "más corto",
        "mas corto",
        "raccourcir",
        "raccourcis",
        "fais plus court",
        "plus court",
        "écourte",
        "ecourte",
        "accorcia",
        "accorciare",
        "fai più corto",
        "fai piu corto",
        "più corto",
        "piu corto",
        "encurta",
        "encurtar",
        "faça mais curto",
        "faca mais curto",
        "mais curto",
        "kısalt",
        "kisalt",
        "daha kısa yap",
        "daha kisa yap",
        "inkorten",
        "kort in",
        "maak korter",
        "korter",
        "сожми текст",
        "сожми",
        "сжать",
        "сжать текст",
        "kürzer machen",
        "kurzer machen",
        "text verdichten",
        "sintetizza",
        "精简",
        "缩短",
        "精简一点",
        "精简文本",
        "写短一点",
    ];
    for kw in shorten_keywords {
        if core == kw || (word_count <= 8 && core.starts_with(&format!("{kw} "))) {
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
        "erweitern",
        "erweitere",
        "ausführlicher",
        "ausfuhrlicher",
        "mach länger",
        "mach langer",
        "mehr details",
        "expandir",
        "expande",
        "elaborar",
        "elabora",
        "más detallado",
        "mas detallado",
        "más largo",
        "mas largo",
        "développer",
        "developper",
        "développe",
        "developpe",
        "plus détaillé",
        "plus detaille",
        "plus long",
        "espandi",
        "espandere",
        "più dettagli",
        "piu dettagli",
        "più lungo",
        "piu lungo",
        "elabore",
        "mais detalhes",
        "mais longo",
        "genişlet",
        "genislet",
        "ayrıntılandır",
        "ayrintilandir",
        "detaylandır",
        "detaylandir",
        "uitbreiden",
        "breid uit",
        "maak langer",
        "meer details",
        "uitwerken",
        "разверни мысль",
        "развернуть мысль",
        "добавь деталей",
        "добавить деталей",
        "gedanken ausführen",
        "gedanken ausfuhren",
        "details hinzufügen",
        "details hinzufugen",
        "explica con más detalle",
        "explica con mas detalle",
        "añade detalles",
        "anade detalles",
        "détaille",
        "detaille",
        "ajoute des détails",
        "ajoute des details",
        "approfondisci",
        "aggiungi dettagli",
        "detalhe mais",
        "adicione detalhes",
        "daha ayrıntılı yaz",
        "daha ayrintili yaz",
        "展开说明",
        "写详细一点",
        "补充细节",
        "扩写",
        "展开",
        "详细说明",
    ];
    for kw in expand_keywords {
        if core == kw || (word_count <= 8 && core.starts_with(&format!("{kw} "))) {
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

    // 5. REWRITE FORMAL
    let formal_keywords = [
        "формальнее",
        "сделай формальнее",
        "официальнее",
        "деловой стиль",
        "в деловом стиле",
        "деловой тон",
        "деловым стилем",
        "сделай официально",
        "официальный стиль",
        "вежливый тон",
        "вежливо",
        "make formal",
        "make it formal",
        "formal tone",
        "professional tone",
        "business tone",
        "professional",
        "more formal",
        "formeller",
        "mach formeller",
        "formeller machen",
        "formell machen",
        "geschäftlicher ton",
        "geschaftlicher ton",
        "höflich formulieren",
        "hoflich formulieren",
        "offizieller",
        "más formal",
        "mas formal",
        "hazlo formal",
        "tono formal",
        "tono profesional",
        "estilo formal",
        "plus formel",
        "fais plus formel",
        "rendre formel",
        "ton formel",
        "style professionnel",
        "rends formel",
        "più formale",
        "piu formale",
        "fai più formale",
        "fai piu formale",
        "tono professionale",
        "stile formale",
        "mais formal",
        "faça mais formal",
        "faca mais formal",
        "tom formal",
        "tom profissional",
        "estilo corporativo",
        "daha resmi",
        "resmi yap",
        "daha resmi yap",
        "kurumsal üslup",
        "kurumsal uslup",
        "profesyonel dil",
        "formeler",
        "maak formeler",
        "formele toon",
        "更正式",
        "商务语气",
        "专业语气",
        "书面语",
    ];
    for kw in formal_keywords {
        if core == kw || (word_count <= 8 && core.starts_with(&format!("{kw} "))) {
            return EditIntent::RewriteFormal;
        }
    }

    // 6. REWRITE CASUAL
    let casual_keywords = [
        "проще",
        "сделай проще",
        "простыми словами",
        "простым языком",
        "дружелюбнее",
        "дружеский тон",
        "разговорнее",
        "разговорный стиль",
        "неформально",
        "непринужденный стиль",
        "непринужденно",
        "в непринужденном стиле",
        "make casual",
        "make it casual",
        "casual tone",
        "friendly tone",
        "conversational",
        "simpler",
        "lockerer",
        "mach lockerer",
        "lockerer ton",
        "freundlich",
        "einfache sprache",
        "informeller",
        "más casual",
        "mas casual",
        "hazlo informal",
        "tono casual",
        "tono amigable",
        "más natural",
        "mas natural",
        "plus décontracté",
        "plus decontracte",
        "plus informel",
        "ton décontracté",
        "ton decontracte",
        "ton amical",
        "fais décontracté",
        "fais decontracte",
        "simple et naturel",
        "più informale",
        "piu informale",
        "fai informale",
        "rendi informale",
        "tono amichevole",
        "più naturale",
        "piu naturale",
        "mais casual",
        "mais informal",
        "tom casual",
        "torne informal",
        "tom amigável",
        "tom amigavel",
        "daha samimi",
        "samimi yap",
        "günlük dil",
        "gunluk dil",
        "daha doğal",
        "daha dogal",
        "daha rahat",
        "informeler",
        "maak informeler",
        "lossere toon",
        "通俗点",
        "随和语气",
        "口语化",
        "更随意",
        "轻松语气",
    ];
    for kw in casual_keywords {
        if core == kw || (word_count <= 8 && core.starts_with(&format!("{kw} "))) {
            return EditIntent::RewriteCasual;
        }
    }

    // 7. SUMMARIZE
    let summarize_keywords = [
        "краткое содержание",
        "главное",
        "главные тезисы",
        "тезисы",
        "краткая суть",
        "сделай саммари",
        "саммари",
        "сделай выжимку",
        "выжимку",
        "выжимка",
        "суть",
        "в двух словах",
        "резюмируй",
        "главные мысли",
        "кратко",
        "summarize",
        "summary",
        "tldr",
        "key takeaways",
        "key points",
        "zusammenfassen",
        "fasse zusammen",
        "zusammenfassung",
        "kernaussagen",
        "kurzfassung",
        "resumir",
        "resume",
        "resumen",
        "puntos clave",
        "haz un resumen",
        "résumer",
        "resumer",
        "résume",
        "résumé",
        "points essentiels",
        "fais un résumé",
        "fais un resume",
        "riassumi",
        "riassumere",
        "riassunto",
        "punti chiave",
        "fai un sommario",
        "resuma",
        "resumo",
        "principais pontos",
        "faça um resumo",
        "faca um resumo",
        "özetle",
        "ozetle",
        "özet",
        "ozet",
        "ana noktalar",
        "özet çıkar",
        "ozet cikar",
        "samenvatten",
        "vat samen",
        "samenvatting",
        "总结",
        "概括",
        "总结要点",
        "核心摘要",
        "提炼精髓",
    ];
    for kw in summarize_keywords {
        if core == kw || (word_count <= 8 && core.starts_with(&format!("{kw} "))) {
            return EditIntent::Summarize;
        }
    }

    // 8. BULLETIZE / LIST
    let list_keywords = [
        "списком",
        "по пунктам",
        "сделай список",
        "сделать список",
        "сделай списком",
        "оформи списком",
        "маркированный список",
        "в виде списка",
        "bullet points",
        "bulletize",
        "bullet list",
        "as a list",
        "format as list",
        "make a list",
        "als liste",
        "liste erstellen",
        "stichpunkte",
        "in stichpunkten",
        "aufzählung",
        "aufzahlung",
        "haz una lista",
        "en viñetas",
        "en vinetas",
        "como lista",
        "formatea como lista",
        "fais une liste",
        "sous forme de liste",
        "en puces",
        "liste à puces",
        "liste a puces",
        "elenco puntato",
        "in punti",
        "come elenco",
        "fai un elenco",
        "faça uma lista",
        "faca uma lista",
        "formate em lista",
        "em tópicos",
        "em topicos",
        "liste yap",
        "maddeler halinde",
        "madde imleri",
        "madde işaretleri",
        "madde isaretleri",
        "liste olarak",
        "opsommingstekens",
        "als lijst",
        "做成列表",
        "列要点",
        "分点列出",
        "项目符号",
        "列成清单",
    ];
    for kw in list_keywords {
        if core == kw || (word_count <= 8 && core.starts_with(&format!("{kw} "))) {
            return EditIntent::Bulletize;
        }
    }

    // 9. CUSTOM COMMANDS (must be imperative and concise: under 12 words)
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
            "schreibe um ",
            "formuliere um ",
            "reescribe ",
            "reformula ",
            "réécris ",
            "reecris ",
            "riscrivi ",
            "riformula ",
            "reescreva ",
            "reformule ",
            "yeniden yaz ",
            "herschrijf ",
            "改写 ",
            "重写 ",
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
            parse_edit_intent("Sekreti.", "en", "Текст", "ru"),
            EditIntent::Shorten
        );
        assert_eq!(
            parse_edit_intent("sokrati", "en", "Текст", "ru"),
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

    #[test]
    fn parses_multilingual_commands() {
        // German
        assert_eq!(
            parse_edit_intent("Bitte übersetze auf Englisch", "en", "Hallo Welt", "de"),
            EditIntent::Translate { target_lang: "en".to_string() }
        );
        assert_eq!(
            parse_edit_intent("mach kürzer", "en", "Langer Text", "de"),
            EditIntent::Shorten
        );
        assert_eq!(
            parse_edit_intent("korrigiere fehler", "en", "Fehlerhafter Text", "de"),
            EditIntent::FixGrammar
        );
        assert_eq!(
            parse_edit_intent("zusammenfassen", "en", "Langer Text", "de"),
            EditIntent::Summarize
        );
        assert_eq!(
            parse_edit_intent("als liste", "en", "Text", "de"),
            EditIntent::Bulletize
        );

        // French
        assert_eq!(
            parse_edit_intent("S'il vous plaît traduis en allemand", "en", "Bonjour", "fr"),
            EditIntent::Translate { target_lang: "de".to_string() }
        );
        assert_eq!(
            parse_edit_intent("fais plus court", "en", "Texte", "fr"),
            EditIntent::Shorten
        );
        assert_eq!(
            parse_edit_intent("corrige la grammaire", "en", "Texte", "fr"),
            EditIntent::FixGrammar
        );
        assert_eq!(
            parse_edit_intent("résumé", "en", "Texte", "fr"),
            EditIntent::Summarize
        );

        // Spanish
        assert_eq!(
            parse_edit_intent("por favor traduce al francés", "en", "Hola", "es"),
            EditIntent::Translate { target_lang: "fr".to_string() }
        );
        assert_eq!(
            parse_edit_intent("hazlo más corto", "en", "Texto", "es"),
            EditIntent::Shorten
        );
        assert_eq!(
            parse_edit_intent("corrige errores", "en", "Texto", "es"),
            EditIntent::FixGrammar
        );
        assert_eq!(
            parse_edit_intent("en viñetas", "en", "Texto", "es"),
            EditIntent::Bulletize
        );

        // Italian
        assert_eq!(
            parse_edit_intent("traduci in inglese", "en", "Ciao", "it"),
            EditIntent::Translate { target_lang: "en".to_string() }
        );
        assert_eq!(
            parse_edit_intent("accorcia", "en", "Testo", "it"),
            EditIntent::Shorten
        );
        assert_eq!(
            parse_edit_intent("riassunto", "en", "Testo", "it"),
            EditIntent::Summarize
        );

        // Portuguese
        assert_eq!(
            parse_edit_intent("traduza para espanhol", "en", "Olá", "pt"),
            EditIntent::Translate { target_lang: "es".to_string() }
        );
        assert_eq!(
            parse_edit_intent("encurta", "en", "Texto", "pt"),
            EditIntent::Shorten
        );

        // Turkish
        assert_eq!(
            parse_edit_intent("lütfen türkçeye çevir", "en", "Hello", "tr"),
            EditIntent::Translate { target_lang: "tr".to_string() }
        );
        assert_eq!(
            parse_edit_intent("kısalt", "en", "Metin", "tr"),
            EditIntent::Shorten
        );
        assert_eq!(
            parse_edit_intent("özetle", "en", "Metin", "tr"),
            EditIntent::Summarize
        );

        // Dutch
        assert_eq!(
            parse_edit_intent("vertaal naar het engels", "en", "Hallo", "nl"),
            EditIntent::Translate { target_lang: "en".to_string() }
        );
        assert_eq!(
            parse_edit_intent("kort in", "en", "Tekst", "nl"),
            EditIntent::Shorten
        );

        // Chinese
        assert_eq!(
            parse_edit_intent("请翻译成英语", "en", "你好世界", "zh"),
            EditIntent::Translate { target_lang: "en".to_string() }
        );
        assert_eq!(
            parse_edit_intent("请总结", "en", "长文本", "zh"),
            EditIntent::Summarize
        );
        assert_eq!(
            parse_edit_intent("修改语法", "en", "文本", "zh"),
            EditIntent::FixGrammar
        );
        assert_eq!(
            parse_edit_intent("分点列出", "en", "文本", "zh"),
            EditIntent::Bulletize
        );
    }

    #[test]
    fn rejects_conversational_idioms_and_long_sentences() {
        assert_eq!(
            parse_edit_intent("короче говоря это было отличное решение", "en", "Текст", "ru"),
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
            EditIntent::Translate { target_lang: "en".to_string() }
        );

        // English text with German UI and default target "en" -> translates to German UI lang
        assert_eq!(
            parse_edit_intent("translate", "en", "Hello world", "de"),
            EditIntent::Translate { target_lang: "de".to_string() }
        );

        // English text with German UI and default target "es" -> translates to Spanish
        assert_eq!(
            parse_edit_intent("translate", "es", "Hello world", "de"),
            EditIntent::Translate { target_lang: "es".to_string() }
        );

        // English text with Russian UI -> translates to Russian
        assert_eq!(
            parse_edit_intent("переведи", "en", "Hello world", "ru"),
            EditIntent::Translate { target_lang: "ru".to_string() }
        );
    }

    #[test]
    fn test_commands_guide_chips_synchronization() {
        // Fix Grammar chips
        assert_eq!(parse_edit_intent("исправь ошибки", "en", "Текст", "ru"), EditIntent::FixGrammar);
        assert_eq!(parse_edit_intent("проверь текст", "en", "Текст", "ru"), EditIntent::FixGrammar);
        assert_eq!(parse_edit_intent("исправь пунктуацию", "en", "Текст", "ru"), EditIntent::FixGrammar);

        // Translate chips
        assert_eq!(parse_edit_intent("переведи", "en", "Текст", "ru"), EditIntent::Translate { target_lang: "en".into() });
        assert_eq!(parse_edit_intent("переведи на английский", "en", "Текст", "ru"), EditIntent::Translate { target_lang: "en".into() });
        assert_eq!(parse_edit_intent("переведи на немецкий", "en", "Текст", "ru"), EditIntent::Translate { target_lang: "de".into() });

        // Shorten chips
        assert_eq!(parse_edit_intent("сократи", "en", "Текст", "ru"), EditIntent::Shorten);
        assert_eq!(parse_edit_intent("сделай короче", "en", "Текст", "ru"), EditIntent::Shorten);
        assert_eq!(parse_edit_intent("сожми текст", "en", "Текст", "ru"), EditIntent::Shorten);

        // Expand chips
        assert_eq!(parse_edit_intent("распиши подробнее", "en", "Текст", "ru"), EditIntent::Expand);
        assert_eq!(parse_edit_intent("разверни мысль", "en", "Текст", "ru"), EditIntent::Expand);
        assert_eq!(parse_edit_intent("добавь деталей", "en", "Текст", "ru"), EditIntent::Expand);

        // Bulletize chips
        assert_eq!(parse_edit_intent("сделай список", "en", "Текст", "ru"), EditIntent::Bulletize);
        assert_eq!(parse_edit_intent("по пунктам", "en", "Текст", "ru"), EditIntent::Bulletize);
        assert_eq!(parse_edit_intent("оформи списком", "en", "Текст", "ru"), EditIntent::Bulletize);

        // Summarize chips
        assert_eq!(parse_edit_intent("главные тезисы", "en", "Текст", "ru"), EditIntent::Summarize);
        assert_eq!(parse_edit_intent("краткая суть", "en", "Текст", "ru"), EditIntent::Summarize);
        assert_eq!(parse_edit_intent("сделай саммари", "en", "Текст", "ru"), EditIntent::Summarize);
        assert_eq!(parse_edit_intent("сделай выжимку", "en", "Текст", "ru"), EditIntent::Summarize);
        assert_eq!(parse_edit_intent("выжимку", "en", "Текст", "ru"), EditIntent::Summarize);

        // Rewrite Formal chips
        assert_eq!(parse_edit_intent("деловой стиль", "en", "Текст", "ru"), EditIntent::RewriteFormal);
        assert_eq!(parse_edit_intent("в деловом стиле", "en", "Текст", "ru"), EditIntent::RewriteFormal);
        assert_eq!(parse_edit_intent("сделай официально", "en", "Текст", "ru"), EditIntent::RewriteFormal);
        assert_eq!(parse_edit_intent("вежливый тон", "en", "Текст", "ru"), EditIntent::RewriteFormal);

        // Rewrite Casual chips
        assert_eq!(parse_edit_intent("простыми словами", "en", "Текст", "ru"), EditIntent::RewriteCasual);
        assert_eq!(parse_edit_intent("дружеский тон", "en", "Текст", "ru"), EditIntent::RewriteCasual);
        assert_eq!(parse_edit_intent("неформально", "en", "Текст", "ru"), EditIntent::RewriteCasual);
        assert_eq!(parse_edit_intent("непринужденный стиль", "en", "Текст", "ru"), EditIntent::RewriteCasual);
    }
}
