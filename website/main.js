/**
 * AURA — Interactive Website Core Script
 * Features:
 * - Live Dictation Typing Simulator (Original Replica with realistic overlay pill)
 * - 5-Tab Settings Showcase (Interactive Tabs, Mic Test, Hardware/Engine Pickers, Live History Search)
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
   Live Dictation Typing Simulator (Monolith + Kinetic Luminescence)
   ========================================================================== */
let simulationRunId = 0;
let simulationTimeouts = [];

function startTypingSimulator() {
  const currentRunId = ++simulationRunId;
  const canvas = document.getElementById('monolith-canvas');
  const editorText = document.getElementById('demo-editor-text');
  const overlayPill = document.getElementById('mock-overlay-pill');
  const overlayStatus = document.getElementById('mock-overlay-status');
  const soundBars = document.querySelectorAll('#mock-overlay-pill .sound-bar');
  
  if (!editorText || !overlayPill || !overlayStatus) return;

  // Cancel all pending timeouts
  simulationTimeouts.forEach(t => clearTimeout(t));
  simulationTimeouts = [];

  const isRu = (document.documentElement.lang || 'ru') === 'ru';
  const phrases = isRu ? [
    {
      raw: "Привет, во сколько созвонимся, чтобы обсудить новый релиз",
      clean: "Привет, во сколько созвонимся, чтобы обсудить новый релиз?"
    },
    {
      raw: "Напиши функцию валидации: проверь email, пароль и токен доступа",
      clean: "Напиши функцию валидации: проверь email, пароль и токен доступа."
    },
    {
      raw: "Почему падает тест авторизации, если передать пустой заголовок",
      clean: "Почему падает тест авторизации, если передать пустой заголовок?"
    },
    {
      raw: "Идея для фичи — добавить умную паузу, пока пользователь думает",
      clean: "Идея для фичи — добавить умную паузу, пока пользователь думает."
    },
    {
      raw: "Спасибо за отчёт, все правки внесены — отправляю клиенту на подпись",
      clean: "Спасибо за отчёт, все правки внесены — отправляю клиенту на подпись."
    },
    {
      raw: "Проверь логи сервера: время ответа базы данных резко выросло",
      clean: "Проверь логи сервера: время ответа базы данных резко выросло!"
    }
  ] : [
    {
      raw: "Hey, what time can we sync to discuss the new release",
      clean: "Hey, what time can we sync to discuss the new release?"
    },
    {
      raw: "Write a validation function: check email, password, and auth token",
      clean: "Write a validation function: check email, password, and auth token."
    },
    {
      raw: "Why does the auth test fail when an empty header is passed",
      clean: "Why does the auth test fail when an empty header is passed?"
    },
    {
      raw: "Feature idea — add a smart pause while the user is thinking",
      clean: "Feature idea — add a smart pause while the user is thinking."
    },
    {
      raw: "Thanks for the report, edits are done — sending to the client",
      clean: "Thanks for the report, edits are done — sending to the client."
    },
    {
      raw: "Check the server logs: database response time spiked suddenly",
      clean: "Check the server logs: database response time spiked suddenly!"
    }
  ];

  let phraseIndex = 0;
  let breathingInterval = null;
  let timerInterval = null;

  function stopAllAnimations() {
    if (breathingInterval) {
      clearInterval(breathingInterval);
      breathingInterval = null;
    }
    if (timerInterval) {
      clearInterval(timerInterval);
      timerInterval = null;
    }
    
    soundBars.forEach(bar => {
      bar.removeAttribute("style");
      bar.setAttribute("height", "6");
      bar.setAttribute("y", "11.6001");
    });
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

    const currentItem = phrases[phraseIndex % phrases.length];
    phraseIndex++;

    // Clean reset of editor state
    editorText.innerHTML = "";
    if (canvas) canvas.classList.remove('recording', 'processing');
    overlayPill.classList.remove('active', 'processing');
    overlayStatus.textContent = "0:00";
    
    await sleep(1000);
    if (currentRunId !== simulationRunId) return;
    
    // Step 1: Open Dictation Pill Overlay & activate Acoustic Resonance Ripples
    overlayPill.classList.add('active');
    if (canvas) canvas.classList.add('recording');
    
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
    
    // Step 2: Dictate phrase with kinetic luminescence on each character
    for (let i = 0; i < currentItem.raw.length; i++) {
      if (currentRunId !== simulationRunId) return;
      const char = currentItem.raw[i];
      const token = document.createElement('span');
      token.className = 'kinetic-token';
      token.textContent = char;
      editorText.appendChild(token);
      await sleep(26 + Math.random() * 30);
    }
    
    if (currentRunId !== simulationRunId) return;
    if (timerInterval) clearInterval(timerInterval);
    await sleep(350);
    if (currentRunId !== simulationRunId) return;
    
    // Step 3: Transcription processing state
    if (canvas) {
      canvas.classList.remove('recording');
      canvas.classList.add('processing');
    }
    overlayPill.classList.add('processing');
    overlayStatus.textContent = isRu ? "Вставка…" : "Transcribing...";
    
    let angle = 0;
    breathingInterval = setInterval(() => {
      if (currentRunId !== simulationRunId) {
        clearInterval(breathingInterval);
        return;
      }
      soundBars.forEach((bar, index) => {
        const distFromCenter = Math.abs(index - 4.5);
        const h = 5 + Math.sin(angle - distFromCenter * 0.45) * 5.5;
        const y = 14.6 - (h / 2);
        bar.setAttribute("height", Math.max(2, h).toString());
        bar.setAttribute("y", y.toString());
      });
      angle += 0.08;
    }, 16);
    
    await sleep(800);
    if (currentRunId !== simulationRunId) return;
    
    // Step 4: Final punctuation & insertion completed
    editorText.textContent = currentItem.clean;
    
    stopAllAnimations();
    if (canvas) canvas.classList.remove('recording', 'processing');
    overlayPill.classList.remove('active', 'processing');
    
    // Pause before repeating next cycle
    await sleep(3600);
    if (currentRunId !== simulationRunId) return;
    runSimulationLoop();
  }

  stopAllAnimations();
  editorText.innerHTML = "";
  if (canvas) canvas.classList.remove('recording', 'processing');
  overlayPill.classList.remove('active', 'processing');
  overlayStatus.textContent = "0:00";

  runSimulationLoop();
}

/* ==========================================================================
   Interface Showcase Mockup Interactivity (Authentic Desktop UI)
   ========================================================================== */
let micTestInterval = null;
let isMicTesting = false;

function initInterfaceShowcase() {
  const showcase = document.querySelector('.app-showcase-window');
  if (!showcase) return;

  const tabs = showcase.querySelectorAll('.nav-tab[data-tab]');
  const panels = showcase.querySelectorAll('.tab-panel');
  if (!tabs.length || !panels.length) return;

  const isRu = (document.documentElement.lang || 'ru') === 'ru';

  // 1. Tab Switching
  tabs.forEach(tab => {
    tab.addEventListener('click', (e) => {
      e.preventDefault();
      const targetTab = tab.getAttribute('data-tab');

      tabs.forEach(t => {
        t.classList.remove('active');
        t.setAttribute('aria-selected', 'false');
      });
      panels.forEach(p => {
        p.classList.remove('active');
        p.style.display = 'none';
      });

      tab.classList.add('active');
      tab.setAttribute('aria-selected', 'true');

      const targetPanel = showcase.querySelector(`#panel-${targetTab}`);
      if (targetPanel) {
        targetPanel.classList.add('active');
        targetPanel.style.display = 'flex';
      }
    });
  });

  // 2. Microphone VU Meter & VAD Test Button
  const btnMicTest = showcase.querySelector('#btn-toggle-mic-meter');
  const micFill = showcase.querySelector('#mic-meter-fill');
  const micPeak = showcase.querySelector('#mic-meter-peak');
  const vadIndicator = showcase.querySelector('#mic-vad-indicator');
  const vadText = showcase.querySelector('#mic-vad-text');

  if (btnMicTest && micFill) {
    btnMicTest.addEventListener('click', (e) => {
      e.preventDefault();
      const currentLangRu = (document.documentElement.lang || 'ru') === 'ru';

      if (isMicTesting) {
        // Stop test
        isMicTesting = false;
        if (micTestInterval) clearInterval(micTestInterval);
        micTestInterval = null;
        micFill.style.width = '0%';
        if (micPeak) micPeak.style.opacity = '0';
        if (vadIndicator) vadIndicator.classList.remove('speaking');
        if (vadText) vadText.textContent = currentLangRu ? 'Тишина' : 'Silence';
        btnMicTest.textContent = currentLangRu ? 'Запустить тест' : 'Test Audio';
        btnMicTest.classList.remove('active');
      } else {
        // Start test
        isMicTesting = true;
        btnMicTest.textContent = currentLangRu ? 'Остановить тест' : 'Stop Test';
        btnMicTest.classList.add('active');

        let micPhase = 0;
        micTestInterval = setInterval(() => {
          micPhase += 0.25;
          const level = Math.max(8, Math.min(92, 45 + Math.sin(micPhase) * 35 + (Math.random() * 20 - 10)));
          micFill.style.width = `${level.toFixed(0)}%`;

          if (micPeak) {
            micPeak.style.opacity = '0.9';
            micPeak.style.left = `${Math.min(98, level + 4).toFixed(0)}%`;
          }

          if (level > 28) {
            if (vadIndicator) vadIndicator.classList.add('speaking');
            if (vadText) vadText.textContent = currentLangRu ? 'Обнаружен голос' : 'Voice detected';
          } else {
            if (vadIndicator) vadIndicator.classList.remove('speaking');
            if (vadText) vadText.textContent = currentLangRu ? 'Тишина' : 'Silence';
          }
        }, 75);
      }
    });
  }

  // 3. Engine Mode Toggle (Cloud vs Local)
  const engineRadios = showcase.querySelectorAll('input[name="transcription_mode"]');
  const localSection = showcase.querySelector('#local-engine-section');
  if (engineRadios.length && localSection) {
    engineRadios.forEach(radio => {
      radio.addEventListener('change', () => {
        localSection.style.display = (radio.value === 'local' && radio.checked) ? 'flex' : 'none';
      });
    });
  }

  // 4. GPU Acceleration & Model Cards Selection
  const gpuCards = showcase.querySelectorAll('.model-card[data-gpu]');
  gpuCards.forEach(card => {
    card.addEventListener('click', () => {
      gpuCards.forEach(c => {
        c.classList.remove('active');
        c.setAttribute('aria-checked', 'false');
      });
      card.classList.add('active');
      card.setAttribute('aria-checked', 'true');
    });
  });

  const whisperCards = showcase.querySelectorAll('.model-card[data-model]');
  whisperCards.forEach(card => {
    card.addEventListener('click', () => {
      whisperCards.forEach(c => {
        c.classList.remove('active');
        c.setAttribute('aria-checked', 'false');
      });
      card.classList.add('active');
      card.setAttribute('aria-checked', 'true');
    });
  });

  // 5. Sound Volume Slider
  const volumeSlider = showcase.querySelector('#range-sound-volume');
  const volumeLabel = showcase.querySelector('#volume-value-label');
  if (volumeSlider && volumeLabel) {
    volumeSlider.addEventListener('input', () => {
      volumeLabel.textContent = `${volumeSlider.value}%`;
    });
  }

  // 6. History Filter Buttons & Live Search
  const filterBtns = showcase.querySelectorAll('.history-filter-btn[data-filter]');
  const searchInput = showcase.querySelector('#input-history-search');
  const historyItems = showcase.querySelectorAll('.history-item');
  const dayHeaders = showcase.querySelectorAll('.history-day-header');
  let activeHistoryFilter = 'all';

  function applyHistoryFilter() {
    const query = (searchInput ? searchInput.value.toLowerCase().trim() : '');

    historyItems.forEach(item => {
      const itemType = item.getAttribute('data-type') || '';
      const textEl = item.querySelector('.history-item-text');
      const text = textEl ? textEl.textContent.toLowerCase() : '';

      const matchesFilter = activeHistoryFilter === 'all' || itemType === activeHistoryFilter;
      const matchesSearch = !query || text.includes(query);

      if (matchesFilter && matchesSearch) {
        item.style.display = 'flex';
      } else {
        item.style.display = 'none';
      }
    });

    // Update day headers visibility
    dayHeaders.forEach(header => {
      let sibling = header.nextElementSibling;
      let hasVisible = false;
      while (sibling && !sibling.classList.contains('history-day-header')) {
        if (sibling.classList.contains('history-item') && sibling.style.display !== 'none') {
          hasVisible = true;
          break;
        }
        sibling = sibling.nextElementSibling;
      }
      header.style.display = hasVisible ? 'flex' : 'none';
    });
  }

  filterBtns.forEach(btn => {
    btn.addEventListener('click', () => {
      filterBtns.forEach(b => b.classList.remove('active'));
      btn.classList.add('active');
      activeHistoryFilter = btn.getAttribute('data-filter') || 'all';
      applyHistoryFilter();
    });
  });

  if (searchInput) {
    searchInput.addEventListener('input', applyHistoryFilter);
  }

  // 7. Copy Buttons on History Items
  const copyButtons = showcase.querySelectorAll('.btn-copy-history');
  copyButtons.forEach(btn => {
    btn.addEventListener('click', async (e) => {
      e.stopPropagation();
      const item = btn.closest('.history-item');
      const textEl = item ? item.querySelector('.history-item-text') : null;
      if (textEl) {
        const text = textEl.textContent.trim();
        try {
          await navigator.clipboard.writeText(text);
        } catch (err) {}

        btn.classList.add('is-copied');
        const oldTitle = btn.getAttribute('title');
        const currentLangRu = (document.documentElement.lang || 'ru') === 'ru';
        btn.setAttribute('title', currentLangRu ? 'Скопировано!' : 'Copied!');
        setTimeout(() => {
          btn.classList.remove('is-copied');
          btn.setAttribute('title', oldTitle || (currentLangRu ? 'Копировать' : 'Copy'));
        }, 1400);
      }
    });
  });

  // 8. Clear & Restore History Button
  const btnClearHist = showcase.querySelector('#btn-clear-history');
  const historyContainer = showcase.querySelector('#history-items-container');
  let isHistoryCleared = false;
  if (btnClearHist && historyContainer) {
    const originalHistoryHtml = historyContainer.innerHTML;
    btnClearHist.addEventListener('click', () => {
      const currentLangRu = (document.documentElement.lang || 'ru') === 'ru';
      if (!isHistoryCleared) {
        isHistoryCleared = true;
        historyContainer.innerHTML = `<div class="history-empty-state">${currentLangRu ? 'История транскрипций очищена' : 'Transcription history cleared'}</div>`;
        btnClearHist.textContent = currentLangRu ? 'Восстановить' : 'Restore';
      } else {
        isHistoryCleared = false;
        historyContainer.innerHTML = originalHistoryHtml;
        btnClearHist.textContent = currentLangRu ? 'Очистить историю' : 'Clear History';
        initInterfaceShowcase();
      }
    });
  }

  // 9. Gaming Mode Manual Toggle in Showcase
  const btnToggleGaming = showcase.querySelector('#btn-toggle-gaming-mode');
  const gamingDot = showcase.querySelector('#gaming-status-dot');
  const gamingText = showcase.querySelector('#gaming-status-text');
  if (btnToggleGaming && gamingDot && gamingText) {
    let isGamingActive = false;
    btnToggleGaming.addEventListener('click', () => {
      const currentLangRu = (document.documentElement.lang || 'ru') === 'ru';
      isGamingActive = !isGamingActive;
      if (isGamingActive) {
        gamingDot.classList.add('active');
        gamingText.classList.add('active');
        gamingText.textContent = currentLangRu ? 'Активен' : 'Active';
        btnToggleGaming.textContent = currentLangRu ? 'Выключить вручную' : 'Disable Manually';
      } else {
        gamingDot.classList.remove('active');
        gamingText.classList.remove('active');
        gamingText.textContent = currentLangRu ? 'Не активен' : 'Inactive';
        btnToggleGaming.textContent = currentLangRu ? 'Включить вручную' : 'Enable Manually';
      }
    });
  }

  // 10. Check Updates Action Button
  const btnUpdates = showcase.querySelector('#btn-check-updates');
  if (btnUpdates) {
    btnUpdates.addEventListener('click', () => {
      const currentLangRu = (document.documentElement.lang || 'ru') === 'ru';
      const originalText = btnUpdates.textContent;
      btnUpdates.textContent = currentLangRu ? 'Проверка…' : 'Checking…';
      btnUpdates.disabled = true;

      setTimeout(() => {
        btnUpdates.textContent = currentLangRu ? '✓ Актуальная версия v1.1.1' : '✓ Aura is up to date (v1.1.1)';
        setTimeout(() => {
          btnUpdates.textContent = originalText;
          btnUpdates.disabled = false;
        }, 2200);
      }, 700);
    });
  }

  // 11. Copy Diagnostics Report Button
  const btnDiag = showcase.querySelector('#btn-copy-diagnostics');
  if (btnDiag) {
    btnDiag.addEventListener('click', async () => {
      const currentLangRu = (document.documentElement.lang || 'ru') === 'ru';
      const originalText = btnDiag.textContent;
      const report = `Aura System Diagnostic Report\nVersion: 1.1.1 (x64)\nOS: Windows 10/11 x64\nEngine: Whisper.cpp / NVIDIA Parakeet (CUDA Active)\nVRAM: NVIDIA GeForce RTX (Detected)\nDPAPI Storage: Encrypted\nStatus: Nominal (Ready)`;
      try {
        await navigator.clipboard.writeText(report);
      } catch (err) {}

      btnDiag.textContent = currentLangRu ? '✓ Отчет скопирован!' : '✓ Report Copied!';
      btnDiag.classList.add('is-success');
      setTimeout(() => {
        btnDiag.textContent = originalText;
        btnDiag.classList.remove('is-success');
      }, 1800);
    });
  }

  // 12. Save Settings Mock Button
  const btnSave = showcase.querySelector('#btn-save-settings');
  const footerStatusText = showcase.querySelector('#footer-status-text');
  if (btnSave && footerStatusText) {
    btnSave.addEventListener('click', () => {
      const currentLangRu = (document.documentElement.lang || 'ru') === 'ru';
      const originalText = footerStatusText.textContent;
      footerStatusText.textContent = currentLangRu ? 'Сохранено' : 'Saved';
      setTimeout(() => {
        footerStatusText.textContent = originalText;
      }, 1500);
    });
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
      vec3 obsidian = vec3(0.015, 0.015, 0.02);  // Deep Obsidian Black
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
    indicator = document.createElement('div');
    indicator.className = 'nav-pill-indicator';
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

  const slotConfigs = [
    { top: 0, scale: 0.94, zIndex: 1, opacity: 0.35 },
    { top: 38, scale: 0.96, zIndex: 2, opacity: 0.55 },
    { top: 76, scale: 0.98, zIndex: 3, opacity: 0.75 },
    { top: 114, scale: 1.0, zIndex: 10, opacity: 1.0 }
  ];

  function applySlots(targetIndex) {
    const total = cards.length;
    // Circular rolling wheel assignment:
    // Slot 3 (front active) = cards[targetIndex]
    // Slot 2 (1 step behind) = cards[(targetIndex + 1) % total]
    // Slot 1 (2 steps behind) = cards[(targetIndex + 2) % total]
    // Slot 0 (furthest back)  = cards[(targetIndex + 3) % total]
    for (let offset = 1; offset < total; offset++) {
      const cardIdx = (targetIndex + offset) % total;
      const card = cards[cardIdx];
      const slotIdx = total - 1 - offset;
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
    const conf = slotConfigs[3];
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
