const { listen } = window.__TAURI__.event;

const bars = document.querySelectorAll(".sound-bar");
const statusEl = document.getElementById("overlay-status");
const pill = document.querySelector(".overlay-pill");
let currentState = "recording";
let recordStart = null;
let angle = 0;

let audioCtx = null;

function initAudioCtx() {
  if (!audioCtx) {
    audioCtx = new (window.AudioContext || window.webkitAudioContext)();
  }
}

// Multi-oscillator physical bell modeling synth using exponential decay envelope
function playBell(freq, duration, gainStart = 0.07) {
  initAudioCtx();
  if (audioCtx.state === "suspended") {
    audioCtx.resume();
  }

  const now = audioCtx.currentTime;

  // Layer 1: Fundamental frequency (sine wave for warm clean body)
  const osc1 = audioCtx.createOscillator();
  const gain1 = audioCtx.createGain();
  osc1.type = "sine";
  osc1.frequency.setValueAtTime(freq, now);

  // Layer 2: Warm overtone (sine wave octave higher for bell clarity)
  const osc2 = audioCtx.createOscillator();
  const gain2 = audioCtx.createGain();
  osc2.type = "sine";
  osc2.frequency.setValueAtTime(freq * 2.0, now);

  // Layer 3: Metallic chime ring (sine wave at 3x frequency)
  const osc3 = audioCtx.createOscillator();
  const gain3 = audioCtx.createGain();
  osc3.type = "sine";
  osc3.frequency.setValueAtTime(freq * 3.0, now);

  // Soft attack (6ms) to prevent clipping click, followed by natural exponential decay
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
  gain1.connect(audioCtx.destination);

  osc2.connect(gain2);
  gain2.connect(audioCtx.destination);

  osc3.connect(gain3);
  gain3.connect(audioCtx.destination);

  osc1.start(now);
  osc1.stop(now + duration + 0.1);

  osc2.start(now);
  osc2.stop(now + duration + 0.1);

  osc3.start(now);
  osc3.stop(now + duration + 0.1);
}

function playStartSound() {
  // Warm, consonant perfect fifth chime: A4 (440Hz) -> E5 (659Hz)
  playBell(440.00, 0.45, 0.06);
  setTimeout(() => {
    playBell(659.25, 0.60, 0.06);
  }, 90);
}

function playSuccessSound() {
  // Melodic, premium ascending major triad bell chord: A4 (440Hz) -> C#5 (554Hz) -> E5 (659Hz) -> A5 (880Hz)
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
}

function playErrorSound() {
  // Soft, calming descending fifth chime: E5 (659Hz) -> A4 (440Hz)
  playBell(659.25, 0.35, 0.06);
  setTimeout(() => {
    playBell(440.00, 0.50, 0.06);
  }, 120);
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
    // Smooth sine wave scanning animation
    bars.forEach((bar, index) => {
      const h = 6 + Math.sin(angle + index * 0.8) * 8;
      const y = 14.6 - (h / 2);
      bar.setAttribute("height", h.toString());
      bar.setAttribute("y", y.toString());
    });
    angle += 0.12;
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

// Listen for recording-state updates: "recording" | "processing" | "error"
listen("recording-state", (event) => {
  currentState = event.payload;

  if (currentState === "recording") {
    recordStart = Date.now();
    statusEl.textContent = "0:00";
    statusEl.classList.remove("error");
    setBarColor("#FE4200");
    resetBars();

    // Animate show and play startup sound
    pill.classList.add("visible");
    playStartSound();
  } else if (currentState === "processing") {
    angle = 0;
    statusEl.textContent = "Обработка…";
    statusEl.classList.remove("error");
  } else if (currentState.startsWith("error")) {
    recordStart = null;
    let errMsg = "Ошибка распознавания";
    if (currentState.includes(":")) {
      errMsg = currentState.substring(currentState.indexOf(":") + 1);
    }
    statusEl.textContent = errMsg;
    statusEl.classList.add("error");
    setBarColor("#666666");
    resetBars();

    // Play error sound and animate hide after a brief delay
    playErrorSound();
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
    playSuccessSound();
  } else {
    playErrorSound();
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
