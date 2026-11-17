// @ts-check
// Aura Minimalist Desktop Tray Menu Controller
import { getIcon, iconBody } from "./icon-system-api.js";
import { hydrateAppIcons } from "./icon-system-integration.js";

try {
  hydrateAppIcons();
} catch (err) {
  console.error("Icon hydration failed", err);
}

/**
 * @typedef {(cmd: string, args?: Record<string, unknown>) => Promise<unknown>} TauriInvoke
 * @typedef {<T = unknown>(event: string, handler: (event: { payload: T }) => void) => Promise<() => void> | void} TauriListen
 */

/**
 * @typedef {Object} TrayLocaleStrings
 * @property {string} settings
 * @property {string} recog
 * @property {string} lang
 * @property {string} cloud
 * @property {string} local
 * @property {string} gaming
 * @property {string} exit
 * @property {string} active
 * @property {string} inactive
 * @property {string} back
 * @property {string} opt_auto
 * @property {string} opt_layout
 * @property {string} badge_auto
 * @property {string} badge_layout
 */

/**
 * @typedef {Object} TrayLangOption
 * @property {string} code
 * @property {"opt_auto" | "opt_layout"} [isSpecial]
 * @property {string} [name]
 * @property {string} badge
 */

/**
 * @typedef {Object} TraySettingsSnapshot
 * @property {string} [ui_language]
 * @property {string} [language]
 * @property {string} [transcription_mode]
 * @property {boolean} [gaming_mode_enabled]
 */

const tauriWindow = /** @type {Window & typeof globalThis & { __TAURI__?: { core?: { invoke: TauriInvoke }, event?: { listen: TauriListen } } }} */ (window);
/** @type {TauriInvoke} */
const invoke = tauriWindow.__TAURI__?.core?.invoke || (async () => null);
/** @type {TauriListen} */
const listen = tauriWindow.__TAURI__?.event?.listen || (() => {});

/** @type {Record<string, TrayLocaleStrings>} */
const I18N = {
  ru: {
    settings: "Открыть настройки",
    recog: "Способ распознавания",
    lang: "Язык распознавания",
    cloud: "Облачный ИИ",
    local: "Локальный ИИ",
    gaming: "Игровой режим",
    exit: "Выход",
    active: "Активен",
    inactive: "Выкл",
    back: "Назад в меню",
    opt_auto: "Автоопределение",
    opt_layout: "По раскладке клавиатуры",
    badge_auto: "АВТО",
    badge_layout: "КЛАВ"
  },
  en: {
    settings: "Open settings",
    recog: "Recognition mode",
    lang: "Recognition language",
    cloud: "Cloud AI",
    local: "Local AI",
    gaming: "Gaming Mode",
    exit: "Quit",
    active: "Active",
    inactive: "Off",
    back: "Back to menu",
    opt_auto: "Auto-detect",
    opt_layout: "Follow keyboard layout",
    badge_auto: "AUTO",
    badge_layout: "KEYB"
  },
  de: {
    settings: "Einstellungen öffnen",
    recog: "Erkennungsmodus",
    lang: "Erkennungssprache",
    cloud: "Cloud-KI",
    local: "Lokale KI",
    gaming: "Spielemodus",
    exit: "Beenden",
    active: "Aktiv",
    inactive: "Aus",
    back: "Zurück zum Menü",
    opt_auto: "Automatisch",
    opt_layout: "Nach Tastaturlayout",
    badge_auto: "AUTO",
    badge_layout: "TAST"
  },
  es: {
    settings: "Abrir ajustes",
    recog: "Modo de reconocimiento",
    lang: "Idioma de reconocimiento",
    cloud: "IA en la nube",
    local: "IA local",
    gaming: "Modo de juego",
    exit: "Salir",
    active: "Activo",
    inactive: "Desactivado",
    back: "Volver al menú",
    opt_auto: "Detección automática",
    opt_layout: "Por distribución de teclado",
    badge_auto: "AUTO",
    badge_layout: "TECL"
  },
  fr: {
    settings: "Ouvrir les paramètres",
    recog: "Mode de reconnaissance",
    lang: "Langue de reconnaissance",
    cloud: "IA cloud",
    local: "IA locale",
    gaming: "Mode jeu",
    exit: "Quitter",
    active: "Actif",
    inactive: "Désactivé",
    back: "Retour au menu",
    opt_auto: "Détection automatique",
    opt_layout: "Selon la disposition",
    badge_auto: "AUTO",
    badge_layout: "CLAV"
  },
  it: {
    settings: "Apri impostazioni",
    recog: "Modalità di riconoscimento",
    lang: "Lingua di riconoscimento",
    cloud: "IA cloud",
    local: "IA locale",
    gaming: "Modalità gioco",
    exit: "Esci",
    active: "Attivo",
    inactive: "Disattivo",
    back: "Torna al menu",
    opt_auto: "Rilevamento automatico",
    opt_layout: "Da layout tastiera",
    badge_auto: "AUTO",
    badge_layout: "TAST"
  },
  zh: {
    settings: "打开设置",
    recog: "识别模式",
    lang: "识别语言",
    cloud: "云端 AI",
    local: "本地 AI",
    gaming: "游戏模式",
    exit: "退出",
    active: "已开启",
    inactive: "已关闭",
    back: "返回菜单",
    opt_auto: "自动检测",
    opt_layout: "按键盘布局",
    badge_auto: "自动",
    badge_layout: "键盘"
  },
  pt: {
    settings: "Abrir configurações",
    recog: "Modo de reconhecimento",
    lang: "Idioma de reconhecimento",
    cloud: "IA na nuvem",
    local: "IA local",
    gaming: "Modo de Jogo",
    exit: "Sair",
    active: "Ativo",
    inactive: "Desactivado",
    back: "Voltar ao menu",
    opt_auto: "Detecção automática",
    opt_layout: "Por layout do teclado",
    badge_auto: "AUTO",
    badge_layout: "TECL"
  },
  tr: {
    settings: "Ayarları aç",
    recog: "Tanıma modu",
    lang: "Tanıma dili",
    cloud: "Bulut AI",
    local: "Yerel AI",
    gaming: "Oyun Modu",
    exit: "Çıkış",
    active: "Aktif",
    inactive: "Kapalı",
    back: "Menüye dön",
    opt_auto: "Otomatik algılama",
    opt_layout: "Klavye düzenine göre",
    badge_auto: "OTO",
    badge_layout: "KLAV"
  }
};

const SUPPORTED_LANGS = [
  { code: "auto", isSpecial: "opt_auto", badge: "AUTO" },
  { code: "layout", isSpecial: "opt_layout", badge: "LAYOUT" },
  { code: "ru", name: "Русский", badge: "RU" },
  { code: "en", name: "English", badge: "EN" },
  { code: "de", name: "Deutsch", badge: "DE" },
  { code: "es", name: "Español", badge: "ES" },
  { code: "fr", name: "Français", badge: "FR" },
  { code: "it", name: "Italiano", badge: "IT" },
  { code: "zh", name: "中文", badge: "ZH" },
  { code: "pt", name: "Português", badge: "PT" },
  { code: "tr", name: "Türkçe", badge: "TR" },
  { code: "nl", name: "Nederlands", badge: "NL" }
];

const CLOUD_ICON_SVG = iconBody("cloud");
const LOCAL_ICON_SVG = iconBody("device");

/** @type {TraySettingsSnapshot | null} */
let currentSettings = null;
/** @type {TraySettingsSnapshot | null} */
let lastConfirmedSettings = null;
/** @type {ReturnType<typeof setTimeout> | null} */
let switchViewTimer = null;
/** @type {string | null} */
let lastSubmenuTriggerId = null;

/**
 * @param {TraySettingsSnapshot | null | undefined} settings
 * @returns {TraySettingsSnapshot | null}
 */
function cloneSettingsSnapshot(settings) {
  if (!settings || typeof settings !== "object") return null;
  if (typeof structuredClone === "function") {
    return structuredClone(settings);
  }
  return JSON.parse(JSON.stringify(settings));
}

function clearSwitchViewTimer() {
  if (switchViewTimer !== null) {
    clearTimeout(switchViewTimer);
    switchViewTimer = null;
  }
}

/**
 * @param {boolean} [restoreFocus]
 */
function scheduleSwitchToMain(restoreFocus = false) {
  clearSwitchViewTimer();
  switchViewTimer = setTimeout(() => {
    switchViewTimer = null;
    switchView("main", { restoreFocus });
  }, 180);
}

/**
 * @param {string} lang
 */
function applyTranslations(lang) {
  const effectiveLang = I18N[lang] ? lang : "ru";
  if (document.documentElement) {
    document.documentElement.lang = effectiveLang;
  }
  const t = I18N[effectiveLang] || I18N.ru;
  /**
   * @param {string} id
   * @param {string} text
   */
  const setEl = (id, text) => {
    const el = document.getElementById(id);
    if (el) el.textContent = text;
  };
  setEl("label-settings", t.settings);
  setEl("label-recog", t.recog);
  setEl("label-recog-sub", t.recog);
  setEl("label-lang", t.lang);
  setEl("label-lang-sub", t.lang);
  setEl("label-cloud", t.cloud);
  setEl("label-local", t.local);
  setEl("label-gaming", t.gaming);
  setEl("label-exit", t.exit);
  const btnBackHeader = document.getElementById("btn-back-header");
  if (btnBackHeader) btnBackHeader.setAttribute("title", t.back);
  const btnBackLangHeader = document.getElementById("btn-back-lang-header");
  if (btnBackLangHeader) btnBackLangHeader.setAttribute("title", t.back);

  updateLanguageBadge((currentSettings && currentSettings.language) || "ru");
  renderLanguageList();
  if (currentSettings) {
    updateEngineSelection(currentSettings.transcription_mode || "local");
  }
}

/**
 * @param {string} [mode]
 */
function updateEngineSelection(mode) {
  const isCloud = mode === "cloud";
  const subCloud = document.getElementById("sub-cloud");
  const subLocal = document.getElementById("sub-local");
  if (subCloud) {
    subCloud.classList.toggle("is-selected", isCloud);
    subCloud.setAttribute("aria-checked", isCloud ? "true" : "false");
  }
  if (subLocal) {
    subLocal.classList.toggle("is-selected", !isCloud);
    subLocal.setAttribute("aria-checked", !isCloud ? "true" : "false");
  }

  const statusIcon = document.getElementById("recog-status-icon");
  if (statusIcon) {
    const lang = (currentSettings && currentSettings.ui_language) || "ru";
    const t = I18N[lang] || I18N.ru;
    statusIcon.innerHTML = isCloud ? CLOUD_ICON_SVG : LOCAL_ICON_SVG;
    statusIcon.setAttribute("title", isCloud ? t.cloud : t.local);
  }
}

/**
 * @param {string} [code]
 */
function updateLanguageBadge(code) {
  const badge = document.getElementById("lang-status-badge");
  if (!badge) return;
  const lang = (currentSettings && currentSettings.ui_language) || "ru";
  const t = I18N[lang] || I18N.ru;

  if (code === "auto") {
    const label = t.opt_auto || t.badge_auto || "Auto";
    badge.classList.add("is-icon");
    badge.setAttribute("aria-label", label);
    badge.setAttribute("title", label);
    badge.innerHTML = getIcon("globe", { size: 14 });
  } else if (code === "layout") {
    const label = t.opt_layout || t.badge_layout || "Layout";
    badge.classList.add("is-icon");
    badge.setAttribute("aria-label", label);
    badge.setAttribute("title", label);
    badge.innerHTML = getIcon("keyboard", { size: 14 });
  } else {
    badge.classList.remove("is-icon");
    badge.removeAttribute("aria-label");
    badge.removeAttribute("title");
    const found = SUPPORTED_LANGS.find(l => l.code === code);
    badge.textContent = found ? found.badge : (code ? code.toUpperCase() : "RU");
  }
}

function renderLanguageList() {
  const container = document.getElementById("lang-list-container");
  if (!container) return;

  const currentLang = (currentSettings && currentSettings.language) || "ru";
  const uiLang = (currentSettings && currentSettings.ui_language) || "ru";
  const t = I18N[uiLang] || I18N.ru;

  container.innerHTML = "";
  const fragment = document.createDocumentFragment();

  SUPPORTED_LANGS.forEach(langItem => {
    const itemEl = document.createElement("div");
    itemEl.className = "lang-item";
    const isSelected = langItem.code === currentLang;
    if (isSelected) {
      itemEl.classList.add("is-selected");
    }
    itemEl.setAttribute("role", "menuitemradio");
    itemEl.setAttribute("aria-checked", isSelected ? "true" : "false");
    itemEl.setAttribute("tabindex", "0");

    const labelText =
      (langItem.isSpecial
        ? t[/** @type {"opt_auto" | "opt_layout"} */ (langItem.isSpecial)]
        : langItem.name) ||
      langItem.name ||
      "";
    let codeBadge = "";
    if (langItem.code === "auto") {
      codeBadge = `<span class="lang-code lang-code-icon" aria-hidden="true">${getIcon("globe", { size: 14 })}</span>`;
    } else if (langItem.code === "layout") {
      codeBadge = `<span class="lang-code lang-code-icon" aria-hidden="true">${getIcon("keyboard", { size: 14 })}</span>`;
    } else {
      const langCode = langItem.isSpecial ? "" : String(langItem.code || "").toUpperCase();
      codeBadge = `<span class="lang-code" aria-hidden="true">${langCode}</span>`;
    }

    itemEl.innerHTML = `
      ${codeBadge}
      <span class="menu-text">${labelText}</span>
      ${getIcon("check", { size: 13, className: "check-mark" })}
    `;

    itemEl.addEventListener("click", async (e) => {
      e.stopPropagation();
      const hadKeyboardFocus = document.activeElement === itemEl;
      if (lastConfirmedSettings || currentSettings) {
        const base = cloneSettingsSnapshot(lastConfirmedSettings || currentSettings);
        const nextSettings = { ...(base || {}), language: langItem.code };
        currentSettings = nextSettings;
        updateLanguageBadge(langItem.code);
        renderLanguageList();
        try {
          await invoke("set_recognition_language", { language: langItem.code });
          lastConfirmedSettings = cloneSettingsSnapshot(nextSettings);
          scheduleSwitchToMain(hadKeyboardFocus);
        } catch (err) {
          console.error("Failed to save language setting:", err);
          currentSettings = cloneSettingsSnapshot(lastConfirmedSettings) || base;
          if (currentSettings) {
            updateLanguageBadge(currentSettings.language);
            renderLanguageList();
          }
        }
      }
    });

    fragment.appendChild(itemEl);
  });

  container.appendChild(fragment);
}

/**
 * @param {unknown} isActive
 */
function updateGamingState(isActive) {
  const isGaming = !!isActive;
  const item = document.getElementById("item-gaming");
  if (item) {
    item.classList.toggle("gaming-active", isGaming);
    item.setAttribute("aria-checked", isGaming ? "true" : "false");
  }
}

/**
 * @param {"main" | "recog" | "lang"} viewName
 * @param {{ restoreFocus?: boolean }} [options]
 */
function switchView(viewName, { restoreFocus = false } = {}) {
  clearSwitchViewTimer();
  const mainView = document.getElementById("view-main");
  const recogView = document.getElementById("view-recog");
  const langView = document.getElementById("view-lang");
  if (!mainView || !recogView || !langView) return;

  const trayRoot = document.getElementById("tray-menu");
  const focusWasInsideMenu = Boolean(
    restoreFocus || (trayRoot && document.activeElement && trayRoot.contains(document.activeElement))
  );

  mainView.classList.remove("active");
  recogView.classList.remove("active");
  langView.classList.remove("active");

  if (viewName === "recog") {
    lastSubmenuTriggerId = "item-recog-trigger";
    recogView.classList.add("active");
    if (focusWasInsideMenu) {
      const target = /** @type {HTMLElement | null} */ (recogView.querySelector(".submenu-item.is-selected"))
        || document.getElementById("btn-back-header");
      target?.focus();
    }
  } else if (viewName === "lang") {
    lastSubmenuTriggerId = "item-lang-trigger";
    langView.classList.add("active");
    renderLanguageList();
    if (focusWasInsideMenu) {
      const target = /** @type {HTMLElement | null} */ (langView.querySelector(".lang-item.is-selected"))
        || document.getElementById("btn-back-lang-header");
      if (target) {
        target.focus();
        target.scrollIntoView?.({ block: "nearest" });
      }
    }
  } else {
    mainView.classList.add("active");
    if (focusWasInsideMenu) {
      const target = (lastSubmenuTriggerId && document.getElementById(lastSubmenuTriggerId))
        || document.getElementById("item-open-settings");
      target?.focus();
    }
  }
}

async function refresh() {
  try {
    clearSwitchViewTimer();
    switchView("main");
    const loaded = /** @type {TraySettingsSnapshot | null} */ (await invoke("get_settings"));
    if (loaded) {
      lastConfirmedSettings = cloneSettingsSnapshot(loaded);
      currentSettings = cloneSettingsSnapshot(loaded);
      applyTranslations((currentSettings && currentSettings.ui_language) || "ru");
      if (currentSettings) {
        updateEngineSelection(currentSettings.transcription_mode);
        updateLanguageBadge(currentSettings.language);
      }
    }
    const isGaming = await invoke("get_gaming_mode_state");
    updateGamingState(isGaming);
  } catch (err) {
    console.error("Failed to load settings in tray menu:", err);
  }
}

/**
 * @param {"cloud" | "local"} mode
 * @param {EventTarget | null} sourceEl
 */
async function applyTrayTranscriptionMode(mode, sourceEl) {
  const hadKeyboardFocus = Boolean(sourceEl && document.activeElement === sourceEl);
  if (!lastConfirmedSettings && !currentSettings) {
    scheduleSwitchToMain(hadKeyboardFocus);
    return;
  }
  const base = cloneSettingsSnapshot(lastConfirmedSettings || currentSettings);
  const nextSettings = { ...(base || {}), transcription_mode: mode };
  currentSettings = nextSettings;
  updateEngineSelection(mode);
  try {
    await invoke("set_transcription_mode", { mode });
    lastConfirmedSettings = cloneSettingsSnapshot(nextSettings);
    scheduleSwitchToMain(hadKeyboardFocus);
  } catch (err) {
    console.error(`Failed to save ${mode} setting:`, err);
    currentSettings = cloneSettingsSnapshot(lastConfirmedSettings) || base;
    if (currentSettings) {
      updateEngineSelection(currentSettings.transcription_mode);
    }
  }
}

// 1. Open Settings
document.getElementById("item-open-settings")?.addEventListener("click", async () => {
  try {
    await invoke("show_settings_window");
  } catch (err) {
    console.error("Failed to open settings window:", err);
  }
});

// 2. Recognition Submenu Navigation
document.getElementById("item-recog-trigger")?.addEventListener("click", () => {
  switchView("recog");
});

document.getElementById("btn-back-header")?.addEventListener("click", () => {
  switchView("main", { restoreFocus: true });
});

// 2b. Language Submenu Navigation
document.getElementById("item-lang-trigger")?.addEventListener("click", () => {
  switchView("lang");
});

document.getElementById("btn-back-lang-header")?.addEventListener("click", () => {
  switchView("main", { restoreFocus: true });
});

// 3. Engine Selection (Cloud)
document.getElementById("sub-cloud")?.addEventListener("click", async (e) => {
  e.stopPropagation();
  await applyTrayTranscriptionMode("cloud", e.currentTarget);
});

// 4. Engine Selection (Local)
document.getElementById("sub-local")?.addEventListener("click", async (e) => {
  e.stopPropagation();
  await applyTrayTranscriptionMode("local", e.currentTarget);
});

// 5. Gaming Mode Toggle
document.getElementById("item-gaming")?.addEventListener("click", async () => {
  try {
    const newState = await invoke("toggle_gaming_mode");
    const nextManual = !!newState;
    if (currentSettings) currentSettings.gaming_mode_enabled = nextManual;
    if (lastConfirmedSettings) lastConfirmedSettings.gaming_mode_enabled = nextManual;
    updateGamingState(nextManual);
  } catch (err) {
    console.error("Failed to toggle gaming mode:", err);
  }
});

// 6. Exit App
document.getElementById("item-exit")?.addEventListener("click", async () => {
  try {
    await invoke("exit_app");
  } catch (err) {
    console.error("Failed to exit app:", err);
  }
});

/**
 * @returns {HTMLElement[]}
 */
function getActiveViewFocusableItems() {
  const activeView = document.querySelector(".menu-view.active");
  if (!activeView) return [];
  return /** @type {HTMLElement[]} */ (Array.from(activeView.querySelectorAll('[tabindex="0"]')));
}

// Keyboard accessibility: roving ArrowUp/ArrowDown/Home/End + Enter/Space + Escape
window.addEventListener("keydown", (e) => {
  if (e.key === "ArrowDown" || e.key === "ArrowUp" || e.key === "Home" || e.key === "End") {
    const items = getActiveViewFocusableItems();
    if (items.length > 0) {
      e.preventDefault();
      const currentIdx = document.activeElement
        ? items.indexOf(/** @type {HTMLElement} */ (document.activeElement))
        : -1;
      let nextIdx = 0;
      if (e.key === "Home") {
        nextIdx = 0;
      } else if (e.key === "End") {
        nextIdx = items.length - 1;
      } else if (currentIdx === -1) {
        nextIdx = e.key === "ArrowUp" ? items.length - 1 : 0;
      } else {
        const delta = e.key === "ArrowDown" ? 1 : -1;
        nextIdx = (currentIdx + delta + items.length) % items.length;
      }
      const nextEl = items[nextIdx];
      if (nextEl) {
        nextEl.focus();
        nextEl.scrollIntoView?.({ block: "nearest" });
      }
      return;
    }
  }

  if (e.key === "Enter" || e.key === " ") {
    const active = /** @type {HTMLElement | null} */ (document.activeElement);
    if (
      active &&
      (active.getAttribute("role") === "button" ||
        active.getAttribute("role") === "switch" ||
        active.getAttribute("role") === "menuitemradio" ||
        active.classList.contains("lang-item") ||
        active.classList.contains("aura-menu-item") ||
        active.classList.contains("submenu-item") ||
        active.classList.contains("drilldown-header"))
    ) {
      e.preventDefault();
      active.click();
      return;
    }
  }

  if (e.key === "Escape") {
    const recogView = document.getElementById("view-recog");
    const langView = document.getElementById("view-lang");
    if ((recogView && recogView.classList.contains("active")) || (langView && langView.classList.contains("active"))) {
      switchView("main", { restoreFocus: true });
    } else {
      invoke("close_window");
    }
  }
});

// Close if clicking outside menu container on transparent backdrop
window.addEventListener("click", (e) => {
  const targetEl = /** @type {Element | null} */ (e.target);
  if (!targetEl || !targetEl.closest("#tray-menu")) {
    invoke("close_window");
  }
});

// Real-time synchronization
listen("gaming-mode-changed", /** @param {{ payload: unknown }} event */ async (event) => {
  updateGamingState(event.payload);
  try {
    const latest = /** @type {TraySettingsSnapshot | null} */ (await invoke("get_settings"));
    if (latest) {
      if (lastConfirmedSettings) {
        lastConfirmedSettings.gaming_mode_enabled = !!latest.gaming_mode_enabled;
      }
      if (currentSettings) {
        currentSettings.gaming_mode_enabled = !!latest.gaming_mode_enabled;
      }
    }
  } catch (_) {}
});

listen("settings-changed", () => {
  refresh();
});

// Initial load
refresh();
