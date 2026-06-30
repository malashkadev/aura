// Retrieve Tauri APIs from window.__TAURI__
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

document.addEventListener("DOMContentLoaded", () => {
  // Navigation Tabs
  const tabs = document.querySelectorAll(".nav-tab");
  const panels = document.querySelectorAll(".tab-panel");
  
  tabs.forEach(tab => {
    tab.addEventListener("click", () => {
      tabs.forEach(t => t.classList.remove("active"));
      panels.forEach(p => p.style.display = "none");
      
      tab.classList.add("active");
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

  // Settings elements
  const selectModel = document.getElementById("select-model");
  const selectProvider = document.getElementById("select-provider");
  const selectHotkey = document.getElementById("select-hotkey");
  const btnDownloadModel = document.getElementById("btn-download-model");
  const btnSaveSettings = document.getElementById("btn-save-settings");
  
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
        selectProvider.value = settings.api_provider || "gemini";
        apiKeyInput.value = settings.api_key || "";
        if (selectHotkey) {
          selectHotkey.value = settings.hotkey || "Alt+V";
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
        
        if (downloaded.includes(val)) {
          opt.textContent = `${baseText} ✓`;
        } else {
          opt.textContent = baseText;
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
      const settings = {
        transcription_mode: radioLocal.checked ? "local" : "cloud",
        api_provider: selectProvider.value,
        api_key: apiKeyInput.value,
        model_name: selectModel.value,
        hotkey: selectHotkey ? selectHotkey.value : "Alt+V",
        streaming_enabled: checkboxStreaming ? checkboxStreaming.checked : false
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

  // Initialize
  loadSettings();
});
