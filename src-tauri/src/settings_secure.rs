use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, MutexGuard, OnceLock};
use tauri::Manager;
use zeroize::Zeroize;

use crate::secure_storage;

const ALLOWED_MODES: &[&str] = &["cloud", "local"];
const ALLOWED_PROVIDERS: &[&str] = &["gemini", "openai", "groq", "huggingface", "custom"];
const ALLOWED_MODELS: &[&str] = &[
    "tiny",
    "base",
    "small",
    "medium",
    "large-v3-turbo-q5_0",
    "large-v3-turbo",
    "parakeet-v3",
];
const ALLOWED_ENGINES: &[&str] = &["whisper", "parakeet"];
const ALLOWED_ACCELERATION: &[&str] = &["cpu", "cuda"];
const ALLOWED_LANGUAGES: &[&str] = &[
    "auto", "layout", "ru", "en", "de", "es", "fr", "it", "zh", "pt", "tr",
];
const ALLOWED_UI_LANGUAGES: &[&str] = &["ru", "en", "de", "es", "fr", "it", "zh", "pt", "tr"];
const ALLOWED_SOUND_THEMES: &[&str] = &["zen", "rhodes", "scifi", "classic", "bubble", "haptic"];
const MAX_DICTIONARY_CHARS: usize = 4096;
const MAX_HOTKEY_CHARS: usize = 64;

static SETTINGS_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

fn lock_settings() -> MutexGuard<'static, ()> {
    let mutex = SETTINGS_LOCK.get_or_init(|| Mutex::new(()));
    match mutex.lock() {
        Ok(guard) => guard,
        Err(poisoned) => {
            crate::logger::log(
                "ERROR",
                "Settings",
                None,
                "Recovering poisoned settings mutex",
            );
            poisoned.into_inner()
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Settings {
    #[serde(default = "default_ui_language")]
    pub ui_language: String,
    pub transcription_mode: String,
    pub api_provider: String,
    #[serde(default, skip_serializing)]
    pub api_key: String,
    #[serde(default, skip_serializing)]
    pub api_key_gemini: String,
    #[serde(default, skip_serializing)]
    pub api_key_openai: String,
    #[serde(default, skip_serializing)]
    pub api_key_groq: String,
    #[serde(default, skip_serializing)]
    pub api_key_huggingface: String,
    #[serde(default, skip_serializing)]
    pub api_key_custom: String,
    pub custom_api_url: String,
    pub custom_model_name: String,
    pub model_name: String,
    pub hotkey: String,
    pub streaming_enabled: bool,
    pub toggle_enabled: bool,
    pub language: String,
    pub dictionary: String,
    pub voice_punctuation: bool,
    pub autostart: bool,
    pub automatic_update_checks: bool,
    pub overlay_sounds: bool,
    pub overlay_sound_theme: String,
    pub overlay_sound_volume: f32,
    pub cloud_fallback_enabled: bool,
    pub local_engine: String,
    pub copy_context_on_start: bool,
    pub local_acceleration: String,
    pub log_speech_text: bool,
    pub overlay_show_timer: bool,
    pub whisper_keep_in_memory: bool,
    #[serde(default = "default_audio_device")]
    pub audio_input_device: String,
    #[serde(default = "default_vram_standby_timeout_mins")]
    pub vram_standby_timeout_mins: u32,
    #[serde(default)]
    pub gaming_mode_enabled: bool,
    #[serde(default)]
    pub gaming_mode_auto_detect: bool,
    #[serde(default)]
    pub text_replacements: Vec<TextReplacement>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct TextReplacement {
    pub trigger: String,
    pub replacement: String,
}

fn default_vram_standby_timeout_mins() -> u32 {
    30
}

fn default_audio_device() -> String {
    "default".to_string()
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            ui_language: default_ui_language(),
            transcription_mode: "cloud".to_string(),
            api_provider: "gemini".to_string(),
            api_key: String::new(),
            api_key_gemini: String::new(),
            api_key_openai: String::new(),
            api_key_groq: String::new(),
            api_key_huggingface: String::new(),
            api_key_custom: String::new(),
            custom_api_url: "https://api.deepinfra.com/v1/openai".to_string(),
            custom_model_name: "openai/whisper-large-v3-turbo".to_string(),
            model_name: "base".to_string(),
            hotkey: "Alt+V".to_string(),
            streaming_enabled: false,
            toggle_enabled: false,
            language: "auto".to_string(),
            dictionary: String::new(),
            voice_punctuation: false,
            autostart: false,
            automatic_update_checks: false,
            overlay_sounds: true,
            overlay_sound_theme: "zen".to_string(),
            overlay_sound_volume: 0.8,
            cloud_fallback_enabled: true,
            local_engine: "whisper".to_string(),
            copy_context_on_start: false,
            local_acceleration: "cpu".to_string(),
            log_speech_text: false,
            overlay_show_timer: true,
            whisper_keep_in_memory: true,
            audio_input_device: "default".to_string(),
            vram_standby_timeout_mins: 30,
            gaming_mode_enabled: false,
            gaming_mode_auto_detect: false,
            text_replacements: Vec::new(),
        }
    }
}

impl Settings {
    pub fn active_api_key(&self) -> &str {
        match self.api_provider.as_str() {
            "gemini" => &self.api_key_gemini,
            "openai" => &self.api_key_openai,
            "groq" => &self.api_key_groq,
            "huggingface" => &self.api_key_huggingface,
            "custom" => &self.api_key_custom,
            _ => "",
        }
    }

    pub fn sync_active_api_key(&mut self) {
        self.api_key = self.active_api_key().to_string();
    }

    pub fn validate(&self) -> Result<(), String> {
        validate_choice(
            "transcription mode",
            &self.transcription_mode,
            ALLOWED_MODES,
        )?;
        validate_choice("API provider", &self.api_provider, ALLOWED_PROVIDERS)?;
        validate_choice("model", &self.model_name, ALLOWED_MODELS)?;
        validate_choice("local engine", &self.local_engine, ALLOWED_ENGINES)?;
        validate_choice(
            "local acceleration",
            &self.local_acceleration,
            ALLOWED_ACCELERATION,
        )?;
        validate_choice("language", &self.language, ALLOWED_LANGUAGES)?;
        validate_choice(
            "interface language",
            &self.ui_language,
            ALLOWED_UI_LANGUAGES,
        )?;
        validate_choice(
            "sound theme",
            &self.overlay_sound_theme,
            ALLOWED_SOUND_THEMES,
        )?;

        let hotkey_len = self.hotkey.chars().count();
        if hotkey_len == 0 || hotkey_len > MAX_HOTKEY_CHARS {
            return Err("Hotkey must contain between 1 and 64 characters".to_string());
        }
        if self.hotkey.chars().any(|ch| ch.is_control()) {
            return Err("Hotkey must not contain control characters".to_string());
        }
        if self.dictionary.chars().count() > MAX_DICTIONARY_CHARS {
            return Err(format!(
                "Dictionary exceeds the {MAX_DICTIONARY_CHARS}-character limit"
            ));
        }
        if !self.overlay_sound_volume.is_finite()
            || !(0.0..=1.0).contains(&self.overlay_sound_volume)
        {
            return Err("Overlay sound volume must be between 0 and 1".to_string());
        }
        if self.local_engine == "parakeet" && self.model_name != "parakeet-v3" {
            return Err("Parakeet engine requires the parakeet-v3 model".to_string());
        }
        if self.local_engine == "whisper" && self.model_name == "parakeet-v3" {
            return Err("Whisper engine cannot use the parakeet-v3 model".to_string());
        }
        if self.audio_input_device.chars().count() > 128
            || self.audio_input_device.chars().any(|ch| ch.is_control())
        {
            return Err("Audio input device is invalid or too long".to_string());
        }
        if self.vram_standby_timeout_mins > 1440 {
            return Err("VRAM standby timeout cannot exceed 1440 minutes (24 hours)".to_string());
        }
        if self.text_replacements.len() > 500 {
            return Err("Text replacements count cannot exceed 500 entries".to_string());
        }
        for (idx, r) in self.text_replacements.iter().enumerate() {
            if r.trigger.trim().is_empty() {
                return Err(format!(
                    "Replacement trigger at index {idx} cannot be empty"
                ));
            }
            if r.trigger.chars().count() > 128 {
                return Err(format!(
                    "Replacement trigger '{}' exceeds 128 characters",
                    r.trigger
                ));
            }
            if r.replacement.chars().count() > 2048 {
                return Err(format!(
                    "Replacement value for '{}' exceeds 2048 characters",
                    r.trigger
                ));
            }
        }
        Ok(())
    }
}

fn ui_language_for_windows_lang_id(language_id: u16) -> &'static str {
    const PRIMARY_LANGUAGE_MASK: u16 = 0x03ff;
    match language_id & PRIMARY_LANGUAGE_MASK {
        0x0004 => "zh",
        0x0007 => "de",
        0x0009 => "en",
        0x000a => "es",
        0x000c => "fr",
        0x0010 => "it",
        0x0016 => "pt",
        0x0019 => "ru",
        0x001f => "tr",
        _ => "en",
    }
}

fn default_ui_language() -> String {
    #[cfg(target_os = "windows")]
    {
        let language_id = unsafe { windows_sys::Win32::Globalization::GetUserDefaultUILanguage() };
        ui_language_for_windows_lang_id(language_id).to_string()
    }
    #[cfg(not(target_os = "windows"))]
    {
        "en".to_string()
    }
}

fn validate_choice(label: &str, value: &str, allowed: &[&str]) -> Result<(), String> {
    if allowed.contains(&value) {
        Ok(())
    } else {
        Err(format!("Unsupported {label}: {value}"))
    }
}

#[derive(Serialize)]
pub struct SettingsView {
    #[serde(flatten)]
    pub settings: Settings,
    pub has_api_key_gemini: bool,
    pub has_api_key_openai: bool,
    pub has_api_key_groq: bool,
    pub has_api_key_huggingface: bool,
    pub has_api_key_custom: bool,
}

impl SettingsView {
    pub fn from_settings(settings: &Settings) -> Self {
        let mut view = settings.clone();
        // The view is serialized to the webview; keep a second in-memory copy
        // of the secrets only for as long as it takes to scrub it.
        view.api_key.zeroize();
        view.api_key_gemini.zeroize();
        view.api_key_openai.zeroize();
        view.api_key_groq.zeroize();
        view.api_key_huggingface.zeroize();
        view.api_key_custom.zeroize();
        Self {
            settings: view,
            has_api_key_gemini: !settings.api_key_gemini.is_empty(),
            has_api_key_openai: !settings.api_key_openai.is_empty(),
            has_api_key_groq: !settings.api_key_groq.is_empty(),
            has_api_key_huggingface: !settings.api_key_huggingface.is_empty(),
            has_api_key_custom: !settings.api_key_custom.is_empty(),
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
struct ProviderSecrets {
    gemini: String,
    openai: String,
    groq: String,
    #[serde(default)]
    huggingface: String,
    #[serde(default)]
    custom: String,
}

impl ProviderSecrets {
    fn from_settings(settings: &Settings) -> Self {
        Self {
            gemini: settings.api_key_gemini.trim().to_string(),
            openai: settings.api_key_openai.trim().to_string(),
            groq: settings.api_key_groq.trim().to_string(),
            huggingface: settings.api_key_huggingface.trim().to_string(),
            custom: settings.api_key_custom.trim().to_string(),
        }
    }

    fn apply_to(self, settings: &mut Settings) {
        settings.api_key_gemini = self.gemini;
        settings.api_key_openai = self.openai;
        settings.api_key_groq = self.groq;
        settings.api_key_huggingface = self.huggingface;
        settings.api_key_custom = self.custom;
        settings.sync_active_api_key();
    }

    fn is_empty(&self) -> bool {
        self.gemini.is_empty()
            && self.openai.is_empty()
            && self.groq.is_empty()
            && self.huggingface.is_empty()
            && self.custom.is_empty()
    }
}

pub fn get_settings_path<R: tauri::Runtime>(
    app_handle: &tauri::AppHandle<R>,
) -> Result<PathBuf, String> {
    let config_dir = app_handle
        .path()
        .app_config_dir()
        .map_err(|error| format!("Failed to get app config directory: {error}"))?;
    Ok(config_dir.join("settings.json"))
}

fn get_secrets_path<R: tauri::Runtime>(
    app_handle: &tauri::AppHandle<R>,
) -> Result<PathBuf, String> {
    let config_dir = app_handle
        .path()
        .app_config_dir()
        .map_err(|error| format!("Failed to get app config directory: {error}"))?;
    Ok(config_dir.join("secrets.dpapi"))
}

fn load_config(path: &Path) -> Result<Settings, String> {
    if !path.exists() {
        return Ok(Settings::default());
    }
    let content = fs::read_to_string(path)
        .map_err(|error| format!("Failed to read settings file: {error}"))?;
    serde_json::from_str(&content).map_err(|error| {
        let backup = path.with_extension(format!("corrupt-{}", timestamp_millis()));
        let _ = fs::rename(path, &backup);
        format!(
            "Settings file was invalid and moved to {}: {error}",
            backup.display()
        )
    })
}

fn unreadable_secrets_backup_path(path: &Path) -> PathBuf {
    let file_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("secrets.dpapi");
    path.with_file_name(format!(
        "{file_name}.unreadable-{}-{}",
        timestamp_millis(),
        std::process::id()
    ))
}

fn recover_unreadable_secrets(path: &Path, error: &str) -> ProviderSecrets {
    let secrets = ProviderSecrets::default();
    let backup = unreadable_secrets_backup_path(path);

    match fs::rename(path, &backup) {
        Ok(()) => {
            crate::logger::log(
                "WARN",
                "Settings",
                None,
                &format!(
                    "Encrypted API key store could not be read and was preserved at {}: {error}. API keys were reset and must be entered again.",
                    backup.display()
                ),
            );
            if let Err(reset_error) = save_secrets(path, &secrets) {
                crate::logger::log(
                    "ERROR",
                    "Settings",
                    None,
                    &format!(
                        "Could not initialize a replacement API key store after recovery: {reset_error}"
                    ),
                );
            }
        }
        Err(rename_error) => {
            crate::logger::log(
                "ERROR",
                "Settings",
                None,
                &format!(
                    "Encrypted API key store could not be read ({error}) or preserved as {} ({rename_error}). Continuing without API keys; the original file was left unchanged.",
                    backup.display()
                ),
            );
        }
    }

    secrets
}

fn load_secrets(path: &Path) -> ProviderSecrets {
    if !path.exists() {
        return ProviderSecrets::default();
    }

    let encrypted = match fs::read(path) {
        Ok(encrypted) => encrypted,
        Err(error) => {
            crate::logger::log(
                "ERROR",
                "Settings",
                None,
                &format!(
                    "Failed to read encrypted API keys: {error}. Continuing without API keys."
                ),
            );
            return ProviderSecrets::default();
        }
    };
    let mut plaintext = match secure_storage::unprotect_for_current_user(&encrypted) {
        Ok(plaintext) => plaintext,
        Err(error) => return recover_unreadable_secrets(path, &error),
    };
    let result = serde_json::from_slice(&plaintext);
    plaintext.zeroize();
    match result {
        Ok(secrets) => secrets,
        Err(error) => recover_unreadable_secrets(
            path,
            &format!("Encrypted API key store is invalid: {error}"),
        ),
    }
}
fn save_secrets(path: &Path, secrets: &ProviderSecrets) -> Result<(), String> {
    let mut plaintext = serde_json::to_vec(secrets)
        .map_err(|error| format!("Failed to serialize API keys: {error}"))?;
    let encrypted_result = secure_storage::protect_for_current_user(&plaintext);
    plaintext.zeroize();
    let mut encrypted = encrypted_result?;
    let result = secure_storage::atomic_write(path, &encrypted);
    encrypted.zeroize();
    result
}

fn save_config(path: &Path, settings: &Settings) -> Result<(), String> {
    let content = serde_json::to_vec_pretty(settings)
        .map_err(|error| format!("Failed to serialize settings: {error}"))?;
    secure_storage::atomic_write(path, &content)
}

fn timestamp_millis() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis())
        .unwrap_or_default()
}

fn load_settings_unlocked<R: tauri::Runtime>(
    app_handle: &tauri::AppHandle<R>,
) -> Result<Settings, String> {
    let config_path = get_settings_path(app_handle)?;
    let secrets_path = get_secrets_path(app_handle)?;
    let mut settings = load_config(&config_path)?;

    let migrated_unsupported_acceleration = settings.local_acceleration == "directml";
    if migrated_unsupported_acceleration {
        settings.local_acceleration = "cpu".to_string();
    }
    let config_contained_plaintext = !settings.api_key.is_empty()
        || !settings.api_key_gemini.is_empty()
        || !settings.api_key_openai.is_empty()
        || !settings.api_key_groq.is_empty()
        || !settings.api_key_huggingface.is_empty()
        || !settings.api_key_custom.is_empty();
    let legacy_secrets = ProviderSecrets {
        gemini: if settings.api_key_gemini.is_empty() && settings.api_provider == "gemini" {
            settings.api_key.clone()
        } else {
            settings.api_key_gemini.clone()
        },
        openai: if settings.api_key_openai.is_empty() && settings.api_provider == "openai" {
            settings.api_key.clone()
        } else {
            settings.api_key_openai.clone()
        },
        groq: if settings.api_key_groq.is_empty() && settings.api_provider == "groq" {
            settings.api_key.clone()
        } else {
            settings.api_key_groq.clone()
        },
        huggingface: if settings.api_key_huggingface.is_empty()
            && settings.api_provider == "huggingface"
        {
            settings.api_key.clone()
        } else {
            settings.api_key_huggingface.clone()
        },
        custom: if settings.api_key_custom.is_empty() && settings.api_provider == "custom" {
            settings.api_key.clone()
        } else {
            settings.api_key_custom.clone()
        },
    };

    let secrets = if secrets_path.exists() {
        load_secrets(&secrets_path)
    } else {
        if !legacy_secrets.is_empty() {
            save_secrets(&secrets_path, &legacy_secrets)?;
        }
        legacy_secrets
    };
    if config_contained_plaintext || migrated_unsupported_acceleration {
        save_config(&config_path, &settings)?;
    }
    secrets.apply_to(&mut settings);
    settings.validate()?;
    Ok(settings)
}

pub fn load_settings<R: tauri::Runtime>(
    app_handle: &tauri::AppHandle<R>,
) -> Result<Settings, String> {
    let _guard = lock_settings();
    load_settings_unlocked(app_handle)
}

pub fn save_settings<R: tauri::Runtime>(
    app_handle: &tauri::AppHandle<R>,
    settings: &Settings,
) -> Result<(), String> {
    let mut normalized = settings.clone();
    // Keys are never serialized into settings.json; scrub the transient copy
    // instead of letting it linger until the struct is dropped.
    normalized.api_key.zeroize();
    normalized.api_key_gemini.zeroize();
    normalized.api_key_openai.zeroize();
    normalized.api_key_groq.zeroize();
    normalized.api_key_huggingface.zeroize();
    normalized.api_key_custom.zeroize();
    normalized.dictionary = normalized.dictionary.trim().to_string();
    normalized.hotkey = normalized.hotkey.trim().to_string();
    normalized.custom_api_url = normalized.custom_api_url.trim().to_string();
    normalized.custom_model_name = normalized.custom_model_name.trim().to_string();
    normalized.validate()?;
    let _guard = lock_settings();

    // API keys are write-only and are updated solely by set_provider_key.
    // Saving a redacted SettingsView must never erase the encrypted store.
    save_config(&get_settings_path(app_handle)?, &normalized)
}

pub fn set_ui_language<R: tauri::Runtime>(
    app_handle: &tauri::AppHandle<R>,
    ui_language: &str,
) -> Result<(), String> {
    validate_choice("interface language", ui_language, ALLOWED_UI_LANGUAGES)?;
    let _guard = lock_settings();
    let mut settings = load_settings_unlocked(app_handle)?;
    settings.ui_language = ui_language.to_string();
    save_config(&get_settings_path(app_handle)?, &settings)
}

pub fn set_provider_key<R: tauri::Runtime>(
    app_handle: &tauri::AppHandle<R>,
    provider: &str,
    key: &str,
) -> Result<(), String> {
    validate_choice("API provider", provider, ALLOWED_PROVIDERS)?;
    if key.chars().count() > 1024 || key.chars().any(|ch| ch.is_control() && !ch.is_whitespace()) {
        return Err("API key is invalid or too long".to_string());
    }

    let _guard = lock_settings();
    let mut settings = load_settings_unlocked(app_handle)?;
    let normalized = key.trim().to_string();
    match provider {
        // Scrub the replaced secret's buffer before overwriting it.
        "gemini" => {
            settings.api_key_gemini.zeroize();
            settings.api_key_gemini = normalized;
        }
        "openai" => {
            settings.api_key_openai.zeroize();
            settings.api_key_openai = normalized;
        }
        "groq" => {
            settings.api_key_groq.zeroize();
            settings.api_key_groq = normalized;
        }
        "huggingface" => {
            settings.api_key_huggingface.zeroize();
            settings.api_key_huggingface = normalized;
        }
        "custom" => {
            settings.api_key_custom.zeroize();
            settings.api_key_custom = normalized;
        }
        _ => return Err(format!("Unsupported API provider: {provider}")),
    }
    settings.sync_active_api_key();
    let secrets = ProviderSecrets::from_settings(&settings);
    save_secrets(&get_secrets_path(app_handle)?, &secrets)?;
    save_config(&get_settings_path(app_handle)?, &settings)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_private_and_safe() {
        let settings = Settings::default();
        assert!(!settings.copy_context_on_start);
        assert_eq!(settings.local_acceleration, "cpu");
        assert!(!settings.log_speech_text);
        assert_eq!(settings.active_api_key(), "");
        settings.validate().unwrap();
    }

    #[test]
    fn windows_ui_language_uses_supported_locale_or_english_fallback() {
        assert_eq!(ui_language_for_windows_lang_id(0x0804), "zh");
        assert_eq!(ui_language_for_windows_lang_id(0x0407), "de");
        assert_eq!(ui_language_for_windows_lang_id(0x0409), "en");
        assert_eq!(ui_language_for_windows_lang_id(0x0c0a), "es");
        assert_eq!(ui_language_for_windows_lang_id(0x040c), "fr");
        assert_eq!(ui_language_for_windows_lang_id(0x0410), "it");
        assert_eq!(ui_language_for_windows_lang_id(0x0416), "pt");
        assert_eq!(ui_language_for_windows_lang_id(0x0419), "ru");
        assert_eq!(ui_language_for_windows_lang_id(0x0819), "ru");
        assert_eq!(ui_language_for_windows_lang_id(0x041f), "tr");
        assert_eq!(ui_language_for_windows_lang_id(0x0413), "en");
        assert_eq!(ui_language_for_windows_lang_id(0x0411), "en");
    }

    #[test]
    fn settings_view_never_serializes_secrets() {
        let settings = Settings {
            api_key: "legacy-secret".to_string(),
            api_key_gemini: "gemini-secret".to_string(),
            api_key_huggingface: "hf-secret".to_string(),
            api_key_custom: "custom-secret".to_string(),
            ..Settings::default()
        };
        let json = serde_json::to_string(&SettingsView::from_settings(&settings)).unwrap();
        assert!(!json.contains("legacy-secret"));
        assert!(!json.contains("gemini-secret"));
        assert!(!json.contains("hf-secret"));
        assert!(!json.contains("custom-secret"));
        assert!(json.contains("\"has_api_key_gemini\":true"));
        assert!(json.contains("\"has_api_key_huggingface\":true"));
        assert!(json.contains("\"has_api_key_custom\":true"));
    }

    #[test]
    fn new_providers_active_key_and_validation() {
        let mut settings = Settings {
            api_provider: "huggingface".to_string(),
            api_key_huggingface: "hf_test".to_string(),
            ..Settings::default()
        };
        assert_eq!(settings.active_api_key(), "hf_test");
        assert!(settings.validate().is_ok());

        settings.api_provider = "custom".to_string();
        settings.api_key_custom = "custom_token".to_string();
        assert_eq!(settings.active_api_key(), "custom_token");
        assert!(settings.validate().is_ok());
    }

    #[test]
    fn validation_rejects_unknown_values_and_oversized_dictionary() {
        let invalid_provider = Settings {
            api_provider: "unknown".to_string(),
            ..Settings::default()
        };
        assert!(invalid_provider.validate().is_err());

        let invalid_dictionary = Settings {
            dictionary: "x".repeat(MAX_DICTIONARY_CHARS + 1),
            ..Settings::default()
        };
        assert!(invalid_dictionary.validate().is_err());
    }

    #[test]
    fn engine_and_model_must_match() {
        let invalid = Settings {
            local_engine: "parakeet".to_string(),
            model_name: "base".to_string(),
            ..Settings::default()
        };
        assert!(invalid.validate().is_err());
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn unreadable_secret_store_is_preserved_and_reinitialized() {
        let directory = std::env::temp_dir().join(format!(
            "aura-secrets-recovery-{}-{}",
            std::process::id(),
            timestamp_millis()
        ));
        let path = directory.join("secrets.dpapi");
        let corrupted = b"AURA-DPAPI-1\0invalid-dpapi-payload";
        fs::create_dir_all(&directory).unwrap();
        fs::write(&path, corrupted).unwrap();

        let secrets = load_secrets(&path);

        assert!(secrets.is_empty());
        let backups: Vec<PathBuf> = fs::read_dir(&directory)
            .unwrap()
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|candidate| {
                candidate
                    .file_name()
                    .and_then(|name| name.to_str())
                    .is_some_and(|name| name.starts_with("secrets.dpapi.unreadable-"))
            })
            .collect();
        assert_eq!(backups.len(), 1);
        assert_eq!(fs::read(&backups[0]).unwrap(), corrupted);

        let encrypted = fs::read(&path).unwrap();
        let mut plaintext = secure_storage::unprotect_for_current_user(&encrypted).unwrap();
        let replacement: ProviderSecrets = serde_json::from_slice(&plaintext).unwrap();
        plaintext.zeroize();
        assert!(replacement.is_empty());

        let _ = fs::remove_dir_all(directory);
    }
}
