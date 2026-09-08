use std::collections::HashSet;
use std::sync::OnceLock;

/// List of lowercase Russian words (conjunctions, prepositions, particles, pronouns, adverbs)
/// that should almost never be capitalized inside a sentence unless starting a new sentence after a period.
static RU_LOWERCASE_WORDS: OnceLock<HashSet<&'static str>> = OnceLock::new();

fn get_ru_lowercase_words() -> &'static HashSet<&'static str> {
    RU_LOWERCASE_WORDS.get_or_init(|| {
        [
            // Conjunctions & Connectors
            "и",
            "что",
            "то",
            "если",
            "или",
            "но",
            "либо",
            "а",
            "да",
            "не",
            "ни",
            "как",
            "также",
            "тоже",
            "хотя",
            "чтобы",
            "потому",
            "отчего",
            "зачем",
            "почему",
            "который",
            "которая",
            "которое",
            "которые",
            "которого",
            "которой",
            "которым",
            "которых",
            // Prepositions
            "в",
            "во",
            "на",
            "с",
            "со",
            "к",
            "ко",
            "по",
            "из",
            "изо",
            "от",
            "ото",
            "до",
            "для",
            "без",
            "безо",
            "под",
            "подо",
            "над",
            "надо",
            "о",
            "об",
            "обо",
            "у",
            "при",
            "про",
            "через",
            "сквозь",
            "между",
            "среди",
            "после",
            "вместо",
            "около",
            // Pronouns & Particles
            "это",
            "эта",
            "этот",
            "эти",
            "этого",
            "этой",
            "этом",
            "этих",
            "этим",
            "этими",
            "тот",
            "та",
            "те",
            "того",
            "той",
            "тех",
            "тем",
            "теми",
            "том",
            "я",
            "ты",
            "он",
            "она",
            "оно",
            "они",
            "мы",
            "вы",
            "меня",
            "тебя",
            "его",
            "ее",
            "её",
            "их",
            "нас",
            "вас",
            "мне",
            "тебе",
            "ему",
            "ей",
            "им",
            "нам",
            "вам",
            "мной",
            "тобой",
            "им",
            "ей",
            "ею",
            "ими",
            "нами",
            "вами",
            "мой",
            "моя",
            "мое",
            "моё",
            "мои",
            "твой",
            "твоя",
            "твое",
            "твоё",
            "твои",
            "наш",
            "наша",
            "наше",
            "наши",
            "ваш",
            "ваша",
            "ваше",
            "ваши",
            "свой",
            "своя",
            "свое",
            "своё",
            "свои",
            "сам",
            "сама",
            "само",
            "сами",
            "себя",
            "себе",
            "собой",
            "все",
            "всё",
            "весь",
            "вся",
            "всего",
            "всей",
            "всем",
            "всеми",
            "всех",
            "только",
            "уже",
            "еще",
            "ещё",
            "лишь",
            "даже",
            "ведь",
            "вот",
            "вон",
            "ну",
            "же",
            "ли",
            "бы",
            "где",
            "куда",
            "откуда",
            "когда",
            "тогда",
            "там",
            "тут",
            "здесь",
            "просто",
            "пойми",
            "хоть",
            "вдруг",
            "разве",
            "неужели",
            "никак",
            "один",
            "одна",
            "одно",
            "одни",
            "одного",
            "одной",
            "одним",
            "одних",
            "много",
            "мало",
            "немного",
            "несколько",
            "больше",
            "меньше",
            "нет",
        ]
        .into_iter()
        .collect()
    })
}

/// Words that strongly indicate the start of an independent new thought/sentence
/// when spoken with a pause and capitalized by Whisper.
static RU_SENTENCE_STARTERS: OnceLock<HashSet<&'static str>> = OnceLock::new();

fn get_ru_sentence_starters() -> &'static HashSet<&'static str> {
    RU_SENTENCE_STARTERS.get_or_init(|| {
        [
            "иногда",
            "однако",
            "кроме",
            "затем",
            "потом",
            "сейчас",
            "теперь",
            "кстати",
            "наверное",
            "возможно",
            "конечно",
            "впрочем",
            "наконец",
            "действительно",
            "видимо",
            "оказывается",
            "следовательно",
            "соответственно",
            "нет",
            "да",
            "ну",
        ]
        .into_iter()
        .collect()
    })
}

/// Removes hallucinated noise tags in brackets/parentheses like [Музыка], (Смех).
pub fn clean_hallucinated_brackets(text: &str) -> String {
    let mut cleaned = text.trim().to_string();
    let noise_words = [
        "музыка",
        "music",
        "laughter",
        "смех",
        "background noise",
        "шум",
        "applause",
        "аплодисменты",
        "silence",
        "тишина",
        "sigh",
        "вздох",
        "cough",
        "кашель",
        "crying",
        "плач",
        "whispering",
        "шепот",
    ];

    for term in &noise_words {
        let mut capitalize = term.chars();
        let term_upper = match capitalize.next() {
            None => String::new(),
            Some(first) => first.to_uppercase().collect::<String>() + capitalize.as_str(),
        };

        cleaned = cleaned.replace(&format!("[{term_upper}]"), "");
        cleaned = cleaned.replace(&format!("[{term}]"), "");
        cleaned = cleaned.replace(&format!("({term_upper})"), "");
        cleaned = cleaned.replace(&format!("({term})"), "");
    }
    cleaned.trim().to_string()
}

/// Replaces mid-sentence line breaks (`\n`) emitted by whisper segment boundaries with spaces,
/// while preserving genuine paragraph breaks (double newlines or lines following terminal punctuation).
pub fn smooth_line_breaks(text: &str) -> String {
    let normalized_crlf = text.replace("\r\n", "\n").replace('\r', "\n");
    let lines: Vec<&str> = normalized_crlf.split('\n').collect();
    if lines.len() <= 1 {
        return text.trim().to_string();
    }

    let mut result = String::with_capacity(text.len());

    for (i, line) in lines.iter().enumerate() {
        let trimmed_line = line.trim();
        if trimmed_line.is_empty() {
            // Preserve explicit blank lines (paragraph breaks)
            if !result.ends_with("\n\n") && !result.is_empty() {
                result.push('\n');
            }
            continue;
        }

        if result.is_empty() {
            result.push_str(trimmed_line);
            continue;
        }

        let prev_char = result.trim_end().chars().last();
        let is_prev_terminal = match prev_char {
            Some(c) => matches!(c, '.' | '!' | '?' | ':' | ';' | '—'),
            None => false,
        };

        let is_double_newline = result.ends_with('\n');

        if is_double_newline || (is_prev_terminal && i > 0 && lines[i - 1].is_empty()) {
            // Keep paragraph break
            if !result.ends_with('\n') {
                result.push('\n');
            }
            result.push_str(trimmed_line);
        } else if is_prev_terminal {
            // Preceded by dot/exclamation/question: add a space
            result.push(' ');
            result.push_str(trimmed_line);
        } else {
            // Preceded by an open sentence: stitch together seamlessly with a space
            result.push(' ');
            result.push_str(trimmed_line);
        }
    }

    result.trim().to_string()
}

/// Normalizes spacing around punctuation marks (removes spaces before commas/dots, adds space after).
pub fn normalize_punctuation_spacing(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();

    let mut i = 0;
    while i < len {
        let ch = chars[i];

        // 1. Remove space before punctuation: "слово , слово" -> "слово, слово"
        if matches!(ch, ',' | '.' | '!' | '?' | ':' | ';' | ')') && result.ends_with(' ') {
            // Keep space if the preceding character was an opening bracket or dash
            let trimmed = result.trim_end();
            let last_before_space = trimmed.chars().last();
            if !matches!(
                last_before_space,
                Some('(') | Some('[') | Some('{') | Some('—')
            ) {
                result.truncate(trimmed.len());
            }
        }

        // 2. Normalize repeated commas/dots: ",," -> ",", ".." -> "." (but keep "...")
        if ch == ',' && i + 1 < len && chars[i + 1] == ',' {
            i += 1;
            continue;
        }
        if ch == '.' && i + 1 < len && chars[i + 1] == '.' {
            if i + 2 < len && chars[i + 2] == '.' {
                // Ellipsis "..."
                result.push_str("...");
                i += 3;
                continue;
            } else {
                // Double dot -> single dot
                result.push('.');
                i += 2;
                continue;
            }
        }

        result.push(ch);

        // 3. Ensure space after punctuation if directly followed by a letter/digit (not space, not closing quote/bracket)
        // Except when punctuation is inside a numeric/decimal context (e.g. 1.0, 3.14, 12:30)
        if matches!(ch, ',' | '.' | '!' | '?' | ':' | ';') && i + 1 < len {
            let next_ch = chars[i + 1];
            let prev_char = result.trim_end().chars().last();
            let is_number_context = (ch == '.' || ch == ',' || ch == ':')
                && prev_char.is_some_and(|c| c.is_ascii_digit())
                && next_ch.is_ascii_digit();

            if next_ch.is_alphanumeric() && !is_number_context {
                result.push(' ');
            }
        }

        i += 1;
    }

    result
}

fn collapse_version_prefix(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];
        if (ch == 'v' || ch == 'V')
            && i + 2 < len
            && chars[i + 1] == ' '
            && chars[i + 2].is_ascii_digit()
        {
            let prev_is_boundary =
                i == 0 || chars[i - 1].is_whitespace() || "([{\"'«".contains(chars[i - 1]);
            if prev_is_boundary {
                result.push(ch);
                result.push(chars[i + 2]);
                i += 3;
                continue;
            }
        }
        result.push(ch);
        i += 1;
    }
    result
}

fn collapse_file_extensions_and_domains(text: &str) -> String {
    const KNOWN_EXTENSIONS: &[&str] = &[
        "rs", "ts", "tsx", "js", "json", "py", "md", "wav", "png", "jpg", "jpeg", "svg", "html",
        "css", "zip", "tar", "gz", "pdf", "txt", "bin", "onnx", "exe", "dll", "toml", "yaml",
        "yml", "csv", "log", "sh", "bat", "com", "ru", "org", "net", "io", "dev", "ai", "app",
    ];

    let mut result = text.to_string();
    for ext in KNOWN_EXTENSIONS {
        let pattern = format!(". {ext}");
        let mut start = 0;
        while let Some(pos) = result[start..].find(&pattern) {
            let match_pos = start + pos;
            if match_pos > 0 {
                let prev_char = result[..match_pos].chars().last();
                if let Some(prev) = prev_char {
                    if prev.is_ascii_alphanumeric() {
                        let after_idx = match_pos + pattern.len();
                        let is_end = after_idx >= result.len();
                        let next_is_boundary = if !is_end {
                            let next_char = result[after_idx..].chars().next();
                            next_char.is_none_or(|c| {
                                c.is_whitespace() || ".,!?:;-—\"'«»()[]{} \t\r\n".contains(c)
                            })
                        } else {
                            true
                        };

                        if next_is_boundary {
                            result = format!(
                                "{}{}.{}{}",
                                &result[..match_pos],
                                "",
                                ext,
                                &result[after_idx..]
                            );
                            start = match_pos + 1 + ext.len();
                            continue;
                        }
                    }
                }
            }
            start = match_pos + pattern.len();
        }
    }
    result
}

/// Collapses bogus spaces introduced between digits, dots, colons, version numbers and technical extensions:
/// e.g. "1. 0. 9" -> "1.0.9", "3. 14" -> "3.14", "v1. 0. 8" -> "v1.0.8", "12 : 30" -> "12:30", "app. exe" -> "app.exe".
pub fn collapse_number_and_version_spacings(text: &str) -> String {
    if text.is_empty() {
        return String::new();
    }

    let mut result = text.to_string();

    // 1. Collapse spaces between digits around dots/commas/colons
    result = collapse_split_decimals_and_time(&result);

    // 2. Collapse "v 1." -> "v1."
    result = collapse_version_prefix(&result);

    // 3. Collapse technical file extensions and domains
    result = collapse_file_extensions_and_domains(&result);

    result
}

fn collapse_split_decimals_and_time(text: &str) -> String {
    let mut result = text.to_string();

    for sep in ['.', ',', ':'] {
        loop {
            let mut new_result = String::with_capacity(result.len());
            let chars: Vec<char> = result.chars().collect();
            let len = chars.len();
            let mut i = 0;
            let mut modified = false;

            while i < len {
                let ch = chars[i];
                if ch.is_ascii_digit() {
                    new_result.push(ch);

                    let mut peek = i + 1;
                    while peek < len && chars[peek] == ' ' {
                        peek += 1;
                    }

                    if peek < len && chars[peek] == sep {
                        let mut after_sep = peek + 1;
                        let mut has_spaces_after = false;
                        while after_sep < len && chars[after_sep] == ' ' {
                            has_spaces_after = true;
                            after_sep += 1;
                        }

                        if after_sep < len
                            && chars[after_sep].is_ascii_digit()
                            && (peek > i + 1 || has_spaces_after)
                        {
                            new_result.push(sep);
                            new_result.push(chars[after_sep]);
                            i = after_sep + 1;
                            modified = true;
                            continue;
                        }
                    }
                } else {
                    new_result.push(ch);
                }
                i += 1;
            }

            result = new_result;
            if !modified {
                break;
            }
        }
    }

    result
}

/// Eliminates repeated words / stutters created across sliding window boundaries
/// e.g. "не... не превратиться" -> "не превратиться", "слово слово" -> "слово".
pub fn reduce_boundary_stutters(text: &str) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() <= 1 {
        return text.to_string();
    }

    let mut filtered_words: Vec<String> = Vec::with_capacity(words.len());

    for word in words {
        let clean_current = word
            .trim_matches(|c: char| c.is_ascii_punctuation() || ".,!?:;-—\"'«»()[]{}".contains(c))
            .to_lowercase();

        if let Some(prev) = filtered_words.last() {
            let clean_prev = prev
                .trim_matches(|c: char| {
                    c.is_ascii_punctuation() || ".,!?:;-—\"'«»()[]{}".contains(c)
                })
                .to_lowercase();

            // If the previous word was an exact duplicate of a short conjunction/particle or word with ellipsis
            if !clean_current.is_empty()
                && clean_prev == clean_current
                && (prev.ends_with("...") || prev.ends_with(".."))
            {
                // "не... не" -> replace the stuttered prefix with the clean continuation
                filtered_words.pop();
                filtered_words.push(word.to_string());
                continue;
            }
        }
        filtered_words.push(word.to_string());
    }

    filtered_words.join(" ")
}

/// Normalizes sentence boundaries and capitalization:
/// - Capitalizes the very first letter of the text.
/// - Lowers bogus uppercase on Russian conjunctions/prepositions/particles inside a sentence.
/// - Inserts a period before capitalized words that clearly start a new independent thought after a pause.
pub fn normalize_sentence_boundaries_and_case(text: &str, _language: &str) -> String {
    if text.trim().is_empty() {
        return String::new();
    }

    let ru_lower = get_ru_lowercase_words();
    let ru_starters = get_ru_sentence_starters();

    let tokens: Vec<&str> = text.split_whitespace().collect();
    if tokens.is_empty() {
        return String::new();
    }

    let mut result_tokens: Vec<String> = Vec::with_capacity(tokens.len());

    for (i, token) in tokens.iter().enumerate() {
        if i == 0 {
            // Capitalize first character of the entire transcription
            let mut chars = token.chars();
            let first_cap = match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            };
            result_tokens.push(first_cap);
            continue;
        }

        let prev_token = result_tokens.last().cloned().unwrap_or_default();
        let prev_ends_with_terminal = prev_token.ends_with('.')
            || prev_token.ends_with('!')
            || prev_token.ends_with('?')
            || prev_token.ends_with(':')
            || prev_token.ends_with(';');

        let mut chars = token.chars();
        let first_char = chars.next();

        if let Some(first) = first_char {
            if first.is_uppercase() {
                let rest_str: String = chars.collect();
                let lower_token = token.to_lowercase();
                let lower_clean = lower_token
                    .trim_matches(|c: char| !c.is_alphabetic())
                    .to_string();

                if !prev_ends_with_terminal {
                    // Case 1: The word is a known sentence starter ("Иногда", "Однако", "Кстати", "Нет", "Да", "Ну")
                    if ru_starters.contains(lower_clean.as_str()) {
                        // Add a period to previous token if it doesn't have any punctuation
                        if let Some(prev) = result_tokens.last_mut() {
                            if !prev.ends_with(|c: char| ".,!?:;-—".contains(c)) {
                                prev.push('.');
                            }
                        }
                        result_tokens.push(token.to_string());
                        continue;
                    }

                    // Case 2: The word is in our known lowercase vocabulary (e.g. "Что", "То", "И", "Если", "Не", "Который")
                    if ru_lower.contains(lower_clean.as_str()) {
                        // De-capitalize: "ты написал Что единица" -> "ты написал что единица"
                        let decapitalized = first.to_lowercase().collect::<String>() + &rest_str;
                        result_tokens.push(decapitalized);
                        continue;
                    }

                    // Case 3: For other capitalized words inside a sentence:
                    // If previous token ended with a comma or dash, lowercase unless all caps (acronym)
                    let prev_is_comma = prev_token.ends_with(',') || prev_token.ends_with('—');
                    let is_all_caps = token
                        .chars()
                        .filter(|c| c.is_alphabetic())
                        .all(|c| c.is_uppercase());

                    if prev_is_comma && !is_all_caps && token.chars().count() > 1 {
                        let decapitalized = first.to_lowercase().collect::<String>() + &rest_str;
                        result_tokens.push(decapitalized);
                        continue;
                    }
                }
            }
        }

        result_tokens.push(token.to_string());
    }

    result_tokens.join(" ")
}

/// Strips typical Whisper subtitle / metadata hallucinated prefixes like "Текст фильма:", "Субтитры:", "Автор субтитров:".
fn remove_inline_hallucination_phrase(
    text: &str,
    phrase: &str,
    require_colon_or_punct_suffix: bool,
) -> String {
    let mut result = text.to_string();
    loop {
        let lower = result.to_lowercase();
        let Some(pos) = lower.find(phrase) else {
            break;
        };

        let before = &result[..pos];
        let after_raw = &result[pos + phrase.len()..];

        // Check following characters
        let trimmed_after = after_raw.trim_start_matches([' ', '\t']);

        let has_separator = trimmed_after.starts_with([':', '-', '—', '.', '…', '!', '?', ',']);

        if require_colon_or_punct_suffix
            && !has_separator
            && !trimmed_after.is_empty()
            && !before.ends_with(['.', '!', '?', '\n'])
        {
            // Not a subtitle marker (e.g. legitimate dictation like "текст фильма был...")
            break;
        }

        // Consume trailing separator characters (like ":", "...", "…", "-", etc.)
        let rest_after_sep =
            trimmed_after.trim_start_matches([':', '-', '—', '.', '…', '!', '?', ',', ' ', '\t']);
        let skip_len = result.len() - before.len() - rest_after_sep.len();

        let after = &result[pos + skip_len..];

        // Build stitched string
        let before_trimmed = before.trim_end();
        let after_trimmed = after.trim_start();

        if before_trimmed.is_empty() {
            result = after_trimmed.to_string();
        } else if after_trimmed.is_empty() {
            result = before_trimmed.to_string();
        } else {
            // Stitch before and after
            let ends_with_terminal = before_trimmed.ends_with(['.', '!', '?', '—', '\n']);
            if ends_with_terminal {
                result = format!("{before_trimmed} {after_trimmed}");
            } else {
                result = format!("{before_trimmed}. {after_trimmed}");
            }
        }
    }
    result
}

fn clean_midtext_subtitle_artifacts(text: &str) -> String {
    // Targeted cleaning for the specific phrases encountered in dictations:
    // 1. "продолжение следует"
    let text = remove_inline_hallucination_phrase(text, "продолжение следует", false);
    // 2. "текст фильма" (when accompanied by ':' / '-' / punctuation or at boundary)
    remove_inline_hallucination_phrase(&text, "текст фильма", true)
}

/// Strips typical Whisper subtitle / metadata hallucinated prefixes and artifacts.
pub fn strip_hallucinated_prefixes(text: &str) -> String {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let lower = trimmed.to_lowercase();

    // Check standalone silence hallucinations first (including ellipsis and unicode punctuation)
    let clean_punct = lower
        .trim_matches(|c: char| {
            c.is_ascii_punctuation() || ".,!?:;-—\"'«»()[]{}… \t\r\n".contains(c)
        })
        .to_string();

    let standalone_hallucinations = [
        "текст фильма",
        "текст видео",
        "субтитры",
        "автор субтитров",
        "продолжение следует",
        "конец фильма",
        "спасибо за просмотр",
        "подписывайтесь на канал",
        "ставьте лайки",
        "terms",
        "термины",
        "vocabulary",
        "glossary",
        "словарь",
        "thank you for watching",
        "thanks for watching",
        "to be continued",
        "subscribe to my channel",
    ];

    if standalone_hallucinations.iter().any(|&s| clean_punct == s) {
        return String::new();
    }

    let mut current_text = trimmed.to_string();

    // Strip leaked prompt metadata labels (e.g., "Terms:", "Термины:", "Vocabulary:", "Glossary:", "Словарь:")
    let prompt_metadata_labels = ["terms", "термины", "vocabulary", "glossary", "словарь"];

    for label in &prompt_metadata_labels {
        let current_lower = current_text.to_lowercase();
        if current_lower.starts_with(label) {
            let remainder = current_text[label.len()..].trim_start();
            if let Some(first_char) = remainder.chars().next() {
                if matches!(first_char, ':' | '-' | '—') {
                    let after_sep = remainder
                        .trim_start_matches([':', '-', '—', ' ', '\t'])
                        .trim();
                    if after_sep.is_empty() {
                        return String::new();
                    } else {
                        current_text = after_sep.to_string();
                        break;
                    }
                }
            }
        }
    }

    // List of known hallucinated prefix stems.
    let prefix_stems = [
        "текст фильма",
        "текст видео",
        "текст аудио",
        "текст записи",
        "текст диалога",
        "продолжение следует",
        "субтитры созданы",
        "субтитры делал",
        "субтитры делала",
        "субтитры сделал",
        "субтитры подготовил",
        "субтитры подготовила",
        "субтитры добавлены",
        "субтитры от",
        "субтитры",
        "автор субтитров",
        "перевод субтитров",
        "редактор субтитров",
        "текст читал",
        "текст читает",
        "текст предоставил",
        "subtitles by",
        "subtitles",
        "translated by",
        "captions by",
        "movie text",
        "video text",
    ];

    for stem in &prefix_stems {
        let current_lower = current_text.to_lowercase();
        if current_lower.starts_with(stem) {
            let remainder = current_text[stem.len()..].trim_start();
            let clean_rem_punct = remainder.trim_matches(|c: char| {
                c.is_ascii_punctuation() || ".,!?:;-—\"'«»()[]{}… \t\r\n".contains(c)
            });
            if clean_rem_punct.is_empty() {
                return String::new();
            }

            // If followed by punctuation like ':', '-', '—', '.', '…'
            if let Some(first_char) = remainder.chars().next() {
                if matches!(first_char, ':' | '-' | '—' | '.' | '…' | '!' | '?' | ',') {
                    let after_punct = remainder
                        .trim_start_matches([':', '-', '—', '.', '…', '!', '?', ',', ' ', '\t'])
                        .trim();
                    if after_punct.is_empty() {
                        return String::new();
                    } else {
                        current_text = after_punct.to_string();
                        break;
                    }
                }
            }

            // Also check if followed by author name (up to 35 chars) + colon
            if let Some(colon_pos) = remainder.find([':', '-', '—']) {
                if colon_pos <= 35 {
                    let after_sep = remainder[colon_pos + 1..].trim_start();
                    if !after_sep.is_empty() {
                        current_text = after_sep.to_string();
                        break;
                    } else {
                        return String::new();
                    }
                }
            } else {
                let clean_rem = remainder.trim();
                if clean_rem.is_empty() {
                    return String::new();
                }
                current_text = clean_rem.to_string();
                break;
            }
        }
    }

    // Remove inline / mid-text / trailing artifacts
    current_text = clean_midtext_subtitle_artifacts(&current_text);

    current_text.trim().to_string()
}

/// Full comprehensive normalization pipeline for speech recognition transcripts.
pub fn normalize_transcription_text(raw_text: &str, language: &str) -> String {
    let unbracketed = clean_hallucinated_brackets(raw_text);
    if unbracketed.trim().is_empty() {
        return String::new();
    }

    let unhallucinated = strip_hallucinated_prefixes(&unbracketed);
    if unhallucinated.trim().is_empty() {
        return String::new();
    }

    let smoothed = smooth_line_breaks(&unhallucinated);
    let unstuttered = reduce_boundary_stutters(&smoothed);
    let spaced = normalize_punctuation_spacing(&unstuttered);
    let cased = normalize_sentence_boundaries_and_case(&spaced, language);
    let final_spaced = normalize_punctuation_spacing(&cased);
    let formatted = collapse_number_and_version_spacings(&final_spaced);

    formatted.trim().to_string()
}

/// Smooths speech boundaries between committed left text and newly decoded right text in streaming mode.
/// Prevents awkward periods and capitalized words when user stutters or pauses before conjunctions (e.g. "и", "но", "что").
pub fn smooth_conjunction_boundary(left: &str, right: &str) -> String {
    let left = left.trim();
    let right = right.trim();
    if left.is_empty() {
        return right.to_string();
    }
    if right.is_empty() {
        return left.to_string();
    }

    // Check if left ends with a single period (not ellipsis "...")
    let left_ends_with_period = left.ends_with('.') && !left.ends_with("..");
    if !left_ends_with_period {
        return format!("{left} {right}");
    }

    let (first_token, remainder) = match right.split_once(char::is_whitespace) {
        Some((w, r)) => (w, r),
        None => (right, ""),
    };

    let clean_first = first_token
        .trim_matches(|c: char| !c.is_alphabetic())
        .to_lowercase();

    // Conjunctions requiring comma in Russian / English
    const COMMA_CONJUNCTIONS: &[&str] = &[
        "а",
        "но",
        "что",
        "чтобы",
        "чтоб",
        "хотя",
        "когда",
        "если",
        "потому",
        "оттого",
        "зато",
        "однако",
        "поскольку",
        "причем",
        "причём",
        "то",
        "but",
        "because",
        "although",
        "though",
        "whereas",
        "while",
        "since",
    ];

    // Conjunctions without comma in Russian / English (connectors)
    const NO_COMMA_CONJUNCTIONS: &[&str] = &["и", "или", "либо", "да", "and", "or", "nor", "so"];

    let is_comma = COMMA_CONJUNCTIONS.contains(&clean_first.as_str());
    let is_no_comma = NO_COMMA_CONJUNCTIONS.contains(&clean_first.as_str());

    if !is_comma && !is_no_comma {
        return format!("{left} {right}");
    }

    // Strip trailing period from left
    let left_stripped = left.strip_suffix('.').unwrap_or(left).trim_end();

    // Lowercase the first token of right
    let lower_first = first_token
        .chars()
        .next()
        .map(|c| c.to_lowercase().collect::<String>() + &first_token[c.len_utf8()..])
        .unwrap_or_else(|| first_token.to_string());

    let right_rest = if remainder.is_empty() {
        lower_first
    } else {
        format!("{lower_first} {remainder}")
    };

    if is_comma {
        if left_stripped.ends_with(|c: char| ",;:-—".contains(c)) {
            format!("{left_stripped} {right_rest}")
        } else {
            format!("{left_stripped}, {right_rest}")
        }
    } else {
        format!("{left_stripped} {right_rest}")
    }
}

fn get_current_local_datetime() -> (String, String, String, String) {
    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::Foundation::SYSTEMTIME;
        use windows_sys::Win32::System::SystemInformation::GetLocalTime;
        let mut st: SYSTEMTIME = unsafe { std::mem::zeroed() };
        unsafe { GetLocalTime(&mut st) };
        let date_iso = format!("{:04}-{:02}-{:02}", st.wYear, st.wMonth, st.wDay);
        let date_ru = format!("{:02}.{:02}.{:04}", st.wDay, st.wMonth, st.wYear);
        let time_str = format!("{:02}:{:02}", st.wHour, st.wMinute);
        let datetime_str = format!("{} {}", date_iso, time_str);
        (date_iso, date_ru, time_str, datetime_str)
    }
    #[cfg(not(target_os = "windows"))]
    {
        let now = std::time::SystemTime::now();
        let secs = now
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        let days = secs / 86400;
        let rem_secs = secs % 86400;
        let hours = rem_secs / 3600;
        let minutes = (rem_secs % 3600) / 60;
        let date_iso = format!("day-{days}");
        let date_ru = format!("day-{days}");
        let time_str = format!("{:02}:{:02}", hours, minutes);
        let datetime_str = format!("{date_iso} {time_str}");
        (date_iso, date_ru, time_str, datetime_str)
    }
}

/// Applies user-configured custom replacements and dynamic macros (e.g. {date}, {time})
/// across the text with whole-token boundary checking and case-insensitivity.
pub fn apply_text_replacements(
    text: &str,
    replacements: &[crate::settings::TextReplacement],
) -> String {
    if text.trim().is_empty() || replacements.is_empty() {
        return text.to_string();
    }

    let (date_iso, date_ru, time_str, datetime_str) = get_current_local_datetime();
    let mut result = text.to_string();

    for rule in replacements {
        let trigger = rule.trigger.trim();
        if trigger.is_empty() {
            continue;
        }

        // Expand dynamic macros in replacement
        let mut expanded_replacement = rule.replacement.clone();
        if expanded_replacement.contains("{date}") {
            expanded_replacement = expanded_replacement.replace("{date}", &date_iso);
        }
        if expanded_replacement.contains("{date_ru}") {
            expanded_replacement = expanded_replacement.replace("{date_ru}", &date_ru);
        }
        if expanded_replacement.contains("{time}") {
            expanded_replacement = expanded_replacement.replace("{time}", &time_str);
        }
        if expanded_replacement.contains("{datetime}") {
            expanded_replacement = expanded_replacement.replace("{datetime}", &datetime_str);
        }

        result = replace_case_insensitive_boundary(&result, trigger, &expanded_replacement);
    }

    result
}

fn replace_case_insensitive_boundary(haystack: &str, trigger: &str, replacement: &str) -> String {
    if haystack.is_empty() || trigger.is_empty() {
        return haystack.to_string();
    }

    let orig_chars: Vec<char> = haystack.chars().collect();
    let trigger_chars: Vec<char> = trigger.chars().collect();
    let trigger_len = trigger_chars.len();

    if trigger_len == 0 || trigger_len > orig_chars.len() {
        return haystack.to_string();
    }

    let mut output = String::with_capacity(haystack.len());
    let mut i = 0;

    while i < orig_chars.len() {
        if i + trigger_len <= orig_chars.len() {
            let matches = (0..trigger_len).all(|j| {
                orig_chars[i + j]
                    .to_lowercase()
                    .eq(trigger_chars[j].to_lowercase())
            });

            if matches {
                let left_ok = i == 0 || !orig_chars[i - 1].is_alphanumeric();
                let right_ok = (i + trigger_len == orig_chars.len())
                    || !orig_chars[i + trigger_len].is_alphanumeric();

                if left_ok && right_ok {
                    output.push_str(replacement);
                    i += trigger_len;
                    continue;
                }
            }
        }
        output.push(orig_chars[i]);
        i += 1;
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_and_version_collapsing() {
        assert_eq!(
            normalize_transcription_text("версия 1. 0. 9 нашей программы", "ru"),
            "Версия 1.0.9 нашей программы"
        );
        assert_eq!(
            normalize_transcription_text("просто берем 1. 0. 9, то он", "ru"),
            "Просто берем 1.0.9, то он"
        );
        assert_eq!(
            normalize_transcription_text("версия v1. 0. 8", "ru"),
            "Версия v1.0.8"
        );
        assert_eq!(
            normalize_transcription_text("число 3. 14 и 0. 5", "ru"),
            "Число 3.14 и 0.5"
        );
        assert_eq!(
            normalize_transcription_text("встреча в 12: 30", "ru"),
            "Встреча в 12:30"
        );
        assert_eq!(
            normalize_transcription_text("файл main. rs и app. exe", "ru"),
            "Файл main.rs и app.exe"
        );
        assert_eq!(
            normalize_transcription_text("сайт github. com", "ru"),
            "Сайт github.com"
        );
        assert_eq!(
            normalize_transcription_text("1. Первый пункт. 2. Второй пункт.", "ru"),
            "1. Первый пункт. 2. Второй пункт."
        );
    }

    #[test]
    fn test_strip_hallucinated_prefixes() {
        let input = "Текст фильма: Хотел в сталкер зайти, но там код нужен от тебя.";
        let output = normalize_transcription_text(input, "ru");
        assert_eq!(output, "Хотел в сталкер зайти, но там код нужен от тебя.");

        let input_subtitles = "Субтитры делал Dima: Всем привет, это проверка.";
        let output_subtitles = normalize_transcription_text(input_subtitles, "ru");
        assert_eq!(output_subtitles, "Всем привет, это проверка.");

        let standalone = "Текст фильма.";
        let output_standalone = normalize_transcription_text(standalone, "ru");
        assert_eq!(output_standalone, "");

        let standalone_colon = "Текст фильма:";
        let output_standalone_colon = normalize_transcription_text(standalone_colon, "ru");
        assert_eq!(output_standalone_colon, "");

        let to_be_continued = "Продолжение следует...";
        let output_tbc = normalize_transcription_text(to_be_continued, "ru");
        assert_eq!(output_tbc, "");

        let to_be_continued_unicode = "Продолжение следует…";
        let output_tbc_unicode = normalize_transcription_text(to_be_continued_unicode, "ru");
        assert_eq!(output_tbc_unicode, "");

        let mid_text_tbc = "Я сделал задачу. Продолжение следует... Теперь проверим её.";
        let output_mid_tbc = normalize_transcription_text(mid_text_tbc, "ru");
        assert_eq!(output_mid_tbc, "Я сделал задачу. Теперь проверим её.");

        let mid_text_film = "Я запустил проект. Текст фильма: Там возникла ошибка в коде.";
        let output_mid_film = normalize_transcription_text(mid_text_film, "ru");
        assert_eq!(
            output_mid_film,
            "Я запустил проект. Там возникла ошибка в коде."
        );

        let prefix_tbc = "Продолжение следует... Хотел в сталкер зайти, но там код нужен.";
        let output_prefix_tbc = normalize_transcription_text(prefix_tbc, "ru");
        assert_eq!(
            output_prefix_tbc,
            "Хотел в сталкер зайти, но там код нужен."
        );

        let watching = "Спасибо за просмотр!";
        let output_watching = normalize_transcription_text(watching, "ru");
        assert_eq!(output_watching, "");
    }

    #[test]
    fn legitimate_dictation_must_not_be_wiped_as_hallucinations() {
        assert_eq!(normalize_transcription_text("Конец!", "ru"), "Конец!");
        assert_eq!(
            normalize_transcription_text("Спасибо за внимание.", "ru"),
            "Спасибо за внимание."
        );
        assert_eq!(
            normalize_transcription_text("Перевод: готово к вечеру", "ru"),
            "Перевод: готово к вечеру"
        );
        assert_eq!(
            normalize_transcription_text("Расшифровка встречи: встреча началась в десять", "ru"),
            "Расшифровка встречи: встреча началась в десять"
        );
        assert_eq!(
            normalize_transcription_text("Транскрипция занимает секунды", "ru"),
            "Транскрипция занимает секунды"
        );
    }

    #[test]
    fn subtitle_credit_prefixes_still_stripped() {
        assert_eq!(
            normalize_transcription_text("Перевод субтитров: Dima, всем привет.", "ru"),
            "Dima, всем привет."
        );
        assert_eq!(
            normalize_transcription_text("Субтитры: студия дубляжа", "ru"),
            "Студия дубляжа"
        );
        assert_eq!(
            normalize_transcription_text("Terms: вот пример моей диктовки", "ru"),
            "Вот пример моей диктовки"
        );
        assert_eq!(
            normalize_transcription_text("Термины: проверка качества записи", "ru"),
            "Проверка качества записи"
        );
        assert_eq!(
            normalize_transcription_text("Vocabulary: clean speech dictation", "en"),
            "Clean speech dictation"
        );
        assert_eq!(
            normalize_transcription_text("Terms and conditions apply.", "en"),
            "Terms and conditions apply."
        );
    }

    #[test]
    fn test_smooth_line_breaks_mid_sentence() {
        let input = "текст разбивается, будто бы запись прерывается в процессе\n диктовки предложения И вот, допустим";
        let smoothed = smooth_line_breaks(input);
        assert!(!smoothed.contains('\n'));
        assert!(smoothed.contains("в процессе диктовки предложения"));
    }

    #[test]
    fn test_normalize_leading_letter() {
        let input = "анимация слишком быстрая тебе не кажется";
        let output = normalize_transcription_text(input, "ru");
        assert!(output.starts_with("Анимация"));
    }

    #[test]
    fn test_decapitalize_conjunctions() {
        let input = "В первом пункте ты написал Что единица отображалась только потому, что она была выбрана вручную Нет, это не правда";
        let output = normalize_transcription_text(input, "ru");
        assert!(output.contains("написал что единица"));
        assert!(output.contains("вручную. Нет, это"));
    }

    #[test]
    fn test_sentence_starter_period_insertion() {
        let input = "может разбиваться по абзацам Иногда эти абзацы будто бы расставлены";
        let output = normalize_transcription_text(input, "ru");
        assert!(output.contains("по абзацам. Иногда"));
    }

    #[test]
    fn test_stutter_reduction() {
        let input = "часть моей речи может просто не... не превратиться в текст";
        let output = normalize_transcription_text(input, "ru");
        assert!(output.contains("просто не превратиться в текст"));
    }

    #[test]
    fn test_punctuation_spacing() {
        let input = "слово ,еще слово .И следующее слово";
        let output = normalize_transcription_text(input, "ru");
        assert_eq!(output, "Слово, еще слово. И следующее слово");
    }

    #[test]
    fn test_smooth_conjunction_boundary() {
        // "И" connector removes period
        let res1 = smooth_conjunction_boundary("Мы пошли в магазин.", "И купили хлеб.");
        assert_eq!(res1, "Мы пошли в магазин и купили хлеб.");

        // "А" / "Но" / "Что" replaces period with comma
        let res2 = smooth_conjunction_boundary("Мы пошли в магазин.", "А потом в кино.");
        assert_eq!(res2, "Мы пошли в магазин, а потом в кино.");

        let res3 = smooth_conjunction_boundary("Я думал.", "Что всё готово.");
        assert_eq!(res3, "Я думал, что всё готово.");

        // English conjunctions
        let res4 = smooth_conjunction_boundary("We arrived home.", "And went to bed.");
        assert_eq!(res4, "We arrived home and went to bed.");

        let res5 = smooth_conjunction_boundary("We arrived home.", "Because we were tired.");
        assert_eq!(res5, "We arrived home, because we were tired.");

        // Regular sentence boundaries remain intact
        let res6 = smooth_conjunction_boundary("Первое предложение.", "Второе предложение.");
        assert_eq!(res6, "Первое предложение. Второе предложение.");
    }

    #[test]
    fn test_apply_text_replacements() {
        let replacements = vec![
            crate::settings::TextReplacement {
                trigger: "кубернетис".to_string(),
                replacement: "Kubernetes".to_string(),
            },
            crate::settings::TextReplacement {
                trigger: "мой имейл".to_string(),
                replacement: "admin@aura.app".to_string(),
            },
            crate::settings::TextReplacement {
                trigger: "сегодня".to_string(),
                replacement: "{date}".to_string(),
            },
        ];

        let text1 = "Мы развернули кубернетис на сервере.";
        let res1 = apply_text_replacements(text1, &replacements);
        assert_eq!(res1, "Мы развернули Kubernetes на сервере.");

        let text2 = "Напиши на Мой Имейл прямо сейчас.";
        let res2 = apply_text_replacements(text2, &replacements);
        assert_eq!(res2, "Напиши на admin@aura.app прямо сейчас.");

        let text3 = "Дата релиза сегодня.";
        let res3 = apply_text_replacements(text3, &replacements);
        assert!(!res3.contains("{date}"));
        assert!(res3.starts_with("Дата релиза 20"));
    }
}
