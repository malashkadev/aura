import { getIcon } from "./icon-system-api.js";

/**
 * Owns the model-card surface: installed-state rendering, download and delete
 * progress, display names, and the per-card delete affordances.
 *
 * Selection state stays in main.js: the module receives `isModelActive`,
 * `selectModelCard` and the selected-model getters, so the radio-group rules
 * keep one owner while the DOM lifecycle moves here.
 */
export function createModelCards({
  invoke,
  listen,
  showConfirm,
  showStatus,
  getDict,
  getTranslation,
  localizedBackendError,
  isModelActive,
  selectModelCard,
  getSelectedModel,
  getSelectedLlm,
  setSelectedLlm,
  notifySelectionChanged,
  onModelDeleted,
  onRefreshComplete,
}) {
  const modelCards = document.querySelectorAll(".model-card[data-model]");
  let downloadedModelList = [];
  const inFlightModelDownloads = new Set();

  function getModelDisplayName(model) {
    if (!model) return "";
    const card = document.querySelector(`.model-card[data-model="${model}"]`);
    if (card) {
      const nameEl = card.querySelector(".model-name");
      if (nameEl && nameEl.textContent) {
        return nameEl.textContent.trim();
      }
    }
    const names = {
      "tiny": "Whisper Tiny",
      "base": "Whisper Base",
      "small": "Whisper Small",
      "medium": "Whisper Medium",
      "large-v3-turbo-q5_0": "Whisper Large v3 Turbo",
      "parakeet-v3": "NVIDIA Parakeet v3",
      "qwen3-0.6b": "Qwen3 0.6B",
      "qwen3-1.7b": "Qwen3 1.7B",
      "qwen3-4b": "Qwen3 4B",
    };
    return names[model] || model;
  }

  function syncActions() {
    const dict = getDict();
    document.querySelectorAll(".model-card[data-model]").forEach(card => {
      const model = card.dataset.model;
      const button = card.querySelector(".btn-delete-card-model");
      if (!button) {
        return;
      }

      const active = isModelActive(model);
      const label = active
        ? (dict.model_status_active || "Используется")
        : (dict.model_action_delete || "Удалить");

      button.disabled = active;
      button.setAttribute("aria-disabled", active ? "true" : "false");
      button.title = label;
      button.setAttribute("aria-label", label);
    });
  }

  async function refresh() {
    try {
      const downloaded = await invoke("get_downloaded_models");
      downloadedModelList = Array.isArray(downloaded) ? downloaded : [];
      const dict = getDict();
      modelCards.forEach(card => {
        const model = card.dataset.model;
        if (inFlightModelDownloads.has(model)) {
          return;
        }
        const isDownloaded = downloadedModelList.includes(model);
        const actionEl = document.getElementById(`action-${model}`);
        if (!actionEl) {
          return;
        }

        const progressEl = document.getElementById(`progress-${model}`);
        if (progressEl) {
          const cancelBtn = progressEl.querySelector(".btn-cancel-download");
          if (cancelBtn) cancelBtn.remove();
          progressEl.style.display = "none";
        }
        actionEl.style.display = "flex";

        const isActiveModel = isModelActive(model);
        const deleteLabel = isActiveModel
          ? (dict.model_status_active || "Используется")
          : (dict.model_action_delete || "Удалить");

        if (isDownloaded) {
          actionEl.innerHTML = `
            <span class="status-ready-badge" title="${dict.model_status_ready || "Установлено"}" aria-label="${dict.model_status_ready || "Установлено"}" data-i18n-title="model_status_ready">
              ${getIcon("check", { size: 12, className: "status-ready-icon", strokeWidth: 2.2 })}
            </span>
            <button type="button" class="btn-delete-card-model" title="${deleteLabel}" aria-label="${deleteLabel}"${isActiveModel ? " disabled aria-disabled=\"true\"" : ""} data-model="${model}">
              ${getIcon("trash", { size: 14, className: "btn-icon" })}
            </button>
          `;
          actionEl.querySelector(".btn-delete-card-model").addEventListener("click", (e) => {
            e.stopPropagation();
            remove(model);
          });
        } else {
          actionEl.innerHTML = `
            <button type="button" class="btn-download-card-model" data-model="${model}">
              ${getIcon("download", { size: 14, className: "btn-icon" })}
              ${dict.model_action_download || "Скачать"}
            </button>
          `;
          actionEl.querySelector(".btn-download-card-model").addEventListener("click", (e) => {
            e.stopPropagation();
            download(model);
          });
        }
      });

      const downloadedLlm = downloadedModelList.filter(id => id.startsWith("qwen3-"));
      if (downloadedLlm.length === 0) {
        modelCards.forEach(c => {
          if (c.dataset.model.startsWith("qwen3-")) {
            c.classList.remove("active");
            c.setAttribute("aria-checked", "false");
          }
        });
      } else if (downloadedLlm.includes(getSelectedLlm())) {
        selectModelCard(getSelectedLlm(), false);
      } else {
        selectModelCard(downloadedLlm[0], false);
      }

      syncActions();
      onRefreshComplete();
    } catch (err) {
      console.error("Failed to check downloaded models", err);
    }
  }

  async function download(model) {
    if (inFlightModelDownloads.has(model)) {
      return;
    }
    inFlightModelDownloads.add(model);
    const actionEl = document.getElementById(`action-${model}`);
    const progressEl = document.getElementById(`progress-${model}`);
    const fillEl = document.getElementById(`fill-${model}`);
    const pctEl = document.getElementById(`pct-${model}`);
    const modelAtStart = getSelectedModel();
    const llmAtStart = getSelectedLlm();

    try {
      showStatus(getTranslation("model_downloading_pattern", { model: getModelDisplayName(model) }));

      if (actionEl) actionEl.style.display = "none";
      if (progressEl) {
        progressEl.classList.remove("installing");
        progressEl.style.display = "flex";
        const oldBtn = progressEl.querySelector(".btn-cancel-download");
        if (oldBtn) oldBtn.remove();
        const cancelBtn = document.createElement("button");
        cancelBtn.type = "button";
        cancelBtn.className = "btn-cancel-download";
        const cancelLabel = getTranslation("model_cancel_download") || "Отменить загрузку";
        cancelBtn.title = cancelLabel;
        cancelBtn.setAttribute("aria-label", cancelLabel);
        cancelBtn.innerHTML = getIcon("close", { size: 16 });
        cancelBtn.addEventListener("click", (e) => {
          e.stopPropagation();
          cancelBtn.disabled = true;
          invoke("cancel_model_download", { modelName: model }).catch((e2) => {
            console.error("Cancel model download error:", e2);
            cancelBtn.disabled = false;
          });
        });
        progressEl.appendChild(cancelBtn);
      }
      if (fillEl) {
        fillEl.style.width = "0%";
      }
      if (pctEl) {
        pctEl.textContent = "0%";
      }

      await invoke("download_model_command", { modelName: model });
      const isLlmCard = model.startsWith("qwen3-");
      const selectionUntouched = isLlmCard
        ? getSelectedLlm() === llmAtStart
        : getSelectedModel() === modelAtStart;
      if (selectionUntouched) {
        selectModelCard(model);
      }
    } catch (err) {
      console.error(err);
      const errStr = String(err).toLowerCase();
      if (errStr.includes("cancel")) {
        showStatus(getTranslation("model_download_cancelled") || "Загрузка отменена", "neutral");
      } else {
        showStatus(getTranslation("model_download_error_pattern", { err: localizedBackendError(err) }), true);
      }
    } finally {
      inFlightModelDownloads.delete(model);
      if (progressEl) {
        progressEl.classList.remove("installing");
        const cancelBtn = progressEl.querySelector(".btn-cancel-download");
        if (cancelBtn) cancelBtn.remove();
        progressEl.style.display = "none";
      }
      if (actionEl) {
        actionEl.style.display = "flex";
      }
      await refresh();
    }
  }

  async function remove(model) {
    const modelDisplayName = getModelDisplayName(model);
    if (isModelActive(model)) {
      showStatus(getTranslation("model_delete_active_error"), true);
      return;
    }
    const confirmTitle = getTranslation("delete_model_title");
    const confirmMsg = getTranslation("delete_model_confirm_pattern", { model: modelDisplayName });
    const confirmBtn = getTranslation("delete_model_btn");
    const cancelBtn = getTranslation("confirm_cancel");

    const confirmed = await showConfirm(confirmTitle, confirmMsg, confirmBtn, cancelBtn);
    if (!confirmed) {
      return;
    }
    try {
      showStatus(getTranslation("model_deleting_pattern", { model: modelDisplayName }));
      await invoke("delete_model_command", { modelName: model });

      showStatus(getTranslation("model_deleted_success"), "neutral");
      onModelDeleted(model);

      if (model.startsWith("qwen3-") && getSelectedLlm() === model) {
        try {
          const downloaded = await invoke("get_downloaded_models");
          const replacement = (Array.isArray(downloaded) ? downloaded : []).find(
            id => id.startsWith("qwen3-") && id !== model
          );
          if (replacement) {
            selectModelCard(replacement);
            notifySelectionChanged();
          } else {
            setSelectedLlm("qwen3-1.7b");
            notifySelectionChanged();
          }
        } catch (err) {
          console.error("Failed to reselect local edit model", err);
        }
      }
      await refresh();
    } catch (err) {
      console.error(err);
      showStatus(getTranslation("model_delete_error_pattern", { err: localizedBackendError(err) }), true);
    }
  }

  listen("model-download-progress", (event) => {
    const payload = event.payload;
    if (!payload) return;

    const model = payload.model;
    const percent = typeof payload.percentage === "number" ? Math.round(payload.percentage) : 0;

    const fillEl = document.getElementById(`fill-${model}`);
    const pctEl = document.getElementById(`pct-${model}`);
    const progressEl = document.getElementById(`progress-${model}`);
    const actionEl = document.getElementById(`action-${model}`);

    if (fillEl && pctEl) {
      if (payload.status === "installing" || (percent >= 100 && !payload.done)) {
        if (progressEl) progressEl.classList.add("installing");
        fillEl.style.width = "100%";
        const installText = getTranslation("gpu_status_installing") || "Установка...";
        pctEl.innerHTML = `<span class="spinner-inline" aria-hidden="true"></span> ${installText}`;
      } else {
        if (progressEl) progressEl.classList.remove("installing");
        fillEl.style.width = `${percent}%`;
        pctEl.textContent = `${percent}%`;
      }
    }

    if (payload.done) {
      const isInternalPayload = typeof model === "string" && model.startsWith("llm-runtime-");
      if (!isInternalPayload) {
        showStatus(getTranslation("model_downloaded_success_pattern", { model: getModelDisplayName(model) }), "success");
      }
      if (progressEl) {
        progressEl.classList.remove("installing");
        const cancelBtn = progressEl.querySelector(".btn-cancel-download");
        if (cancelBtn) cancelBtn.remove();
        progressEl.style.display = "none";
      }
      if (actionEl) actionEl.style.display = "flex";
      refresh();
    }
  });

  return {
    getDownloadedModels: () => downloadedModelList,
    isDownloaded: (model) => downloadedModelList.includes(model),
    getModelDisplayName,
    refresh,
    download,
    remove,
    syncActions,
  };
}
