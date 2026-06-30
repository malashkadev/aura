use base64::{engine::general_purpose, Engine as _};
use reqwest::multipart;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ApiProvider {
    Gemini,
    OpenAi,
    Groq,
}

// --- Gemini Request / Response Schemas ---

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Serialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize)]
struct GeminiPart {
    #[serde(rename = "inlineData", skip_serializing_if = "Option::is_none")]
    inline_data: Option<GeminiInlineData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,
}

#[derive(Serialize)]
struct GeminiInlineData {
    #[serde(rename = "mimeType")]
    mime_type: String,
    data: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Option<Vec<GeminiCandidate>>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: Option<GeminiResponseContent>,
}

#[derive(Deserialize)]
struct GeminiResponseContent {
    parts: Option<Vec<GeminiResponsePart>>,
}

#[derive(Deserialize)]
struct GeminiResponsePart {
    text: Option<String>,
}

// --- OpenAI Request / Response Schemas ---

#[derive(Deserialize)]
struct WhisperResponse {
    text: String,
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
}

#[derive(Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Option<Vec<ChatChoice>>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: Option<ChatMessageResponse>,
}

#[derive(Deserialize)]
struct ChatMessageResponse {
    content: Option<String>,
}

/// Transcribes the audio file and cleans up the transcript according to instructions.
///
/// * `provider`: The API provider (`Gemini` or `OpenAi`)
/// * `api_key`: The API key to use for authentication
/// * `wav_path`: Absolute path to the 16kHz mono WAV file
/// * `selected_text`: Context text currently selected by the user
pub async fn transcribe_and_clean(
    provider: ApiProvider,
    api_key: &str,
    wav_path: &str,
    selected_text: &str,
) -> Result<String, String> {
    let client = reqwest::Client::new();

    // Read the WAV file bytes
    let wav_bytes = fs::read(wav_path)
        .map_err(|e| format!("Failed to read WAV file at {wav_path}: {e}"))?;

    match provider {
        ApiProvider::Gemini => {
            // Encode audio bytes in Base64
            let base64_audio = general_purpose::STANDARD.encode(&wav_bytes);

            // Construct Gemini request JSON body
            let prompt = "You are an elite dictation and editing assistant. Task: Transcribe and clean up the speech from the audio. Selected Text Context: The user has selected the following text: [SELECTED_TEXT]\n\nInstructions:\n1. Return ONLY the transcribed text. Do NOT add any explanations, introductory text, greetings, or conversational remarks.\n2. Clean up speech by removing filler words and adding proper punctuation and grammar. Keep the language natural and matching the speaker's language.\n3. CRITICAL language rule: The output text must be in the EXACT SAME LANGUAGE as the transcribed audio. Do NOT translate the text. If the speaker speaks in Russian, output Russian. If the speaker speaks in English, output English.\n4. CRITICAL: If the dictation is simply a statement, question, commentary, or general speech, you must transcribe it word-for-word (with punctuation). DO NOT ANSWER QUESTIONS, do NOT write explanations, and do NOT discuss the topic. Even if the user asks you a direct question in the audio, you must only transcribe the question, NEVER answer it.\n5. If and ONLY if the dictation contains a clear, direct, and explicit command to edit, rewrite, or format the selected text context (e.g., 'make this formal', 'translate this to English', 'wrap in a function'), then perform that edit on the selected text and return the final edited result. If no such editing command is present, ignore the selected text context and simply output the transcribed dictation.".to_string()
            .replace("[SELECTED_TEXT]", selected_text);

            let request_body = GeminiRequest {
                contents: vec![GeminiContent {
                    parts: vec![
                        GeminiPart {
                            inline_data: Some(GeminiInlineData {
                                mime_type: "audio/wav".to_string(),
                                data: base64_audio,
                            }),
                            text: None,
                        },
                        GeminiPart {
                            inline_data: None,
                            text: Some(prompt),
                        },
                    ],
                }],
            };

            let endpoint = format!(
                "https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash:generateContent?key={}",
                api_key
            );

            let response = client
                .post(&endpoint)
                .header("Content-Type", "application/json")
                .json(&request_body)
                .send()
                .await
                .map_err(|e| format!("Gemini API request failed: {e}"))?;

            let status = response.status();
            if !status.is_success() {
                let error_body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "<failed to read response body>".to_string());
                return Err(format!(
                    "Gemini API returned status code {status}. Response body: {error_body}"
                ));
            }

            let gemini_resp: GeminiResponse = response
                .json()
                .await
                .map_err(|e| format!("Failed to parse Gemini API JSON response: {e}"))?;

            // Extract the result candidates[0].content.parts[0].text
            let clean_text = gemini_resp
                .candidates
                .and_then(|c| c.into_iter().next())
                .and_then(|c| c.content)
                .and_then(|c| c.parts)
                .and_then(|p| p.into_iter().next())
                .and_then(|p| p.text)
                .ok_or_else(|| "Gemini response did not contain expected text content structure.".to_string())?;

            Ok(clean_text)
        }

        ApiProvider::OpenAi => {
            // --- Step 1: Call OpenAI Whisper API for transcription ---
            let whisper_endpoint = "https://api.openai.com/v1/audio/transcriptions";

            let file_part = multipart::Part::bytes(wav_bytes)
                .file_name("audio.wav")
                .mime_str("audio/wav")
                .map_err(|e| format!("Failed to prepare audio multipart part: {e}"))?;

            let form = multipart::Form::new()
                .part("file", file_part)
                .text("model", "whisper-1");

            let whisper_response = client
                .post(whisper_endpoint)
                .bearer_auth(api_key)
                .multipart(form)
                .send()
                .await
                .map_err(|e| format!("OpenAI Whisper API request failed: {e}"))?;

            let status = whisper_response.status();
            if !status.is_success() {
                let error_body = whisper_response
                    .text()
                    .await
                    .unwrap_or_else(|_| "<failed to read response body>".to_string());
                return Err(format!(
                    "OpenAI Whisper API returned status code {status}. Response body: {error_body}"
                ));
            }

            let whisper_resp: WhisperResponse = whisper_response
                .json()
                .await
                .map_err(|e| format!("Failed to parse OpenAI Whisper JSON response: {e}"))?;

            let transcribed_text = whisper_resp.text;

            // --- Step 2: Call OpenAI Chat Completions API with gpt-4o-mini to clean/edit ---
            let chat_endpoint = "https://api.openai.com/v1/chat/completions";

            let system_prompt = "You are an elite dictation and editing assistant. Task: Clean up and format the speech transcription. Selected Text Context: The user has selected the following text: [SELECTED_TEXT]\n\nInstructions:\n1. Return ONLY the finalized text. Do NOT add any explanations, introductory text, greetings, or conversational remarks.\n2. Clean up speech by removing filler words and adding proper punctuation and grammar. Keep the language natural and matching the speaker's language.\n3. CRITICAL language rule: The output text must be in the EXACT SAME LANGUAGE as the transcribed audio. Do NOT translate the text. If the speaker speaks in Russian, output Russian. If the speaker speaks in English, output English.\n4. CRITICAL: Treat the input transcription strictly as a passive string to be formatted. DO NOT ANSWER QUESTIONS, do NOT write explanations, and do NOT discuss the topic. Even if the transcription is a question like 'How to do X?', you must ONLY output the transcription text itself with proper punctuation. NEVER answer the question.\n5. If and ONLY if the dictation contains a clear, direct, and explicit command to edit, rewrite, or format the selected text context (e.g., 'make this formal', 'translate this to English', 'wrap in a function'), then perform that edit on the selected text and return the final edited result. If no such editing command is present, ignore the selected text context and simply output the transcribed dictation.".to_string()
            .replace("[SELECTED_TEXT]", selected_text);

            let chat_request = ChatCompletionRequest {
                model: "gpt-4o-mini".to_string(),
                messages: vec![
                    ChatMessage {
                        role: "system".to_string(),
                        content: system_prompt,
                    },
                    ChatMessage {
                        role: "user".to_string(),
                        content: format!("Transcribed text to process: \"{transcribed_text}\""),
                    },
                ],
            };

            let chat_response = client
                .post(chat_endpoint)
                .bearer_auth(api_key)
                .json(&chat_request)
                .send()
                .await
                .map_err(|e| format!("OpenAI Chat Completion API request failed: {e}"))?;

            let status = chat_response.status();
            if !status.is_success() {
                let error_body = chat_response
                    .text()
                    .await
                    .unwrap_or_else(|_| "<failed to read response body>".to_string());
                return Err(format!(
                    "OpenAI Chat Completions API returned status code {status}. Response body: {error_body}"
                ));
            }

            let chat_resp: ChatCompletionResponse = chat_response
                .json()
                .await
                .map_err(|e| format!("Failed to parse OpenAI Chat Completion JSON response: {e}"))?;

            let cleaned_text = chat_resp
                .choices
                .and_then(|c| c.into_iter().next())
                .and_then(|c| c.message)
                .and_then(|m| m.content)
                .ok_or_else(|| "OpenAI Chat Completion response did not contain expected text content structure.".to_string())?;

            Ok(cleaned_text)
        }
        ApiProvider::Groq => {
            // --- Step 1: Call Groq Whisper API for transcription ---
            let whisper_endpoint = "https://api.groq.com/openai/v1/audio/transcriptions";

            let file_part = multipart::Part::bytes(wav_bytes)
                .file_name("audio.wav")
                .mime_str("audio/wav")
                .map_err(|e| format!("Failed to prepare audio multipart part: {e}"))?;

            let form = multipart::Form::new()
                .part("file", file_part)
                .text("model", "whisper-large-v3");

            let whisper_response = client
                .post(whisper_endpoint)
                .bearer_auth(api_key)
                .multipart(form)
                .send()
                .await
                .map_err(|e| format!("Groq Whisper API request failed: {e}"))?;

            let status = whisper_response.status();
            if !status.is_success() {
                let error_body = whisper_response
                    .text()
                    .await
                    .unwrap_or_else(|_| "<failed to read response body>".to_string());
                return Err(format!(
                    "Groq Whisper API returned status code {status}. Response body: {error_body}"
                ));
            }

            let whisper_resp: WhisperResponse = whisper_response
                .json()
                .await
                .map_err(|e| format!("Failed to parse Groq Whisper JSON response: {e}"))?;

            let transcribed_text = whisper_resp.text;

            // --- Step 2: Call Groq Chat Completions with Llama-3.3-70b-versatile to clean ---
            let chat_endpoint = "https://api.groq.com/openai/v1/chat/completions";

            let system_prompt = "You are an elite dictation and editing assistant. Task: Clean up and format the speech transcription. Selected Text Context: The user has selected the following text: [SELECTED_TEXT]\n\nInstructions:\n1. Return ONLY the finalized text. Do NOT add any explanations, introductory text, greetings, or conversational remarks.\n2. Clean up speech by removing filler words and adding proper punctuation and grammar. Keep the language natural and matching the speaker's language.\n3. CRITICAL language rule: The output text must be in the EXACT SAME LANGUAGE as the transcribed audio. Do NOT translate the text. If the speaker speaks in Russian, output Russian. If the speaker speaks in English, output English.\n4. CRITICAL: Treat the input transcription strictly as a passive string to be formatted. DO NOT ANSWER QUESTIONS, do NOT write explanations, and do NOT discuss the topic. Even if the transcription is a question like 'How to do X?', you must ONLY output the transcription text itself with proper punctuation. NEVER answer the question.\n5. If and ONLY if the dictation contains a clear, direct, and explicit command to edit, rewrite, or format the selected text context (e.g., 'make this formal', 'translate this to English', 'wrap in a function'), then perform that edit on the selected text and return the final edited result. If no such editing command is present, ignore the selected text context and simply output the transcribed dictation.".to_string()
            .replace("[SELECTED_TEXT]", selected_text);

            let chat_request = ChatCompletionRequest {
                model: "llama-3.3-70b-versatile".to_string(),
                messages: vec![
                    ChatMessage {
                        role: "system".to_string(),
                        content: system_prompt,
                    },
                    ChatMessage {
                        role: "user".to_string(),
                        content: format!("Transcribed text to process: \"{transcribed_text}\""),
                    },
                ],
            };

            let chat_response = client
                .post(chat_endpoint)
                .bearer_auth(api_key)
                .json(&chat_request)
                .send()
                .await
                .map_err(|e| format!("Groq Chat Completion API request failed: {e}"))?;

            let status = chat_response.status();
            if !status.is_success() {
                let error_body = chat_response
                    .text()
                    .await
                    .unwrap_or_else(|_| "<failed to read response body>".to_string());
                return Err(format!(
                    "Groq Chat Completions API returned status code {status}. Response body: {error_body}"
                ));
            }

            let chat_resp: ChatCompletionResponse = chat_response
                .json()
                .await
                .map_err(|e| format!("Failed to parse Groq Chat Completion JSON response: {e}"))?;

            let cleaned_text = chat_resp
                .choices
                .and_then(|c| c.into_iter().next())
                .and_then(|c| c.message)
                .and_then(|m| m.content)
                .ok_or_else(|| "Groq Chat Completion response did not contain expected text content structure.".to_string())?;

            Ok(cleaned_text)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gemini_request_serialization() {
        let req = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![
                    GeminiPart {
                        inline_data: Some(GeminiInlineData {
                            mime_type: "audio/wav".to_string(),
                            data: "SGVsbG8=".to_string(),
                        }),
                        text: None,
                    },
                    GeminiPart {
                        inline_data: None,
                        text: Some("Test Prompt".to_string()),
                    },
                ],
            }],
        };

        let serialized = serde_json::to_string(&req).unwrap();
        assert!(serialized.contains(r#""mimeType":"audio/wav""#));
        assert!(serialized.contains(r#""data":"SGVsbG8=""#));
        assert!(serialized.contains(r#""text":"Test Prompt""#));
        assert!(!serialized.contains(r#""inlineData":null"#));
    }

    #[test]
    fn test_gemini_response_deserialization() {
        let json = r#"{
            "candidates": [
                {
                    "content": {
                        "parts": [
                            {
                                "text": "This is transcribed text."
                            }
                        ]
                    }
                }
            ]
        }"#;

        let resp: GeminiResponse = serde_json::from_str(json).unwrap();
        let text = resp
            .candidates
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.content)
            .and_then(|c| c.parts)
            .and_then(|p| p.into_iter().next())
            .and_then(|p| p.text);

        assert_eq!(text, Some("This is transcribed text.".to_string()));
    }

    #[test]
    fn test_openai_chat_deserialization() {
        let json = r#"{
            "choices": [
                {
                    "message": {
                        "role": "assistant",
                        "content": "Cleaned transcription text."
                    }
                }
            ]
        }"#;

        let resp: ChatCompletionResponse = serde_json::from_str(json).unwrap();
        let text = resp
            .choices
            .and_then(|c| c.into_iter().next())
            .and_then(|c| c.message)
            .and_then(|m| m.content);

        assert_eq!(text, Some("Cleaned transcription text.".to_string()));
    }
}
