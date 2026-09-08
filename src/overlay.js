const { listen } = window.__TAURI__.event;

const bars = document.querySelectorAll(".sound-bar");
const statusEl = document.getElementById("overlay-status");
const statusTextEl = document.getElementById("overlay-status-text");
const contextBadgeEl = document.getElementById("overlay-context-badge");
const pill = document.querySelector(".overlay-pill");
let currentState = "recording";

function updateStatusVisibility() {
  const hasText = statusTextEl
    ? Boolean(statusTextEl.textContent && statusTextEl.textContent.trim())
    : Boolean(statusEl && statusEl.textContent && statusEl.textContent.trim());
  const hasContext = Boolean(
    contextBadgeEl &&
    contextBadgeEl.style.display !== "none" &&
    contextBadgeEl.style.display !== ""
  );
  if (statusEl) {
    statusEl.style.display = (hasText || hasContext) ? "inline-block" : "none";
  }
}

function setOverlayText(text) {
  if (statusTextEl) {
    statusTextEl.textContent = text;
  } else if (statusEl) {
    statusEl.textContent = text;
  }
  updateStatusVisibility();
}

const contextIndicatorTranslations = {
  ru: "Редактирование",
  en: "Edit",
  de: "Bearbeiten",
  fr: "Modifier",
  it: "Modifica",
  es: "Editar",
  pt: "Editar",
  zh: "编辑",
  ja: "編集",
  tr: "Düzenle"
};

listen("selection-context-active", (event) => {
  const active = Boolean(event.payload);
  if (contextBadgeEl) {
    contextBadgeEl.style.display = active ? "inline-flex" : "none";
    const textEl = document.getElementById("overlay-context-text");
    if (textEl) {
      const uiLang = localStorage.getItem("aura_ui_lang") || "ru";
      const txt = contextIndicatorTranslations[uiLang] || contextIndicatorTranslations.en;
      textEl.textContent = txt;
      if (contextBadgeEl) contextBadgeEl.title = txt;
    }
  }
  updateStatusVisibility();
});
// Monotonic counter advanced on every state event. The hide guard compares
// it so a hide request from an old session can never mask a brand-new
// recording (state strings alone cannot distinguish two sessions).
let stateCycle = 0;
let recordStart = null;
let angle = 0;
// Pending auto-hide timer from a previous state (e.g. an error). Cleared when
// a newer session owns the overlay, so it can never hide a fresh recording.
let hideTimerId = null;

// Windows "Show animations" off: static bars, no pill transition.
const reduceMotionQuery = window.matchMedia("(prefers-reduced-motion: reduce)");

let audioCtx = null;
let soundVolume = 0.8;
let globalGain = null;

function initAudioCtx() {
  if (!audioCtx) {
    audioCtx = new (window.AudioContext || window.webkitAudioContext)();
    globalGain = audioCtx.createGain();
    globalGain.gain.value = soundVolume;
    globalGain.connect(audioCtx.destination);
  }
}

// Tibetan singing bowl physical modeling synth.
// Layers inharmonic overtones and detuned oscillator pairs to generate natural shimmering phase beating (tremolo).
function playBowl(freq, duration, gainStart = 0.08) {
  initAudioCtx();
  if (audioCtx.state === "suspended") {
    audioCtx.resume();
  }

  const now = audioCtx.currentTime;

  // We layer 3 resonant components: fundamental, warm overtone, and metallic ring
  const components = [
    { f: freq, gain: gainStart, decay: duration },
    { f: freq * 1.5, gain: gainStart * 0.45, decay: duration * 0.8 },
    { f: freq * 2.76, gain: gainStart * 0.3, decay: duration * 0.55 }
  ];

  components.forEach(comp => {
    // Oscillator pair detuned by ~1.4Hz creates natural acoustic shimmering tremolo (beating)
    const oscA = audioCtx.createOscillator();
    const gainA = audioCtx.createGain();
    oscA.type = "sine";
    oscA.frequency.setValueAtTime(comp.f - 0.7, now);

    const oscB = audioCtx.createOscillator();
    const gainB = audioCtx.createGain();
    oscB.type = "sine";
    oscB.frequency.setValueAtTime(comp.f + 0.7, now);

    // Soft mallet attack (15ms) prevents harsh transients
    const attack = 0.015;
    
    gainA.gain.setValueAtTime(0.0001, now);
    gainA.gain.linearRampToValueAtTime(comp.gain, now + attack);
    gainA.gain.exponentialRampToValueAtTime(0.0001, now + comp.decay);

    gainB.gain.setValueAtTime(0.0001, now);
    gainB.gain.linearRampToValueAtTime(comp.gain, now + attack);
    gainB.gain.exponentialRampToValueAtTime(0.0001, now + comp.decay);

    oscA.connect(gainA);
    gainA.connect(globalGain);

    oscB.connect(gainB);
    gainB.connect(globalGain);

    oscA.start(now);
    oscA.stop(now + comp.decay + 0.1);

    oscB.start(now);
    oscB.stop(now + comp.decay + 0.1);
  });
}

function playRhodes(freq, duration, gainStart = 0.08) {
  initAudioCtx();
  if (audioCtx.state === "suspended") {
    audioCtx.resume();
  }
  const now = audioCtx.currentTime;
  const osc1 = audioCtx.createOscillator();
  const osc2 = audioCtx.createOscillator();
  const gain1 = audioCtx.createGain();
  const gain2 = audioCtx.createGain();

  osc1.type = "triangle";
  osc1.frequency.setValueAtTime(freq, now);

  osc2.type = "sine";
  osc2.frequency.setValueAtTime(freq, now);

  const attack = 0.010;
  gain1.gain.setValueAtTime(0.0001, now);
  gain1.gain.linearRampToValueAtTime(gainStart * 0.4, now + attack);
  gain1.gain.exponentialRampToValueAtTime(0.0001, now + duration);

  gain2.gain.setValueAtTime(0.0001, now);
  gain2.gain.linearRampToValueAtTime(gainStart * 0.7, now + attack);
  gain2.gain.exponentialRampToValueAtTime(0.0001, now + duration);

  osc1.connect(gain1);
  gain1.connect(globalGain);
  osc2.connect(gain2);
  gain2.connect(globalGain);

  osc1.start(now);
  osc1.stop(now + duration + 0.1);
  osc2.start(now);
  osc2.stop(now + duration + 0.1);
}

function playSciFi(freqStart, freqEnd, duration, gainStart = 0.06) {
  initAudioCtx();
  if (audioCtx.state === "suspended") {
    audioCtx.resume();
  }
  const now = audioCtx.currentTime;
  const osc1 = audioCtx.createOscillator();
  const osc2 = audioCtx.createOscillator();
  const gainNode = audioCtx.createGain();

  osc1.type = "sine";
  osc1.frequency.setValueAtTime(freqStart, now);
  osc1.frequency.exponentialRampToValueAtTime(freqEnd, now + duration);

  osc2.type = "sine";
  osc2.frequency.setValueAtTime(freqStart + 3.0, now);
  osc2.frequency.exponentialRampToValueAtTime(freqEnd + 3.0, now + duration);

  const attack = 0.012;
  gainNode.gain.setValueAtTime(0.0001, now);
  gainNode.gain.linearRampToValueAtTime(gainStart, now + attack);
  gainNode.gain.exponentialRampToValueAtTime(0.0001, now + duration);

  osc1.connect(gainNode);
  osc2.connect(gainNode);
  gainNode.connect(globalGain);

  osc1.start(now);
  osc1.stop(now + duration + 0.1);
  osc2.start(now);
  osc2.stop(now + duration + 0.1);
}

function playBell(freq, duration, gainStart = 0.07) {
  initAudioCtx();
  if (audioCtx.state === "suspended") {
    audioCtx.resume();
  }
  const now = audioCtx.currentTime;
  const osc1 = audioCtx.createOscillator();
  const osc2 = audioCtx.createOscillator();
  const osc3 = audioCtx.createOscillator();
  const gain1 = audioCtx.createGain();
  const gain2 = audioCtx.createGain();
  const gain3 = audioCtx.createGain();

  osc1.type = "sine";
  osc1.frequency.setValueAtTime(freq, now);
  osc2.type = "sine";
  osc2.frequency.setValueAtTime(freq * 2.0, now);
  osc3.type = "sine";
  osc3.frequency.setValueAtTime(freq * 3.0, now);

  const attack = 0.006;
  gain1.gain.setValueAtTime(0.0001, now);
  gain1.gain.linearRampToValueAtTime(gainStart, now + attack);
  gain1.gain.exponentialRampToValueAtTime(0.0001, now + duration);

  gain2.gain.setValueAtTime(0.0001, now);
  gain2.gain.linearRampToValueAtTime(gainStart * 0.45, now + attack);
  gain2.gain.exponentialRampToValueAtTime(0.0001, now + duration * 0.7);

  gain3.gain.setValueAtTime(0.0001, now);
  gain3.gain.linearRampToValueAtTime(gainStart * 0.25, now + attack);
  gain3.gain.exponentialRampToValueAtTime(0.0001, now + duration * 0.4);

  osc1.connect(gain1);
  gain1.connect(globalGain);
  osc2.connect(gain2);
  gain2.connect(globalGain);
  osc3.connect(gain3);
  gain3.connect(globalGain);

  osc1.start(now);
  osc1.stop(now + duration + 0.1);
  osc2.start(now);
  osc2.stop(now + duration + 0.1);
  osc3.start(now);
  osc3.stop(now + duration + 0.1);
}

function playWaterDrop(fStart, fEnd, duration, gainStart = 0.08) {
  initAudioCtx();
  if (audioCtx.state === "suspended") {
    audioCtx.resume();
  }
  const now = audioCtx.currentTime;
  const osc = audioCtx.createOscillator();
  const gain = audioCtx.createGain();

  osc.type = "sine";
  osc.frequency.setValueAtTime(fStart, now);
  osc.frequency.exponentialRampToValueAtTime(fEnd, now + (duration * 0.65));
  osc.frequency.exponentialRampToValueAtTime(fEnd * 0.9, now + duration);

  gain.gain.setValueAtTime(0.0001, now);
  gain.gain.linearRampToValueAtTime(gainStart, now + 0.008);
  gain.gain.exponentialRampToValueAtTime(0.0001, now + duration);

  osc.connect(gain);
  gain.connect(globalGain);

  osc.start(now);
  osc.stop(now + duration + 0.02);
}

function playHapticClick(clickFreq, duration, gainStart = 0.10) {
  initAudioCtx();
  if (audioCtx.state === "suspended") {
    audioCtx.resume();
  }
  const now = audioCtx.currentTime;
  const osc = audioCtx.createOscillator();
  const gain = audioCtx.createGain();

  osc.type = "sine";
  osc.frequency.setValueAtTime(clickFreq * 2.5, now);
  osc.frequency.exponentialRampToValueAtTime(clickFreq * 0.6, now + duration);

  gain.gain.setValueAtTime(0.0001, now);
  gain.gain.linearRampToValueAtTime(gainStart, now + 0.002);
  gain.gain.exponentialRampToValueAtTime(0.0001, now + duration);

  const bufferSize = Math.floor(audioCtx.sampleRate * 0.008);
  const noiseBuffer = audioCtx.createBuffer(1, bufferSize, audioCtx.sampleRate);
  const output = noiseBuffer.getChannelData(0);
  for (let i = 0; i < bufferSize; i++) {
    output[i] = Math.random() * 2 - 1;
  }
  const noise = audioCtx.createBufferSource();
  noise.buffer = noiseBuffer;
  const filter = audioCtx.createBiquadFilter();
  filter.type = "bandpass";
  filter.frequency.value = 2800;
  const noiseGain = audioCtx.createGain();
  noiseGain.gain.setValueAtTime(gainStart * 0.35, now);
  noiseGain.gain.exponentialRampToValueAtTime(0.0001, now + 0.008);

  noise.connect(filter);
  filter.connect(noiseGain);
  noiseGain.connect(globalGain);
  noise.start(now);

  osc.connect(gain);
  gain.connect(globalGain);
  osc.start(now);
  osc.stop(now + duration);
}

let soundsEnabled = true;
let soundTheme = "zen";
let overlayShowTimer = true;

function applyActiveThemeSettings(settings) {
  if (!settings || typeof settings !== "object") {
    return;
  }
  if (settings.ui_language) {
    try {
      localStorage.setItem("aura_ui_lang", settings.ui_language);
    } catch (_) {}
    const textEl = document.getElementById("overlay-context-text");
    if (textEl) {
      const txt = contextIndicatorTranslations[settings.ui_language] || contextIndicatorTranslations.en;
      textEl.textContent = txt;
      if (contextBadgeEl) contextBadgeEl.title = txt;
    }
  }
  soundsEnabled = settings.overlay_sounds !== false;
  soundTheme = settings.overlay_sound_theme || "zen";
  soundVolume = typeof settings.overlay_sound_volume === "number" ? settings.overlay_sound_volume : 0.8;
  overlayShowTimer = settings.overlay_show_timer !== false;
  if (!overlayShowTimer && !currentState.startsWith("error")) {
    setOverlayText("");
  }
  if (globalGain && audioCtx) {
    globalGain.gain.setValueAtTime(soundVolume, audioCtx.currentTime);
  }
}

listen("overlay-preferences", (event) => {
  applyActiveThemeSettings(event.payload);
});

function playThemeStart() {
  if (!soundsEnabled) return;
  if (soundTheme === "rhodes") {
    playRhodes(440.00, 0.65, 0.07);
    setTimeout(() => { playRhodes(554.37, 0.80, 0.07); }, 90);
  } else if (soundTheme === "scifi") {
    playSciFi(440.00, 880.00, 0.22, 0.055);
  } else if (soundTheme === "classic") {
    playBell(440.00, 0.45, 0.06);
    setTimeout(() => { playBell(659.25, 0.60, 0.06); }, 90);
  } else if (soundTheme === "bubble") {
    playWaterDrop(400, 950, 0.12, 0.08);
  } else if (soundTheme === "haptic") {
    playHapticClick(180, 0.03, 0.12);
  } else {
    // Default: zen
    playBowl(174.61, 1.2, 0.08);
  }
}

function playThemeSuccess() {
  if (!soundsEnabled) return;
  if (soundTheme === "rhodes") {
    playRhodes(220.00, 0.45, 0.06);
    setTimeout(() => {
      playRhodes(329.63, 0.45, 0.06);
      setTimeout(() => {
        playRhodes(440.00, 0.45, 0.06);
        setTimeout(() => {
          playRhodes(554.37, 0.45, 0.06);
          setTimeout(() => {
            playRhodes(659.25, 0.85, 0.06);
          }, 70);
        }, 70);
      }, 70);
    }, 70);
  } else if (soundTheme === "scifi") {
    playSciFi(523.25, 1046.50, 0.18, 0.05);
    setTimeout(() => { playSciFi(783.99, 1567.98, 0.18, 0.045); }, 65);
  } else if (soundTheme === "classic") {
    playBell(440.00, 0.35, 0.05);
    setTimeout(() => {
      playBell(554.37, 0.35, 0.05);
      setTimeout(() => {
        playBell(659.25, 0.45, 0.05);
        setTimeout(() => {
          playBell(880.00, 0.75, 0.05);
        }, 70);
      }, 70);
    }, 70);
  } else if (soundTheme === "bubble") {
    playWaterDrop(380, 850, 0.10, 0.07);
    setTimeout(() => { playWaterDrop(500, 1200, 0.15, 0.08); }, 90);
  } else if (soundTheme === "haptic") {
    playHapticClick(240, 0.025, 0.10);
    setTimeout(() => { playHapticClick(360, 0.04, 0.12); }, 70);
  } else {
    // Default: zen
    playBowl(174.61, 1.8, 0.07);
    setTimeout(() => { playBowl(261.63, 1.8, 0.06); }, 60);
  }
}

function playThemeError() {
  if (!soundsEnabled) return;
  if (soundTheme === "rhodes") {
    playRhodes(554.37, 0.35, 0.07);
    setTimeout(() => { playRhodes(440.00, 0.55, 0.07); }, 120);
  } else if (soundTheme === "scifi") {
    playSciFi(587.33, 293.66, 0.28, 0.06);
  } else if (soundTheme === "classic") {
    playBell(659.25, 0.35, 0.06);
    setTimeout(() => { playBell(440.00, 0.50, 0.06); }, 120);
  } else if (soundTheme === "bubble") {
    playWaterDrop(650, 280, 0.18, 0.08);
  } else if (soundTheme === "haptic") {
    playHapticClick(120, 0.05, 0.12);
    setTimeout(() => { playHapticClick(90, 0.08, 0.12); }, 100);
  } else {
    // Default: zen
    playBowl(174.61, 0.45, 0.07);
  }
}

// Track current and target heights for smooth linear interpolation (lerp)
const barStates = Array.from(bars).map(() => ({
  currentHeight: 6.0,
  targetHeight: 6.0
}));

// Reset all bars to default quiet state
function resetBars() {
  barStates.forEach(state => {
    state.targetHeight = 6.0;
    state.currentHeight = 6.0;
  });
  bars.forEach(bar => {
    bar.setAttribute("height", "6");
    bar.setAttribute("y", "11.6001");
  });
}

function setBarColor(color) {
  bars.forEach(bar => bar.setAttribute("fill", color));
}

// Recording timer shown under the pill
function updateTimer() {
  if (currentState === "recording" && recordStart) {
    if (!overlayShowTimer) {
      setOverlayText("");
      return;
    }
    const secs = Math.floor((Date.now() - recordStart) / 1000);
    const m = Math.floor(secs / 60);
    const s = String(secs % 60).padStart(2, "0");
    setOverlayText(`${m}:${s}`);
  }
}
setInterval(updateTimer, 500);

// Main animation and physics loop (60fps)
function updateAnimation() {
  if (reduceMotionQuery.matches) {
    // Static rendering: no lerping, no breathing wave. Volume still moves the
    // bars in recording state, but instantly and without oscillation.
    if (currentState === "recording") {
      bars.forEach((bar, index) => {
        const h = barStates[index].targetHeight;
        bar.setAttribute("height", h.toString());
        bar.setAttribute("y", (14.6 - h / 2).toString());
      });
    } else if (currentState === "processing") {
      bars.forEach(bar => {
        bar.setAttribute("height", "10.5");
        bar.setAttribute("y", "9.35");
      });
    }
    requestAnimationFrame(updateAnimation);
    return;
  }

  if (currentState === "processing") {
    // Slow, organic breathing wave centered around the middle bar (index 4)
    bars.forEach((bar, index) => {
      const distFromCenter = Math.abs(index - 4);
      const h = 5 + Math.sin(angle - distFromCenter * 0.45) * 5.5;
      const y = 14.6 - (h / 2);
      bar.setAttribute("height", h.toString());
      bar.setAttribute("y", y.toString());
    });
    angle += 0.05;
  } else if (currentState === "recording") {
    // Smoothly slide currentHeight towards targetHeight
    bars.forEach((bar, index) => {
      const state = barStates[index];
      // Lerp: current = current + (target - current) * factor
      state.currentHeight += (state.targetHeight - state.currentHeight) * 0.20;

      const y = 14.6 - (state.currentHeight / 2);
      bar.setAttribute("height", state.currentHeight.toString());
      bar.setAttribute("y", y.toString());
    });
  }

  requestAnimationFrame(updateAnimation);
}

// Start the animation loop
updateAnimation();

const processingTranslations = {
  ru: "Обработка…",
  en: "Processing…",
  de: "Verarbeitung…",
  fr: "Traitement…",
  it: "Elaborazione…",
  es: "Procesando…",
  pt: "Processamento…",
  zh: "处理中…",
  ja: "処理中…",
  tr: "İşleniyor…"
};

const noticeTranslations = {
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
    tr: "Sistem etkin pencereye yapıştırmayı engelledi. Metin panoda saklandı"
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
    tr: "Yerel mod (ağ hatası)"
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
    tr: "Kopyalandı (odak değişti)"
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
    tr: "Metin kopyalandı"
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
    tr: "Model yükleniyor…"
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
    tr: "Başlatılıyor…"
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
  }
};

function translateError(errStr, lang) {
  const match = errorTranslations[errStr];
  if (match && match[lang]) {
    return match[lang];
  }
  if (errStr.startsWith("No network:")) {
    const details = errStr.replace("No network:", "").trim();
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
  currentState = event.payload;
  stateCycle += 1;

  if (currentState === "recording") {
    recordStart = Date.now();
    setOverlayText(overlayShowTimer ? "0:00" : "");
    statusEl.classList.remove("error");
    setBarColor("#FE4200");
    resetBars();

    // Animate show and play startup sound
    pill.classList.add("visible");
    playThemeStart();
  } else if (currentState === "processing") {
    angle = 0;
    if (contextBadgeEl) {
      contextBadgeEl.style.display = "none";
    }
    const uiLang = localStorage.getItem("aura_ui_lang") || "ru";
    setOverlayText(overlayShowTimer ? (processingTranslations[uiLang] || processingTranslations.en) : "");
    statusEl.classList.remove("error");
  } else if (currentState.startsWith("notice:")) {
    // Non-alarming transient message (e.g. cloud->local fallback); no error sound.
    if (contextBadgeEl) {
      contextBadgeEl.style.display = "none";
    }
    const notice = currentState.substring("notice:".length);
    const uiLang = localStorage.getItem("aura_ui_lang") || "ru";
    const translations = noticeTranslations[notice];
    setOverlayText(overlayShowTimer ? (translations?.[uiLang] || translations?.en || notice) : "");
    statusEl.classList.remove("error");
    setBarColor("#FE4200");
    pill.classList.add("visible");
  } else if (currentState.startsWith("error")) {
    recordStart = null;
    if (contextBadgeEl) {
      contextBadgeEl.style.display = "none";
    }
    let errMsg = "Ошибка распознавания";
    if (currentState.includes(":")) {
      errMsg = currentState.substring(currentState.indexOf(":") + 1);
    }
    const uiLang = localStorage.getItem("aura_ui_lang") || "ru";
    setOverlayText(translateError(errMsg, uiLang));
    statusEl.classList.add("error");
    setBarColor("#666666");
    resetBars();
    
    // Ensure the pill is visible to display the error
    pill.classList.add("visible");

    // Play error sound and animate hide after a brief delay
    playThemeError();
    const stateAtError = currentState;
    const cycleAtError = stateCycle;
    hideTimerId = setTimeout(() => {
      hideTimerId = null;
      // Only hide if the overlay still shows this same error; a newer
      // session (recording/processing/notice) owns the overlay otherwise.
      if (currentState !== stateAtError || stateCycle !== cycleAtError) return;
      pill.classList.remove("visible");
      hideOverlayAfterPillFade(stateAtError, cycleAtError);
    }, 2500);
  }
});

// Hides the overlay window once the pill fade-out is done. With reduced
// motion the transition is disabled, so the hide must happen immediately.
// `expectedCycle` guards against a hide request from an old session firing
// after a brand-new session already took over the overlay.
function hideOverlayAfterPillFade(expectedState, expectedCycle) {
  const guardExpired = () =>
    currentState !== expectedState || stateCycle !== expectedCycle;

  const hideNow = () => {
    if (guardExpired()) return;
    if (contextBadgeEl) {
      contextBadgeEl.style.display = "none";
    }
    window.__TAURI__.core
      .invoke("hide_overlay_window")
      .catch((err) => console.error("hide_overlay_window failed:", err));
  };

  // Failsafe: if the fade transition never completes (or the event is
  // missed), still hide the window instead of leaving it stranded.
  const failsafeId = setTimeout(() => {
    pill.classList.remove("visible");
    hideNow();
  }, 1200);

  if (reduceMotionQuery.matches) {
    clearTimeout(failsafeId);
    hideNow();
    return;
  }
  pill.addEventListener("transitionend", function handler() {
    pill.removeEventListener("transitionend", handler);
    clearTimeout(failsafeId);
    hideNow();
  });
}

// Listen to the new event "hide-overlay-requested"
listen("hide-overlay-requested", (event) => {
  const payload = event.payload || {};
  const status = payload.status || "success";

  if (contextBadgeEl) {
    contextBadgeEl.style.display = "none";
  }
  updateStatusVisibility();

  if (status === "success") {
    playThemeSuccess();
  } else {
    playThemeError();
  }

  const stateAtRequest = currentState;
  const cycleAtRequest = stateCycle;
  pill.classList.remove("visible");
  // A brand-new recording may have started while the overlay was playing
  // its hide animation — never hide a newer session's pill.
  hideOverlayAfterPillFade(stateAtRequest, cycleAtRequest);
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
    let h = 6.0;
    // Make wave react to even tiny volume fluctuations
    if (volume > 0.00005) {
      const boosted = Math.sqrt(volume); // e.g. sqrt(0.001) = 0.031

      // Generate organic ripples based on time and index to simulate frequency bands
      const time = Date.now() * 0.007;
      const wave = 0.35 +
        Math.sin(time * (0.9 + index * 0.12) + index * 1.6) * 0.4 +
        Math.cos(time * 0.65 - index * 0.95) * 0.25;

      // Apply multiplier (32.0) to normalized wave coefficient
      h = 6.0 + (boosted * 32.0 * Math.max(0.05, wave));
      h = Math.min(16.0, Math.max(6.0, h));
    }
    state.targetHeight = h;
  });
});
