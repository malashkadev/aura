/**
 * AURA — Interactive Website Core Script
 * Features:
 * - Live Dictation Typing Simulator (Original Replica with realistic overlay pill)
 * - 6-Tab Settings Showcase (Interactive Tabs, Mic Test, Hardware/Engine Pickers, Live History Search)
 * - Engine Matrix Tab Switcher & Benchmark Progress Fill
 * - Pure Acoustic Tensor Waves (High-Performance Autonomous FBM WebGL Shader with Organic Hero Mask, Zero Mouse Ripple)
 * - Hero ↔ How-it-Works Kinetic Snap Transition (Instant 0ms-Latency Response, Bidirectional Wheel/Touch/Keyboard, Auto-Hiding Indicator)
 * - Floating Morphing Navigation Capsule & Header CTA Reveal
 * - Advantages Interactive Cascading 3D Deck with Drop-Fade & Rise
 * - Floating Back to Top Button
 * - Clean Bilingual i18n Engine (Instant & Pure without Matrix Noise)
 */

document.addEventListener('DOMContentLoaded', () => {
  initLanguageAutoDetect();
  initLanguageSwitcher();
  startTypingSimulator();
  initInterfaceShowcase();
  initEngineTabs();
  initBenchmarkAnimation();
  initFluidGradientCanvas();
  initHeroScrollTransition();
  initNavCapsule();
  initHeaderCtaReveal();
  initFeaturesDeck();
  initBackToTop();
  initFooterAuraLetters();
  initDownloadModal();
  
  if (window.location.hash) {
    const target = document.querySelector(window.location.hash);
    if (target) {
      setTimeout(() => {
        if (window.auraSmoothScrollTo) {
          const header = document.querySelector('.site-header');
          const headerHeight = header ? header.offsetHeight + 14 : 74;
          const targetTop = Math.max(0, target.getBoundingClientRect().top + window.scrollY - headerHeight);
          window.auraSmoothScrollTo(targetTop, 350);
        } else {
          target.scrollIntoView();
        }
      }, 50);
    }
  }
});

/* ==========================================================================
   Live Dictation & Voice Editing Simulator (Monolith + Dual-Beam HUD)
   ========================================================================== */
let simulationRunId = 0;
let simulationTimeouts = [];

function escapeHtml(text) {
  return String(text)
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;')
    .replace(/'/g, '&#039;');
}

function startTypingSimulator() {
  const currentRunId = ++simulationRunId;
  const canvas = document.getElementById('monolith-canvas');
  const editorText = document.getElementById('demo-editor-text');
  const overlayPill = document.getElementById('mock-overlay-pill');
  const overlayCard = document.getElementById('mock-overlay-card') || (overlayPill ? overlayPill.querySelector('.mock-overlay-card') : null);
  const overlayStatus = document.getElementById('mock-overlay-status');
  const soundBars = document.querySelectorAll('#mock-overlay-pill .sound-bar');
  const kineticCursor = canvas ? canvas.querySelector('.kinetic-cursor') : null;
  
  if (!editorText || !overlayPill || !overlayStatus) return;

  // Cancel all pending timeouts
  simulationTimeouts.forEach(t => clearTimeout(t));
  simulationTimeouts = [];

  const isRu = (document.documentElement.lang || 'ru') === 'ru';
  const scenarios = isRu ? [
    {
      type: 'dictation',
      raw: "Привет, во сколько созвонимся, чтобы обсудить новый релиз",
      clean: "Привет, во сколько созвонимся, чтобы обсудить новый релиз?"
    },
    {
      type: 'edit',
      initialBefore: "",
      initialSelected: "Отправте пажалуста отчет клиенту как можно скорее.",
      initialAfter: "",
      command: "«Исправь ошибки»",
      finalText: "Отправьте, пожалуйста, отчёт клиенту как можно скорее."
    },
    {
      type: 'dictation',
      raw: "Напиши функцию валидации: проверь email, пароль и токен доступа",
      clean: "Напиши функцию валидации: проверь email, пароль и токен доступа."
    },
    {
      type: 'edit',
      initialBefore: "",
      initialSelected: "Создай асинхронный обработчик запросов с логированием ошибок.",
      initialAfter: "",
      command: "«Переведи на английский»",
      finalText: "Create an asynchronous request handler with error logging."
    },
    {
      type: 'dictation',
      raw: "Идея для фичи — добавить умную паузу, пока пользователь думает",
      clean: "Идея для фичи — добавить умную паузу, пока пользователь думает."
    },
    {
      type: 'edit',
      initialBefore: "",
      initialSelected: "Вся история транскрипций храница в открытом виде на локальном диске.",
      initialAfter: "",
      command: "«Исправь грамматику»",
      finalText: "Вся история транскрипций хранится в зашифрованном виде на локальном диске."
    }
  ] : [
    {
      type: 'dictation',
      raw: "Hey, what time can we sync to discuss the new release",
      clean: "Hey, what time can we sync to discuss the new release?"
    },
    {
      type: 'edit',
      initialBefore: "",
      initialSelected: "Plz send the reprot to the client as soon as possible.",
      initialAfter: "",
      command: "“Fix errors”",
      finalText: "Please send the report to the client as soon as possible."
    },
    {
      type: 'dictation',
      raw: "Write a validation function: check email, password, and auth token",
      clean: "Write a validation function: check email, password, and auth token."
    },
    {
      type: 'edit',
      initialBefore: "",
      initialSelected: "Function must validate the access token and expire time.",
      initialAfter: "",
      command: "“Translate to French”",
      finalText: "La fonction doit valider le jeton d'accès et le délai d'expiration."
    }
  ];

  let scenarioIndex = 0;
  let timerInterval = null;

  // Authentic Sound Wave Engine directly imported from src/overlay.js
  const restHeight = 2.5;
  const maxHeight = 16.0;
  const COLOR_SILENCE = { r: 114, g: 34, b: 5 };
  const COLOR_SPEECH = { r: 254, g: 66, b: 0 };

  function getBarHeatColor(energy) {
    const clamped = Math.max(0, Math.min(1, energy));
    const r = Math.round(COLOR_SILENCE.r + (COLOR_SPEECH.r - COLOR_SILENCE.r) * clamped);
    const g = Math.round(COLOR_SILENCE.g + (COLOR_SPEECH.g - COLOR_SILENCE.g) * clamped);
    const b = Math.round(COLOR_SILENCE.b + (COLOR_SPEECH.b - COLOR_SILENCE.b) * clamped);
    return `rgb(${r}, ${g}, ${b})`;
  }

  // Exact bar tracking from src/overlay.js
  const barStates = Array.from(soundBars).map(() => ({
    currentHeight: restHeight,
    targetHeight: restHeight
  }));

  function resetBars() {
    barStates.forEach(state => {
      state.targetHeight = restHeight;
      state.currentHeight = restHeight;
    });
    soundBars.forEach(bar => {
      bar.setAttribute("height", restHeight.toString());
      bar.setAttribute("y", (15.0 - restHeight / 2).toString());
      bar.setAttribute("rx", (restHeight / 2).toString());
      bar.setAttribute("fill", "rgb(114, 34, 5)");
    });
  }

  let waveAnimId = null;
  let waveMode = 'idle'; // 'speech' | 'processing' | 'idle'
  let waveAngle = 0;

  function runWaveLoop() {
    if (currentRunId !== simulationRunId) return;

    const now = performance.now();
    const isProc = waveMode === 'processing';
    const isSpeech = waveMode === 'speech';

    if (isSpeech) {
      // Authentic speech frequency ripples from src/overlay.js
      const volume = 0.04;
      const boosted = Math.sqrt(volume);
      const time = now * 0.007;

      barStates.forEach((state, index) => {
        const wave = 0.35 +
          Math.sin(time * (0.9 + index * 0.12) + index * 1.6) * 0.4 +
          Math.cos(time * 0.65 - index * 0.95) * 0.25;

        let h = restHeight + (boosted * 36.0 * Math.max(0.05, wave));
        h = Math.min(maxHeight, Math.max(restHeight, h));
        state.targetHeight = h;
      });
    } else if (waveMode === 'idle') {
      barStates.forEach(state => { state.targetHeight = restHeight; });
    }

    waveAngle += 0.05;

    soundBars.forEach((bar, index) => {
      const state = barStates[index];
      let blendedTarget = state.targetHeight;

      if (isProc) {
        const distFromCenter = Math.abs(index - 4.5);
        blendedTarget = 4.0 + Math.sin(waveAngle - distFromCenter * 0.45) * 3.5;
      }

      // Smooth linear interpolation (lerp) at 60fps matching src/overlay.js
      state.currentHeight += (blendedTarget - state.currentHeight) * 0.16;

      const y = 15.0 - (state.currentHeight / 2);
      bar.setAttribute("height", state.currentHeight.toFixed(2));
      bar.setAttribute("y", y.toFixed(2));
      bar.setAttribute("rx", (Math.min(1.5, state.currentHeight / 2)).toFixed(2));

      const barEnergy = Math.max(0, Math.min(1, (state.currentHeight - restHeight) / 7.0));
      const effectiveEnergy = isProc ? 1.0 : barEnergy;
      bar.setAttribute("fill", getBarHeatColor(effectiveEnergy));
    });

    waveAnimId = requestAnimationFrame(runWaveLoop);
  }

  function startEqualizerAnimation(mode = 'speech') {
    waveMode = mode;
    if (!waveAnimId) {
      waveAnimId = requestAnimationFrame(runWaveLoop);
    }
  }

  function stopAllAnimations() {
    if (kineticCursor) kineticCursor.classList.remove('hidden');
    waveMode = 'idle';
    if (waveAnimId) {
      cancelAnimationFrame(waveAnimId);
      waveAnimId = null;
    }
    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = null;
    }
    resetBars();
    simulationTimeouts.forEach(t => clearTimeout(t));
    simulationTimeouts = [];
  }

  function sleep(ms) {
    return new Promise(resolve => {
      const id = setTimeout(resolve, ms);
      simulationTimeouts.push(id);
    });
  }

  async function runSimulationLoop() {
    if (currentRunId !== simulationRunId) return;
    stopAllAnimations();

    const scenario = scenarios[scenarioIndex % scenarios.length];
    scenarioIndex++;

    // Clean reset of editor state
    editorText.innerHTML = "";
    if (canvas) canvas.classList.remove('recording', 'processing');
    overlayPill.classList.remove('active', 'edit-mode', 'processing');
    if (overlayCard) {
      overlayCard.classList.remove('edit-context', 'processing');
      overlayCard.style.setProperty('--beam-opacity', '0');
    }
    overlayStatus.textContent = "0:00";
    resetBars();
    
    if (scenario.type === 'dictation') {
      // SCENARIO 1: DIRECT VOICE DICTATION
      await sleep(900);
      if (currentRunId !== simulationRunId) return;
      
      // Step 1: Open Dictation Pill Overlay (Zero dual beam during voice recording!)
      if (kineticCursor) kineticCursor.classList.remove('hidden');
      overlayPill.classList.add('active');
      if (overlayCard) {
        overlayCard.classList.remove('edit-context', 'processing');
        overlayCard.style.setProperty('--beam-opacity', '0');
      }
      if (canvas) canvas.classList.add('recording');
      startEqualizerAnimation('speech');
      
      let seconds = 0;
      timerInterval = setInterval(() => {
        if (currentRunId !== simulationRunId) {
          clearInterval(timerInterval);
          return;
        }
        seconds++;
        overlayStatus.textContent = `0:0${seconds}`;
      }, 1000);
      
      await sleep(600);
      if (currentRunId !== simulationRunId) return;
      
      // Step 2: Dictate phrase character by character
      for (let i = 0; i < scenario.raw.length; i++) {
        if (currentRunId !== simulationRunId) return;
        const char = scenario.raw[i];
        const token = document.createElement('span');
        token.className = 'kinetic-token';
        token.textContent = char;
        editorText.appendChild(token);
        await sleep(24 + Math.random() * 26);
      }
      
      if (currentRunId !== simulationRunId) return;
      if (timerInterval) clearInterval(timerInterval);
      await sleep(300);
      if (currentRunId !== simulationRunId) return;
      
      // Step 3: Transcription processing state (Dual rotating beam activates ONLY here!)
      if (canvas) {
        canvas.classList.remove('recording');
        canvas.classList.add('processing');
      }
      overlayPill.classList.add('processing');
      if (overlayCard) {
        overlayCard.classList.add('processing');
        overlayCard.style.setProperty('--beam-opacity', '0.85');
      }
      overlayStatus.textContent = isRu ? "Вставка…" : "Transcribing...";
      
      startEqualizerAnimation('processing');
      await sleep(800);
      if (currentRunId !== simulationRunId) return;
      
      // Step 4: Final punctuation & insertion completed
      editorText.textContent = scenario.clean;
      
      stopAllAnimations();
      if (canvas) canvas.classList.remove('recording', 'processing');
      overlayPill.classList.remove('active', 'processing');
      if (overlayCard) {
        overlayCard.classList.remove('processing');
        overlayCard.style.setProperty('--beam-opacity', '0');
      }
      
      await sleep(3400);
      if (currentRunId !== simulationRunId) return;
      runSimulationLoop();

    } else {
      // SCENARIO 2: VOICE EDITING OF SELECTED TEXT
      // Step 1: Smooth appearance of initial rough text (keystroke stream with kinetic cursor)
      editorText.innerHTML = "";
      if (kineticCursor) kineticCursor.classList.remove('hidden');
      const fullInitialText = scenario.initialBefore + scenario.initialSelected + scenario.initialAfter;
      const initialStep = 3;
      for (let i = 0; i <= fullInitialText.length; i += initialStep) {
        if (currentRunId !== simulationRunId) return;
        editorText.textContent = fullInitialText.slice(0, i);
        await sleep(16);
      }
      editorText.textContent = fullInitialText;
      await sleep(750);
      if (currentRunId !== simulationRunId) return;

      // Step 2: Progressive Left-to-Right Selection (Smooth & fast, Caret suppressed)
      if (kineticCursor) kineticCursor.classList.add('hidden');
      const targetText = scenario.initialSelected;
      const charDelay = Math.max(10, Math.min(16, Math.round(750 / targetText.length)));
      for (let i = 1; i <= targetText.length; i++) {
        if (currentRunId !== simulationRunId) return;
        const selected = escapeHtml(targetText.slice(0, i));
        const unselected = escapeHtml(targetText.slice(i));
        editorText.innerHTML = escapeHtml(scenario.initialBefore) + 
                               '<span class="real-text-selection">' + selected + '</span>' + 
                               unselected + 
                               escapeHtml(scenario.initialAfter);
        await sleep(charDelay);
      }
      await sleep(350);
      if (currentRunId !== simulationRunId) return;

      // Step 3: Overlay pops up with Diffuse Contour Glow (.edit-context, ZERO dual beam!)
      if (overlayCard) {
        overlayCard.classList.remove('processing');
        overlayCard.classList.add('edit-context');
        overlayCard.style.setProperty('--beam-opacity', '0');
      }
      overlayPill.classList.add('active', 'edit-mode');
      if (canvas) canvas.classList.add('recording');
      // Real Aura 1:1 Achromatic Recording Timer (0:01)
      overlayStatus.textContent = "0:01";

      // Authentic speech wave matching dictation mode exactly!
      startEqualizerAnimation('speech');
      await sleep(1100);
      if (currentRunId !== simulationRunId) return;

      // Step 4: Switch to AI command processing (Dual rotating beam activates ONLY here!)
      if (canvas) {
        canvas.classList.remove('recording');
        canvas.classList.add('processing');
      }
      overlayPill.classList.remove('edit-mode');
      overlayPill.classList.add('processing');
      if (overlayCard) {
        overlayCard.classList.remove('edit-context');
        overlayCard.classList.add('processing');
        overlayCard.style.setProperty('--beam-opacity', '0.85');
      }
      // Real Aura 1:1 Achromatic Processing Status
      overlayStatus.textContent = isRu ? "Обработка…" : "Processing...";

      startEqualizerAnimation('processing');
      await sleep(850);
      if (currentRunId !== simulationRunId) return;

      // Step 5: Replace text seamlessly with fast keystroke stream (Aura 1:1 Win32 Hook Emulation)
      stopAllAnimations();
      if (canvas) canvas.classList.remove('recording', 'processing');
      overlayPill.classList.remove('active', 'edit-mode', 'processing');
      if (overlayCard) {
        overlayCard.classList.remove('edit-context', 'processing');
        overlayCard.style.setProperty('--beam-opacity', '0');
      }

      editorText.innerHTML = "";
      if (kineticCursor) kineticCursor.classList.remove('hidden');

      const replacementText = scenario.finalText;
      const stepSize = 2;
      for (let i = 0; i <= replacementText.length; i += stepSize) {
        if (currentRunId !== simulationRunId) return;
        editorText.textContent = replacementText.slice(0, i);
        await sleep(14);
      }
      editorText.textContent = replacementText;

      await sleep(3800);
      if (currentRunId !== simulationRunId) return;
      runSimulationLoop();
    }
  }

  stopAllAnimations();
  editorText.innerHTML = "";
  if (canvas) canvas.classList.remove('recording', 'processing');
  overlayPill.classList.remove('active', 'edit-mode', 'processing');
  if (overlayCard) {
    overlayCard.classList.remove('edit-context', 'processing');
    overlayCard.style.setProperty('--beam-opacity', '0');
  }
  overlayStatus.textContent = "0:00";
  resetBars();

  runSimulationLoop();
}

/* ==========================================================================
   Interface Showcase Mockup Interactivity (Authentic Desktop UI)
   ========================================================================== */
function initInterfaceShowcase() {
  const showcase = document.querySelector(".app-showcase-window");
  if (!showcase) return;

  const tabs = Array.from(showcase.querySelectorAll(".nav-tab[data-tab]"));
  const panels = Array.from(showcase.querySelectorAll(".tab-panel"));
  if (!tabs.length || !panels.length) return;

  const isRu = (document.documentElement.lang || "ru") === "ru";
  const $id = (id) => showcase.querySelector("#" + id);

  const L = isRu
    ? {
        demoInfo: "Демо-превью окна настроек Aura",
        autostartOn: "Автозапуск при старте Windows включён",
        autostartOff: "Автозапуск выключен",
        uiLangChanged: "Язык интерфейса: ",
        micTest: "Проверить",
        micStop: "Остановить",
        modeCloud: "Быстрое распознавание через внешние нейросети",
        modeLocal: "Полная приватность, работает автономно на ПК",
        installedTitle: "Установлено",
        downloadLabel: "Скачать",
        deleteTitle: "Удалить",
        modelLoaded: " загружена и выбрана",
        modelDeleted: "Модель удалена",
        vramMap: { "15": "15 мин", "30": "30 мин", "60": "1 час", "0": "никогда" },
        vramDefault: "30 мин",
        gamingOn: "вкл.",
        gamingAuto: "авто",
        gamingOff: "выкл.",
        repsEmpty: "Нет правил автозамены",
        repMissing: "Укажите слово и замену",
        repAdded: "Правило автозамены добавлено",
        modeAuto: "Авто",
        modeLocalLabel: "Локальный",
        modeCloudLabel: "Облачный",
        pressCombo: "Нажмите комбинацию...",
        pressPrefix: "Нажмите",
        hotkeyUpdated: "Горячая клавиша обновлена: ",
        hotkeyReset: "Сброшено на Alt + V",
        micDevices: [
          ["quadcast", "Микрофон (HyperX QuadCast S)"],
          ["realtek", "Микрофон (Realtek(R) Audio)"]
        ],
        commands: [
          { id: "fix_grammar", name: "Исправление грамматики", chips: ["«исправь ошибки»", "«проверь текст»", "«исправь пунктуацию»"] },
          { id: "translate", name: "Перевод текста", chips: ["«переведи»", "«переведи на английский»", "«переведи на немецкий»"] }
        ],
        historySamples: [
          { day: "Сегодня", time: "14:22", mode: "local", engine: "Локально", dur: "3.4с", text: "Отправил пул-реквест с оптимизацией конвейера распознавания." },
          { day: "Сегодня", time: "11:05", mode: "cloud", engine: "Cloud AI", dur: "4.1с", text: "Привет! Созвонимся в пятницу после обеда, обсудим детали релиза." },
          { day: "Вчера", time: "19:48", mode: "local", engine: "Локально", dur: "2.8с", text: "Не забыть обновить зависимости Cargo и прогнать интеграционные тесты." }
        ],
        histEmptyCleared: "История пуста. Нажмите «Восстановить», чтобы вернуть примеры.",
        histEmptySearch: "Ничего не найдено по вашему запросу.",
        histRestore: "Восстановить",
        histClear: "Очистить историю",
        histClearedToast: "История очищена",
        histRestoredToast: "История восстановлена",
        copyTitle: "Скопировать",
        copiedToast: "Текст скопирован в буфер",
        onboardingToast: "Вводный экскурс запускается в приложении",
        whatsNewToast: "Aura v1.2.5: Parakeet TDT, Qwen3 и голосовая правка",
        exitToast: "Завершение работы отключено в веб-демо",
        checkingUpdates: "Проверка...",
        upToDateBtn: "Установлена актуальная версия 1.2.5",
        upToDateToast: "Обновлений нет (v1.2.5 актуальна)",
        diagCopied: "Отчёт диагностики скопирован в буфер",
        exportedToast: "Настройки экспортированы в JSON (без API-ключей)",
        importToast: "Импорт конфигурации из JSON доступен в программе",
        savedStatus: "Сохранено",
        savedToast: "Настройки сохранены"
      }
    : {
        demoInfo: "Interactive preview of Aura settings",
        autostartOn: "Launch at Windows startup enabled",
        autostartOff: "Launch at Windows startup disabled",
        uiLangChanged: "Interface language: ",
        micTest: "Check",
        micStop: "Stop",
        modeCloud: "Fast transcription via external cloud APIs",
        modeLocal: "Full privacy, runs locally on your PC",
        installedTitle: "Installed",
        downloadLabel: "Download",
        deleteTitle: "Delete",
        modelLoaded: " downloaded and selected",
        modelDeleted: "Model removed",
        vramMap: { "15": "15 min", "30": "30 min", "60": "1 hour", "0": "never" },
        vramDefault: "30 min",
        gamingOn: "on",
        gamingAuto: "auto",
        gamingOff: "off",
        repsEmpty: "No replacement rules yet",
        repMissing: "Enter both trigger and replacement",
        repAdded: "Replacement rule added",
        modeAuto: "Auto",
        modeLocalLabel: "Local",
        modeCloudLabel: "Cloud",
        pressCombo: "Press key combination...",
        pressPrefix: "Press",
        hotkeyUpdated: "Hotkey updated: ",
        hotkeyReset: "Reset to Alt + V",
        micDevices: [
          ["quadcast", "Microphone (HyperX QuadCast S)"],
          ["realtek", "Microphone (Realtek(R) Audio)"]
        ],
        commands: [
          { id: "fix_grammar", name: "Grammar & Spelling Fix", chips: ['"fix grammar"', '"check text"', '"fix punctuation"'] },
          { id: "translate", name: "Voice Translation", chips: ['"translate"', '"translate to English"', '"translate to German"'] }
        ],
        historySamples: [
          { day: "Today", time: "14:22", mode: "local", engine: "Local", dur: "3.4s", text: "Pushed the pull request optimizing the local audio recognition pipeline." },
          { day: "Today", time: "11:05", mode: "cloud", engine: "Cloud AI", dur: "4.1s", text: "Hey! Let's sync on Friday afternoon to review the final release notes." },
          { day: "Yesterday", time: "19:48", mode: "local", engine: "Local", dur: "2.8s", text: "Remember to update Cargo dependencies and run integration tests." }
        ],
        histEmptyCleared: 'History is empty. Click "Restore" to bring back samples.',
        histEmptySearch: "No matches found for your search.",
        histRestore: "Restore",
        histClear: "Clear History",
        histClearedToast: "History cleared",
        histRestoredToast: "History restored",
        copyTitle: "Copy",
        copiedToast: "Copied to clipboard",
        onboardingToast: "Guided tour runs inside the desktop app",
        whatsNewToast: "Aura v1.2.5: Parakeet TDT, Qwen3 & Voice Editing",
        exitToast: "Exit is disabled in the web preview",
        checkingUpdates: "Checking...",
        upToDateBtn: "You are on the latest version 1.2.5",
        upToDateToast: "No updates available (v1.2.5 is current)",
        diagCopied: "Diagnostic report copied to clipboard",
        exportedToast: "Settings exported as JSON (without API keys)",
        importToast: "Importing JSON configuration is available in the desktop app",
        savedStatus: "Saved",
        savedToast: "Settings saved"
      };

  function getIcon(name, opts) {
    if (!window.AuraIcons) return "";
    const options = opts || {};
    if (String(name).startsWith("provider-") && window.AuraIcons.brandSvg) {
      const branded = window.AuraIcons.brandSvg(name, options);
      if (branded) return branded;
    }
    return window.AuraIcons.svg ? window.AuraIcons.svg(name, options) : "";
  }

  // ---------- Toast ----------
  let toastTimer = null;
  function showToast(text, kind) {
    const toast = $id("app-toast");
    const toastIcon = $id("toast-icon");
    const toastText = $id("toast-text");
    if (!toast || !toastText) return;
    if (toastTimer) {
      clearTimeout(toastTimer);
      toastTimer = null;
    }
    toastText.textContent = text;
    if (toastIcon) {
      const iconName = kind === "info" ? "info" : "check";
      toastIcon.innerHTML = getIcon(iconName, { size: 16 });
    }
    toast.classList.remove("is-leaving");
    toast.removeAttribute("hidden");
    toastTimer = setTimeout(hideToast, 2600);
  }

  function hideToast() {
    const toast = $id("app-toast");
    if (!toast || toast.hasAttribute("hidden")) return;
    toast.classList.add("is-leaving");
    setTimeout(() => {
      toast.setAttribute("hidden", "");
      toast.classList.remove("is-leaving");
    }, 150);
  }

  $id("toast-close")?.addEventListener("click", hideToast);

  // ---------- Sliding indicators & Tabs ----------
  function positionSlidingIndicator(indicator, left, top, width, height, immediate) {
    if (!indicator || width <= 0 || height <= 0) return;
    if (immediate) indicator.style.transition = "none";
    indicator.style.transform = "translate3d(" + left + "px, " + top + "px, 0)";
    indicator.style.width = width + "px";
    indicator.style.height = height + "px";
    indicator.style.opacity = "1";
    if (immediate) {
      void indicator.offsetHeight;
      indicator.style.transition = "";
    }
  }

  const navContainer = showcase.querySelector(".settings-nav");
  const navIndicator = $id("nav-indicator");

  function updateNavIndicator(tab, immediate) {
    if (!navIndicator || !navContainer || !tab) return;
    const navRect = navContainer.getBoundingClientRect();
    const tabRect = tab.getBoundingClientRect();
    if (tabRect.width <= 0 || tabRect.height <= 0) return;
    navContainer.classList.add("has-indicator");
    positionSlidingIndicator(
      navIndicator,
      tabRect.left - navRect.left + navContainer.scrollLeft,
      tabRect.top - navRect.top + navContainer.scrollTop,
      tabRect.width,
      tabRect.height,
      Boolean(immediate)
    );
  }

  const engineTrack = $id("engine-segmented-track");
  const engineIndicator = $id("engine-indicator");
  function updateEngineIndicator(immediate) {
    if (!engineTrack || !engineIndicator) return;
    const checked = engineTrack.querySelector('input[type="radio"]:checked');
    const seg = checked && checked.closest(".engine-segment");
    if (!seg || seg.offsetWidth <= 0) return;
    engineTrack.classList.add("has-indicator");
    positionSlidingIndicator(
      engineIndicator,
      seg.offsetLeft,
      seg.offsetTop,
      seg.offsetWidth,
      seg.offsetHeight,
      Boolean(immediate)
    );
  }

  const historyFilters = $id("history-filters");
  const historyFilterIndicator = $id("history-filter-indicator");
  function updateHistoryFilterIndicator(immediate) {
    if (!historyFilters || !historyFilterIndicator) return;
    const active = historyFilters.querySelector(".history-filter-btn.active");
    if (!active || active.offsetWidth <= 0) return;
    historyFilters.classList.add("has-indicator");
    positionSlidingIndicator(
      historyFilterIndicator,
      active.offsetLeft,
      active.offsetTop,
      active.offsetWidth,
      active.offsetHeight,
      Boolean(immediate)
    );
  }

  function triggerTabStagger(panel) {
    if (!panel) return;
    const targets = panel.querySelectorAll(":scope > .card, :scope > #local-engine-section > .summary-block, :scope > .summary-block");
    targets.forEach((el, idx) => {
      el.style.setProperty("--stagger-i", String(Math.min(idx, 6)));
    });
    panel.classList.remove("is-tab-entering");
    void panel.offsetWidth;
    panel.classList.add("is-tab-entering");
  }

  function activateTab(tabName, animate) {
    let activeTabEl = null;
    let activePanelEl = null;
    tabs.forEach((btn) => {
      const isMatch = btn.dataset.tab === tabName;
      btn.classList.toggle("active", isMatch);
      btn.setAttribute("aria-selected", isMatch ? "true" : "false");
      if (isMatch) activeTabEl = btn;
    });
    panels.forEach((panel) => {
      const isMatch = panel.id === "panel-" + tabName;
      panel.style.display = isMatch ? "flex" : "none";
      panel.classList.toggle("active", isMatch);
      if (isMatch) activePanelEl = panel;
    });

    closeSelectPanels(true);
    updateNavIndicator(activeTabEl, !animate);
    if (animate && activePanelEl) triggerTabStagger(activePanelEl);

    if (tabName === "speech") {
      requestAnimationFrame(() => updateEngineIndicator(true));
    } else if (tabName === "history") {
      requestAnimationFrame(() => updateHistoryFilterIndicator(true));
    }
  }

  tabs.forEach((btn) => {
    btn.addEventListener("click", (e) => {
      e.preventDefault();
      if (btn.classList.contains("active")) return;
      activateTab(btn.dataset.tab, true);
    });
  });

  // ---------- Custom Select Panels (Desktop Parity) ----------
  function getSelectOptionIconKey(selectId, optionValue) {
    if (!selectId || !optionValue) return null;
    const id = selectId.toLowerCase();
    const val = String(optionValue).toLowerCase();
    if (id.includes("provider")) return "provider-" + val;
    if (id.includes("local-engine")) return "engine-" + val;
    if (id.includes("edit-mode")) {
      if (val === "auto") return "action-sparkles";
      if (val === "local") return "cpu";
      if (val === "cloud") return "cloud";
      return null;
    }
    if (id.includes("sound-theme") || id.includes("sound")) return "sound-" + val;
    if (id.includes("lang") || id.includes("language") || id.includes("translate-target")) return "lang-" + val;
    if (id.includes("audio-device")) return "mic";
    return null;
  }

  function syncPanelSelection(select) {
    const wrap = select.closest(".select-wrap");
    const panel = wrap && wrap.querySelector(".select-panel");
    if (!wrap) return;
    if (wrap._syncSelectValue) wrap._syncSelectValue();
    if (!panel) return;
    const value = select.value;
    panel.querySelectorAll(".select-panel-item").forEach((item) => {
      const isSelected = item.dataset.value === value;
      item.classList.toggle("is-selected", isSelected);
      item.setAttribute("aria-selected", isSelected ? "true" : "false");
    });
  }

  function closeSelectPanels(immediate) {
    showcase.querySelectorAll(".select-panel.open, .select-panel.closing").forEach((panel) => {
      const wrap = panel.closest(".select-wrap");
      if (wrap) wrap.classList.remove("select-active");
      if (panel._closeTimer) {
        clearTimeout(panel._closeTimer);
        panel._closeTimer = null;
      }
      if (immediate) {
        if (wrap) wrap.classList.remove("select-closing");
        panel.classList.remove("open", "closing");
        panel.style.display = "none";
        return;
      }
      if (wrap) wrap.classList.add("select-closing");
      panel.classList.remove("open");
      panel.classList.add("closing");
      panel._closeTimer = setTimeout(() => {
        panel._closeTimer = null;
        if (wrap) wrap.classList.remove("select-closing");
        if (panel.classList.contains("closing")) {
          panel.classList.remove("closing");
          panel.style.display = "none";
        }
      }, 150);
    });
    showcase.querySelectorAll("select.custom-select[aria-expanded]").forEach((el) => {
      el.setAttribute("aria-expanded", "false");
    });
  }

  function toggleSelectPanel(select) {
    const wrap = select.closest(".select-wrap");
    const panel = wrap && wrap.querySelector(".select-panel");
    if (!panel) return;
    const willOpen = !panel.classList.contains("open");
    closeSelectPanels(false);
    if (willOpen) {
      if (panel._closeTimer) {
        clearTimeout(panel._closeTimer);
        panel._closeTimer = null;
      }
      wrap.classList.remove("select-closing");
      panel.classList.remove("closing");
      syncPanelSelection(select);
      panel.style.display = "block";
      void panel.offsetHeight;
      panel.classList.add("open");
      wrap.classList.add("select-active");
      select.setAttribute("aria-expanded", "true");
    }
  }

  function pickSelectValue(select, value) {
    if (select.value !== value) {
      select.value = value;
      select.dispatchEvent(new Event("change", { bubbles: true }));
    }
    syncPanelSelection(select);
    closeSelectPanels(false);
  }

  function buildSelectPanel(select) {
    const wrap = select.closest(".select-wrap");
    const panel = wrap && wrap.querySelector(".select-panel");
    if (!panel) return;
    panel.textContent = "";

    const head = document.createElement("div");
    head.className = "select-panel-head";
    head.setAttribute("aria-hidden", "true");
    head.addEventListener("mousedown", (e) => {
      e.preventDefault();
      e.stopPropagation();
      closeSelectPanels(false);
    });
    panel.appendChild(head);

    const body = document.createElement("div");
    body.className = "select-panel-body";
    panel.appendChild(body);

    const track = document.createElement("div");
    track.className = "select-panel-track";
    body.appendChild(track);

    const divider = document.createElement("div");
    divider.className = "select-panel-divider";
    track.appendChild(divider);

    const list = document.createElement("div");
    list.className = "select-panel-list";
    track.appendChild(list);

    Array.from(select.options).forEach((option, idx) => {
      if (option.disabled) return;
      const item = document.createElement("div");
      item.className = "select-panel-item";
      item.id = (select.id || panel.id || "select") + "-option-" + idx;
      item.dataset.value = option.value;
      item.setAttribute("role", "option");

      const iconKey = getSelectOptionIconKey(select.id, option.value);
      const langCode = iconKey && iconKey.startsWith("lang-") ? iconKey.slice(5) : null;
      let iconSvg = null;
      if (langCode === "auto") iconSvg = getIcon("globe", { size: 16, className: "select-panel-item-icon" });
      else if (langCode === "layout") iconSvg = getIcon("keyboard", { size: 16, className: "select-panel-item-icon" });
      else if (iconKey && !langCode) iconSvg = getIcon(iconKey, { size: 16, className: "select-panel-item-icon" });

      if (iconSvg) {
        item.classList.add("has-icon");
        const iconWrap = document.createElement("span");
        iconWrap.className = "select-panel-item-icon";
        iconWrap.setAttribute("aria-hidden", "true");
        iconWrap.innerHTML = iconSvg;
        item.appendChild(iconWrap);
      } else if (langCode) {
        const badge = document.createElement("span");
        badge.className = "select-panel-item-code";
        badge.textContent = langCode.toUpperCase();
        item.appendChild(badge);
      }

      const main = document.createElement("div");
      main.className = "select-panel-item-main";
      const name = document.createElement("span");
      name.className = "select-panel-item-name";
      name.textContent = option.textContent.trim();
      main.appendChild(name);

      const check = document.createElement("span");
      check.className = "select-panel-item-check";
      check.innerHTML = getIcon("check", { size: 15, strokeWidth: 2.2 });

      item.appendChild(main);
      item.appendChild(check);
      item.addEventListener("click", () => pickSelectValue(select, option.value));
      list.appendChild(item);
    });
    syncPanelSelection(select);
  }

  function initSelectPanels() {
    const audioSelect = $id("select-audio-device");
    if (audioSelect && audioSelect.options.length === 1) {
      L.micDevices.forEach(([val, label]) => {
        const opt = document.createElement("option");
        opt.value = val;
        opt.textContent = label;
        audioSelect.appendChild(opt);
      });
    }

    showcase.querySelectorAll("select.custom-select").forEach((select) => {
      const wrap = select.closest(".select-wrap");
      const panel = wrap && wrap.querySelector(".select-panel");
      if (!wrap || !panel) return;
      if (wrap.querySelector(".select-catcher")) return;

      const catcher = document.createElement("div");
      catcher.className = "select-catcher";
      wrap.insertBefore(catcher, select);
      catcher.addEventListener("mousedown", (e) => {
        if (e.button !== 0) return;
        e.preventDefault();
        toggleSelectPanel(select);
      });

      const chevron = document.createElement("span");
      chevron.className = "select-chevron";
      chevron.setAttribute("aria-hidden", "true");
      chevron.innerHTML = getIcon("chevron-down", { size: 14, strokeWidth: 2 });
      wrap.appendChild(chevron);

      const valueSpan = document.createElement("span");
      valueSpan.className = "select-value";
      valueSpan.setAttribute("aria-hidden", "true");
      const syncValue = () => {
        const opt = select.selectedOptions[0];
        const text = opt ? opt.textContent.trim() : "";
        valueSpan.textContent = text;
        wrap.classList.toggle("is-enhanced", Boolean(text));
      };
      syncValue();
      select.addEventListener("change", syncValue);
      wrap._syncSelectValue = syncValue;
      wrap.appendChild(valueSpan);

      buildSelectPanel(select);
    });

    document.addEventListener("mousedown", (e) => {
      if (!e.target.closest(".app-showcase-window .select-wrap")) closeSelectPanels(false);
    });
  }

  // ---------- Window header & General ----------
  $id("btn-window-minimize")?.addEventListener("click", () => showToast(L.demoInfo, "info"));
  $id("btn-window-close")?.addEventListener("click", () => showToast(L.demoInfo, "info"));

  $id("checkbox-autostart")?.addEventListener("change", (e) => {
    showToast(e.target.checked ? L.autostartOn : L.autostartOff);
  });

  $id("select-ui-lang")?.addEventListener("change", (e) => {
    const label = e.target.selectedOptions[0]?.textContent.trim() || e.target.value;
    showToast(L.uiLangChanged + label);
  });

  // ---------- Speech: Microphone VU Meter ----------
  let micTesting = false;
  let micRaf = null;
  let micLevel = 0;
  let micPeak = 0;
  const btnMicTest = $id("btn-toggle-mic-meter");
  const micCollapsible = $id("mic-meter-collapsible");
  const micFill = $id("mic-meter-fill");
  const micPeakEl = $id("mic-meter-peak");

  function stopMicMeter() {
    micTesting = false;
    if (micRaf) {
      cancelAnimationFrame(micRaf);
      micRaf = null;
    }
    if (btnMicTest) {
      btnMicTest.classList.remove("active");
      btnMicTest.textContent = L.micTest;
    }
    if (micCollapsible) micCollapsible.classList.remove("is-active");
    if (micFill) micFill.style.width = "0%";
    if (micPeakEl) micPeakEl.style.opacity = "0";
  }

  function tickMicMeter(ts) {
    if (!micTesting) return;
    const wave =
      Math.sin(ts * 0.009) * 18 +
      Math.sin(ts * 0.023) * 14 +
      Math.cos(ts * 0.0041) * 16 +
      40;
    const target = Math.max(8, Math.min(86, wave));
    micLevel = micLevel * 0.72 + target * 0.28;
    micPeak = Math.max(micLevel, micPeak - 0.35);
    if (micFill) micFill.style.width = micLevel.toFixed(1) + "%";
    if (micPeakEl) {
      micPeakEl.style.left = Math.min(98, micPeak).toFixed(1) + "%";
      micPeakEl.style.opacity = "0.9";
    }
    micRaf = requestAnimationFrame(tickMicMeter);
  }

  btnMicTest?.addEventListener("click", () => {
    if (micTesting) {
      stopMicMeter();
    } else {
      micTesting = true;
      micLevel = 16;
      micPeak = 20;
      btnMicTest.classList.add("active");
      btnMicTest.textContent = L.micStop;
      micCollapsible?.classList.add("is-active");
      micRaf = requestAnimationFrame(tickMicMeter);
    }
  });

  // ---------- Speech: Cloud vs Local ----------
  const radioCloud = $id("radio-cloud");
  const radioLocal = $id("radio-local");
  const engineCaptionDesc = $id("engine-caption-desc");
  const cardApiKeys = $id("card-api-keys");
  const cardCloudFallback = $id("card-cloud-fallback");
  const localEngineSection = $id("local-engine-section");

  function applyTranscriptionMode(mode) {
    const isCloud = mode === "cloud";
    if (engineCaptionDesc) {
      engineCaptionDesc.textContent = isCloud ? L.modeCloud : L.modeLocal;
    }
    if (cardApiKeys) cardApiKeys.style.display = isCloud ? "flex" : "none";
    if (cardCloudFallback) cardCloudFallback.style.display = isCloud ? "flex" : "none";
    if (localEngineSection) localEngineSection.style.display = isCloud ? "none" : "flex";
    updateEngineIndicator(false);
  }

  radioCloud?.addEventListener("change", () => radioCloud.checked && applyTranscriptionMode("cloud"));
  radioLocal?.addEventListener("change", () => radioLocal.checked && applyTranscriptionMode("local"));

  const selectProvider = $id("select-provider");
  const customProviderUrl = $id("custom-provider-url-group");
  const customProviderModel = $id("custom-provider-model-group");
  const linkGetKey = $id("link-get-key");
  const providerLinks = {
    gemini: "https://aistudio.google.com/",
    openai: "https://platform.openai.com/api-keys",
    groq: "https://console.groq.com/keys",
    huggingface: "https://huggingface.co/settings/tokens",
    custom: ""
  };
  selectProvider?.addEventListener("change", () => {
    const val = selectProvider.value;
    const isCustom = val === "custom";
    if (customProviderUrl) customProviderUrl.style.display = isCustom ? "flex" : "none";
    if (customProviderModel) customProviderModel.style.display = isCustom ? "flex" : "none";
    if (linkGetKey) {
      linkGetKey.style.display = isCustom ? "none" : "inline-flex";
      if (!isCustom) linkGetKey.href = providerLinks[val] || "#";
    }
  });

  function bindKeyVisibility(btnId, inputId) {
    const btn = $id(btnId);
    const input = $id(inputId);
    if (!btn || !input) return;
    btn.addEventListener("click", () => {
      const show = input.type === "password";
      input.type = show ? "text" : "password";
      btn.classList.toggle("visible", show);
      btn.innerHTML = getIcon(show ? "eye-off" : "eye", { size: 18, className: "eye-icon" });
    });
  }
  bindKeyVisibility("btn-toggle-key", "input-api-key");
  bindKeyVisibility("btn-toggle-key-edit", "input-edit-api-key");

  // ---------- Disclosure Accordions ----------
  function bindDisclosure(btnId, bodyId, initialOpen) {
    const btn = $id(btnId);
    const body = $id(bodyId);
    if (!btn || !body) return;
    const block = btn.closest(".summary-block");
    if (initialOpen) {
      body.classList.add("is-expanded");
      btn.setAttribute("aria-expanded", "true");
      if (block) block.classList.add("is-open");
    }
    btn.addEventListener("click", () => {
      const open = body.classList.toggle("is-expanded");
      btn.setAttribute("aria-expanded", open ? "true" : "false");
      if (block) block.classList.toggle("is-open", open);
    });
  }

  bindDisclosure("btn-local-model-summary", "local-model-body", false);
  bindDisclosure("btn-performance-summary", "performance-body", false);
  bindDisclosure("btn-commands-summary", "commands-body", false);
  bindDisclosure("btn-edit-engine-summary", "edit-engine-body", false);

  // ---------- Local Models, GPU & Qwen3 State ----------
  const MODEL_LABELS = {
    tiny: "Whisper Tiny",
    base: "Whisper Base",
    small: "Whisper Small",
    medium: "Whisper Medium",
    "large-v3-turbo-q5_0": "Large v3 Turbo (Q5)",
    "large-v3-turbo": "Large v3 Turbo",
    "parakeet-v3": "Parakeet TDT",
    "qwen3-0.6b": "Qwen3 0.6B",
    "qwen3-1.7b": "Qwen3 1.7B",
    "qwen3-4b": "Qwen3 4B"
  };

  const installedModels = new Set(["base", "large-v3-turbo-q5_0", "qwen3-1.7b"]);
  const installedGpus = new Set(["cpu", "cuda"]);
  let activeWhisperModel = "base";
  let activeParakeetModel = "parakeet-v3";
  let activeLlmModel = "qwen3-1.7b";
  let activeGpu = "cpu";

  function readyBadgeHtml() {
    return (
      '<span class="status-ready-badge" title="' + L.installedTitle + '" aria-label="' + L.installedTitle + '">' +
      getIcon("check", { size: 12, className: "status-ready-icon", strokeWidth: 2.2 }) +
      "</span>"
    );
  }

  function downloadBtnHtml(attrName, attrVal) {
    return (
      '<button type="button" class="btn-download-card-model" data-' + attrName + '="' + attrVal + '">' +
      getIcon("download", { size: 14, className: "btn-icon" }) +
      "<span>" + L.downloadLabel + "</span></button>"
    );
  }

  function deleteBtnHtml(attrName, attrVal, disabled) {
    return (
      '<button type="button" class="btn-delete-card-model" title="' + L.deleteTitle + '" aria-label="' + L.deleteTitle + '" data-' +
      attrName + '="' + attrVal + '"' + (disabled ? " disabled" : "") + ">" +
      getIcon("trash", { size: 14, className: "btn-icon" }) +
      "</button>"
    );
  }

  function syncCardGroup(containerId, attr, installedSet, activeId, requireOneActive) {
    const container = $id(containerId);
    if (!container) return;
    const cards = container.querySelectorAll(".model-card[data-" + attr + "]");
    cards.forEach((card) => {
      const id = card.dataset[attr];
      const isInstalled = installedSet.has(id);
      const isSelected = isInstalled && id === activeId;
      card.classList.toggle("active", isSelected);
      card.setAttribute("aria-checked", isSelected ? "true" : "false");
      const actionEl = card.querySelector(".model-card-action");
      if (!actionEl) return;

      if (isInstalled) {
        let html = readyBadgeHtml();
        if (id !== "cpu") {
          html += deleteBtnHtml(attr, id, requireOneActive && isSelected);
        }
        actionEl.innerHTML = html;
      } else {
        actionEl.innerHTML = downloadBtnHtml(attr, id);
      }
    });
  }

  function simulateDownload(card, id, attr, onComplete) {
    const progressEl = card.querySelector(".model-card-progress");
    const fillEl = card.querySelector(".progress-bar-fill");
    const pctEl = card.querySelector(".progress-pct");
    if (!progressEl || !fillEl || !pctEl) {
      onComplete();
      return;
    }
    const actionBtn = card.querySelector(".btn-download-card-model");
    if (actionBtn) actionBtn.disabled = true;
    progressEl.style.display = "flex";
    let pct = 0;
    const stepTimer = setInterval(() => {
      pct = Math.min(100, pct + 18);
      fillEl.style.width = pct + "%";
      pctEl.textContent = pct + "%";
      if (pct >= 100) {
        clearInterval(stepTimer);
        setTimeout(() => {
          progressEl.style.display = "none";
          fillEl.style.width = "0%";
          onComplete();
        }, 180);
      }
    }, 90);
  }

  function updateLocalSummarySpecs() {
    const engine = $id("select-local-engine")?.value || "whisper";
    const modelEl = $id("local-model-spec-value");
    const accelEl = $id("local-accel-spec-value");
    if (modelEl) {
      modelEl.textContent = engine === "parakeet" ? "Parakeet TDT" : (MODEL_LABELS[activeWhisperModel] || "Whisper Base");
    }
    if (accelEl) {
      accelEl.textContent = activeGpu === "cuda" ? "NVIDIA CUDA" : "CPU";
    }
  }

  function renderAllModelCards() {
    syncCardGroup("gpu-modes-container", "gpu", installedGpus, activeGpu, true);
    syncCardGroup("model-cards-container", "model", installedModels, activeWhisperModel, true);
    syncCardGroup("parakeet-model-cards-container", "model", installedModels, activeParakeetModel, false);
    syncCardGroup("llm-model-cards-container", "model", installedModels, activeLlmModel, true);
    updateLocalSummarySpecs();
    updateEditEngineSummary();
  }

  function bindCardGroupEvents(containerId, attr, installedSet, getActive, setActive) {
    const container = $id(containerId);
    if (!container) return;
    container.addEventListener("click", (e) => {
      const dlBtn = e.target.closest(".btn-download-card-model");
      const delBtn = e.target.closest(".btn-delete-card-model");
      const card = e.target.closest(".model-card[data-" + attr + "]");
      if (!card) return;
      const id = card.dataset[attr];

      if (dlBtn) {
        e.stopPropagation();
        simulateDownload(card, id, attr, () => {
          installedSet.add(id);
          setActive(id);
          renderAllModelCards();
          showToast((MODEL_LABELS[id] || id.toUpperCase()) + L.modelLoaded);
        });
        return;
      }
      if (delBtn) {
        e.stopPropagation();
        if (delBtn.disabled || id === getActive()) return;
        installedSet.delete(id);
        renderAllModelCards();
        showToast(L.modelDeleted, "info");
        return;
      }
      if (installedSet.has(id) && id !== getActive()) {
        setActive(id);
        renderAllModelCards();
      }
    });
  }

  bindCardGroupEvents("gpu-modes-container", "gpu", installedGpus, () => activeGpu, (v) => { activeGpu = v; });
  bindCardGroupEvents("model-cards-container", "model", installedModels, () => activeWhisperModel, (v) => { activeWhisperModel = v; });
  bindCardGroupEvents("parakeet-model-cards-container", "model", installedModels, () => activeParakeetModel, (v) => { activeParakeetModel = v; });
  bindCardGroupEvents("llm-model-cards-container", "model", installedModels, () => activeLlmModel, (v) => { activeLlmModel = v; });

  const selectLocalEngine = $id("select-local-engine");
  const groupWhisperModels = $id("group-whisper-models");
  const groupParakeetModels = $id("group-parakeet-models");
  selectLocalEngine?.addEventListener("change", () => {
    const isParakeet = selectLocalEngine.value === "parakeet";
    if (groupWhisperModels) groupWhisperModels.style.display = isParakeet ? "none" : "flex";
    if (groupParakeetModels) groupParakeetModels.style.display = isParakeet ? "flex" : "none";
    updateLocalSummarySpecs();
  });

  // ---------- Performance Summary ----------
  const selectVram = $id("select-vram-standby");
  const cbGamingAuto = $id("checkbox-gaming-mode-auto-detect");
  const cbGamingManual = $id("checkbox-gaming-mode");
  function updatePerformanceSummary() {
    const vramEl = $id("performance-vram-spec-value");
    const gamingEl = $id("performance-gaming-spec-value");
    if (vramEl && selectVram) {
      vramEl.textContent = L.vramMap[selectVram.value] || L.vramDefault;
    }
    if (gamingEl) {
      if (cbGamingManual?.checked) gamingEl.textContent = L.gamingOn;
      else if (cbGamingAuto?.checked) gamingEl.textContent = L.gamingAuto;
      else gamingEl.textContent = L.gamingOff;
    }
  }
  selectVram?.addEventListener("change", updatePerformanceSummary);
  cbGamingAuto?.addEventListener("change", updatePerformanceSummary);
  cbGamingManual?.addEventListener("change", updatePerformanceSummary);

  // ---------- Text: Replacements, Smart Context, Selection Edit ----------
  let replacements = isRu
    ? [
        { trigger: "почта", value: "name@mail.ru|" },
        { trigger: "спс", value: "Спасибо!" }
      ]
    : [
        { trigger: "email", value: "name@mail.com|" },
        { trigger: "ty", value: "Thank you!" }
      ];
  const repList = $id("replacements-list");
  const inputRepTrigger = $id("input-rep-trigger");
  const inputRepValue = $id("input-rep-value");

  function renderReplacements() {
    if (!repList) return;
    repList.innerHTML = "";
    if (!replacements.length) {
      const empty = document.createElement("div");
      empty.className = "replacements-empty";
      empty.textContent = L.repsEmpty;
      repList.appendChild(empty);
      return;
    }
    replacements.forEach((item, idx) => {
      const row = document.createElement("div");
      row.className = "replacement-item";
      const info = document.createElement("div");
      info.className = "replacement-info";

      const trig = document.createElement("span");
      trig.className = "replacement-trigger";
      trig.textContent = item.trigger;
      const arrow = document.createElement("span");
      arrow.className = "replacement-arrow";
      arrow.setAttribute("aria-hidden", "true");
      arrow.innerHTML = getIcon("arrow-right", { size: 14 });
      const val = document.createElement("span");
      val.className = "replacement-value";
      val.textContent = item.value;

      info.appendChild(trig);
      info.appendChild(arrow);
      info.appendChild(val);

      const del = document.createElement("button");
      del.type = "button";
      del.className = "replacement-delete";
      del.title = L.deleteTitle;
      del.innerHTML = getIcon("close", { size: 14 });
      del.addEventListener("click", () => {
        row.classList.add("is-deleting");
        setTimeout(() => {
          replacements.splice(idx, 1);
          renderReplacements();
        }, 160);
      });

      row.appendChild(info);
      row.appendChild(del);
      repList.appendChild(row);
    });
  }

  $id("btn-add-replacement")?.addEventListener("click", () => {
    const trig = inputRepTrigger?.value.trim();
    const val = inputRepValue?.value.trim();
    if (!trig || !val) {
      showToast(L.repMissing, "info");
      return;
    }
    replacements.push({ trigger: trig, value: val });
    inputRepTrigger.value = "";
    inputRepValue.value = "";
    renderReplacements();
    showToast(L.repAdded);
  });

  const cbSmartContext = $id("checkbox-smart-context-enabled");
  const smartContextSub = $id("smart-context-suboptions");
  const cbTrailingPeriod = $id("checkbox-smart-context-trailing-period");
  if (cbSmartContext) cbSmartContext.checked = true;
  if (cbTrailingPeriod) cbTrailingPeriod.checked = true;
  if (smartContextSub) smartContextSub.classList.add("is-expanded");
  cbSmartContext?.addEventListener("change", () => {
    smartContextSub?.classList.toggle("is-expanded", cbSmartContext.checked);
  });

  const cbSelectionEdit = $id("checkbox-selection-edit-enabled");
  const selectionEditInner = $id("selection-edit-inner");
  if (cbSelectionEdit) cbSelectionEdit.checked = true;
  if (selectionEditInner) selectionEditInner.classList.add("is-expanded");
  cbSelectionEdit?.addEventListener("change", () => {
    selectionEditInner?.classList.toggle("is-expanded", cbSelectionEdit.checked);
  });

  function renderCommandsGuide() {
    const guide = $id("commands-info-guide");
    if (!guide) return;
    guide.innerHTML = "";
    L.commands.forEach((cmd) => {
      const item = document.createElement("div");
      item.className = "guide-item";
      const header = document.createElement("div");
      header.className = "guide-item-header";
      const iconWrap = document.createElement("span");
      iconWrap.className = "guide-item-icon";
      iconWrap.innerHTML = getIcon(cmd.id, { size: 16 });
      const title = document.createElement("span");
      title.className = "guide-item-title";
      title.textContent = cmd.name;
      header.appendChild(iconWrap);
      header.appendChild(title);

      const chips = document.createElement("div");
      chips.className = "command-chips";
      cmd.chips.forEach((c) => {
        const chip = document.createElement("span");
        chip.className = "chip";
        chip.textContent = c;
        chips.appendChild(chip);
      });
      item.appendChild(header);
      item.appendChild(chips);
      guide.appendChild(item);
    });
  }

  const selectAiEditMode = $id("select-ai-edit-mode");
  const selectEditCloudProvider = $id("select-edit-cloud-provider");
  const groupLlmModels = $id("group-llm-models");
  const groupEditCloud = $id("group-edit-cloud-provider");
  const editCustomUrl = $id("edit-custom-provider-url-group");
  const editCustomModel = $id("edit-custom-provider-model-group");
  const linkGetKeyEdit = $id("link-get-key-edit");

  function updateEditEngineSummary() {
    const mode = selectAiEditMode?.value || "auto";
    const modeLabelMap = { auto: L.modeAuto, local: L.modeLocalLabel, cloud: L.modeCloudLabel };
    const modeEl = $id("edit-engine-mode-spec-value");
    const modelEl = $id("edit-engine-model-spec-value");
    if (modeEl) modeEl.textContent = modeLabelMap[mode] || L.modeAuto;
    if (modelEl) {
      if (mode === "cloud") {
        modelEl.textContent = selectEditCloudProvider?.selectedOptions[0]?.textContent.trim() || "Gemini";
      } else {
        modelEl.textContent = MODEL_LABELS[activeLlmModel] || "Qwen3 1.7B";
      }
    }
    if (groupLlmModels) groupLlmModels.style.display = mode === "cloud" ? "none" : "flex";
    if (groupEditCloud) groupEditCloud.style.display = mode === "local" ? "none" : "flex";
  }

  selectAiEditMode?.addEventListener("change", updateEditEngineSummary);
  selectEditCloudProvider?.addEventListener("change", () => {
    const val = selectEditCloudProvider.value;
    const isCustom = val === "custom";
    if (editCustomUrl) editCustomUrl.style.display = isCustom ? "flex" : "none";
    if (editCustomModel) editCustomModel.style.display = isCustom ? "flex" : "none";
    if (linkGetKeyEdit) {
      linkGetKeyEdit.style.display = isCustom ? "none" : "inline-flex";
      if (!isCustom) linkGetKeyEdit.href = providerLinks[val] || "#";
    }
    updateEditEngineSummary();
  });

  // ---------- Hotkeys & Sounds ----------
  const inputHotkey = $id("input-hotkey");
  if (inputHotkey) inputHotkey.value = "Alt + V";
  let recordingHotkey = false;
  inputHotkey?.addEventListener("click", () => {
    recordingHotkey = true;
    inputHotkey.classList.add("recording");
    inputHotkey.value = L.pressCombo;
  });
  inputHotkey?.addEventListener("blur", () => {
    if (recordingHotkey) {
      recordingHotkey = false;
      inputHotkey.classList.remove("recording");
      if (inputHotkey.value.startsWith(L.pressPrefix)) inputHotkey.value = "Alt + V";
    }
  });
  document.addEventListener("keydown", (e) => {
    if (!recordingHotkey || !inputHotkey) return;
    e.preventDefault();
    if (e.key === "Escape") {
      recordingHotkey = false;
      inputHotkey.classList.remove("recording");
      inputHotkey.value = "Alt + V";
      inputHotkey.blur();
      return;
    }
    const parts = [];
    if (e.ctrlKey) parts.push("Ctrl");
    if (e.altKey) parts.push("Alt");
    if (e.shiftKey) parts.push("Shift");
    const k = e.key;
    if (!["Control", "Alt", "Shift", "Meta"].includes(k)) {
      parts.push(k.length === 1 ? k.toUpperCase() : k);
      inputHotkey.value = parts.join(" + ");
      recordingHotkey = false;
      inputHotkey.classList.remove("recording");
      inputHotkey.blur();
      showToast(L.hotkeyUpdated + inputHotkey.value);
    }
  });
  $id("btn-reset-hotkey")?.addEventListener("click", () => {
    recordingHotkey = false;
    if (inputHotkey) {
      inputHotkey.classList.remove("recording");
      inputHotkey.value = "Alt + V";
    }
    showToast(L.hotkeyReset);
  });

  const cbSounds = $id("checkbox-sounds");
  const soundSub = $id("sound-suboptions");
  const rangeVol = $id("range-sound-volume");
  const volLabel = $id("volume-value-label");
  if (cbSounds) cbSounds.checked = true;
  if (soundSub) soundSub.classList.add("is-expanded");
  cbSounds?.addEventListener("change", () => {
    soundSub?.classList.toggle("is-expanded", cbSounds.checked);
  });
  function syncSlider() {
    if (!rangeVol) return;
    const v = Number(rangeVol.value || 80);
    if (volLabel) volLabel.textContent = v + "%";
    rangeVol.style.background =
      "linear-gradient(to right, var(--accent-color) 0%, var(--accent-color) " +
      v + "%, hsl(0, 0%, 20%) " + v + "%, hsl(0, 0%, 20%) 100%)";
  }
  rangeVol?.addEventListener("input", syncSlider);
  syncSlider();

  // ---------- History ----------
  let historyCleared = false;
  let activeFilter = "all";

  const historyListEl = $id("history-items-container");
  const historySearchInput = $id("input-history-search");
  const btnClearSearch = $id("btn-clear-history-search");
  const btnClearHistory = $id("btn-clear-history");

  function renderHistory() {
    if (!historyListEl) return;
    const q = (historySearchInput?.value || "").trim().toLowerCase();
    if (btnClearSearch) btnClearSearch.style.display = q ? "flex" : "none";

    if (historyCleared) {
      historyListEl.innerHTML = '<div class="history-empty-state">' + L.histEmptyCleared + "</div>";
      return;
    }

    const filtered = L.historySamples.filter((item) => {
      if (activeFilter !== "all" && item.mode !== activeFilter) return false;
      if (q && !item.text.toLowerCase().includes(q)) return false;
      return true;
    });

    if (!filtered.length) {
      historyListEl.innerHTML = '<div class="history-empty-state">' + L.histEmptySearch + "</div>";
      return;
    }

    historyListEl.innerHTML = "";
    let lastDay = null;
    filtered.forEach((item) => {
      if (item.day !== lastDay) {
        lastDay = item.day;
        const dh = document.createElement("div");
        dh.className = "history-day-header";
        dh.innerHTML = '<span class="history-day-title">' + item.day + '</span><span class="history-day-line"></span>';
        historyListEl.appendChild(dh);
      }
      const row = document.createElement("div");
      row.className = "history-item";
      row.innerHTML =
        '<div class="history-item-body">' +
        '  <div class="history-item-meta">' +
        '    <span class="history-item-time">' + item.time + "</span>" +
        '    <span class="history-item-badge ' + (item.mode === "cloud" ? "badge-cloud" : "badge-local") + '">' + item.engine + "</span>" +
        '    <span class="history-item-duration">' + item.dur + "</span>" +
        "  </div>" +
        '  <div class="history-item-text"></div>' +
        "</div>" +
        '<button type="button" class="btn-copy-history" title="' + L.copyTitle + '" aria-label="' + L.copyTitle + '">' +
        getIcon("copy", { size: 16, className: "copy-icon" }) +
        getIcon("check", { size: 16, className: "check-icon" }) +
        "</button>";
      row.querySelector(".history-item-text").textContent = item.text;
      const copyBtn = row.querySelector(".btn-copy-history");
      copyBtn.addEventListener("click", () => {
        if (navigator.clipboard && navigator.clipboard.writeText) {
          navigator.clipboard.writeText(item.text).catch(() => {});
        }
        copyBtn.classList.add("copied");
        showToast(L.copiedToast);
        setTimeout(() => copyBtn.classList.remove("copied"), 1300);
      });
      historyListEl.appendChild(row);
    });
  }

  historySearchInput?.addEventListener("input", renderHistory);
  btnClearSearch?.addEventListener("click", () => {
    if (historySearchInput) historySearchInput.value = "";
    renderHistory();
  });
  showcase.querySelectorAll(".history-filter-btn[data-filter]").forEach((btn) => {
    btn.addEventListener("click", () => {
      activeFilter = btn.dataset.filter;
      showcase.querySelectorAll(".history-filter-btn").forEach((b) => b.classList.toggle("active", b === btn));
      updateHistoryFilterIndicator(false);
      renderHistory();
    });
  });
  btnClearHistory?.addEventListener("click", () => {
    historyCleared = !historyCleared;
    btnClearHistory.textContent = historyCleared ? L.histRestore : L.histClear;
    renderHistory();
    showToast(historyCleared ? L.histClearedToast : L.histRestoredToast, "info");
  });

  // ---------- About & Footer Actions ----------
  $id("btn-replay-onboarding")?.addEventListener("click", () => showToast(L.onboardingToast, "info"));
  $id("btn-whats-new")?.addEventListener("click", () => showToast(L.whatsNewToast, "info"));
  $id("btn-exit-app")?.addEventListener("click", () => showToast(L.exitToast, "info"));
  $id("btn-check-updates")?.addEventListener("click", (e) => {
    const btn = e.currentTarget;
    btn.textContent = L.checkingUpdates;
    setTimeout(() => {
      btn.textContent = L.upToDateBtn;
      showToast(L.upToDateToast);
    }, 450);
  });
  $id("btn-copy-diagnostics")?.addEventListener("click", () => showToast(L.diagCopied));
  $id("btn-export-settings")?.addEventListener("click", () => showToast(L.exportedToast));
  $id("btn-import-settings")?.addEventListener("click", () => showToast(L.importToast, "info"));

  const btnSave = $id("btn-save-settings");
  const footerStatusText = $id("footer-status-text");
  if (btnSave && footerStatusText) {
    btnSave.addEventListener("click", () => {
      const originalText = footerStatusText.textContent;
      footerStatusText.textContent = L.savedStatus;
      showToast(L.savedToast);
      setTimeout(() => {
        footerStatusText.textContent = originalText;
      }, 1500);
    });
  }

  // ---------- Init ----------
  if (typeof window.hydrateAppIcons === "function") {
    window.hydrateAppIcons(showcase);
  }
  initSelectPanels();
  renderCommandsGuide();
  renderReplacements();
  renderAllModelCards();
  updatePerformanceSummary();
  renderHistory();
  applyTranscriptionMode("local");
  activateTab("general", false);

  const syncAllIndicators = () => {
    const activeTab = showcase.querySelector(".nav-tab.active");
    if (activeTab) updateNavIndicator(activeTab, true);
    updateEngineIndicator(true);
    updateHistoryFilterIndicator(true);
  };
  window.addEventListener("resize", syncAllIndicators);
  window.addEventListener("load", syncAllIndicators);
  if (document.fonts && document.fonts.ready) {
    document.fonts.ready.then(syncAllIndicators).catch(() => {});
  }
}
/* ==========================================================================
   Engine Matrix Tab Switcher with Sliding Indicator
   ========================================================================== */
function initEngineTabs() {
  const tabsContainer = document.querySelector('.engine-tabs');
  const tabs = document.querySelectorAll('.engine-tab-btn');
  const panels = document.querySelectorAll('.engine-tab-content');
  if (!tabsContainer || !tabs.length) return;

  let indicator = tabsContainer.querySelector('.engine-tab-indicator');
  if (!indicator) {
    indicator = document.createElement('div');
    indicator.className = 'engine-tab-indicator';
    tabsContainer.prepend(indicator);
  }

  function updateIndicator(activeTab) {
    if (!activeTab) return;
    requestAnimationFrame(() => {
      indicator.style.width = `${activeTab.offsetWidth}px`;
      indicator.style.transform = `translate3d(${activeTab.offsetLeft}px, 0, 0)`;
    });
  }

  const initialActive = tabsContainer.querySelector('.engine-tab-btn.active') || tabs[0];
  requestAnimationFrame(() => updateIndicator(initialActive));

  window.addEventListener('resize', () => {
    const current = tabsContainer.querySelector('.engine-tab-btn.active');
    updateIndicator(current);
  });

  tabs.forEach(tab => {
    tab.addEventListener('click', () => {
      const target = tab.getAttribute('data-tab');

      tabs.forEach(t => t.classList.remove('active'));
      panels.forEach(p => p.classList.remove('active'));

      tab.classList.add('active');
      updateIndicator(tab);

      const targetPanel = document.getElementById(`engine-panel-${target}`);
      if (targetPanel) {
        targetPanel.classList.add('active');
      }
    });
  });
}

/* ==========================================================================
   Benchmark Visual Progress Fill
   ========================================================================== */
function initBenchmarkAnimation() {
  const fills = document.querySelectorAll('.bench-bar-fill');
  if (!fills.length) return;

  const observer = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
      if (entry.isIntersecting) {
        fills.forEach(fill => {
          const width = fill.getAttribute('data-width') || '100%';
          fill.style.width = width;
          fill.style.transition = 'width 800ms cubic-bezier(0.16, 1, 0.3, 1)';
        });
        observer.disconnect();
      }
    });
  }, { threshold: 0.2 });

  const benchmarkCard = document.querySelector('.benchmark-widget');
  if (benchmarkCard) {
    observer.observe(benchmarkCard);
  }
}

/* ==========================================================================
   Pure Acoustic Tensor Waves (WebGL Shader with Hero Scroll Fade-out)
   ========================================================================== */
function initFluidGradientCanvas() {
  const canvas = document.getElementById('aura-fluid-canvas');
  if (!canvas) return;

  const gl = canvas.getContext('webgl') || canvas.getContext('experimental-webgl');
  if (!gl) {
    return;
  }

  const vsSource = `
    attribute vec2 a_position;
    void main() {
      gl_Position = vec4(a_position, 0.0, 1.0);
    }
  `;

  const fsSource = `
    precision highp float;
    uniform vec2 u_resolution;
    uniform float u_time;
    uniform float u_shield;
    uniform float u_footer;

    float hash(vec2 p) {
      p = fract(p * vec2(123.34, 456.21));
      p += dot(p, p + 45.32);
      return fract(p.x * p.y);
    }

    float noise(vec2 p) {
      vec2 i = floor(p);
      vec2 f = fract(p);
      vec2 u = f * f * (3.0 - 2.0 * f);
      float a = hash(i + vec2(0.0, 0.0));
      float b = hash(i + vec2(1.0, 0.0));
      float c = hash(i + vec2(0.0, 1.0));
      float d = hash(i + vec2(1.0, 1.0));
      return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
    }

    float fbm(vec2 p) {
      float v = 0.0;
      float a = 0.5;
      mat2 rot = mat2(0.8, -0.6, 0.6, 0.8);
      for (int i = 0; i < 4; i++) {
        v += a * noise(p);
        p = rot * p * 2.0 + vec2(1.2, 3.4);
        a *= 0.5;
      }
      return v;
    }

    void main() {
      vec2 uv = gl_FragCoord.xy / u_resolution.xy;
      float aspect = u_resolution.x / u_resolution.y;
      vec2 p = (uv - 0.5) * 2.0;
      p.x *= aspect;

      // Slow, majestic wave tempo
      float t = u_time * 0.2;

      // 1. Digital 3D Acoustic Wave Curvature
      float angle = 0.45;
      mat2 rot = mat2(cos(angle), -sin(angle), sin(angle), cos(angle));
      vec2 rp = rot * p;

      float w1 = sin(rp.x * 1.2 + t * 0.65) * cos(rp.y * 1.0 - t * 0.5) * 0.55;
      float w2 = sin(rp.x * 2.2 - rp.y * 1.6 + t * 0.85) * 0.3;
      float w3 = cos(rp.x * 0.7 + rp.y * 1.4 + t * 0.4) * 0.35;
      float dNoise = fbm(rp * 0.7 + vec2(t * 0.06, t * 0.04)) * 0.4;
      
      float elevation = w1 + w2 + w3 + dNoise;

      // 2. Subtle Acoustic Wave Contours (Topography of sound)
      float bandVal = fract((elevation + 1.5) * 4.5);
      float bandEdge = smoothstep(0.0, 0.06, bandVal) * (1.0 - smoothstep(0.06, 0.14, bandVal));

      // 3. Palette Definition
      vec3 obsidian = vec3(0.012, 0.012, 0.012);  // Pure Achromatic Charcoal Black (Zero Blue/Indigo Tint)
      vec3 ember    = vec3(0.18, 0.035, 0.005);  // Warm Obsidian Amber
      vec3 auraCore = vec3(0.95, 0.26, 0.0);      // Pure Aura Flame (#ff4200)
      vec3 laser    = vec3(1.0, 0.58, 0.15);     // Radiant Highlight

      // Organic 2D Soft Radial Hero Mask:
      // In Hero (u_shield = 1.0), light originates naturally from the right demo card area (x ~ 0.5, y ~ 0.0)
      // and falls off with a super-wide, buttery-smooth radial curve towards the left text.
      float rightField = smoothstep(-0.95, 0.55, p.x + 0.15 * p.y);
      float heroLight = pow(rightField, 1.6);
      float textShield = mix(1.0, heroLight, u_shield);

      // Hero mode vs Darker, Subdued Ambient mode vs Warm Glowing Footer mode
      float baseElev = clamp((elevation + 0.75) / 1.9, 0.0, 1.0);
      float heroElev = baseElev * (0.2 + 0.8 * textShield);
      float midElev = baseElev * 0.65 + 0.05;
      float footerElev = baseElev * 0.88 + 0.08;
      
      float normElev = mix(mix(midElev, heroElev, u_shield), footerElev, u_footer);

      // Color blending: Obsidian base -> Dark amber ember
      vec3 col = mix(obsidian, ember, smoothstep(0.20, 0.80, normElev));
      
      // Fiery intensity:
      // Hero: vivid on right demo side (0.85 * textShield)
      // Below Hero / Mid sections: very dark, subdued ambient glow (0.16)
      // Footer: smoothly undarkens as user arrives at the footer (0.75)
      float heroFire = 0.85 * textShield;
      float midFire = 0.16;
      float footerFire = 0.75;

      float fireIntensity = mix(mix(midFire, heroFire, u_shield), footerFire, u_footer);
      col = mix(col, auraCore, pow(smoothstep(0.42, 0.94, normElev), 2.2) * fireIntensity);
      col = mix(col, laser, pow(smoothstep(0.68, 0.99, normElev), 2.6) * (fireIntensity * 0.55));

      // Fine contour lines
      float lineIntensity = mix(mix(0.025, 0.07 * textShield, u_shield), 0.065, u_footer);
      col += auraCore * (bandEdge * lineIntensity);

      // Smooth vignette to keep margins pristine
      float vignette = smoothstep(1.5, 0.2, length(uv - 0.5));
      col *= (0.70 + 0.30 * vignette);

      gl_FragColor = vec4(col, 1.0);
    }
  `;

  function createShader(type, source) {
    const shader = gl.createShader(type);
    gl.shaderSource(shader, source);
    gl.compileShader(shader);
    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
      console.error('Shader compile error:', gl.getShaderInfoLog(shader));
    }
    return shader;
  }

  const program = gl.createProgram();
  gl.attachShader(program, createShader(gl.VERTEX_SHADER, vsSource));
  gl.attachShader(program, createShader(gl.FRAGMENT_SHADER, fsSource));
  gl.linkProgram(program);
  if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
    console.error('Program link error:', gl.getProgramInfoLog(program));
  }
  gl.useProgram(program);

  // Quad geometry
  const positionBuffer = gl.createBuffer();
  gl.bindBuffer(gl.ARRAY_BUFFER, positionBuffer);
  gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([
    -1, -1,
     1, -1,
    -1,  1,
    -1,  1,
     1, -1,
     1,  1
  ]), gl.STATIC_DRAW);

  const posAttr = gl.getAttribLocation(program, 'a_position');
  gl.enableVertexAttribArray(posAttr);
  gl.vertexAttribPointer(posAttr, 2, gl.FLOAT, false, 0, 0);

  const uResolution = gl.getUniformLocation(program, 'u_resolution');
  const uTime = gl.getUniformLocation(program, 'u_time');
  const uShield = gl.getUniformLocation(program, 'u_shield');
  const uFooter = gl.getUniformLocation(program, 'u_footer');

  let currentShield = 1.0;
  let targetFooter = 0.0;
  let currentFooter = 0.0;
  let lastTime = performance.now();
  let totalElapsed = 0;
  let animFrame = null;
  let isHidden = false;

  function drawFrame() {
    gl.uniform2f(uResolution, canvas.width, canvas.height);
    gl.uniform1f(uTime, totalElapsed);
    gl.uniform1f(uShield, currentShield);
    gl.uniform1f(uFooter, currentFooter);
    gl.drawArrays(gl.TRIANGLES, 0, 6);
  }

  function resize() {
    const scale = Math.min(window.devicePixelRatio || 1, 1.5);
    const width = Math.floor(window.innerWidth * scale);
    const height = Math.floor(window.innerHeight * scale);
    if (canvas.width !== width || canvas.height !== height) {
      canvas.width = width;
      canvas.height = height;
      gl.viewport(0, 0, width, height);
      drawFrame();
    }
  }

  window.addEventListener('resize', resize, { passive: true });
  resize();

  // Check if hero section is currently visible in viewport
  function isHeroVisible() {
    const heroSection = document.querySelector('.hero-section');
    if (!heroSection) return window.scrollY < 200;
    const rect = heroSection.getBoundingClientRect();
    // Hero is active only when its bottom is still prominently visible in the upper viewport.
    // When snapped to how-it-works, rect.bottom is ~72px (behind header), so rect.bottom <= 120 means hero is off-screen.
    return rect.bottom > 140;
  }

  // Check if footer/download section is currently in view
  function isFooterVisible() {
    const finaleEl = document.getElementById('download') || document.querySelector('.site-finale-footer');
    if (!finaleEl) return false;
    const rect = finaleEl.getBoundingClientRect();
    // Only active when the download section has actually entered the viewport
    return rect.top < window.innerHeight * 0.75 && rect.bottom > 0;
  }

  // Smooth scroll handler: updates u_shield and targetFooter in real-time
  function updateScroll() {
    const scrollY = window.scrollY;

    // Dissolve the left shield smoothly as user leaves hero (1.0 at top -> 0.0 when scrolled > 300px)
    const shieldProgress = Math.max(0, Math.min(1, scrollY / 320));
    currentShield = 1.0 - shieldProgress;

    // Footer undarkening bloom: smoothly increases from 0.0 to 1.0 as user enters footer
    const finaleEl = document.getElementById('download') || document.querySelector('.site-finale-footer');
    if (finaleEl) {
      const rect = finaleEl.getBoundingClientRect();
      const enterDist = window.innerHeight * 0.85;
      targetFooter = Math.max(0, Math.min(1, (enterDist - rect.top) / (enterDist * 0.75)));
    } else {
      targetFooter = 0.0;
    }

    drawFrame();
  }

  window.addEventListener('scroll', updateScroll, { passive: true });
  updateScroll();

  function render(now) {
    if (isHidden) return;

    const isInHero = isHeroVisible();
    const isInFooter = isFooterVisible();

    // Run animation ONLY when actually inside Hero or Footer (pauses in How-it-Works, Advantages, Interface, Comparison)
    if (!isInHero && !isInFooter) {
      lastTime = now;
      animFrame = requestAnimationFrame(render);
      return;
    }

    const delta = Math.min((now - lastTime) * 0.001, 0.1);
    lastTime = now;
    totalElapsed += delta;

    // Smooth lerp for footer undarkening transition
    currentFooter += (targetFooter - currentFooter) * 0.08;
    if (Math.abs(targetFooter - currentFooter) < 0.001) {
      currentFooter = targetFooter;
    }

    drawFrame();
    animFrame = requestAnimationFrame(render);
  }

  animFrame = requestAnimationFrame(render);

  document.addEventListener('visibilitychange', () => {
    if (document.hidden) {
      isHidden = true;
      if (animFrame) cancelAnimationFrame(animFrame);
    } else {
      isHidden = false;
      lastTime = performance.now();
      animFrame = requestAnimationFrame(render);
    }
  });
}

/* ==========================================================================
   Hero ↔ How-it-Works Kinetic Snap Transition (Instant 0ms-Latency Response)
   ========================================================================== */
function initHeroScrollTransition() {
  const howItWorks = document.getElementById('how-it-works');
  const scrollIndicator = document.getElementById('hero-scroll-indicator') || document.querySelector('.hero-scroll-indicator');
  if (!howItWorks) return;

  let activeAnimFrame = null;
  let isSnapping = false;
  let snapCooldownUntil = 0;
  let lastSnapDirection = 0; // 1 = snapped down to how-it-works, -1 = snapped up to hero

  function getHeaderHeight() {
    const header = document.querySelector('.site-header');
    return header ? header.offsetHeight : 72;
  }

  function getHowItWorksTargetTop() {
    const headerHeight = getHeaderHeight();
    return Math.max(0, howItWorks.getBoundingClientRect().top + window.scrollY - headerHeight);
  }

  function updateIndicatorVisibility() {
    if (!scrollIndicator) return;
    if (window.scrollY > 20) {
      scrollIndicator.classList.add('is-hidden');
    } else {
      scrollIndicator.classList.remove('is-hidden');
    }
  }

  window.addEventListener('scroll', updateIndicatorVisibility, { passive: true });
  updateIndicatorVisibility();

  function smoothScrollTo(targetY, duration = 300, onComplete) {
    if (activeAnimFrame) {
      cancelAnimationFrame(activeAnimFrame);
      activeAnimFrame = null;
    }

    const prefersReducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    if (prefersReducedMotion) {
      window.scrollTo(0, targetY);
      isSnapping = false;
      updateIndicatorVisibility();
      if (onComplete) onComplete();
      return;
    }

    isSnapping = true;
    const startY = window.scrollY;
    const distance = targetY - startY;

    if (Math.abs(distance) < 2) {
      window.scrollTo(0, targetY);
      isSnapping = false;
      updateIndicatorVisibility();
      if (onComplete) onComplete();
      return;
    }

    const startTime = performance.now();

    // Fast-attack Quartic Ease-Out: starts immediately on frame 0, decelerates smoothly
    function easeOutQuart(t) {
      return 1 - Math.pow(1 - t, 4);
    }

    function step(currentTime) {
      const elapsed = currentTime - startTime;
      const progress = Math.min(elapsed / duration, 1);
      const ease = easeOutQuart(progress);

      window.scrollTo(0, startY + distance * ease);
      updateIndicatorVisibility();

      if (progress < 1) {
        activeAnimFrame = requestAnimationFrame(step);
      } else {
        window.scrollTo(0, targetY);
        activeAnimFrame = null;
        isSnapping = false;
        updateIndicatorVisibility();
        if (onComplete) onComplete();
      }
    }

    activeAnimFrame = requestAnimationFrame(step);
  }

  // Wheel listener: instant snap on first wheel tick, seamless bidirectional transitions
  window.addEventListener('wheel', (e) => {
    const now = performance.now();
    const currentDelta = e.deltaY;
    const threshold = 8;

    // While animating, strictly lock out wheel events so momentum doesn't disrupt RAF
    if (isSnapping) {
      e.preventDefault();
      return;
    }

    // If within cooldown period after landing:
    // Only absorb residual inertial ticks in the SAME direction as the completed snap.
    // If the user intentionally reversed wheel direction, immediately allow the reverse snap!
    if (now < snapCooldownUntil) {
      if (Math.sign(currentDelta) === lastSnapDirection) {
        e.preventDefault();
        return;
      }
    }

    const howItWorksTop = getHowItWorksTargetTop();
    const scrollY = window.scrollY;

    // ZONE 1: Anywhere in Hero or between Hero and How-it-Works
    if (scrollY < howItWorksTop - 8) {
      if (currentDelta > threshold) {
        e.preventDefault();
        lastSnapDirection = 1;
        smoothScrollTo(howItWorksTop, 300, () => {
          snapCooldownUntil = performance.now() + 200;
        });
      } else if (currentDelta < -threshold && scrollY > 8) {
        e.preventDefault();
        lastSnapDirection = -1;
        smoothScrollTo(0, 260, () => {
          snapCooldownUntil = performance.now() + 150;
        });
      }
    }
    // ZONE 2: At or near top of How-it-Works (within top 120px)
    else if (scrollY >= howItWorksTop - 8 && scrollY <= howItWorksTop + 120) {
      if (currentDelta < -threshold) {
        e.preventDefault();
        lastSnapDirection = -1;
        smoothScrollTo(0, 300, () => {
          snapCooldownUntil = performance.now() + 200;
        });
      }
    }
  }, { passive: false });

  // Touch / Mobile gesture support
  let touchStartY = 0;
  let touchStartX = 0;

  window.addEventListener('touchstart', (e) => {
    if (e.touches.length > 0) {
      touchStartY = e.touches[0].clientY;
      touchStartX = e.touches[0].clientX;
    }
  }, { passive: true });

  window.addEventListener('touchmove', (e) => {
    if (!e.touches.length) return;
    if (isSnapping) {
      e.preventDefault();
      return;
    }

    const touchCurrentY = e.touches[0].clientY;
    const touchCurrentX = e.touches[0].clientX;
    const deltaY = touchStartY - touchCurrentY; // Positive = swipe up = scroll down
    const deltaX = touchStartX - touchCurrentX;

    if (Math.abs(deltaY) < Math.abs(deltaX) || Math.abs(deltaY) < 18) {
      return;
    }

    const howItWorksTop = getHowItWorksTargetTop();
    const scrollY = window.scrollY;

    if (scrollY < howItWorksTop - 8 && deltaY > 18) {
      e.preventDefault();
      lastSnapDirection = 1;
      smoothScrollTo(howItWorksTop, 300, () => {
        snapCooldownUntil = performance.now() + 200;
      });
    } else if (scrollY <= howItWorksTop + 140 && deltaY < -18) {
      e.preventDefault();
      lastSnapDirection = -1;
      smoothScrollTo(0, 300, () => {
        snapCooldownUntil = performance.now() + 200;
      });
    }
  }, { passive: false });

  // Keyboard navigation support (ArrowUp, ArrowDown, PageUp, PageDown, Space, Home)
  const scrollKeyCodes = ['ArrowUp', 'ArrowDown', 'PageUp', 'PageDown', 'Space', 'Home', 'End'];

  window.addEventListener('keydown', (e) => {
    const activeTag = document.activeElement ? document.activeElement.tagName.toLowerCase() : '';
    if (activeTag === 'input' || activeTag === 'textarea') return;

    if (!scrollKeyCodes.includes(e.code)) return;

    // While animating, strictly prevent native keyboard scroll bleed-through
    if (isSnapping) {
      e.preventDefault();
      return;
    }

    const howItWorksTop = getHowItWorksTargetTop();
    const scrollY = window.scrollY;

    // ZONE 1: Anywhere in Hero or between Hero and How-it-Works
    if (scrollY < howItWorksTop - 8) {
      if (e.code === 'ArrowDown' || e.code === 'PageDown' || (e.code === 'Space' && !e.shiftKey)) {
        e.preventDefault();
        lastSnapDirection = 1;
        smoothScrollTo(howItWorksTop, 300, () => {
          snapCooldownUntil = performance.now() + 150;
        });
      } else if (e.code === 'ArrowUp' || e.code === 'PageUp' || (e.code === 'Space' && e.shiftKey) || e.code === 'Home') {
        if (scrollY > 5) {
          e.preventDefault();
          lastSnapDirection = -1;
          smoothScrollTo(0, 260, () => {
            snapCooldownUntil = performance.now() + 150;
          });
        }
      }
    }
    // ZONE 2: At or near top of How-it-Works (within top 160px)
    else if (scrollY >= howItWorksTop - 8 && scrollY <= howItWorksTop + 160) {
      if (e.code === 'ArrowUp' || e.code === 'PageUp' || (e.code === 'Space' && e.shiftKey) || e.code === 'Home') {
        e.preventDefault();
        lastSnapDirection = -1;
        smoothScrollTo(0, 300, () => {
          snapCooldownUntil = performance.now() + 150;
        });
      }
    }
  });

  // Fast non-nav header links and scroll indicator clicks
  document.querySelectorAll('a[href^="#"]:not(.nav-link)').forEach(anchor => {
    anchor.addEventListener('click', (e) => {
      const href = anchor.getAttribute('href');
      if (href === '#' || href === '#hero') {
        e.preventDefault();
        smoothScrollTo(0, 260);
        if (history.pushState) history.pushState(null, null, ' ');
        return;
      }

      const targetEl = document.querySelector(href);
      if (targetEl) {
        e.preventDefault();
        const headerHeight = getHeaderHeight();
        const targetTop = Math.max(0, targetEl.getBoundingClientRect().top + window.scrollY - headerHeight);
        smoothScrollTo(targetTop, 280);
        if (history.pushState) history.pushState(null, null, href);
      }
    });
  });

  // Export smoothScrollTo for external callers
  window.auraSmoothScrollTo = smoothScrollTo;
}

/* ==========================================================================
   Capsule Navigation Dock & Animated Sliding Pill Indicator
   ========================================================================== */
function initNavCapsule() {
  const navMenu = document.getElementById('nav-menu');
  if (!navMenu) return;

  const links = navMenu.querySelectorAll('.nav-link');
  let indicator = navMenu.querySelector('.nav-pill-indicator');

  if (!indicator) {
    indicator = document.createElement('li');
    indicator.className = 'nav-pill-indicator';
    indicator.setAttribute('aria-hidden', 'true');
    navMenu.prepend(indicator);
  }

  function updateIndicator(link) {
    if (!link) {
      indicator.style.opacity = '0';
      return;
    }
    indicator.style.opacity = '1';
    const navRect = navMenu.getBoundingClientRect();
    const linkRect = link.getBoundingClientRect();
    const left = linkRect.left - navRect.left;
    const width = linkRect.width;

    indicator.style.width = `${width}px`;
    indicator.style.transform = `translate3d(${left}px, 0, 0)`;
  }

  function setActiveLink(link) {
    links.forEach(l => l.classList.remove('active'));
    if (link) {
      link.classList.add('active');
      updateIndicator(link);
    } else {
      updateIndicator(null);
    }
  }

  // Smooth scroll click on nav links
  links.forEach(link => {
    link.addEventListener('click', (e) => {
      const href = link.getAttribute('href');
      if (href && href.startsWith('#')) {
        e.preventDefault();
        const target = document.querySelector(href);
        if (target) {
          const header = document.querySelector('.site-header');
          const headerHeight = header ? header.offsetHeight + 14 : 74;
          const targetTop = Math.max(0, target.getBoundingClientRect().top + window.scrollY - headerHeight);
          setActiveLink(link);
          if (window.auraSmoothScrollTo) {
            window.auraSmoothScrollTo(targetTop, 300);
          } else {
            window.scrollTo({ top: targetTop, behavior: 'smooth' });
          }
        }
      }
    });
  });

  // ScrollSpy
  const sections = [];
  links.forEach(link => {
    const href = link.getAttribute('href');
    if (href && href.startsWith('#')) {
      const sec = document.querySelector(href);
      if (sec) sections.push({ el: sec, link: link });
    }
  });

  function checkActiveSection() {
    const scrollPos = window.scrollY + 160;
    let current = null;

    for (let i = sections.length - 1; i >= 0; i--) {
      const secTop = sections[i].el.offsetTop;
      if (scrollPos >= secTop) {
        current = sections[i].link;
        break;
      }
    }

    if (current) {
      setActiveLink(current);
    } else {
      setActiveLink(null);
    }
  }

  window.addEventListener('scroll', checkActiveSection, { passive: true });
  checkActiveSection();
}

/* ==========================================================================
   Header CTA Scroll Reveal (Download & GitHub Icon Buttons)
   ========================================================================== */
function initHeaderCtaReveal() {
  const ctaGroup = document.getElementById('header-cta-group');
  const heroSection = document.getElementById('hero');
  if (!ctaGroup || !heroSection) return;

  function updateHeaderCta() {
    const heroBottom = heroSection.getBoundingClientRect().bottom;
    if (heroBottom < 100) {
      ctaGroup.classList.add('is-visible');
    } else {
      ctaGroup.classList.remove('is-visible');
    }
  }

  window.addEventListener('scroll', updateHeaderCta, { passive: true });
  updateHeaderCta();
}

/* ==========================================================================
   Features Advantages 3D Cascading Interactive Deck (Drop-Fade & Rise)
   ========================================================================== */
function initFeaturesDeck() {
  const container = document.getElementById('features-deck') || document.querySelector('.features-deck-container');
  if (!container) return;

  const cards = Array.from(container.querySelectorAll('.deck-card'));
  const pills = Array.from(document.querySelectorAll('.deck-segment-pill'));
  if (cards.length === 0) return;

  let activeIndex = 0;
  let isTransitioning = false;

  const slotConfigs = cards.length === 5 ? [
    { top: 0, scale: 0.92, zIndex: 1, opacity: 0.25 },
    { top: 28, scale: 0.94, zIndex: 2, opacity: 0.45 },
    { top: 56, scale: 0.96, zIndex: 3, opacity: 0.65 },
    { top: 84, scale: 0.98, zIndex: 4, opacity: 0.85 },
    { top: 112, scale: 1.0, zIndex: 10, opacity: 1.0 }
  ] : [
    { top: 0, scale: 0.94, zIndex: 1, opacity: 0.35 },
    { top: 38, scale: 0.96, zIndex: 2, opacity: 0.55 },
    { top: 76, scale: 0.98, zIndex: 3, opacity: 0.75 },
    { top: 114, scale: 1.0, zIndex: 10, opacity: 1.0 }
  ];

  function applySlots(targetIndex) {
    const total = cards.length;
    const activeSlotIdx = slotConfigs.length - 1;

    for (let offset = 1; offset < total; offset++) {
      const cardIdx = (targetIndex + offset) % total;
      const card = cards[cardIdx];
      const slotIdx = activeSlotIdx - offset;
      const conf = slotConfigs[slotIdx];

      card.classList.remove('is-active');
      card.setAttribute('aria-expanded', 'false');
      card.style.setProperty('--deck-top', `${conf.top}px`);
      card.style.setProperty('--deck-scale', conf.scale);
      card.style.top = `${conf.top}px`;
      card.style.zIndex = conf.zIndex;
      card.style.opacity = conf.opacity;
    }

    const activeCard = cards[targetIndex];
    activeCard.classList.add('is-active');
    activeCard.setAttribute('aria-expanded', 'true');
    const conf = slotConfigs[activeSlotIdx];
    activeCard.style.setProperty('--deck-top', `${conf.top}px`);
    activeCard.style.setProperty('--deck-scale', conf.scale);
    activeCard.style.top = `${conf.top}px`;
    activeCard.style.zIndex = conf.zIndex;
    activeCard.style.opacity = conf.opacity;

    pills.forEach((pill, idx) => {
      const isActive = idx === targetIndex;
      pill.classList.toggle('is-active', isActive);
      pill.setAttribute('aria-selected', isActive ? 'true' : 'false');
    });
  }

  function setActiveCard(index) {
    if (isTransitioning) return;
    const prevActiveIndex = activeIndex;

    if (index < 0) {
      activeIndex = cards.length - 1;
    } else if (index >= cards.length) {
      activeIndex = 0;
    } else {
      activeIndex = index;
    }

    if (prevActiveIndex === activeIndex) return;

    const prevCard = cards[prevActiveIndex];
    const activeCard = cards[activeIndex];

    isTransitioning = true;
    prevCard.classList.add('is-drop-exiting');

    setTimeout(() => {
      prevCard.classList.remove('is-drop-exiting');
      applySlots(activeIndex);
      activeCard.classList.add('is-rise-entering');

      setTimeout(() => {
        activeCard.classList.remove('is-rise-entering');
        isTransitioning = false;
      }, 280);
    }, 140);
  }

  const btnPrev = document.getElementById('deck-btn-prev');
  const btnNext = document.getElementById('deck-btn-next');

  if (btnPrev) {
    btnPrev.addEventListener('click', (e) => {
      e.preventDefault();
      setActiveCard(activeIndex - 1);
    });
  }

  if (btnNext) {
    btnNext.addEventListener('click', (e) => {
      e.preventDefault();
      setActiveCard(activeIndex + 1);
    });
  }

  cards.forEach((card, idx) => {
    card.addEventListener('click', (e) => {
      if (e.target.closest('a') || e.target.closest('button')) return;
      if (!card.classList.contains('is-active')) {
        setActiveCard(idx);
      }
    });
  });

  pills.forEach((pill, idx) => {
    pill.addEventListener('click', () => {
      setActiveCard(idx);
    });
  });

  applySlots(0);
}

/* ==========================================================================
   Floating Back to Top Button
   ========================================================================== */
function initBackToTop() {
  const btn = document.getElementById('back-to-top');
  if (!btn) return;

  function updateBackToTop() {
    if (window.scrollY > 400) {
      btn.classList.add('is-visible');
    } else {
      btn.classList.remove('is-visible');
    }
  }

  window.addEventListener('scroll', updateBackToTop, { passive: true });
  updateBackToTop();

  btn.addEventListener('click', (e) => {
    e.preventDefault();
    if (window.auraSmoothScrollTo) {
      window.auraSmoothScrollTo(0, 350);
    } else {
      window.scrollTo({ top: 0, behavior: 'smooth' });
    }
  });
}

/* ==========================================================================
   Monolithic 3D Letters: A U R A (Kinetic 3D Magnetic Cursor Tracking)
   ========================================================================== */
function initFooterAuraLetters() {
  const lettersWrap = document.getElementById('aura-letters-wrap') || document.querySelector('.aura-letters-wrap');
  if (!lettersWrap) return;

  const letters = Array.from(lettersWrap.querySelectorAll('.aura-letter'));
  if (letters.length === 0) return;

  // Individual physics state for each letter (A=0, U=1, R=2, A=3)
  const letterStates = letters.map((_, idx) => {
    const baseRy = (idx - 1.5) * 4.5; // Natural architectural fan: -6.75°, -2.25°, +2.25°, +6.75°
    const baseRz = (idx - 1.5) * 1.2;
    return {
      baseRy,
      baseRx: 0,
      baseRz,
      rx: 0,
      ry: baseRy,
      rz: baseRz,
      tz: 0,
      targetRx: 0,
      targetRy: baseRy,
      targetRz: baseRz,
      targetTz: 0
    };
  });

  let isMouseTracking = false;
  let rafId = null;

  function updateLetterTransforms() {
    let hasMotion = false;

    letters.forEach((letter, idx) => {
      const state = letterStates[idx];

      // Smooth buttery lerp (0.09)
      state.rx += (state.targetRx - state.rx) * 0.09;
      state.ry += (state.targetRy - state.ry) * 0.09;
      state.rz += (state.targetRz - state.rz) * 0.09;
      state.tz += (state.targetTz - state.tz) * 0.09;

      const gradAngle = 180 + state.ry * 1.5;

      letter.style.setProperty('--rx', `${state.rx.toFixed(2)}deg`);
      letter.style.setProperty('--ry', `${state.ry.toFixed(2)}deg`);
      letter.style.setProperty('--rz', `${state.rz.toFixed(2)}deg`);
      letter.style.setProperty('--tz', `${state.tz.toFixed(1)}px`);
      letter.style.setProperty('--grad-angle', `${gradAngle.toFixed(1)}deg`);

      if (
        Math.abs(state.targetRx - state.rx) > 0.01 ||
        Math.abs(state.targetRy - state.ry) > 0.01 ||
        Math.abs(state.targetTz - state.tz) > 0.05
      ) {
        hasMotion = true;
      }
    });

    if (hasMotion || isMouseTracking) {
      rafId = requestAnimationFrame(updateLetterTransforms);
    } else {
      rafId = null;
    }
  }

  function onMouseMove(e) {
    const wrapRect = lettersWrap.getBoundingClientRect();
    // Only track when footer is in or near the viewport
    if (wrapRect.bottom < -150 || wrapRect.top > window.innerHeight + 150) {
      if (isMouseTracking) resetTargets();
      return;
    }

    isMouseTracking = true;

    letters.forEach((letter, idx) => {
      const state = letterStates[idx];
      const letterRect = letter.getBoundingClientRect();
      const lx = letterRect.left + letterRect.width / 2;
      const ly = letterRect.top + letterRect.height / 2;

      // Distance from this specific individual letter to mouse
      const dx = e.clientX - lx;
      const dy = e.clientY - ly;
      const dist = Math.hypot(dx, dy);

      // Normalized distance relative to screen
      const nx = dx / (window.innerWidth * 0.45);
      const ny = dy / (window.innerHeight * 0.45);

      // Individual letter magnetic attraction angle
      const rotY = state.baseRy + Math.max(-25, Math.min(25, nx * 22));
      const rotX = Math.max(-15, Math.min(15, -ny * 14));
      const rotZ = state.baseRz + Math.max(-5, Math.min(5, nx * 3));

      // 3D proximity pop: letter smoothly lifts forward when cursor is nearby
      const proximity = Math.max(0, 1.0 - dist / 550);
      const tz = proximity * 22;

      state.targetRx = rotX;
      state.targetRy = rotY;
      state.targetRz = rotZ;
      state.targetTz = tz;
    });

    if (!rafId) {
      rafId = requestAnimationFrame(updateLetterTransforms);
    }
  }

  function resetTargets() {
    isMouseTracking = false;
    letters.forEach((_, idx) => {
      const state = letterStates[idx];
      state.targetRx = state.baseRx;
      state.targetRy = state.baseRy;
      state.targetRz = state.baseRz;
      state.targetTz = 0;
    });
    if (!rafId) {
      rafId = requestAnimationFrame(updateLetterTransforms);
    }
  }

  function onMouseLeave() {
    resetTargets();
  }

  window.addEventListener('mousemove', onMouseMove, { passive: true });
  document.addEventListener('mouseleave', onMouseLeave, { passive: true });
  window.addEventListener('scroll', () => {
    const wrapRect = lettersWrap.getBoundingClientRect();
    if (wrapRect.bottom < -200 || wrapRect.top > window.innerHeight + 200) {
      if (isMouseTracking) resetTargets();
    }
  }, { passive: true });
}

/* ==========================================================================
   Multi-Page Language Routing & First-Visit Auto-Detection
   ========================================================================== */
function initLanguageAutoDetect() {
  const currentHtmlLang = (document.documentElement.lang || '').toLowerCase();
  const path = window.location.pathname.toLowerCase();

  // If already on the Russian page (lang="ru" or /index_ru), NEVER redirect
  if (currentHtmlLang === 'ru' || path.includes('index_ru')) {
    return;
  }

  try {
    const savedLang = localStorage.getItem('aura_user_lang');
    if (savedLang === 'ru') {
      window.location.replace('index_ru.html' + window.location.search + window.location.hash);
      return;
    }
    if (savedLang === 'en') {
      return;
    }

    // First visit: Auto-detect browser language
    const navLangs = navigator.languages ? Array.from(navigator.languages) : [];
    const primaryLang = (navigator.language || navigator.userLanguage || '').toLowerCase();
    const allLangs = [primaryLang, ...navLangs.map(l => (l || '').toLowerCase())];

    const isRuLocale = allLangs.some(l => 
      l.startsWith('ru') || l.startsWith('be') || l.startsWith('uk') || l.startsWith('kk')
    );

    if (isRuLocale) {
      window.location.replace('index_ru.html' + window.location.search + window.location.hash);
    }
  } catch (e) {
    // Graceful fallback if localStorage is unavailable
  }
}

function initLanguageSwitcher() {
  const btnToggle = document.getElementById('btn-lang-toggle');
  if (!btnToggle) return;

  btnToggle.addEventListener('click', () => {
    const targetLang = btnToggle.getAttribute('data-lang-target') || 'en';
    try {
      localStorage.setItem('aura_user_lang', targetLang);
    } catch (err) {}
  });
}

/* ==========================================================================
   Download Confirmation Modal
   ========================================================================== */
function initDownloadModal() {
  const modal = document.getElementById('download-modal');
  if (!modal) return;

  const closeBtn = document.getElementById('modal-download-close');
  const cancelBtn = document.getElementById('modal-download-cancel');
  const confirmBtn = document.getElementById('modal-download-confirm');
  const triggers = document.querySelectorAll('[data-download-trigger]');
  let lastActiveElement = null;

  function openModal(triggerElement) {
    lastActiveElement = triggerElement || document.activeElement;
    modal.classList.add('is-active');
    modal.setAttribute('aria-hidden', 'false');
    document.body.classList.add('modal-open');
    if (confirmBtn) {
      setTimeout(() => confirmBtn.focus(), 60);
    }
  }

  function closeModal() {
    modal.classList.remove('is-active');
    modal.setAttribute('aria-hidden', 'true');
    document.body.classList.remove('modal-open');
    if (lastActiveElement && typeof lastActiveElement.focus === 'function') {
      lastActiveElement.focus();
    }
  }

  triggers.forEach(trigger => {
    trigger.addEventListener('click', (e) => {
      e.preventDefault();
      openModal(trigger);
    });
  });

  if (closeBtn) {
    closeBtn.addEventListener('click', (e) => {
      e.preventDefault();
      closeModal();
    });
  }

  if (cancelBtn) {
    cancelBtn.addEventListener('click', (e) => {
      e.preventDefault();
      closeModal();
    });
  }

  if (confirmBtn) {
    confirmBtn.addEventListener('click', () => {
      setTimeout(() => {
        closeModal();
      }, 400);
    });
  }

  modal.addEventListener('click', (e) => {
    if (e.target === modal) {
      closeModal();
    }
  });

  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape' && modal.classList.contains('is-active')) {
      closeModal();
    }
  });
}
