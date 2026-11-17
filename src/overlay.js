// @ts-check

import { createSoundSynth } from "./sound-themes.js";

/**
 * @typedef {Object} TauriOverlayBridge
 * @property {{ listen: (eventName: string, handler: (event: { payload: any }) => void) => Promise<() => void> | void }} [event]
 * @property {{ invoke: (cmd: string, args?: Record<string, unknown>) => Promise<any> }} [core]
 */

/**
 * @typedef {Object} OverlayPreferencesPayload
 * @property {string} [ui_language]
 * @property {boolean} [overlay_sounds]
 * @property {string} [overlay_sound_theme]
 * @property {number} [overlay_sound_volume]
 * @property {boolean} [overlay_show_timer]
 */

const tauriWindow = /** @type {Window & typeof globalThis & { __TAURI__?: TauriOverlayBridge }} */ (window);
const { listen } = (tauriWindow.__TAURI__ && tauriWindow.__TAURI__.event) || { listen: () => {} };

const bars = /** @type {NodeListOf<SVGElement>} */ (document.querySelectorAll(".sound-bar"));
const statusEl = /** @type {HTMLElement | null} */ (document.getElementById("overlay-status"));
const statusTextEl = /** @type {HTMLElement | null} */ (document.getElementById("overlay-status-text"));
const pill = /** @type {HTMLElement | null} */ (document.querySelector(".overlay-pill"));
const cardEl = /** @type {HTMLElement | null} */ (document.querySelector(".overlay-card"));
const auraSvg = document.getElementById("aura-svg");
const barsGroup = /** @type {HTMLElement | SVGElement | null} */ (document.getElementById("bars-group"));
let currentState = "idle";

let isEditContext = false;

const SUPPORTED_UI_LANGS = ["ru", "en", "de", "es", "fr", "it", "zh", "pt", "tr"];
const SUPPORTED_SOUND_THEMES = ["zen", "rhodes", "scifi", "classic", "bubble", "haptic"];

/** @returns {string} */
function resolveUiLang() {
  let lang = "ru";
  try {
    lang = localStorage.getItem("aura_ui_lang") || "ru";
  } catch (_) {}
  if (!SUPPORTED_UI_LANGS.includes(lang)) {
    lang = "en";
  }
  if (document.documentElement && document.documentElement.lang !== lang) {
    document.documentElement.lang = lang;
  }
  return lang;
}

resolveUiLang();

function updateStatusVisibility() {
  const hasText = statusTextEl
    ? Boolean(statusTextEl.textContent && statusTextEl.textContent.trim())
    : Boolean(statusEl && statusEl.textContent && statusEl.textContent.trim());
  if (statusEl) {
    statusEl.style.display = hasText ? "inline-flex" : "none";
  }
  if (document.body) {
    document.body.classList.toggle("has-status", hasText);
  }
}

/** @param {string} text */
function setOverlayText(text) {
  if (statusTextEl) {
    statusTextEl.textContent = text;
  } else if (statusEl) {
    statusEl.textContent = text;
  }
  updateStatusVisibility();
}

listen("selection-context-active", (event) => {
  const newEditContext = Boolean(event.payload);
  if (isEditContext === newEditContext) return;
  isEditContext = newEditContext;
  if (cardEl) {
    cardEl.classList.toggle("edit-context", isEditContext && currentState === "recording");
  }
  updateStatusVisibility();
});
// Monotonic counter advanced on every state event. The hide guard compares
// it so a hide request from an old session can never mask a brand-new
// recording (state strings alone cannot distinguish two sessions).
let stateCycle = 0;
/** @type {number | null} */
let recordStart = null;
let angle = 0;
// Pending auto-hide timer from a previous state (e.g. an error). Cleared when
// a newer session owns the overlay, so it can never hide a fresh recording.
/** @type {ReturnType<typeof setTimeout> | null} */
let hideTimerId = null;
/** @type {ReturnType<typeof setInterval> | null} */
let recordingTimerId = null;
/** @type {number | null} */
let rafId = null;
/** @type {ReturnType<typeof setTimeout> | null} */
let audioSuspendTimerId = null;

// Windows "Show animations" off: static bars, no pill transition.
const reduceMotionQuery = window.matchMedia("(prefers-reduced-motion: reduce)");

const overlaySynth = createSoundSynth({ initialVolume: 0.8 });
let soundsEnabled = true;
let soundTheme = "zen";
let soundVolume = 0.8;
let overlayShowTimer = true;

function cancelScheduledAudioSuspend() {
  if (audioSuspendTimerId !== null) {
    clearTimeout(audioSuspendTimerId);
    audioSuspendTimerId = null;
  }
}

function scheduleIdleAudioSuspend() {
  cancelScheduledAudioSuspend();
  audioSuspendTimerId = setTimeout(() => {
    audioSuspendTimerId = null;
    if (!pill || !pill.classList.contains("visible")) {
      overlaySynth.suspend();
    }
  }, 2200);
}

/** @param {OverlayPreferencesPayload | null | undefined} settings */
function applyActiveThemeSettings(settings) {
  if (!settings || typeof settings !== "object") {
    return;
  }
  if (settings.ui_language) {
    try {
      localStorage.setItem("aura_ui_lang", settings.ui_language);
    } catch (_) {}
    resolveUiLang();
  }
  soundsEnabled = settings.overlay_sounds !== false;
  const candidateTheme = settings.overlay_sound_theme || "zen";
  soundTheme = SUPPORTED_SOUND_THEMES.includes(candidateTheme) ? candidateTheme : "zen";
  soundVolume = typeof settings.overlay_sound_volume === "number" ? settings.overlay_sound_volume : 0.8;
  overlaySynth.setVolume(soundVolume);
  overlayShowTimer = settings.overlay_show_timer !== false;
  if (document.body) {
    document.body.classList.toggle("no-timer", !overlayShowTimer);
  }
  if (
    !overlayShowTimer &&
    !currentState.startsWith("error") &&
    !currentState.startsWith("notice:")
  ) {
    setOverlayText("");
  }
}

listen("overlay-preferences", (event) => {
  applyActiveThemeSettings(event.payload);
});

function playThemeStart() {
  if (!soundsEnabled) return;
  cancelScheduledAudioSuspend();
  overlaySynth.playStart(soundTheme);
}

function playThemeSuccess() {
  if (!soundsEnabled) return;
  cancelScheduledAudioSuspend();
  overlaySynth.playSuccess(soundTheme);
}

function playThemeError() {
  if (!soundsEnabled) return;
  cancelScheduledAudioSuspend();
  overlaySynth.playError(soundTheme);
}

const restHeight = 2.5;
const maxHeight = 16.0;

// Exact silence reference color (#722205) and Aura tactical orange (#ff4200)
const COLOR_SILENCE = { r: 114, g: 34, b: 5 };
const COLOR_SPEECH = { r: 255, g: 66, b: 0 };

/**
 * @param {number} energy
 * @returns {string}
 */
function getBarHeatColor(energy) {
  const clamped = Math.max(0, Math.min(1, energy));
  const r = Math.round(COLOR_SILENCE.r + (COLOR_SPEECH.r - COLOR_SILENCE.r) * clamped);
  const g = Math.round(COLOR_SILENCE.g + (COLOR_SPEECH.g - COLOR_SILENCE.g) * clamped);
  const b = Math.round(COLOR_SILENCE.b + (COLOR_SPEECH.b - COLOR_SILENCE.b) * clamped);
  return `rgb(${r}, ${g}, ${b})`;
}

// Track current and target heights for smooth linear interpolation (lerp)
const barStates = Array.from(bars).map(() => ({
  currentHeight: restHeight,
  targetHeight: restHeight
}));

// Reset all bars to default quiet state
function resetBars() {
  barStates.forEach(state => {
    state.targetHeight = restHeight;
    state.currentHeight = restHeight;
  });
  bars.forEach(bar => {
    bar.setAttribute("height", restHeight.toString());
    bar.setAttribute("y", (15.0 - restHeight / 2).toString());
    bar.setAttribute("rx", (restHeight / 2).toString());
    if (currentState === "recording") {
      bar.setAttribute("fill", "rgb(114, 34, 5)");
    }
  });
}

/** @param {string} color */
function setBarColor(color) {
  bars.forEach(bar => bar.setAttribute("fill", color));
}

function applyReducedMotionBars() {
  if (currentState === "recording") {
    bars.forEach((bar, index) => {
      const h = barStates[index].targetHeight;
      bar.setAttribute("height", h.toString());
      bar.setAttribute("y", (15.0 - h / 2).toString());
      bar.setAttribute("rx", Math.min(1.5, h / 2).toFixed(2));
      const barEnergy = Math.max(0, Math.min(1, (h - restHeight) / 7.0));
      bar.setAttribute("fill", getBarHeatColor(barEnergy));
    });
  } else if (currentState === "processing") {
    bars.forEach(bar => {
      bar.setAttribute("height", "4.0");
      bar.setAttribute("y", "12.6");
      bar.setAttribute("rx", "1.5");
      bar.setAttribute("fill", "#ff4200");
    });
  }
}

// State Crossfade Tracker (550ms silky smooth transition for beam & wave)
let transitionState = {
  active: false,
  startTime: 0,
  duration: 350,
  startBeam: 0.0,
  targetBeam: 0.0,
  currentBeam: 0.0,
  startWave: 0.0, // 0.0 = live mic, 1.0 = processing breathing
  targetWave: 0.0,
  currentWave: 0.0
};

// Lockstep Morph Tracker (Pill ↔ Square, ZERO SCALE CHANGE: currentW / vbW == 1.30097)
let morphState = {
  active: false,
  startProgress: 0.0,
  targetProgress: 0.0, // 0 = Pill (134px), 1 = Square (39px)
  startTime: 0,
  duration: 380,
  currentProgress: 0.0
};

/**
 * @param {number} t
 * @returns {number}
 */
function solveCubicBezier(t) {
  const c1 = 1 - t;
  return 1 - Math.pow(c1, 3.2);
}

/**
 * @param {number} target
 * @param {number} [duration]
 */
function startMorph(target, duration = 380) {
  if (duration === 0 || reduceMotionQuery.matches) {
    morphState.active = false;
    morphState.currentProgress = target;
    morphState.targetProgress = target;
    morphState.startProgress = target;
    applyMorphProgress(target);
    return;
  }
  morphState = {
    active: true,
    startProgress: morphState.currentProgress,
    targetProgress: target,
    startTime: performance.now(),
    duration: duration,
    currentProgress: morphState.currentProgress
  };
  startAnimationLoop();
}

/** @param {number} prog */
function applyMorphProgress(prog) {
  const currentWidth = 134 - (134 - 39) * prog;
  if (cardEl) {
    cardEl.style.width = currentWidth.toFixed(2) + "px";
  }
  const vbW = currentWidth / (134 / 103);
  const vbX = 0 + (1.2 - 0) * prog;
  const vbY = 0 + (-0.2 - 0) * prog;
  if (auraSvg) {
    auraSvg.setAttribute("viewBox", `${vbX.toFixed(3)} ${vbY.toFixed(3)} ${vbW.toFixed(3)} 30`);
  }
}

// Recording timer shown under the pill
function updateTimer() {
  if (currentState !== "recording" || !recordStart) {
    stopRecordingTimer();
    return;
  }
  if (!overlayShowTimer) {
    setOverlayText("");
    return;
  }
  const elapsed = Date.now() - recordStart;
  if (elapsed < 3000) {
    setOverlayText("");
    return;
  }
  const secs = Math.floor(elapsed / 1000);
  const m = Math.floor(secs / 60);
  const s = String(secs % 60).padStart(2, "0");
  setOverlayText(`${m}:${s}`);
}

function startRecordingTimer() {
  if (recordingTimerId !== null) return;
  recordingTimerId = setInterval(updateTimer, 500);
}

function stopRecordingTimer() {
  if (recordingTimerId !== null) {
    clearInterval(recordingTimerId);
    recordingTimerId = null;
  }
}

function stopAnimationLoop() {
  if (rafId !== null) {
    cancelAnimationFrame(rafId);
    rafId = null;
  }
}

function startAnimationLoop() {
  if (reduceMotionQuery.matches) {
    stopAnimationLoop();
    applyReducedMotionBars();
    return;
  }
  if (rafId !== null) return;
  rafId = requestAnimationFrame(updateAnimation);
}

// Main animation and physics loop (runs ONLY while overlay is visible or morphing)
function updateAnimation() {
  rafId = null;

  if (reduceMotionQuery.matches) {
    applyReducedMotionBars();
    return;
  }

  const isVisible = Boolean(pill && pill.classList.contains("visible"));
  if (!isVisible && !morphState.active && !transitionState.active) {
    return;
  }

  // 1. Handle Morphing Progress (Pill ↔ Square lockstep)
  if (morphState.active) {
    const elapsed = performance.now() - morphState.startTime;
    const t = Math.min(1, elapsed / morphState.duration);
    const ease = solveCubicBezier(t);
    morphState.currentProgress = morphState.startProgress + (morphState.targetProgress - morphState.startProgress) * ease;
    applyMorphProgress(morphState.currentProgress);

    if (t >= 1) {
      morphState.active = false;
      morphState.currentProgress = morphState.targetProgress;
      applyMorphProgress(morphState.targetProgress);
    }
  } else if (morphState.currentProgress > 0 && morphState.currentProgress < 1) {
    applyMorphProgress(morphState.currentProgress);
  }

  // 2. Handle Crossfade Transition Progress (550ms smooth cubic ease-in-out)
  if (transitionState.active) {
    const elapsed = performance.now() - transitionState.startTime;
    const t = Math.min(1, elapsed / transitionState.duration);
    const ease = t < 0.5 ? 4 * t * t * t : 1 - Math.pow(-2 * t + 2, 3) / 2;

    transitionState.currentBeam = transitionState.startBeam + (transitionState.targetBeam - transitionState.startBeam) * ease;
    transitionState.currentWave = transitionState.startWave + (transitionState.targetWave - transitionState.startWave) * ease;

    if (cardEl) {
      cardEl.style.setProperty("--beam-opacity", transitionState.currentBeam.toFixed(3));
    }

    if (t >= 1) {
      transitionState.active = false;
      transitionState.currentBeam = transitionState.targetBeam;
      transitionState.currentWave = transitionState.targetWave;
      if (cardEl) {
        cardEl.style.setProperty("--beam-opacity", transitionState.targetBeam.toFixed(3));
      }
    }
  }

  const procFactor = transitionState.currentWave;
  const recFactor = 1 - procFactor;

  // 3. Audio & wave animation
  angle = (angle + 0.05) % (Math.PI * 2);

  bars.forEach((bar, index) => {
    const state = barStates[index];
    const distFromCenter = Math.abs(index - 4.5);
    const targetProcH = 4.0 + Math.sin(angle - distFromCenter * 0.45) * 3.5;

    const blendedTarget = state.targetHeight * recFactor + targetProcH * procFactor;
    state.currentHeight += (blendedTarget - state.currentHeight) * 0.16;

    const y = 15.0 - (state.currentHeight / 2);
    bar.setAttribute("height", state.currentHeight.toFixed(2));
    bar.setAttribute("y", y.toFixed(2));
    bar.setAttribute("rx", (Math.min(1.5, state.currentHeight / 2)).toFixed(2));

    // Dynamic bar color (Variant 2: Per-bar heat / energy):
    // In silence (restHeight = 2.5), energy = 0 -> #722205 (exact silence reference).
    // As speech elevates bars, energy scales towards 1.0 -> #ff4200 (tactical orange).
    // During transition to processing or in processing, smoothly illuminate to #ff4200.
    if (currentState === "recording" || currentState === "processing") {
      const barEnergy = Math.max(0, Math.min(1, (state.currentHeight - restHeight) / 7.0));
      const effectiveEnergy = procFactor > 0 ? barEnergy + (1 - barEnergy) * procFactor : barEnergy;
      bar.setAttribute("fill", getBarHeatColor(effectiveEnergy));
    }
  });

  if ((pill && pill.classList.contains("visible")) || morphState.active || transitionState.active) {
    rafId = requestAnimationFrame(updateAnimation);
  }
}

const noticeTranslations = {
  "edit-failed": {
    ru: "Текст не изменён",
    en: "Text unchanged",
    de: "Text unverändert",
    fr: "Texte inchangé",
    it: "Testo invariato",
    es: "Texto sin cambios",
    pt: "Texto inalterado",
    zh: "文本未改动",
    ja: "変更なし",
    tr: "Metin değişmedi",
    nl: "Tekst ongewijzigd"
  },
  "edit-model-missing": {
    ru: "Локальная модель не скачана — скачайте её в настройках",
    en: "Local model not downloaded — download it in settings",
    de: "Lokales Modell nicht heruntergeladen — in Einstellungen laden",
    fr: "Modèle local non téléchargé — téléchargez-le dans les paramètres",
    it: "Modello locale non scaricato — scaricalo nelle impostazioni",
    es: "Modelo local no descargado — descárguelo en ajustes",
    pt: "Modelo local não descarregado — descarregue nas definições",
    zh: "本地模型未下载——请在设置中下载",
    ja: "ローカルモデルが未ダウンロードです — 設定からダウンロードしてください",
    tr: "Yerel model indirilmedi — ayarlardan indirin",
    nl: "Lokaal model niet gedownload — download het in instellingen"
  },
  "edit-no-api-key": {
    ru: "Укажите API-ключ в настройках для редактирования",
    en: "Add an API key in settings for text editing",
    de: "Fügen Sie einen API-Schlüssel in den Einstellungen hinzu",
    fr: "Ajoutez une clé API dans les paramètres pour modifier le texte",
    it: "Aggiungi una chiave API nelle impostazioni per modificare il testo",
    es: "Añada una clave API en los ajustes para editar texto",
    pt: "Adicione uma chave API nas definições para editar texto",
    zh: "请在设置中添加 API 密钥以编辑文本",
    ja: "テキスト編集用のAPIキーを設定で入力してください",
    tr: "Metin düzenlemek için ayarlara API anahtarı ekleyin",
    nl: "Voeg een API-sleutel toe in instellingen voor tekstbewerking"
  },
  "edit-local-failed": {
    ru: "Сбой локального сервера — выделенный текст не изменён",
    en: "Local engine error — selected text left unchanged",
    de: "Lokaler Serverfehler — ausgewählter Text unverändert",
    fr: "Erreur du moteur local — texte sélectionné inchangé",
    it: "Errore motore locale — testo selezionato invariato",
    es: "Error del motor local — texto seleccionado sin cambios",
    pt: "Erro no motor local — texto selecionado inalterado",
    zh: "本地引擎故障——选中文本未改动",
    ja: "ローカルエンジンのエラー — 選択テキストは変更されていません",
    tr: "Yerel motor hatası — seçili metin değiştirilmedi",
    nl: "Fout in lokale server — geselecteerde tekst ongewijzigd"
  },
  "edit-cloud-failed": {
    ru: "Сбой облачного ИИ — проверьте ключ и сеть",
    en: "Cloud AI error — check network and API key",
    de: "Cloud-KI-Fehler — Netzwerk und API-Schlüssel prüfen",
    fr: "Erreur de l'IA cloud — vérifiez le réseau et la clé API",
    it: "Errore IA cloud — controlla rete e chiave API",
    es: "Error de IA en la nube — compruebe red y clave API",
    pt: "Erro de IA na nuvem — verifique a rede e a chave API",
    zh: "云端 AI 错误——请检查网络和 API 密钥",
    ja: "クラウドAIエラー — ネットワークとAPIキーを確認してください",
    tr: "Bulut yapay zeka hatası — ağı ve API anahtarını kontrol edin",
    nl: "Cloud-AI fout — controleer netwerk en API-sleutel"
  },
  "edit-timeout": {
    ru: "Время ожидания истекло — текст не изменён",
    en: "Editing timed out — text left unchanged",
    de: "Zeitüberschreitung — Text unverändert",
    fr: "Délai dépassé — texte inchangé",
    it: "Tempo scaduto — testo invariato",
    es: "Tiempo agotado — texto sin cambios",
    pt: "Tempo limite excedido — texto inalterado",
    zh: "处理超时——文本未改动",
    ja: "タイムアウト — テキストは変更されていません",
    tr: "Zaman aşımı — metin değiştirilmedi",
    nl: "Time-out bij bewerking — tekst ongewijzigd"
  },
  "edit-too-large": {
    ru: "Текст слишком длинный для редактирования",
    en: "Selected text too long for editing",
    de: "Ausgewählter Text zu lang für Bearbeitung",
    fr: "Texte sélectionné trop long pour être modifié",
    it: "Testo selezionato troppo lungo da modificare",
    es: "Texto seleccionado demasiado largo para editar",
    pt: "Texto selecionado demasiado longo para editar",
    zh: "选中文本过长，无法处理",
    ja: "選択テキストが長すぎます",
    tr: "Seçili metin düzenleme için çok uzun",
    nl: "Geselecteerde tekst te lang voor bewerking"
  },
  "elevated-paste-blocked": {
    ru: "Система заблокировала вставку в активное окно. Текст сохранён в буфере обмена",
    en: "The system blocked pasting into the active window. Text kept in the clipboard",
    de: "Das System hat das Einfügen in das aktive Fenster blockiert. Text befindet sich in der Zwischenablage",
    fr: "Le système a bloqué le collage dans la fenêtre active. Texte conservé dans le presse-papiers",
    it: "Il sistema ha bloccato l'incolla nella finestra attiva. Testo conservato negli appunti",
    es: "El sistema bloqueó pegar en la ventana activa. Texto guardado en el portapapeles",
    pt: "O sistema bloqueou a colagem na janela ativa. Texto mantido na área de transferência",
    zh: "系统阻止了向活动窗口粘贴。文本已保留在剪贴板中",
    ja: "システムがアクティブなウィンドウへの貼り付けをブロックしました。テキストはクリップボードに保持されています",
    tr: "Sistem etkin pencereye yapıştırmayı engelledi. Metin panoda saklandı",
    nl: "Het systeem heeft plakken in het actieve venster geblokkeerd. Tekst bewaard op het klembord"
  },
  "Cloud unavailable — used local model instead": {
    ru: "Локальный режим (сбой сети)",
    en: "Local fallback (network error)",
    de: "Lokaler Modus (Netzwerkfehler)",
    fr: "Mode local (erreur réseau)",
    it: "Modalità locale (errore di rete)",
    es: "Modo local (error de red)",
    pt: "Modo local (erro de rede)",
    zh: "本地模式（网络错误）",
    ja: "ローカルモード（ネットワークエラー）",
    tr: "Yerel mod (ağ hatası)",
    nl: "Lokale modus (netwerkfout)"
  },
  "focus-changed-copied": {
    ru: "Скопировано (фокус изменен)",
    en: "Copied (focus changed)",
    de: "Kopiert (Fokus geändert)",
    fr: "Copié (focus modifié)",
    it: "Copiato (focus modificato)",
    es: "Copiado (foco cambiado)",
    pt: "Copiado (foco alterado)",
    zh: "已复制（焦点已更改）",
    ja: "コピー完了（フォーカス変更）",
    tr: "Kopyalandı (odak değişti)",
    nl: "Gekopieerd (focus gewijzigd)"
  },
  "final-copied-after-edit": {
    ru: "Текст скопирован",
    en: "Text copied",
    de: "Text kopiert",
    fr: "Texte copié",
    it: "Testo copiato",
    es: "Texto copiado",
    pt: "Texto copiado",
    zh: "文本已复制",
    ja: "テキストをコピーしました",
    tr: "Metin kopyalandı",
    nl: "Tekst gekopieerd"
  },
  "loading-model": {
    ru: "Загрузка модели…",
    en: "Loading model…",
    de: "Modell wird geladen…",
    fr: "Chargement du modèle…",
    it: "Caricamento del modello…",
    es: "Cargando modelo…",
    pt: "Carregando modelo…",
    zh: "正在加载模型…",
    ja: "モデルを読み込み中…",
    tr: "Model yükleniyor…",
    nl: "Model laden…"
  },
  "warming-model": {
    ru: "Инициализация…",
    en: "Initializing…",
    de: "Initialisierung…",
    fr: "Initialisation…",
    it: "Inizializzazione…",
    es: "Inicializando…",
    pt: "Inicializando…",
    zh: "正在初始化…",
    ja: "初期化中…",
    tr: "Başlatılıyor…",
    nl: "Initialiseren…"
  }
};

const errorTranslations = {
  "Access forbidden (403): VPN/proxy IP is blocked. Turn off the VPN for Groq/OpenAI or switch server": {
    ru: "Доступ запрещён (403): IP VPN/прокси заблокирован. Отключите VPN для Groq/OpenAI или смените сервер",
    en: "Access forbidden (403): VPN/proxy IP is blocked. Turn off the VPN for Groq/OpenAI or switch server",
    de: "Zugriff verboten (403): VPN/Proxy-IP ist blockiert. Deaktivieren Sie das VPN für Groq/OpenAI oder wechseln Sie den Server",
    fr: "Accès refusé (403) : l'IP VPN/proxy est bloquée. Désactivez le VPN pour Groq/OpenAI ou changez de serveur",
    it: "Accesso vietato (403): IP VPN/proxy bloccato. Disattiva la VPN per Groq/OpenAI o cambia server",
    es: "Acceso denegado (403): la IP de VPN/proxy está bloqueada. Desactive la VPN para Groq/OpenAI o cambie de servidor",
    pt: "Acesso proibido (403): IP de VPN/proxy bloqueada. Desative a VPN para Groq/OpenAI ou mude de servidor",
    zh: "访问被拒绝 (403)：VPN/代理 IP 已被封禁。请为 Groq/OpenAI 关闭 VPN 或更换服务器",
    ja: "アクセス禁止 (403): VPN/プロキシのIPがブロックされています。Groq/OpenAIでVPNを無効にするか、サーバーを変更してください",
    tr: "Erişim engellendi (403): VPN/proxy IP'si bloklanmış. Groq/OpenAI için VPN'i kapatın veya sunucuyu değiştirin"
  },
  "Microphone start error": {
    ru: "Ошибка запуска микрофона",
    en: "Microphone start error",
    de: "Fehler beim Starten des Mikrofons",
    fr: "Erreur de démarrage du microphone",
    it: "Errore di avvio del microfono",
    es: "Error al iniciar el micrófono",
    pt: "Erro ao iniciar o microfone",
    zh: "无法启动麦克风",
    ja: "マイクの起動エラー",
    tr: "Mikrofon başlatılamadı"
  },
  "Recording stop error": {
    ru: "Ошибка остановки записи",
    en: "Recording stop error",
    de: "Fehler beim Stoppen der Aufnahme",
    fr: "Erreur d'arrêt de l'enregistrement",
    it: "Errore di arresto della registrazione",
    es: "Error al detener la grabación",
    pt: "Erro ao parar a gravação",
    zh: "无法停止录音",
    ja: "録音停止エラー",
    tr: "Kayıt durdurulamadı"
  },
  "Settings load error": {
    ru: "Ошибка загрузки настроек",
    en: "Settings load error",
    de: "Fehler beim Laden der Einstellungen",
    fr: "Erreur de chargement des paramètres",
    it: "Errore di caricamento delle impostazioni",
    es: "Error al cargar los ajustes",
    pt: "Erro ao carregar as configurações",
    zh: "无法加载设置",
    ja: "設定の読み込みエラー",
    tr: "Ayarlar yüklenemedi"
  },
  "Gemini is unavailable in your region. Enable global VPN or choose Groq": {
    ru: "Gemini недоступен в вашем регионе. Включите VPN для всех приложений или выберите Groq",
    en: "Gemini is unavailable in your region. Enable global VPN or choose Groq",
    de: "Gemini ist in Ihrer Region nicht verfügbar. Aktivieren Sie VPN oder wählen Sie Groq",
    fr: "Gemini n'est pas disponible dans votre région. Activez un VPN ou choisissez Groq",
    it: "Gemini non è disponibile nella tua regione. Abilita la VPN o scegli Groq",
    es: "Gemini no está disponible en su región. Active una VPN o elija Groq",
    pt: "Gemini indisponível na sua região. Ative a VPN ou escolha o Groq",
    zh: "Gemini 在您所在地区不可用。请启用全局 VPN 或选择 Groq",
    ja: "Geminiはお住まいの地域では利用できません。VPNを有効にするかGroqを選択してください",
    tr: "Gemini bölgenizde kullanılamıyor. VPN'i etkinleştirin veya Groq'u seçin"
  },
  "Invalid API key in settings": {
    ru: "Неверный API-ключ в настройках",
    en: "Invalid API key in settings",
    de: "Ungültiger API-Schlüssel in den Einstellungen",
    fr: "Clé API non valide dans les paramètres",
    it: "Chiave API non valida nelle impostazioni",
    es: "Clave API no válida en los ajustes",
    pt: "Chave API inválida nas configurações",
    zh: "设置中的 API 密钥无效",
    ja: "設定のAPIキーが無効です",
    tr: "Ayarlardaki API anahtarı geçersiz"
  },
  "Connection error via VPN/Proxy": {
    ru: "Ошибка соединения через VPN/прокси",
    en: "Connection error via VPN/Proxy",
    de: "Verbindungsfehler über VPN/Proxy",
    fr: "Erreur de connexion via VPN/Proxy",
    it: "Errore di connessione tramite VPN/Proxy",
    es: "Error de conexión a través de VPN/Proxy",
    pt: "Erro de conexão via VPN/Proxy",
    zh: "通过 VPN/代理连接失败",
    ja: "VPN/プロキシ経由の接続エラー",
    tr: "VPN/Proxy bağlantı hatası"
  },
  "Local model not downloaded": {
    ru: "Локальная модель не скачана",
    en: "Local model not downloaded",
    de: "Lokales Modell nicht heruntergeladen",
    fr: "Modèle local non téléchargé",
    it: "Modello locale non scaricato",
    es: "Modelo local no descargado",
    pt: "Modelo local não baixado",
    zh: "本地模型未下载",
    ja: "ローカルモデルがダウンロードされていません",
    tr: "Yerel model indirilmedi"
  },
  "Local Whisper client failure": {
    ru: "Сбой локального Whisper-клиента",
    en: "Local Whisper client failure",
    de: "Fehler des lokalen Whisper-Clients",
    fr: "Échec du client Whisper local",
    it: "Errore del client Whisper locale",
    es: "Fallo del cliente Whisper local",
    pt: "Falha no cliente Whisper local",
    zh: "本地 Whisper 客户端崩溃",
    ja: "ローカルWhisperクライアントの起動失敗",
    tr: "Yerel Whisper istemci hatası"
  },
  "Local Parakeet client failure": {
    ru: "Сбой локального Parakeet-клиента",
    en: "Local Parakeet client failure",
    de: "Fehler des lokalen Parakeet-Clients",
    fr: "Échec du client Parakeet local",
    it: "Errore del client Parakeet locale",
    es: "Fallo del cliente Parakeet local",
    pt: "Falha no cliente Parakeet local",
    zh: "本地 Parakeet 客户端崩溃",
    ja: "ローカルParakeetクライアントの起動失敗",
    tr: "Yerel Parakeet istemci hatası"
  },
  "API rate limit reached": {
    ru: "Лимит запросов API исчерпан",
    en: "API rate limit reached",
    de: "API-Anfragenlimit erreicht",
    fr: "Limite de requêtes API atteinte",
    it: "Limite di richieste API raggiunto",
    es: "Límite de solicitudes API agotado",
    pt: "Limite de solicitações da API atingido",
    zh: "已达到 API 请求频率限制",
    ja: "APIリクエストの上限に達しました",
    tr: "API istek limiti tükendi"
  },
  "API key balance exhausted": {
    ru: "Баланс API ключа исчерпан",
    en: "API key balance exhausted",
    de: "Guthaben des API-Schlüssels aufgebraucht",
    fr: "Solde de la clé API épuisé",
    it: "Credito della chiave API esaurito",
    es: "Saldo de la clave API agotado",
    pt: "Saldo da chave API esgotado",
    zh: "API 密钥余额不足",
    ja: "APIキーの残高が不足しています",
    tr: "API anahtar bakiyesi tükendi"
  },
  "Failed to write to clipboard": {
    ru: "Не удалось записать в буфер обмена",
    en: "Failed to write to clipboard",
    de: "Schreiben in die Zwischenablage fehlgeschlagen",
    fr: "Échec de l'écriture dans le presse-papiers",
    it: "Impossibile scrivere negli appunti",
    es: "Error al escribir en el portapapeles",
    pt: "Falha ao gravar na área de transferência",
    zh: "无法写入剪贴板",
    ja: "クリップボードへの書き込みに失敗しました",
    tr: "Panoya yazılamadı"
  },
  "Recognition error": {
    ru: "Ошибка распознавания",
    en: "Recognition error",
    de: "Erkennungsfehler",
    fr: "Erreur de reconnaissance",
    it: "Errore di riconoscimento",
    es: "Error de reconocimiento",
    pt: "Erro de reconhecimento",
    zh: "识别错误",
    ja: "認識エラー",
    tr: "Tanıma hatası"
  }
};

function translateError(/** @type {string} */ errStr, /** @type {string} */ lang) {
  const errorMap = /** @type {Record<string, Record<string, string>>} */ (errorTranslations);
  const match = errorMap[errStr];
  if (match) {
    return match[lang] || match.en || errStr;
  }
  if (errStr.startsWith("No network:")) {
    const details = errStr.replace("No network:", "").trim();
    /** @type {Record<string, string>} */
    const netTranslations = {
      ru: "Нет сети:",
      en: "No network:",
      de: "Kein Netzwerk:",
      fr: "Pas de réseau :",
      it: "Nessuna rete:",
      es: "Sin red:",
      pt: "Sem rede:",
      zh: "无网络：",
      ja: "ネットワークなし：",
      tr: "Ağ bağlantısı yok:"
    };
    const prefix = netTranslations[lang] || netTranslations.en;
    return `${prefix} ${details}`;
  }
  if (errStr.startsWith("custom_api_url_insecure:")) {
    /** @type {Record<string, string>} */
    const insecureTranslations = {
      ru: "Пользовательский сервер должен использовать HTTPS. HTTP разрешён только для localhost.",
      en: "The custom server must use HTTPS. HTTP is allowed only for localhost.",
      de: "Der benutzerdefinierte Server muss HTTPS verwenden. HTTP ist nur für localhost erlaubt.",
      fr: "Le serveur personnalisé doit utiliser HTTPS. HTTP n'est autorisé que pour localhost.",
      it: "Il server personalizzato deve usare HTTPS. HTTP è consentito solo per localhost.",
      es: "El servidor personalizado debe usar HTTPS. HTTP solo se permite para localhost.",
      pt: "O servidor personalizado deve usar HTTPS. HTTP é permitido apenas para localhost.",
      zh: "自定义服务器必须使用 HTTPS。HTTP 仅允许用于 localhost。",
      tr: "Özel sunucu HTTPS kullanmalıdır. HTTP yalnızca localhost için izinlidir."
    };
    return insecureTranslations[lang] || insecureTranslations.en;
  }
  return errStr;
}

// Listen for recording-state updates: "recording" | "processing" | "error"
listen("recording-state", (event) => {
  // Any incoming state invalidates a pending auto-hide from a previous state
  // (an old error timer must never hide a brand-new recording).
  if (hideTimerId !== null) {
    clearTimeout(hideTimerId);
    hideTimerId = null;
  }
  const incomingState = typeof event.payload === "string" ? event.payload : "";
  currentState = incomingState;
  stateCycle += 1;

  if (currentState === "recording") {
    if (cardEl) {
      cardEl.classList.remove("processing");
      cardEl.classList.remove("square");
      cardEl.classList.toggle("edit-context", isEditContext);
    }
    if (barsGroup) barsGroup.style.opacity = "1";
    startMorph(0.0, 0);
    transitionState = {
      active: true,
      startTime: performance.now(),
      duration: 350,
      startBeam: transitionState.currentBeam,
      targetBeam: 0.0,
      currentBeam: transitionState.currentBeam,
      startWave: transitionState.currentWave,
      targetWave: 0.0,
      currentWave: transitionState.currentWave
    };
    recordStart = Date.now();
    resolveUiLang();
    setOverlayText("");
    if (statusEl) statusEl.classList.remove("error");
    setBarColor("rgb(114, 34, 5)");
    resetBars();

    // Animate show and play startup sound
    if (pill) pill.classList.add("visible");
    startRecordingTimer();
    startAnimationLoop();
    playThemeStart();
  } else if (currentState === "processing") {
    stopRecordingTimer();
    isEditContext = false;
    if (cardEl) {
      cardEl.classList.add("processing");
      cardEl.classList.remove("square");
      cardEl.classList.remove("edit-context");
    }
    if (barsGroup) barsGroup.style.opacity = "1";
    transitionState = {
      active: true,
      startTime: performance.now(),
      duration: 350,
      startBeam: transitionState.currentBeam,
      targetBeam: 0.75,
      currentBeam: transitionState.currentBeam,
      startWave: transitionState.currentWave,
      targetWave: 1.0,
      currentWave: transitionState.currentWave
    };
    angle = 0;
    resolveUiLang();
    setOverlayText("");
    if (statusEl) statusEl.classList.remove("error");
    if (pill) pill.classList.add("visible");
    startAnimationLoop();
  } else if (currentState.startsWith("notice:")) {
    stopRecordingTimer();
    isEditContext = false;
    const notice = currentState.substring("notice:".length);
    const uiLang = resolveUiLang();
    const noticeMap = /** @type {Record<string, Record<string, string>>} */ (noticeTranslations);
    const translations = noticeMap[notice];
    const isWarmupNotice = notice === "loading-model" || notice === "warming-model";

    if (isWarmupNotice) {
      currentState = "processing";
      if (cardEl) {
        cardEl.classList.add("processing");
        cardEl.classList.remove("square");
        cardEl.classList.remove("edit-context");
      }
      if (barsGroup) barsGroup.style.opacity = "1";
      startMorph(0.0, 0);
      transitionState = {
        active: true,
        startTime: performance.now(),
        duration: 350,
        startBeam: transitionState.currentBeam,
        targetBeam: 0.75,
        currentBeam: transitionState.currentBeam,
        startWave: transitionState.currentWave,
        targetWave: 1.0,
        currentWave: transitionState.currentWave
      };
      setOverlayText(translations?.[uiLang] || translations?.en || notice);
      if (statusEl) statusEl.classList.remove("error");
      if (pill) pill.classList.add("visible");
      startAnimationLoop();
    } else {
      if (cardEl) {
        cardEl.classList.remove("processing");
        cardEl.classList.remove("square");
        cardEl.classList.remove("edit-context");
        cardEl.style.setProperty("--beam-opacity", "0");
      }
      transitionState.active = false;
      transitionState.currentBeam = 0.0;
      transitionState.targetBeam = 0.0;
      transitionState.currentWave = 0.0;
      transitionState.targetWave = 0.0;
      startMorph(0.0, 0);
      // Non-alarming transient message (e.g. cloud->local fallback); no error sound.
      // A notice is a message, not the countdown: it must stay readable even
      // when "show timer" is off, otherwise a failed edit looks like nothing
      // happened.
      setOverlayText(translations?.[uiLang] || translations?.en || notice);
      if (statusEl) statusEl.classList.remove("error");
      setBarColor("#ff4200");
      if (pill) pill.classList.add("visible");
      stopAnimationLoop();
    }
  } else if (currentState.startsWith("error")) {
    stopRecordingTimer();
    if (cardEl) {
      cardEl.classList.remove("processing");
      cardEl.classList.remove("square");
      cardEl.classList.remove("edit-context");
      cardEl.style.setProperty("--beam-opacity", "0");
    }
    isEditContext = false;
    transitionState.active = false;
    transitionState.currentBeam = 0.0;
    transitionState.targetBeam = 0.0;
    transitionState.currentWave = 0.0;
    transitionState.targetWave = 0.0;
    startMorph(0.0, 0);
    recordStart = null;
    let errMsg = "Recognition error";
    if (currentState.includes(":")) {
      errMsg = currentState.substring(currentState.indexOf(":") + 1);
    }
    const uiLang = resolveUiLang();
    setOverlayText(translateError(errMsg, uiLang));
    if (statusEl) statusEl.classList.add("error");
    setBarColor("#666666");
    resetBars();
    stopAnimationLoop();
    
    // Ensure the pill is visible to display the error
    if (pill) pill.classList.add("visible");

    // Play error sound and animate hide after a brief delay
    playThemeError();
    const stateAtError = currentState;
    const cycleAtError = stateCycle;
    hideTimerId = setTimeout(() => {
      hideTimerId = null;
      // Only hide if the overlay still shows this same error; a newer
      // session (recording/processing/notice) owns the overlay otherwise.
      if (currentState !== stateAtError || stateCycle !== cycleAtError) return;
      if (pill) pill.classList.remove("visible");
      hideOverlayAfterPillFade(stateAtError, cycleAtError);
    }, 2500);
  }
});

// Hides the overlay window once the pill fade-out is done. With reduced
// motion the transition is disabled, so the hide must happen immediately.
// `expectedCycle` guards against a hide request from an old session firing
// after a brand-new session already took over the overlay.
/**
 * @param {string} expectedState
 * @param {number} expectedCycle
 */
function hideOverlayAfterPillFade(expectedState, expectedCycle) {
  const guardExpired = () =>
    currentState !== expectedState || stateCycle !== expectedCycle;

  const hideNow = () => {
    if (guardExpired()) return;
    stopRecordingTimer();
    stopAnimationLoop();
    scheduleIdleAudioSuspend();
    setOverlayText("");
    if (cardEl) {
      cardEl.classList.remove("processing", "edit-context");
      cardEl.classList.add("square");
      cardEl.style.setProperty("--beam-opacity", "0");
    }
    if (tauriWindow.__TAURI__ && tauriWindow.__TAURI__.core) {
      tauriWindow.__TAURI__.core
        .invoke("hide_overlay_window")
        .catch((err) => console.error("hide_overlay_window failed:", err));
    }
  };

  /** @param {TransitionEvent} e */
  function handler(e) {
    if (!pill || e.target !== pill || e.propertyName !== "opacity") return;
    pill.removeEventListener("transitionend", handler);
    clearTimeout(failsafeId);
    hideNow();
  }

  // Failsafe: if the fade transition never completes (or the event is
  // missed), still hide the window instead of leaving it stranded.
  const failsafeId = setTimeout(() => {
    if (pill) {
      pill.removeEventListener("transitionend", handler);
      pill.classList.remove("visible");
    }
    hideNow();
  }, 1200);

  if (reduceMotionQuery.matches || !pill) {
    clearTimeout(failsafeId);
    hideNow();
    return;
  }
  pill.addEventListener("transitionend", handler);
}

// Listen to the new event "hide-overlay-requested"
listen("hide-overlay-requested", (event) => {
  const payload = event.payload || {};
  const status = payload.status || "success";

  stopRecordingTimer();
  recordStart = null;
  isEditContext = false;
  if (cardEl) {
    cardEl.classList.remove("edit-context");
  }
  setOverlayText("");

  if (status === "success") {
    playThemeSuccess();
    currentState = "square";
    if (cardEl) {
      cardEl.classList.remove("processing");
      cardEl.classList.add("square");
      cardEl.style.setProperty("--beam-opacity", "0");
    }
    if (barsGroup) barsGroup.style.opacity = "0";
    transitionState.active = false;
    transitionState.currentBeam = 0.0;
    transitionState.targetBeam = 0.0;
    transitionState.currentWave = 0.0;
    transitionState.targetWave = 0.0;
    startMorph(1.0, 380);

    const stateAtRequest = currentState;
    const cycleAtRequest = stateCycle;
    setTimeout(() => {
      if (stateCycle !== cycleAtRequest) return;
      if (pill) pill.classList.remove("visible");
      hideOverlayAfterPillFade(stateAtRequest, cycleAtRequest);
    }, 520);
  } else {
    playThemeError();
    const stateAtRequest = currentState;
    const cycleAtRequest = stateCycle;
    if (cardEl) {
      cardEl.classList.remove("processing");
      cardEl.classList.remove("square");
      cardEl.style.setProperty("--beam-opacity", "0");
    }
    transitionState.active = false;
    transitionState.currentBeam = 0.0;
    transitionState.targetBeam = 0.0;
    transitionState.currentWave = 0.0;
    transitionState.targetWave = 0.0;
    stopAnimationLoop();
    if (pill) pill.classList.remove("visible");
    hideOverlayAfterPillFade(stateAtRequest, cycleAtRequest);
  }
});


// Listen to real-time voice volume updates from Rust
listen("volume-level", (event) => {
  if (currentState !== "recording") return;

  // Extract raw volume value safely
  let volume = 0.0;
  if (event.payload !== null && event.payload !== undefined) {
    volume = typeof event.payload === "number" ? event.payload : parseFloat(event.payload);
    if (isNaN(volume)) volume = 0.0;
  }

  barStates.forEach((state, index) => {
    let h = restHeight;
    // Make wave react to even tiny volume fluctuations
    if (volume > 0.00005) {
      const boosted = Math.sqrt(volume); // e.g. sqrt(0.001) = 0.031

      // Generate organic ripples based on time and index to simulate frequency bands
      const time = Date.now() * 0.007;
      const wave = 0.35 +
        Math.sin(time * (0.9 + index * 0.12) + index * 1.6) * 0.4 +
        Math.cos(time * 0.65 - index * 0.95) * 0.25;

      // Apply multiplier to normalized wave coefficient
      h = restHeight + (boosted * 36.0 * Math.max(0.05, wave));
      h = Math.min(maxHeight, Math.max(restHeight, h));
    }
    state.targetHeight = h;
  });

  if (reduceMotionQuery.matches) {
    applyReducedMotionBars();
  }
});

// Standalone preview fallback (when opened outside Tauri in browser QA or test runner)
if (!tauriWindow.__TAURI__ && pill) {
  currentState = "recording";
  pill.classList.add("visible");
  setOverlayText("0:00");
  startAnimationLoop();
}
