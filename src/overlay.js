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

function playTone(freq, duration, type = "triangle", gainStart = 0.1) {
  initAudioCtx();
  if (audioCtx.state === "suspended") {
    audioCtx.resume();
  }

  const osc = audioCtx.createOscillator();
  const gainNode = audioCtx.createGain();

  osc.type = type;
  osc.frequency.setValueAtTime(freq, audioCtx.currentTime);

  // Linear ramp decay envelope
  gainNode.gain.setValueAtTime(gainStart, audioCtx.currentTime);
  gainNode.gain.linearRampToValueAtTime(0.0001, audioCtx.currentTime + duration);

  osc.connect(gainNode);
  gainNode.connect(audioCtx.destination);

  osc.start();
  osc.stop(audioCtx.currentTime + duration);
}

function playStartSound() {
  // Warm C5 (523Hz) followed by E5 (659Hz) chime
  playTone(523.25, 0.12, "sine", 0.08);
  setTimeout(() => {
    playTone(659.25, 0.18, "sine", 0.08);
  }, 80);
}

function playSuccessSound() {
  // Bright C5 (523Hz) -> G5 (784Hz) -> C6 (1046Hz) arpeggio
  playTone(523.25, 0.10, "triangle", 0.06);
  setTimeout(() => {
    playTone(783.99, 0.10, "triangle", 0.06);
    setTimeout(() => {
      playTone(1046.50, 0.22, "triangle", 0.06);
    }, 70);
  }, 70);
}

function playErrorSound() {
  // Soft descending G4 (392Hz) -> Eb4 (311Hz) cancel chime
  playTone(392.00, 0.12, "triangle", 0.07);
  setTimeout(() => {
    playTone(311.13, 0.22, "triangle", 0.07);
  }, 100);
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
