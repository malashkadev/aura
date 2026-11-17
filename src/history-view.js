// @ts-check

import { getIcon } from "./icon-system-api.js";

/**
 * @typedef {Object} HistoryEntry
 * @property {string} [text]
 * @property {number} [timestamp_ms]
 * @property {string} [mode]
 * @property {string | null} [engine]
 * @property {number | null} [processing_ms]
 */

/**
 * @typedef {Object} HistoryViewDeps
 * @property {(cmd: string, args?: Record<string, unknown>) => Promise<any>} invoke
 * @property {(title: string, message: string, confirmText?: string, cancelText?: string) => Promise<boolean>} showConfirm
 * @property {() => Record<string, string>} getDict
 * @property {() => string} getLanguage
 * @property {(filter: string) => void} [onFilterChange]
 */

/**
 * Owns the transcript history surface: rendering, day grouping, search,
 * filter chips, copy feedback, and the clear-history confirmation.
 *
 * The host supplies the Tauri bridge, the confirmation dialog and the current
 * locale dictionary so this module stays independent of the settings state.
 *
 * @param {HistoryViewDeps} deps
 */
export function createHistoryView({ invoke, showConfirm, getDict, getLanguage, onFilterChange = undefined }) {
  const historyContainer = document.getElementById("history-items-container");
  const btnClearHistory = document.getElementById("btn-clear-history");
  const inputHistorySearch = /** @type {HTMLInputElement | null} */ (document.getElementById("input-history-search"));
  const btnClearHistorySearch = document.getElementById("btn-clear-history-search");
  const historyFilterButtons = /** @type {NodeListOf<HTMLElement>} */ (document.querySelectorAll(".history-filter-btn"));

  /** @type {HistoryEntry[]} */
  let cachedHistoryList = [];
  let historySearchQuery = "";
  let historyActiveFilter = "all";

  /**
   * @param {string | undefined} text
   * @returns {string}
   */
  function escapeHtml(text) {
    return (text || "")
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;")
      .replace(/'/g, "&#039;");
  }

  /**
   * @param {number | undefined} timestampMs
   * @param {Record<string, string>} dict
   * @param {string} currentLanguage
   * @returns {{ key: string, title: string }}
   */
  function computeDayGroup(timestampMs, dict, currentLanguage) {
    const now = new Date();
    const startOfToday = new Date(now.getFullYear(), now.getMonth(), now.getDate()).getTime();
    const startOfYesterday = startOfToday - 86400000;
    const endOfToday = startOfToday + 86400000;
    if (!timestampMs) return { key: "today", title: dict.history_group_today || "Сегодня" };
    if (timestampMs >= startOfToday && timestampMs < endOfToday) {
      return { key: "today", title: dict.history_group_today || "Сегодня" };
    }
    if (timestampMs >= startOfYesterday && timestampMs < startOfToday) {
      return { key: "yesterday", title: dict.history_group_yesterday || "Вчера" };
    }
    const d = new Date(timestampMs);
    const isCurrentYear = d.getFullYear() === now.getFullYear();
    const dateTitle = d.toLocaleDateString(currentLanguage || "ru", {
      day: "numeric",
      month: "long",
      year: isCurrentYear ? undefined : "numeric"
    });
    const dateKey = `${d.getFullYear()}-${d.getMonth()}-${d.getDate()}`;
    return { key: dateKey, title: dateTitle };
  }

  /**
   * @param {number | null | undefined} ms
   * @param {Record<string, string>} dict
   * @returns {string}
   */
  function formatHistoryDuration(ms, dict) {
    if (!ms) return "";
    if (ms < 1000) return `${ms} ${dict.history_unit_ms || "ms"}`;
    const secs = ms >= 10000 ? Math.round(ms / 1000) : Math.round((ms / 1000) * 10) / 10;
    return `${secs} ${dict.history_unit_sec || "s"}`;
  }

  /**
   * @param {{ key: string, title: string }} dayGroup
   * @returns {HTMLDivElement}
   */
  function buildDayHeaderElement(dayGroup) {
    const headerEl = document.createElement("div");
    headerEl.className = "history-day-header";
    headerEl.dataset.dayKey = dayGroup.key;
    headerEl.innerHTML = `
      <span class="history-day-title">${escapeHtml(dayGroup.title)}</span>
      <span class="history-day-line"></span>
    `;
    return headerEl;
  }

  /**
   * @param {HistoryEntry} entry
   * @param {{ animate: boolean, dict: Record<string, string>, currentLanguage: string }} context
   * @returns {HTMLDivElement}
   */
  function buildHistoryItemElement(entry, { animate, dict, currentLanguage }) {
    const date = new Date(entry.timestamp_ms || Date.now());
    const displayTime = date.toLocaleTimeString(currentLanguage, { hour: '2-digit', minute: '2-digit', second: '2-digit' });

    const itemEl = document.createElement("div");
    itemEl.className = animate ? "history-item" : "history-item history-item-static";

    let badgeHtml;
    const engineLabel = entry.engine ? dict[`history_engine_${entry.engine}`] : undefined;
    const durationHtml = entry.processing_ms
      ? `<span class="history-item-duration">${formatHistoryDuration(entry.processing_ms, dict)}</span>`
      : "";
    if (engineLabel) {
      badgeHtml = `<span class="history-item-badge badge-local">${escapeHtml(engineLabel)}</span>${durationHtml}`;
    } else if (entry.mode === "cloud") {
      badgeHtml = `<span class="history-item-badge badge-cloud">${escapeHtml(dict.history_badge_cloud || "Cloud")}</span>${durationHtml}`;
    } else {
      badgeHtml = `<span class="history-item-badge badge-local">${escapeHtml(dict.history_badge_local || "Local")}</span>${durationHtml}`;
    }

    itemEl.innerHTML = `
      <div class="history-item-body">
        <div class="history-item-meta">
          <span class="history-item-time">${displayTime}</span>
          ${badgeHtml}
        </div>
        <div class="history-item-text">${escapeHtml(entry.text)}</div>
      </div>
      <button type="button" class="btn-copy-history" title="${escapeHtml(dict.history_copy_title || "Copy to clipboard")}" aria-label="${escapeHtml(dict.history_copy_title || "Copy to clipboard")}">
        ${getIcon("copy", { size: 16, className: "copy-icon" })}
        ${getIcon("check", { size: 16, className: "check-icon" })}
      </button>
    `;

    const btnCopy = /** @type {(HTMLButtonElement & { _copyTimeout?: ReturnType<typeof setTimeout> | null }) | null} */ (
      itemEl.querySelector(".btn-copy-history")
    );

    if (btnCopy) {
      btnCopy.addEventListener("click", async () => {
        try {
          await invoke("copy_to_clipboard", { text: entry.text || "" });

          btnCopy.classList.add("copied");

          if (btnCopy._copyTimeout) {
            clearTimeout(btnCopy._copyTimeout);
          }

          btnCopy._copyTimeout = setTimeout(() => {
            btnCopy.classList.remove("copied");
            btnCopy._copyTimeout = null;
          }, 1500);
        } catch (err) {
          console.error("Failed to copy", err);
        }
      });
    }

    return itemEl;
  }

  /**
   * @param {HistoryEntry[]} prev
   * @param {HistoryEntry[]} next
   * @returns {boolean}
   */
  function tryPrependSingleEntry(prev, next) {
    if (!historyContainer || prev.length === 0 || next.length < 2) return false;
    if (historyActiveFilter !== "all" || (historySearchQuery || "").trim() !== "") return false;
    const firstHeader = historyContainer.querySelector(".history-day-header");
    const existingItems = historyContainer.querySelectorAll(".history-item");
    if (!firstHeader || existingItems.length !== prev.length) return false;

    const isAppendedAtHead = next.length === prev.length + 1
      && JSON.stringify(next.slice(1)) === JSON.stringify(prev);
    const isCappedRollAtHead = next.length === prev.length
      && JSON.stringify(next.slice(1)) === JSON.stringify(prev.slice(0, -1));
    if (!isAppendedAtHead && !isCappedRollAtHead) return false;

    const dict = getDict();
    const currentLanguage = getLanguage();
    const newEntry = next[0];
    const newDay = computeDayGroup(newEntry.timestamp_ms, dict, currentLanguage);
    const prevFirstDay = computeDayGroup(prev[0].timestamp_ms, dict, currentLanguage);
    const newItemEl = buildHistoryItemElement(newEntry, { animate: true, dict, currentLanguage });

    if (newDay.key === prevFirstDay.key) {
      firstHeader.after(newItemEl);
    } else {
      const newHeaderEl = buildDayHeaderElement(newDay);
      historyContainer.prepend(newHeaderEl, newItemEl);
    }

    if (isCappedRollAtHead) {
      const lastItem = existingItems[existingItems.length - 1];
      if (lastItem) {
        const prevEl = lastItem.previousElementSibling;
        const nextEl = lastItem.nextElementSibling;
        lastItem.remove();
        if (
          prevEl &&
          prevEl.classList.contains("history-day-header") &&
          (!nextEl || nextEl.classList.contains("history-day-header"))
        ) {
          prevEl.remove();
        }
      }
    }
    return true;
  }

  /**
   * @param {{ animate?: boolean }} [options]
   */
  function renderHistoryItems(options = {}) {
    const { animate = true } = options;
    if (!historyContainer) return;
    const dict = getDict();
    const currentLanguage = getLanguage();

    if (!cachedHistoryList || cachedHistoryList.length === 0) {
      historyContainer.innerHTML = `<div class="history-empty-state" id="history-empty-text" data-i18n="history_empty">${dict.history_empty}</div>`;
      return;
    }

    const q = (historySearchQuery || "").trim().toLowerCase();
    const filtered = cachedHistoryList.filter(entry => {
      if (historyActiveFilter === "cloud" && entry.mode !== "cloud") return false;
      if (historyActiveFilter === "local" && entry.mode === "cloud") return false;

      if (q && !(entry.text || "").toLowerCase().includes(q)) {
        return false;
      }
      return true;
    });

    if (filtered.length === 0) {
      const isFiltered = Boolean(q || (historyActiveFilter && historyActiveFilter !== "all"));
      const emptyMsg = isFiltered ? (dict.history_search_empty || dict.history_empty) : dict.history_empty;
      historyContainer.innerHTML = `<div class="history-empty-state" id="history-empty-text">${escapeHtml(emptyMsg)}</div>`;
      return;
    }

    const fragment = document.createDocumentFragment();
    /** @type {string | null} */
    let lastDayKey = null;

    filtered.forEach(entry => {
      const dayGroup = computeDayGroup(entry.timestamp_ms, dict, currentLanguage);
      if (dayGroup.title && dayGroup.key !== lastDayKey) {
        lastDayKey = dayGroup.key;
        fragment.appendChild(buildDayHeaderElement(dayGroup));
      }

      const itemEl = buildHistoryItemElement(entry, { animate, dict, currentLanguage });
      fragment.appendChild(itemEl);
    });
    historyContainer.replaceChildren(fragment);
  }

  async function loadHistoryList(force = false) {
    if (!historyContainer) return;
    try {
      const next = (await invoke("get_history")) || [];
      const prev = cachedHistoryList;
      const changed = JSON.stringify(next) !== JSON.stringify(cachedHistoryList);
      cachedHistoryList = next;
      if (changed && !force && tryPrependSingleEntry(prev, next)) {
        return;
      }
      if (changed || force) {
        renderHistoryItems({ animate: changed && !force });
      }
    } catch (err) {
      console.error("Failed to load history", err);
    }
  }

  if (inputHistorySearch) {
    inputHistorySearch.addEventListener("input", () => {
      historySearchQuery = inputHistorySearch.value;
      if (btnClearHistorySearch) {
        btnClearHistorySearch.style.display = historySearchQuery ? "flex" : "none";
      }
      renderHistoryItems({ animate: false });
    });
  }

  if (btnClearHistorySearch) {
    btnClearHistorySearch.addEventListener("click", () => {
      if (inputHistorySearch) {
        inputHistorySearch.value = "";
        inputHistorySearch.focus();
      }
      historySearchQuery = "";
      btnClearHistorySearch.style.display = "none";
      renderHistoryItems({ animate: false });
    });
  }

  historyFilterButtons.forEach(btn => {
    btn.addEventListener("click", () => {
      historyFilterButtons.forEach(b => b.classList.remove("active"));
      btn.classList.add("active");
      historyActiveFilter = btn.dataset.filter || "all";
      onFilterChange?.(historyActiveFilter);
      renderHistoryItems({ animate: false });
    });
  });

  if (btnClearHistory) {
    btnClearHistory.addEventListener("click", async () => {
      const dict = getDict();
      const confirmed = await showConfirm(
        dict.confirm_clear_history_title,
        dict.confirm_clear_history_msg,
        dict.confirm_ok,
        dict.confirm_cancel
      );
      if (confirmed) {
        try {
          await invoke("clear_history");
          loadHistoryList();
        } catch (err) {
          console.error("Failed to clear history", err);
        }
      }
    });
  }

  return { loadHistoryList, renderHistoryItems };
}
