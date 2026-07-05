// Interactive behaviors for Aura Website

document.addEventListener('DOMContentLoaded', () => {
  setupSpotlightGlow();
  setupSmoothScrolling();
  setupIntersectionObserver();
  setupMockSettings();
  startTypingSimulator();
  setupRevealOnScroll();
  setupScrollListeners();
  setupFAQAccordions();
});

/**
 * Creates a dynamic spotlight glow effect inside bento cards following the cursor
 */
function setupSpotlightGlow() {
  const cards = document.querySelectorAll('.bento-card');
  const bgContainer = document.querySelector('.aura-bg-container');
  
  // Local card spotlights
  cards.forEach(card => {
    card.addEventListener('mousemove', (e) => {
      const rect = card.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;
      
      card.style.setProperty('--mouse-x', `${x}px`);
      card.style.setProperty('--mouse-y', `${y}px`);
    });
  });

  // Global background spotlight
  if (bgContainer) {
    window.addEventListener('mousemove', (e) => {
      bgContainer.style.setProperty('--mouse-x', `${e.clientX}px`);
      bgContainer.style.setProperty('--mouse-y', `${e.clientY}px`);
    });
  }
}

/**
 * Handles smooth scrolling and updates the active nav link on click
 */
function setupSmoothScrolling() {
  const navLinks = document.querySelectorAll('.nav-link');
  
  navLinks.forEach(link => {
    link.addEventListener('click', (e) => {
      e.preventDefault();
      const targetId = link.getAttribute('href');
      
      if (targetId === '#') {
        window.scrollTo({ top: 0, behavior: 'smooth' });
        return;
      }
      
      const targetSection = document.querySelector(targetId);
      if (targetSection) {
        const offset = 72; // height of fixed header
        const targetPosition = targetSection.getBoundingClientRect().top + window.scrollY - offset;
        
        window.scrollTo({
          top: targetPosition,
          behavior: 'smooth'
        });
      }
    });
  });
}

/**
 * Highlights active navigation link based on scroll position
 */
function setupIntersectionObserver() {
  const sections = document.querySelectorAll('section[id]');
  const navLinks = document.querySelectorAll('.nav-link');
  
  const options = {
    root: null,
    rootMargin: '-80px 0px -60% 0px', // check when section covers top portion of viewport
    threshold: 0
  };
  
  const observer = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
      if (entry.isIntersecting) {
        const id = entry.target.getAttribute('id');
        
        navLinks.forEach(link => {
          if (link.getAttribute('href') === `#${id}`) {
            link.classList.add('active');
          } else {
            link.classList.remove('active');
          }
        });
      }
    });
  }, options);
  
  sections.forEach(section => observer.observe(section));
}

/**
 * Controls the live interactive Mock Settings panel
 */
function setupMockSettings() {
  const tabs = document.querySelectorAll('[data-mock-tab]');
  const panels = document.querySelectorAll('.mock-panel');
  const slider = document.getElementById('mock-sound-vol');
  const sliderVal = document.getElementById('mock-vol-val');
  
  const radioCloud = document.getElementById('mock-radio-cloud');
  const radioLocal = document.getElementById('mock-radio-local');
  const localModelCard = document.getElementById('mock-card-local-model');
  const modelCards = document.querySelectorAll('.mock-model-card');
  
  const apiProviderSelect = document.getElementById('mock-api-provider');
  const apiKeyInput = document.getElementById('mock-api-key');
  
  const clearHistoryBtn = document.getElementById('mock-btn-clear-history');
  const historyList = document.getElementById('mock-history-list');
  const historyEmpty = document.getElementById('mock-history-empty');
  
  // Tab switching for all 6 tabs
  tabs.forEach(tab => {
    tab.addEventListener('click', () => {
      const targetPanelId = `mock-panel-${tab.getAttribute('data-mock-tab')}`;
      
      tabs.forEach(t => t.classList.remove('mock-tab-active'));
      tab.classList.add('mock-tab-active');
      
      panels.forEach(panel => {
        if (panel.getAttribute('id') === targetPanelId) {
          panel.style.display = 'flex';
        } else {
          panel.style.display = 'none';
        }
      });
    });
  });

  // Slider value updates
  if (slider && sliderVal) {
    slider.addEventListener('input', (e) => {
      sliderVal.textContent = `${e.target.value}%`;
    });
  }

  // Toggling Local Whisper models list card based on Engine radio button
  if (radioCloud && radioLocal && localModelCard) {
    const handleEngineChange = () => {
      if (radioLocal.checked) {
        localModelCard.style.display = 'block';
      } else {
        localModelCard.style.display = 'none';
      }
    };
    
    radioCloud.addEventListener('change', handleEngineChange);
    radioLocal.addEventListener('change', handleEngineChange);
  }

  // Local Whisper models selection toggler
  modelCards.forEach(card => {
    card.addEventListener('click', () => {
      modelCards.forEach(c => c.classList.remove('mock-model-active'));
      card.classList.add('mock-model-active');
    });
  });

  // Dynamic API Key input placeholder based on provider selection
  if (apiProviderSelect && apiKeyInput) {
    const isRu = document.documentElement.lang === 'ru';
    apiProviderSelect.addEventListener('change', (e) => {
      const provider = e.target.value;
      if (provider === 'gemini') {
        apiKeyInput.placeholder = isRu ? 'Введите ваш API-ключ Gemini...' : 'Enter your Gemini API key...';
      } else if (provider === 'openai') {
        apiKeyInput.placeholder = isRu ? 'Введите ваш API-ключ OpenAI...' : 'Enter your OpenAI API key...';
      } else if (provider === 'groq') {
        apiKeyInput.placeholder = isRu ? 'Введите ваш API-ключ Groq...' : 'Enter your Groq API key...';
      }
    });
  }

  // Mock clearing history
  if (clearHistoryBtn && historyList && historyEmpty) {
    clearHistoryBtn.addEventListener('click', () => {
      historyList.style.display = 'none';
      historyEmpty.style.display = 'block';
      clearHistoryBtn.style.display = 'none';
    });
  }
}

/**
 * Runs the automatic typing simulator demonstrating Aura's core features
 */
function startTypingSimulator() {
  const editorText = document.getElementById('demo-editor-text');
  const overlayPill = document.getElementById('mock-overlay-pill');
  const overlayStatus = document.getElementById('mock-overlay-status');
  const soundBars = document.querySelectorAll('#mock-overlay-pill .sound-bar');
  
  if (!editorText || !overlayPill || !overlayStatus) return;

  const isRu = document.documentElement.lang === 'ru';
  const rawPhrase = isRu ? "Привет... эээ... запусти локальный Whisper." : "Hello... uh... start local Whisper.";
  const cleanPhrase = isRu ? "Привет, запусти локальный Whisper." : "Hello, start local Whisper.";
  
  let breathingInterval = null;
  let timerInterval = null;

  function stopAllAnimations() {
    if (breathingInterval) clearInterval(breathingInterval);
    if (timerInterval) clearInterval(timerInterval);
    
    // Reset heights to default
    soundBars.forEach(bar => {
      bar.removeAttribute("style"); // Remove any inline styles from JS animation loop
      bar.setAttribute("height", "6");
      bar.setAttribute("y", "11.6001");
    });
  }

  async function runSimulationLoop() {
    stopAllAnimations();

    // Reset editor state
    editorText.textContent = "";
    editorText.style.color = 'var(--text-primary)';
    overlayPill.classList.remove('active', 'processing');
    overlayStatus.textContent = "0:00";
    
    await sleep(1500);
    
    // Step 1: Open Dictation Pill Overlay (Start recording state)
    // Staggered waves are driven smoothly by CSS animations
    overlayPill.classList.add('active');
    
    // Increment timer text: 0:00, 0:01, 0:02
    let seconds = 0;
    timerInterval = setInterval(() => {
      seconds++;
      overlayStatus.textContent = `0:0${seconds}`;
    }, 1000);
    
    await sleep(800);
    
    // Step 2: Type spoken phrase letter-by-letter
    for (let i = 0; i < rawPhrase.length; i++) {
      editorText.textContent += rawPhrase[i];
      await sleep(35 + Math.random() * 70);
    }
    
    // Stop recording state animations
    clearInterval(timerInterval);
    
    await sleep(400);
    
    // Step 3: Trigger AI formatting status ("Обработка…")
    overlayPill.classList.add('processing');
    overlayStatus.textContent = isRu ? "Обработка…" : "Processing...";
    
    // Start actual app's 60fps sinusoidal breathing wave animation
    let angle = 0;
    breathingInterval = setInterval(() => {
      soundBars.forEach((bar, index) => {
        const distFromCenter = Math.abs(index - 4);
        const h = 5 + Math.sin(angle - distFromCenter * 0.45) * 5.5;
        const y = 14.6 - (h / 2);
        bar.setAttribute("height", h.toString());
        bar.setAttribute("y", y.toString());
      });
      angle += 0.05;
    }, 16); // ~60fps
    
    await sleep(1800);
    
    // Step 4: AI Formatting flash transition
    editorText.style.color = 'var(--accent-color)';
    await sleep(180);
    
    // Step 5: Insert clean text, return color, stop animations, hide overlay
    editorText.textContent = cleanPhrase;
    editorText.style.color = 'var(--text-primary)';
    
    stopAllAnimations();
    overlayPill.classList.remove('active', 'processing');
    
    // Delay before looping again
    await sleep(4000);
    runSimulationLoop();
  }

  function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
  }

  runSimulationLoop();
}



/**
 * Triggers smooth entry transitions for key components as they enter the viewport
 */
function setupRevealOnScroll() {
  const reveals = document.querySelectorAll('.reveal-on-scroll');
  
  const observer = new IntersectionObserver((entries) => {
    entries.forEach(entry => {
      if (entry.isIntersecting) {
        entry.target.classList.add('revealed');
        observer.unobserve(entry.target); // Trigger once
      }
    });
  }, {
    root: null,
    threshold: 0.05,
    rootMargin: '0px 0px -50px 0px'
  });
  
  reveals.forEach(el => observer.observe(el));
}

/**
 * Handles scroll-dependent visibility for header buttons and the scroll-to-top button
 */
function setupScrollListeners() {
  const headerActions = document.querySelector('.site-header .header-actions');
  const scrollTopBtn = document.getElementById('scroll-top-btn');
  
  const handleScroll = () => {
    const scrollPos = window.scrollY;
    
    if (scrollPos > 400) {
      if (headerActions) headerActions.classList.add('visible');
      if (scrollTopBtn) scrollTopBtn.classList.add('visible');
    } else {
      if (headerActions) headerActions.classList.remove('visible');
      if (scrollTopBtn) scrollTopBtn.classList.remove('visible');
    }
  };
  
  window.addEventListener('scroll', handleScroll);
  
  // Smoothly scroll back to top on click
  if (scrollTopBtn) {
    scrollTopBtn.addEventListener('click', () => {
      window.scrollTo({
        top: 0,
        behavior: 'smooth'
      });
    });
  }
}

/**
 * Handles independent folding accordion drawers for the FAQ block
 */
function setupFAQAccordions() {
  const faqItems = document.querySelectorAll('.faq-item');
  
  faqItems.forEach(item => {
    const btn = item.querySelector('.faq-question-btn');
    const pane = item.querySelector('.faq-answer-pane');
    
    if (!btn || !pane) return;
    
    btn.addEventListener('click', () => {
      const isActive = item.classList.contains('faq-active');
      
      // Close other items (classic accordion logic)
      faqItems.forEach(otherItem => {
        if (otherItem !== item) {
          otherItem.classList.remove('faq-active');
          const otherBtn = otherItem.querySelector('.faq-question-btn');
          const otherPane = otherItem.querySelector('.faq-answer-pane');
          if (otherBtn) otherBtn.setAttribute('aria-expanded', 'false');
          if (otherPane) otherPane.style.maxHeight = '0';
        }
      });
      
      // Toggle current item
      if (isActive) {
        item.classList.remove('faq-active');
        btn.setAttribute('aria-expanded', 'false');
        pane.style.maxHeight = '0';
      } else {
        item.classList.add('faq-active');
        btn.setAttribute('aria-expanded', 'true');
        pane.style.maxHeight = `${pane.scrollHeight}px`;
      }
    });
  });
}


