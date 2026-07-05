const { listen } = window.__TAURI__.event;

const bars = document.querySelectorAll(".sound-bar");
const statusEl = document.getElementById("overlay-status");
const pill = document.querySelector(".overlay-pill");
let currentState = "recording";
let recordStart = null;
let angle = 0;

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

let soundsEnabled = true;
let soundTheme = "zen";

async function updateActiveThemeSettings() {
  try {
    const settings = await window.__TAURI__.core.invoke("get_settings");
    soundsEnabled = settings.overlay_sounds !== false;
    soundTheme = settings.overlay_sound_theme || "zen";
    soundVolume = typeof settings.overlay_sound_volume === "number" ? settings.overlay_sound_volume : 0.8;
    if (globalGain) {
      globalGain.gain.setValueAtTime(soundVolume, audioCtx.currentTime);
    }
  } catch (e) {
    console.error("Failed to query settings in overlay", e);
  }
}

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
    const secs = Math.floor((Date.now() - recordStart) / 1000);
    const m = Math.floor(secs / 60);
    const s = String(secs % 60).padStart(2, "0");
    statusEl.textContent = `${m}:${s}`;
  }
}
setInterval(updateTimer, 500);

// Main animation and physics loop (60fps)
function updateAnimation() {
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

const errorTranslations = {
  "Ошибка запуска микрофона": {
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
  "Ошибка остановки записи": {
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
  "Ошибка загрузки настроек": {
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
  "Gemini недоступен в вашем регионе. Включите VPN для всех приложений или выберите Groq": {
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
  "Неверный API-ключ в настройках": {
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
  "Ошибка соединения через VPN/прокси": {
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
  "Локальная модель не скачана": {
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
  "Сбой локального Whisper-клиента": {
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
  "Лимит запросов API исчерпан": {
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
  "Баланс API ключа исчерпан": {
    en: "API key balance exhausted",
    de: "Guthaben des API-Schlüssels aufgebraucht",
    fr: "Solde de la clé API épuisé",
    it: "Credito della chiave API esaurito",
    es: "Saldo de la clave API agotado",
    pt: "Saldo da chave API esgotado",
    zh: "API 密钥余额不足",
    ja: "APIキー of 残高が不足しています",
    tr: "API anahtar bakiyesi tükendi"
  }
};

function translateError(errStr, lang) {
  if (lang === "ru") return errStr;
  const match = errorTranslations[errStr];
  if (match && match[lang]) {
    return match[lang];
  }
  if (errStr.startsWith("Нет сети:")) {
    const details = errStr.replace("Нет сети:", "").trim();
    const netTranslations = {
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
listen("recording-state", async (event) => {
  currentState = event.payload;

  if (currentState === "recording") {
    await updateActiveThemeSettings();
    recordStart = Date.now();
    statusEl.textContent = "0:00";
    statusEl.classList.remove("error");
    setBarColor("#FE4200");
    resetBars();

    // Animate show and play startup sound
    pill.classList.add("visible");
    playThemeStart();
  } else if (currentState === "processing") {
    angle = 0;
    const uiLang = localStorage.getItem("aura_ui_lang") || "ru";
    statusEl.textContent = processingTranslations[uiLang] || processingTranslations.en;
    statusEl.classList.remove("error");
  } else if (currentState.startsWith("error")) {
    recordStart = null;
    let errMsg = "Ошибка распознавания";
    if (currentState.includes(":")) {
      errMsg = currentState.substring(currentState.indexOf(":") + 1);
    }
    const uiLang = localStorage.getItem("aura_ui_lang") || "ru";
    statusEl.textContent = translateError(errMsg, uiLang);
    statusEl.classList.add("error");
    setBarColor("#666666");
    resetBars();

    // Play error sound and animate hide after a brief delay
    playThemeError();
    setTimeout(() => {
      pill.classList.remove("visible");
      // Safe hide on animation finish
      pill.addEventListener("transitionend", function handler() {
        pill.removeEventListener("transitionend", handler);
        window.__TAURI__.core.invoke("hide_overlay_window");
      });
    }, 2500);
  }
});

// Listen to the new event "hide-overlay-requested"
listen("hide-overlay-requested", (event) => {
  const payload = event.payload || {};
  const status = payload.status || "success";

  if (status === "success") {
    playThemeSuccess();
  } else {
    playThemeError();
  }

  pill.classList.remove("visible");
  pill.addEventListener("transitionend", function handler() {
    pill.removeEventListener("transitionend", handler);
    window.__TAURI__.core.invoke("hide_overlay_window");
  });
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
