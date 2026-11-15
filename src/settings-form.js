/**
 * Reads the settings form into the payload sent to `set_settings`.
 *
 * Only DOM-backed fields live here. Selection state, API-key bookkeeping,
 * onboarding and command toggles are passed in by `main.js`, which remains
 * the orchestrator. The returned object must match the non-secret fields of
 * `settings_secure::Settings` one-to-one (guarded by a test).
 */
export function collectFormSettings({
  modelName,
  llmModel,
  uiLanguage,
  transcriptionMode,
  apiProvider,
  localAcceleration,
  textReplacements,
  completedOnboarding,
  disabledEditCommands,
}) {
  const byId = (id) => document.getElementById(id);
  const valueOf = (id, fallback) => {
    const element = byId(id);
    return element ? element.value : fallback;
  };
  const checkedOf = (id, fallback) => {
    const element = byId(id);
    return element ? element.checked : fallback;
  };

  return {
    transcription_mode: transcriptionMode,
    ui_language: uiLanguage,
    api_provider: apiProvider,
    custom_api_url: valueOf("input-custom-url", "https://api.deepinfra.com/v1/openai").trim(),
    custom_model_name: valueOf("input-custom-model", "openai/whisper-large-v3-turbo").trim(),
    model_name: modelName,
    hotkey: valueOf("input-hotkey", "Alt+V"),
    streaming_enabled: checkedOf("checkbox-streaming", false),
    toggle_enabled: checkedOf("checkbox-toggle", false),
    language: valueOf("select-language", "auto"),
    dictionary: valueOf("textarea-dictionary", ""),
    cloud_fallback_enabled: checkedOf("checkbox-cloud-fallback", true),
    autostart: checkedOf("checkbox-autostart", false),
    automatic_update_checks: checkedOf("checkbox-automatic-update-checks", false),
    local_engine: valueOf("select-local-engine", "whisper"),
    local_acceleration: localAcceleration,
    overlay_sounds: checkedOf("checkbox-sounds", true),
    overlay_sound_theme: valueOf("select-sound-theme", "zen"),
    overlay_sound_volume: (() => {
      const element = byId("range-sound-volume");
      return element ? parseFloat(element.value) / 100 : 0.8;
    })(),
    copy_context_on_start: checkedOf("checkbox-selection-edit-enabled", false),
    audio_input_device: valueOf("select-audio-device", "default"),
    log_speech_text: checkedOf("setting-log-speech-text", false),
    overlay_show_timer: checkedOf("checkbox-overlay-show-timer", true),
    gaming_mode_enabled: checkedOf("checkbox-gaming-mode", false),
    gaming_mode_auto_detect: checkedOf("checkbox-gaming-mode-auto-detect", false),
    vram_standby_timeout_mins: (() => {
      const element = byId("select-vram-standby");
      return element ? (parseInt(element.value, 10) || 0) : 30;
    })(),
    text_replacements: textReplacements,
    ai_edit_mode: checkedOf("checkbox-selection-edit-enabled", false)
      ? valueOf("select-ai-edit-mode", "auto")
      : "off",
    translate_default_target: valueOf("select-translate-target", "en"),
    llm_model: llmModel,
    has_completed_onboarding: completedOnboarding,
    disabled_edit_commands: disabledEditCommands,
    smart_context_enabled: checkedOf("checkbox-smart-context-enabled", true),
    smart_context_trailing_period: checkedOf("checkbox-smart-context-trailing-period", true),
  };
}
