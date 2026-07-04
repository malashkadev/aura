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
      if (tab.dataset.tab === "history") {
        renderHistory();
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

  // Engine change (Cloud vs Local) toggling card visibility
  const radioCloud = document.getElementById("radio-cloud");
  const radioLocal = document.getElementById("radio-local");
  const localModelCard = document.getElementById("card-local-model");
  const cloudModelCard = document.getElementById("card-cloud-api");

  function updateEngineUI() {
    if (radioLocal.checked) {
      localModelCard.style.display = "flex";
      cloudModelCard.style.display = "none";
    } else {
      localModelCard.style.display = "none";
      cloudModelCard.style.display = "flex";
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
  const selectModel = document.getElementById("select-model");
  const selectProvider = document.getElementById("select-provider");
  const selectHotkey = document.getElementById("select-hotkey");
  const selectLanguage = document.getElementById("select-language");
  const textareaDictionary = document.getElementById("textarea-dictionary");
  const checkboxToggle = document.getElementById("checkbox-toggle");
  const checkboxPunctuation = document.getElementById("checkbox-punctuation");
  const checkboxAutostart = document.getElementById("checkbox-autostart");
  const btnDownloadModel = document.getElementById("btn-download-model");
  const btnDeleteModel = document.getElementById("btn-delete-model");
  const btnSaveSettings = document.getElementById("btn-save-settings");
  const historyList = document.getElementById("history-list");
  const btnClearHistory = document.getElementById("btn-clear-history");
  
  let apiKeys = {
    gemini: "",
    openai: "",
    groq: ""
  };
  let previousSelProvider = selectProvider.value;
  
  const progressContainer = document.getElementById("progress-container");
  const progressStatus = document.getElementById("progress-status");
  const progressPercent = document.getElementById("progress-percent");
  const progressBarFill = document.getElementById("progress-bar-fill");
  const footerStatusText = document.getElementById("footer-status-text");

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
        
        selectModel.value = settings.model_name || "base";
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

        updateEngineUI();
        await refreshDownloadedModels();
        showStatus("Настройки загружены");
      }
    } catch (err) {
      console.error(err);
      showStatus(`Ошибка загрузки настроек: ${err}`, true);
    }
  }

  // Check which models are downloaded and update dropdown list options
  async function refreshDownloadedModels() {
    try {
      const downloaded = await invoke("get_downloaded_models");
      const modelOptions = selectModel.querySelectorAll("option");
      modelOptions.forEach(opt => {
        const val = opt.value;
        let baseText = opt.textContent;
        // Clean up any previously appended checkmarks or labels
        if (baseText.endsWith(" ✓")) {
          baseText = baseText.substring(0, baseText.length - 2);
        }
        if (baseText.endsWith(" [Скачана]")) {
          baseText = baseText.substring(0, baseText.length - 10);
        }
        if (baseText.startsWith("✓ ")) {
          baseText = baseText.substring(2);
        }
        
        if (downloaded.includes(val)) {
          opt.textContent = `✓ ${baseText}`;
          opt.classList.add("downloaded-option");
        } else {
          opt.textContent = baseText;
          opt.classList.remove("downloaded-option");
        }
      });
      await updateDownloadButtonState();
    } catch (err) {
      console.error("Failed to check downloaded models", err);
    }
  }

  // Update Download Button State based on model presence on disk
  async function updateDownloadButtonState() {
    try {
      const downloaded = await invoke("get_downloaded_models");
      const selectedModel = selectModel.value;
      const isDownloaded = downloaded.includes(selectedModel);
      
      if (isDownloaded) {
        btnDownloadModel.innerHTML = `
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="btn-icon"><polyline points="20 6 9 17 4 12"></polyline></svg>
          Модель скачана
        `;
        btnDownloadModel.classList.add("downloaded");
        if (btnDeleteModel) btnDeleteModel.style.display = "flex";
      } else {
        btnDownloadModel.innerHTML = `
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="btn-icon"><path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4"></path><polyline points="7 10 12 15 17 10"></polyline><line x1="12" y1="15" x2="12" y2="3"></line></svg>
          Скачать модель
        `;
        btnDownloadModel.classList.remove("downloaded");
        btnDownloadModel.disabled = false;
        if (btnDeleteModel) btnDeleteModel.style.display = "none";
      }
    } catch (err) {
      console.error("Failed to update download button state", err);
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
        model_name: selectModel.value,
        hotkey: selectHotkey ? selectHotkey.value : "Alt+V",
        streaming_enabled: checkboxStreaming ? checkboxStreaming.checked : false,
        toggle_enabled: checkboxToggle ? checkboxToggle.checked : false,
        language: selectLanguage ? selectLanguage.value : "auto",
        dictionary: textareaDictionary ? textareaDictionary.value : "",
        voice_punctuation: checkboxPunctuation ? checkboxPunctuation.checked : false,
        autostart: checkboxAutostart ? checkboxAutostart.checked : false
      };

      await invoke("set_settings", { settings });
      showStatus("Настройки успешно сохранены!");
      
      // Temporary success animation in footer status
      setTimeout(() => {
        showStatus("Готово");
      }, 3000);
    } catch (err) {
      console.error(err);
      showStatus(`Ошибка сохранения настроек: ${err}`, true);
    }
  }

  // Download Whisper Model
  async function downloadModel() {
    const modelName = selectModel.value;
    try {
      showStatus(`Запуск скачивания для модели '${modelName}'...`);
      btnDownloadModel.disabled = true;
      progressContainer.style.display = "flex";
      progressBarFill.style.width = "0%";
      progressPercent.textContent = "0%";
      progressStatus.textContent = "Подключение к Hugging Face...";
      
      await invoke("download_model_command", { modelName });
    } catch (err) {
      console.error(err);
      showStatus(`Ошибка скачивания: ${err}`, true);
      btnDownloadModel.disabled = false;
      progressStatus.textContent = "Произошла ошибка";
    }
  }

  // Delete Whisper Model from Local Disk
  async function deleteModel() {
    const modelName = selectModel.value;
    if (!confirm(`Вы действительно хотите удалить локальную модель '${modelName}'?`)) {
      return;
    }
    try {
      showStatus(`Удаление модели '${modelName}'...`);
      btnDeleteModel.disabled = true;
      await invoke("delete_model_command", { modelName });
      showStatus("Модель успешно удалена");
      btnDeleteModel.disabled = false;
      
      // Refresh list of downloaded models and button states
      await refreshDownloadedModels();
    } catch (err) {
      console.error(err);
      showStatus(`Ошибка удаления: ${err}`, true);
      btnDeleteModel.disabled = false;
    }
  }

  // Listen to model-download-progress events from Rust
  listen("model-download-progress", (event) => {
    const payload = event.payload;
    if (!payload) return;

    const percent = Math.round(payload.percentage);
    progressBarFill.style.width = `${percent}%`;
    progressPercent.textContent = `${percent}%`;
    
    if (payload.done) {
      progressStatus.textContent = `Модель '${payload.model}' готова`;
      showStatus(`Модель '${payload.model}' скачана!`);
      btnDownloadModel.disabled = false;
      refreshDownloadedModels();
      setTimeout(() => {
        progressContainer.style.display = "none";
      }, 4000);
    } else {
      const mbDownloaded = (payload.downloaded / 1024 / 1024).toFixed(1);
      const totalMb = payload.total ? (payload.total / 1024 / 1024).toFixed(1) : "?";
      progressStatus.textContent = `Скачивание: ${mbDownloaded} / ${totalMb} МБ`;
    }
  });

  // --- Transcription History ---

  function formatHistoryDate(timestampMs) {
    try {
      return new Date(timestampMs).toLocaleString("ru-RU", {
        day: "2-digit", month: "2-digit",
        hour: "2-digit", minute: "2-digit"
      });
    } catch {
      return "";
    }
  }

  async function renderHistory() {
    if (!historyList) return;
    try {
      const entries = await invoke("get_history");
      historyList.innerHTML = "";

      if (!entries || entries.length === 0) {
        const empty = document.createElement("p");
        empty.className = "history-empty";
        empty.textContent = "Пока нет ни одной записи.";
        historyList.appendChild(empty);
        return;
      }

      entries.forEach(entry => {
        const item = document.createElement("div");
        item.className = "history-item";

        const textEl = document.createElement("p");
        textEl.className = "history-text";
        textEl.textContent = entry.text;

        const metaEl = document.createElement("div");
        metaEl.className = "history-meta";

        const dateEl = document.createElement("span");
        dateEl.className = "history-date";
        const modeLabel = entry.mode === "local" ? "локально" : "облако";
        dateEl.textContent = `${formatHistoryDate(entry.timestamp_ms)} · ${modeLabel}`;

        const copyBtn = document.createElement("button");
        copyBtn.type = "button";
        copyBtn.className = "btn-copy";
        copyBtn.textContent = "Копировать";
        copyBtn.addEventListener("click", async () => {
          try {
            await invoke("copy_to_clipboard", { text: entry.text });
            copyBtn.textContent = "Скопировано ✓";
            setTimeout(() => { copyBtn.textContent = "Копировать"; }, 1500);
          } catch (err) {
            console.error("Failed to copy history entry", err);
          }
        });

        metaEl.appendChild(dateEl);
        metaEl.appendChild(copyBtn);
        item.appendChild(textEl);
        item.appendChild(metaEl);
        historyList.appendChild(item);
      });
    } catch (err) {
      console.error("Failed to load history", err);
    }
  }

  if (btnClearHistory) {
    btnClearHistory.addEventListener("click", async () => {
      try {
        await invoke("clear_history");
        renderHistory();
        showStatus("История очищена");
      } catch (err) {
        console.error(err);
        showStatus(`Ошибка очистки истории: ${err}`, true);
      }
    });
  }

  // Refresh the list live when a new transcription is saved
  listen("history-updated", () => {
    const historyPanel = document.getElementById("panel-history");
    if (historyPanel && historyPanel.style.display !== "none") {
      renderHistory();
    }
  });

  // Helpers
  function showStatus(msg, isError = false) {
    footerStatusText.textContent = msg;
    if (isError) {
      footerStatusText.style.color = "hsl(0, 85%, 65%)";
    } else {
      footerStatusText.style.color = "";
    }
  }

  // Bind Events
  btnSaveSettings.addEventListener("click", saveSettings);
  btnDownloadModel.addEventListener("click", downloadModel);
  if (btnDeleteModel) {
    btnDeleteModel.addEventListener("click", deleteModel);
  }
  selectProvider.addEventListener("change", () => {
    // 1. Save input to current provider
    apiKeys[previousSelProvider] = apiKeyInput.value;
    // 2. Switch provider
    previousSelProvider = selectProvider.value;
    // 3. Load input from new provider
    apiKeyInput.value = apiKeys[selectProvider.value] || "";
    updateApiKeyLink();
  });
  selectModel.addEventListener("change", updateDownloadButtonState);

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
