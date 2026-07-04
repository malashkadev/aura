// Retrieve Tauri APIs from window.__TAURI__
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

document.addEventListener("DOMContentLoaded", () => {
  // Navigation Tabs
  const tabs = document.querySelectorAll(".nav-tab");
  const panels = document.querySelectorAll(".tab-panel");
  
  tabs.forEach(tab => {
    tab.addEventListener("click", () => {
      tabs.forEach(t => {
        t.classList.remove("active");
        t.setAttribute("aria-selected", "false");
      });
      panels.forEach(p => p.style.display = "none");

      tab.classList.add("active");
      tab.setAttribute("aria-selected", "true");
      const targetPanel = document.getElementById(`panel-${tab.dataset.tab}`);
      if (targetPanel) {
        targetPanel.style.display = "flex";
      }
    });
  });

  // Toggle API Key visibility
  const apiKeyInput = document.getElementById("input-api-key");
  const toggleKeyBtn = document.getElementById("btn-toggle-key");
  toggleKeyBtn.addEventListener("click", () => {
    if (apiKeyInput.type === "password") {
      apiKeyInput.type = "text";
      toggleKeyBtn.classList.add("visible");
    } else {
      apiKeyInput.type = "password";
      toggleKeyBtn.classList.remove("visible");
    }
  });

  // Engine change (Cloud vs Local) toggling Whisper card visibility
  const radioCloud = document.getElementById("radio-cloud");
  const radioLocal = document.getElementById("radio-local");
  const localModelCard = document.getElementById("card-local-model");

  function updateEngineUI() {
    if (radioLocal.checked) {
      localModelCard.style.display = "flex";
    } else {
      localModelCard.style.display = "none";
    }
  }

  radioCloud.addEventListener("change", updateEngineUI);
  radioLocal.addEventListener("change", updateEngineUI);

  // Dynamic API Key Links
  const linkGetKey = document.getElementById("link-get-key");
  const providerLinks = {
    gemini: { url: "https://aistudio.google.com/", name: "Google AI Studio" },
    openai: { url: "https://platform.openai.com/api-keys", name: "OpenAI Platform" },
    groq: { url: "https://console.groq.com/keys", name: "Groq Console" }
  };
  function updateApiKeyLink() {
    const prov = selectProvider.value;
    const info = providerLinks[prov] || providerLinks.gemini;
    if (linkGetKey) {
      linkGetKey.href = info.url;
      linkGetKey.textContent = `Получить ключ на ${info.name}`;
    }
  }

  // Settings elements
  const selectProvider = document.getElementById("select-provider");
  const selectHotkey = document.getElementById("select-hotkey");
  const selectLanguage = document.getElementById("select-language");
  const textareaDictionary = document.getElementById("textarea-dictionary");
  const checkboxToggle = document.getElementById("checkbox-toggle");
  const checkboxPunctuation = document.getElementById("checkbox-punctuation");
  const checkboxAutostart = document.getElementById("checkbox-autostart");
  const btnSaveSettings = document.getElementById("btn-save-settings");
  
  const checkboxSounds = document.getElementById("checkbox-sounds");
  const selectSoundTheme = document.getElementById("select-sound-theme");

  function updateSoundUI() {
    const themeGroup = document.getElementById("sound-theme-group");
    if (themeGroup) {
      themeGroup.style.display = (checkboxSounds && checkboxSounds.checked) ? "flex" : "none";
    }
  }
  if (checkboxSounds) {
    checkboxSounds.addEventListener("change", updateSoundUI);
  }
  
  let apiKeys = {
    gemini: "",
    openai: "",
    groq: ""
  };
  let previousSelProvider = selectProvider.value;
  
  const footerStatusText = document.getElementById("footer-status-text");

  let selectedModelName = "base";

  let settingsModified = false;
  let isSettingsLoaded = false;

  function markSettingsModified() {
    if (!isSettingsLoaded) return;
    if (!settingsModified) {
      settingsModified = true;
      showStatus("Настройки изменены (не сохранены)", false, true);
    }
  }

  function bindSettingsChangeListeners() {
    const checkboxStreaming = document.getElementById("checkbox-streaming");
    const inputs = [
      radioCloud, radioLocal, selectProvider, apiKeyInput, selectHotkey,
      selectLanguage, textareaDictionary, checkboxToggle, checkboxPunctuation,
      checkboxAutostart, checkboxStreaming, checkboxSounds, selectSoundTheme
    ];
    inputs.forEach(input => {
      if (input) {
        input.addEventListener("change", markSettingsModified);
        input.addEventListener("input", markSettingsModified);
      }
    });
  }
  const modelCards = document.querySelectorAll(".model-card");

  modelCards.forEach(card => {
    card.addEventListener("click", (e) => {
      // Prevent selection trigger when clicking delete/download buttons inside the card
      if (e.target.closest(".btn-delete-card-model") || e.target.closest(".btn-download-card-model")) {
        return;
      }
      selectModelCard(card.dataset.model);
    });

    card.addEventListener("keydown", (e) => {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        selectModelCard(card.dataset.model);
      }
    });
  });

  function selectModelCard(model) {
    if (selectedModelName !== model) {
      selectedModelName = model;
      markSettingsModified();
    }
    modelCards.forEach(c => {
      const isCurrent = c.dataset.model === model;
      c.classList.toggle("active", isCurrent);
      c.setAttribute("aria-checked", isCurrent ? "true" : "false");
    });
  }

  // Load Settings from Backend
  async function loadSettings() {
    try {
      showStatus("Загрузка настроек...");
      const settings = await invoke("get_settings");
      
      if (settings) {
        if (settings.transcription_mode === "local") {
          radioLocal.checked = true;
        } else {
          radioCloud.checked = true;
        }
        
        selectModelCard(settings.model_name || "base");
        apiKeys.gemini = settings.api_key_gemini || "";
        apiKeys.openai = settings.api_key_openai || "";
        apiKeys.groq = settings.api_key_groq || "";
        
        selectProvider.value = settings.api_provider || "gemini";
        previousSelProvider = selectProvider.value;
        apiKeyInput.value = apiKeys[selectProvider.value] || "";
        updateApiKeyLink();
        if (selectHotkey) {
          selectHotkey.value = settings.hotkey || "Alt+V";
        }
        if (selectLanguage) {
          selectLanguage.value = settings.language || "auto";
        }
        if (textareaDictionary) {
          textareaDictionary.value = settings.dictionary || "";
        }
        if (checkboxToggle) {
          checkboxToggle.checked = !!settings.toggle_enabled;
        }
        if (checkboxPunctuation) {
          checkboxPunctuation.checked = !!settings.voice_punctuation;
        }
        if (checkboxAutostart) {
          checkboxAutostart.checked = !!settings.autostart;
        }

        const checkboxStreaming = document.getElementById("checkbox-streaming");
        if (checkboxStreaming) {
          checkboxStreaming.checked = !!settings.streaming_enabled;
        }

        if (checkboxSounds) {
          checkboxSounds.checked = settings.overlay_sounds !== false;
        }
        if (selectSoundTheme) {
          selectSoundTheme.value = settings.overlay_sound_theme || "zen";
        }
        updateSoundUI();

        updateEngineUI();
        await refreshDownloadedModels();
        
        isSettingsLoaded = true;
        settingsModified = false;
        showStatus("Настройки загружены");
        bindSettingsChangeListeners();
      }
    } catch (err) {
      console.error(err);
      showStatus(`Ошибка загрузки настроек: ${err}`, true);
    }
  }

  async function refreshDownloadedModels() {
    try {
      const downloaded = await invoke("get_downloaded_models");
      modelCards.forEach(card => {
        const model = card.dataset.model;
        const isDownloaded = downloaded.includes(model);
        const actionEl = document.getElementById(`action-${model}`);
        
        if (isDownloaded) {
          actionEl.innerHTML = `
            <span class="status-ready">Готова</span>
            <button type="button" class="btn-delete-card-model" title="Удалить модель" data-model="${model}">
              <svg class="btn-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="3 6 5 6 21 6"></polyline><path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6m3 0V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"></path><line x1="10" y1="11" x2="10" y2="17"></line><line x1="14" y1="11" x2="14" y2="17"></line></svg>
            </button>
          `;
          // Bind click to the delete button
          actionEl.querySelector(".btn-delete-card-model").addEventListener("click", () => deleteModelCard(model));
        } else {
          actionEl.innerHTML = `
            <button type="button" class="btn-download-card-model" data-model="${model}">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="btn-icon"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
              Скачать
            </button>
          `;
          // Bind click to the download button
          actionEl.querySelector(".btn-download-card-model").addEventListener("click", () => downloadModelCard(model));
        }
      });
    } catch (err) {
      console.error("Failed to check downloaded models", err);
    }
  }

  // Save Settings to Backend
  async function saveSettings() {
    try {
      showStatus("Сохранение настроек...");
      
      const checkboxStreaming = document.getElementById("checkbox-streaming");
      
      // Update apiKeys cache from active input first:
      apiKeys[selectProvider.value] = apiKeyInput.value;

      const settings = {
        transcription_mode: radioLocal.checked ? "local" : "cloud",
        api_provider: selectProvider.value,
        api_key: apiKeyInput.value,
        api_key_gemini: apiKeys.gemini,
        api_key_openai: apiKeys.openai,
        api_key_groq: apiKeys.groq,
        model_name: selectedModelName,
        hotkey: selectHotkey ? selectHotkey.value : "Alt+V",
        streaming_enabled: checkboxStreaming ? checkboxStreaming.checked : false,
        toggle_enabled: checkboxToggle ? checkboxToggle.checked : false,
        language: selectLanguage ? selectLanguage.value : "auto",
        dictionary: textareaDictionary ? textareaDictionary.value : "",
        voice_punctuation: checkboxPunctuation ? checkboxPunctuation.checked : false,
        autostart: checkboxAutostart ? checkboxAutostart.checked : false,
        overlay_sounds: checkboxSounds ? checkboxSounds.checked : true,
        overlay_sound_theme: selectSoundTheme ? selectSoundTheme.value : "zen"
      };

      await invoke("set_settings", { settings });
      settingsModified = false;
      showStatus("Настройки успешно сохранены!");
      
      // Temporary success animation in footer status
      setTimeout(() => {
        if (!settingsModified) {
          showStatus("Готово");
        }
      }, 3000);
    } catch (err) {
      console.error(err);
      showStatus(`Ошибка сохранения настроек: ${err}`, true);
    }
  }

  async function downloadModelCard(model) {
    try {
      showStatus(`Запуск скачивания для модели '${model}'...`);
      const actionEl = document.getElementById(`action-${model}`);
      const progressEl = document.getElementById(`progress-${model}`);
      const fillEl = document.getElementById(`fill-${model}`);
      const pctEl = document.getElementById(`pct-${model}`);

      // Hide actions, show progress
      actionEl.style.display = "none";
      progressEl.style.display = "flex";
      fillEl.style.width = "0%";
      pctEl.textContent = "0%";

      await invoke("download_model_command", { modelName: model });
    } catch (err) {
      console.error(err);
      showStatus(`Ошибка скачивания: ${err}`, true);
      refreshDownloadedModels();
    }
  }

  async function deleteModelCard(model) {
    const confirmed = await showConfirm(
      "Удаление модели",
      `Вы действительно хотите удалить локальную модель '${model}'?`,
      "Удалить",
      "Отмена"
    );
    if (!confirmed) {
      return;
    }
    try {
      showStatus(`Удаление модели '${model}'...`);
      await invoke("delete_model_command", { modelName: model });
      showStatus("Модель успешно удалена");
      await refreshDownloadedModels();
    } catch (err) {
      console.error(err);
      showStatus(`Ошибка удаления: ${err}`, true);
    }
  }

  // Listen to model-download-progress events from Rust
  listen("model-download-progress", (event) => {
    const payload = event.payload;
    if (!payload) return;

    const model = payload.model;
    const percent = Math.round(payload.percentage);
    
    const fillEl = document.getElementById(`fill-${model}`);
    const pctEl = document.getElementById(`pct-${model}`);
    const progressEl = document.getElementById(`progress-${model}`);
    const actionEl = document.getElementById(`action-${model}`);
    
    if (fillEl && pctEl) {
      fillEl.style.width = `${percent}%`;
      pctEl.textContent = `${percent}%`;
    }

    if (payload.done) {
      showStatus(`Модель '${model}' скачана!`);
      if (progressEl) progressEl.style.display = "none";
      if (actionEl) actionEl.style.display = "flex";
      refreshDownloadedModels();
    }
  });

  // --- Asynchronous Custom Confirm Dialog ---
  function showConfirm(title, message, confirmText = "ОК", cancelText = "Отмена") {
    return new Promise((resolve) => {
      const modal = document.getElementById("custom-confirm-modal");
      const titleEl = document.getElementById("confirm-modal-title");
      const msgEl = document.getElementById("confirm-modal-message");
      const btnOk = document.getElementById("btn-confirm-ok");
      const btnCancel = document.getElementById("btn-confirm-cancel");

      if (!modal) {
        resolve(false);
        return;
      }

      titleEl.textContent = title;
      msgEl.textContent = message;
      btnOk.textContent = confirmText;
      btnCancel.textContent = cancelText;

      modal.style.display = "flex";
      modal.offsetHeight; // force reflow
      modal.classList.add("active");

      function cleanUp(result) {
        modal.classList.remove("active");
        btnOk.removeEventListener("click", onOk);
        btnCancel.removeEventListener("click", onCancel);
        setTimeout(() => {
          modal.style.display = "none";
          resolve(result);
        }, 200);
      }

      function onOk() { cleanUp(true); }
      function onCancel() { cleanUp(false); }

      btnOk.addEventListener("click", onOk);
      btnCancel.addEventListener("click", onCancel);
    });
  }

  // Helpers
  function showStatus(msg, isError = false, isModified = false) {
    footerStatusText.textContent = msg;
    const footerStatus = footerStatusText.closest(".footer-status");
    
    if (footerStatus) {
      footerStatus.classList.remove("modified", "error", "success");
      if (isError) {
        footerStatus.classList.add("error");
        footerStatusText.style.color = "hsl(0, 85%, 65%)";
      } else if (isModified) {
        footerStatus.classList.add("modified");
        footerStatusText.style.color = "hsl(40, 95%, 55%)";
      } else {
        footerStatus.classList.add("success");
        footerStatusText.style.color = "";
      }
    }
  }

  // Bind Events
  btnSaveSettings.addEventListener("click", saveSettings);
  selectProvider.addEventListener("change", () => {
    // 1. Save input to current provider
    apiKeys[previousSelProvider] = apiKeyInput.value;
    // 2. Switch provider
    previousSelProvider = selectProvider.value;
    // 3. Load input from new provider
    apiKeyInput.value = apiKeys[selectProvider.value] || "";
    updateApiKeyLink();
  });

  // Window controls via Tauri IPC commands
  const btnWindowMinimize = document.getElementById("btn-window-minimize");
  const btnWindowClose = document.getElementById("btn-window-close");
  
  if (btnWindowMinimize) {
    btnWindowMinimize.addEventListener("click", () => invoke("minimize_window"));
  }
  if (btnWindowClose) {
    btnWindowClose.addEventListener("click", () => invoke("close_window"));
  }

  // Window dragging via mousedown on header (bypasses click-through/drag bugs in Webview2)
  const appHeader = document.querySelector(".app-header");
  if (appHeader) {
    appHeader.addEventListener("mousedown", (e) => {
      // Only trigger drag on left click and avoid dragging when clicking on control buttons
      if (e.button === 0 && !e.target.closest(".window-control-btn") && !e.target.closest("button")) {
        invoke("start_dragging_command");
      }
    });
  }

  // Initialize
  loadSettings();
});
