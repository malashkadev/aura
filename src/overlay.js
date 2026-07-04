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
  gain1.connect(audioCtx.destination);
  osc2.connect(gain2);
  gain2.connect(audioCtx.destination);

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
  gainNode.connect(audioCtx.destination);

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

let soundsEnabled = true;
let soundTheme = "zen";

async function updateActiveThemeSettings() {
  try {
    const settings = await window.__TAURI__.core.invoke("get_settings");
    soundsEnabled = settings.overlay_sounds !== false;
    soundTheme = settings.overlay_sound_theme || "zen";
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
