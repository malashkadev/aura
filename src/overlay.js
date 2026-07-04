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
    gainA.connect(audioCtx.destination);

    oscB.connect(gainB);
    gainB.connect(audioCtx.destination);

    oscA.start(now);
    oscA.stop(now + comp.decay + 0.1);

    oscB.start(now);
    oscB.stop(now + comp.decay + 0.1);
  });
}

function playStartSound() {
  // Deep meditation bowl strike: F3 (174.61Hz) with a 1.2s decay
  playBowl(174.61, 1.2, 0.08);
}

function playSuccessSound() {
  // Consonant pair of resonant singing bowls: F3 (174.61Hz) and C4 (261.63Hz) struck together.
  // Staggered slightly like a soft mallet sweep.
  playBowl(174.61, 1.8, 0.07);
  setTimeout(() => {
    playBowl(261.63, 1.8, 0.06);
  }, 60);
}

function playErrorSound() {
  // A soft, damped, shorter single bowl sound
  playBowl(174.61, 0.45, 0.07);
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
