// Aura Minimalist Desktop Tray Menu Controller
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;

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
  },
  nl: {
    settings: "Instellingen openen",
    recog: "Herkenningsmodus",
    lang: "Herkenningstaal",
    cloud: "Cloud AI",
    local: "Lokale AI",
    gaming: "Gamingmodus",
    exit: "Afsluiten",
    active: "Actief",
    inactive: "Uit",
    back: "Terug naar menu",
    opt_auto: "Automatisch",
    opt_layout: "Volgens toetsenbordindeling",
    badge_auto: "AUTO",
    badge_layout: "TOETS"
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

const CLOUD_ICON_SVG = '<path d="M17.5 19H9a7 7 0 1 1 6.71-9h1.79a4.5 4.5 0 1 1 0 9Z"/>';
const LOCAL_ICON_SVG = '<rect width="20" height="14" x="2" y="3" rx="2"/><line x1="8" x2="16" y1="21" y2="21"/><line x1="12" x2="12" y1="17" y2="21"/>';

let currentSettings = null;

function applyTranslations(lang) {
  const t = I18N[lang] || I18N.ru;
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
  const toggleGaming = document.getElementById("toggle-gaming");
  if (toggleGaming) toggleGaming.setAttribute("aria-label", t.gaming);

  updateLanguageBadge((currentSettings && currentSettings.language) || "ru");
  renderLanguageList();
  if (currentSettings) {
    updateEngineSelection(currentSettings.transcription_mode || "local");
  }
}

function updateEngineSelection(mode) {
  const isCloud = mode === "cloud";
  const subCloud = document.getElementById("sub-cloud");
  const subLocal = document.getElementById("sub-local");
  if (subCloud) subCloud.classList.toggle("is-selected", isCloud);
  if (subLocal) subLocal.classList.toggle("is-selected", !isCloud);

  const statusIcon = document.getElementById("recog-status-icon");
  if (statusIcon) {
    const lang = (currentSettings && currentSettings.ui_language) || "ru";
    const t = I18N[lang] || I18N.ru;
    statusIcon.innerHTML = isCloud ? CLOUD_ICON_SVG : LOCAL_ICON_SVG;
    statusIcon.setAttribute("title", isCloud ? t.cloud : t.local);
  }
}

function updateLanguageBadge(code) {
  const badge = document.getElementById("lang-status-badge");
  if (!badge) return;
  const lang = (currentSettings && currentSettings.ui_language) || "ru";
  const t = I18N[lang] || I18N.ru;

  if (code === "auto") {
    badge.textContent = t.badge_auto || "AUTO";
  } else if (code === "layout") {
    badge.textContent = t.badge_layout || "КЛАВ";
  } else {
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
    if (langItem.code === currentLang) {
      itemEl.classList.add("is-selected");
    }
    itemEl.setAttribute("role", "button");
    itemEl.setAttribute("tabindex", "0");

    const labelText = langItem.isSpecial ? (t[langItem.isSpecial] || langItem.name) : langItem.name;

    itemEl.innerHTML = `
      <span class="menu-text">${labelText}</span>
      <svg class="check-mark" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
        <polyline points="20 6 9 17 4 12"/>
      </svg>
    `;

    itemEl.addEventListener("click", async (e) => {
      e.stopPropagation();
      if (currentSettings) {
        currentSettings.language = langItem.code;
        updateLanguageBadge(langItem.code);
        renderLanguageList();
        try {
          await invoke("set_settings", { settings: currentSettings });
        } catch (err) {
          console.error("Failed to save language setting:", err);
        }
      }
      setTimeout(() => switchView("main"), 180);
    });

    fragment.appendChild(itemEl);
  });

  container.appendChild(fragment);
}

function updateGamingState(isActive) {
  const isGaming = !!isActive;
  const item = document.getElementById("item-gaming");
  if (item) item.classList.toggle("gaming-active", isGaming);
}

function switchView(viewName) {
  const mainView = document.getElementById("view-main");
  const recogView = document.getElementById("view-recog");
  const langView = document.getElementById("view-lang");
  if (!mainView || !recogView || !langView) return;

  mainView.classList.remove("active");
  recogView.classList.remove("active");
  langView.classList.remove("active");

  if (viewName === "recog") {
    recogView.classList.add("active");
  } else if (viewName === "lang") {
    langView.classList.add("active");
    renderLanguageList();
  } else {
    mainView.classList.add("active");
  }
}

async function refresh() {
  try {
    switchView("main");
    currentSettings = await invoke("get_settings");
    if (currentSettings) {
      applyTranslations(currentSettings.ui_language || "ru");
      updateEngineSelection(currentSettings.transcription_mode);
      updateLanguageBadge(currentSettings.language);
    }
    const isGaming = await invoke("get_gaming_mode_state");
    updateGamingState(isGaming);
  } catch (err) {
    console.error("Failed to load settings in tray menu:", err);
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
  switchView("main");
});

// 2b. Language Submenu Navigation
document.getElementById("item-lang-trigger")?.addEventListener("click", () => {
  switchView("lang");
});

document.getElementById("btn-back-lang-header")?.addEventListener("click", () => {
  switchView("main");
});

// 3. Engine Selection (Cloud)
document.getElementById("sub-cloud")?.addEventListener("click", async (e) => {
  e.stopPropagation();
  updateEngineSelection("cloud");
  if (currentSettings) {
    currentSettings.transcription_mode = "cloud";
    try {
      await invoke("set_settings", { settings: currentSettings });
    } catch (err) {
      console.error("Failed to save cloud setting:", err);
    }
  }
  setTimeout(() => switchView("main"), 180);
});

// 4. Engine Selection (Local)
document.getElementById("sub-local")?.addEventListener("click", async (e) => {
  e.stopPropagation();
  updateEngineSelection("local");
  if (currentSettings) {
    currentSettings.transcription_mode = "local";
    try {
      await invoke("set_settings", { settings: currentSettings });
    } catch (err) {
      console.error("Failed to save local setting:", err);
    }
  }
  setTimeout(() => switchView("main"), 180);
});

// 5. Gaming Mode Toggle
document.getElementById("item-gaming")?.addEventListener("click", async () => {
  try {
    const newState = await invoke("toggle_gaming_mode");
    updateGamingState(newState);
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

// Keyboard accessibility: trigger click on Enter or Space for focused buttons
window.addEventListener("keydown", (e) => {
  if (e.key === "Enter" || e.key === " ") {
    const active = document.activeElement;
    if (active && (active.getAttribute("role") === "button" || active.classList.contains("lang-item") || active.classList.contains("aura-menu-item") || active.classList.contains("submenu-item") || active.classList.contains("drilldown-header"))) {
      e.preventDefault();
      active.click();
      return;
    }
  }

  if (e.key === "Escape") {
    const recogView = document.getElementById("view-recog");
    const langView = document.getElementById("view-lang");
    if ((recogView && recogView.classList.contains("active")) || (langView && langView.classList.contains("active"))) {
      switchView("main");
    } else {
      invoke("close_window");
    }
  }
});

// Close if clicking outside menu container on transparent backdrop
window.addEventListener("click", (e) => {
  if (!e.target.closest("#tray-menu")) {
    invoke("close_window");
  }
});

// Real-time synchronization
listen("gaming-mode-changed", (event) => {
  updateGamingState(event.payload);
});

listen("settings-changed", () => {
  refresh();
});

// Initial load
refresh();
