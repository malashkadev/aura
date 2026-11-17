// @ts-check

/**
 * Shared Web Audio synthesizer for Aura overlay cues and settings previews.
 *
 * Centralizes all six acoustic themes ("zen", "rhodes", "scifi", "classic",
 * "bubble", "haptic"), tracks multi-note chord timers so rapid transitions
 * never overlap, and exposes `suspend()` so hidden windows release their
 * WASAPI audio render stream while idle.
 */

/** @typedef {"zen" | "rhodes" | "scifi" | "classic" | "bubble" | "haptic"} SoundThemeName */

/**
 * @typedef {Object} SoundSynth
 * @property {(nextVolume: number) => void} setVolume
 * @property {() => void} suspend
 * @property {() => void} clearNoteTimers
 * @property {(theme?: string) => void} playStart
 * @property {(theme?: string) => void} playSuccess
 * @property {(theme?: string) => void} playError
 */

/** @type {readonly SoundThemeName[]} */
export const SOUND_THEMES = [
  "zen",
  "rhodes",
  "scifi",
  "classic",
  "bubble",
  "haptic",
];

/**
 * @param {{ initialVolume?: number }} [options]
 * @returns {SoundSynth}
 */
export function createSoundSynth({ initialVolume = 0.8 } = {}) {
  /** @type {AudioContext | null} */
  let audioCtx = null;
  /** @type {GainNode | null} */
  let globalGain = null;
  let soundVolume = Math.max(0, Math.min(1, Number(initialVolume) || 0));
  /** @type {Array<ReturnType<typeof setTimeout>>} */
  const noteTimers = [];

  function clearNoteTimers() {
    while (noteTimers.length > 0) {
      const id = noteTimers.pop();
      if (id !== undefined) clearTimeout(id);
    }
  }

  /**
   * @param {() => void} fn
   * @param {number} delayMs
   */
  function scheduleNote(fn, delayMs) {
    const id = setTimeout(() => {
      const idx = noteTimers.indexOf(id);
      if (idx !== -1) noteTimers.splice(idx, 1);
      fn();
    }, delayMs);
    noteTimers.push(id);
  }

  /** @returns {boolean} */
  function ensureContext() {
    const audioWindow = /** @type {Window & typeof globalThis & { webkitAudioContext?: typeof AudioContext }} */ (window);
    const Ctx = audioWindow.AudioContext || audioWindow.webkitAudioContext;
    if (!Ctx) return false;
    if (!audioCtx) {
      audioCtx = new Ctx();
      globalGain = audioCtx.createGain();
      globalGain.gain.value = soundVolume;
      globalGain.connect(audioCtx.destination);
    }
    if (audioCtx.state === "suspended") {
      audioCtx.resume().catch(() => {});
    }
    if (globalGain) {
      globalGain.gain.setValueAtTime(soundVolume, audioCtx.currentTime);
    }
    return true;
  }

  /** @param {number} nextVolume */
  function setVolume(nextVolume) {
    const num = Number(nextVolume);
    soundVolume = Number.isFinite(num) ? Math.max(0, Math.min(1, num)) : 0.8;
    if (globalGain && audioCtx) {
      globalGain.gain.setValueAtTime(soundVolume, audioCtx.currentTime);
    }
  }

  function suspend() {
    clearNoteTimers();
    if (audioCtx && audioCtx.state === "running") {
      audioCtx.suspend().catch(() => {});
    }
  }

  /**
   * @param {number} freq
   * @param {number} duration
   * @param {number} [gainStart]
   */
  function playBowl(freq, duration, gainStart = 0.08) {
    if (!ensureContext() || !audioCtx || !globalGain) return;
    const ctx = audioCtx;
    const masterGain = globalGain;
    const now = ctx.currentTime;
    const components = [
      { f: freq, gain: gainStart, decay: duration },
      { f: freq * 1.5, gain: gainStart * 0.45, decay: duration * 0.8 },
      { f: freq * 2.76, gain: gainStart * 0.3, decay: duration * 0.55 },
    ];

    components.forEach((comp) => {
      const oscA = ctx.createOscillator();
      const gainA = ctx.createGain();
      oscA.type = "sine";
      oscA.frequency.setValueAtTime(comp.f - 0.7, now);

      const oscB = ctx.createOscillator();
      const gainB = ctx.createGain();
      oscB.type = "sine";
      oscB.frequency.setValueAtTime(comp.f + 0.7, now);

      const attack = 0.015;
      gainA.gain.setValueAtTime(0.0001, now);
      gainA.gain.linearRampToValueAtTime(comp.gain, now + attack);
      gainA.gain.exponentialRampToValueAtTime(0.0001, now + comp.decay);

      gainB.gain.setValueAtTime(0.0001, now);
      gainB.gain.linearRampToValueAtTime(comp.gain, now + attack);
      gainB.gain.exponentialRampToValueAtTime(0.0001, now + comp.decay);

      oscA.connect(gainA);
      gainA.connect(masterGain);
      oscB.connect(gainB);
      gainB.connect(masterGain);

      oscA.start(now);
      oscA.stop(now + comp.decay + 0.1);
      oscB.start(now);
      oscB.stop(now + comp.decay + 0.1);
    });
  }

  /**
   * @param {number} freq
   * @param {number} duration
   * @param {number} [gainStart]
   */
  function playRhodes(freq, duration, gainStart = 0.08) {
    if (!ensureContext() || !audioCtx || !globalGain) return;
    const now = audioCtx.currentTime;
    const osc1 = audioCtx.createOscillator();
    const osc2 = audioCtx.createOscillator();
    const gain1 = audioCtx.createGain();
    const gain2 = audioCtx.createGain();

    osc1.type = "triangle";
    osc1.frequency.setValueAtTime(freq, now);
    osc2.type = "sine";
    osc2.frequency.setValueAtTime(freq, now);

    const attack = 0.01;
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

  /**
   * @param {number} freqStart
   * @param {number} freqEnd
   * @param {number} duration
   * @param {number} [gainStart]
   */
  function playSciFi(freqStart, freqEnd, duration, gainStart = 0.06) {
    if (!ensureContext() || !audioCtx || !globalGain) return;
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

  /**
   * @param {number} freq
   * @param {number} duration
   * @param {number} [gainStart]
   */
  function playBell(freq, duration, gainStart = 0.07) {
    if (!ensureContext() || !audioCtx || !globalGain) return;
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

  /**
   * @param {number} fStart
   * @param {number} fEnd
   * @param {number} duration
   * @param {number} [gainStart]
   */
  function playWaterDrop(fStart, fEnd, duration, gainStart = 0.08) {
    if (!ensureContext() || !audioCtx || !globalGain) return;
    const now = audioCtx.currentTime;
    const osc = audioCtx.createOscillator();
    const gain = audioCtx.createGain();

    osc.type = "sine";
    osc.frequency.setValueAtTime(fStart, now);
    osc.frequency.exponentialRampToValueAtTime(fEnd, now + duration * 0.65);
    osc.frequency.exponentialRampToValueAtTime(fEnd * 0.9, now + duration);

    gain.gain.setValueAtTime(0.0001, now);
    gain.gain.linearRampToValueAtTime(gainStart, now + 0.008);
    gain.gain.exponentialRampToValueAtTime(0.0001, now + duration);

    osc.connect(gain);
    gain.connect(globalGain);

    osc.start(now);
    osc.stop(now + duration + 0.02);
  }

  /**
   * @param {number} clickFreq
   * @param {number} duration
   * @param {number} [gainStart]
   */
  function playHapticClick(clickFreq, duration, gainStart = 0.1) {
    if (!ensureContext() || !audioCtx || !globalGain) return;
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

  /** @param {string} [theme] */
  function playStart(theme = "zen") {
    if (soundVolume <= 0) return;
    clearNoteTimers();
    try {
      if (theme === "rhodes") {
        playRhodes(440.0, 0.65, 0.07);
        scheduleNote(() => playRhodes(554.37, 0.8, 0.07), 90);
      } else if (theme === "scifi") {
        playSciFi(440.0, 880.0, 0.22, 0.055);
      } else if (theme === "classic") {
        playBell(440.0, 0.45, 0.06);
        scheduleNote(() => playBell(659.25, 0.6, 0.06), 90);
      } else if (theme === "bubble") {
        playWaterDrop(400, 950, 0.12, 0.08);
      } else if (theme === "haptic") {
        playHapticClick(180, 0.03, 0.12);
      } else {
        playBowl(174.61, 1.2, 0.08);
      }
    } catch (_) {}
  }

  /** @param {string} [theme] */
  function playSuccess(theme = "zen") {
    if (soundVolume <= 0) return;
    clearNoteTimers();
    try {
      if (theme === "rhodes") {
        playRhodes(220.0, 0.45, 0.06);
        scheduleNote(() => playRhodes(329.63, 0.45, 0.06), 70);
        scheduleNote(() => playRhodes(440.0, 0.45, 0.06), 140);
        scheduleNote(() => playRhodes(554.37, 0.45, 0.06), 210);
        scheduleNote(() => playRhodes(659.25, 0.85, 0.06), 280);
      } else if (theme === "scifi") {
        playSciFi(523.25, 1046.5, 0.18, 0.05);
        scheduleNote(() => playSciFi(783.99, 1567.98, 0.18, 0.045), 65);
      } else if (theme === "classic") {
        playBell(440.0, 0.35, 0.05);
        scheduleNote(() => playBell(554.37, 0.35, 0.05), 70);
        scheduleNote(() => playBell(659.25, 0.45, 0.05), 140);
        scheduleNote(() => playBell(880.0, 0.75, 0.05), 210);
      } else if (theme === "bubble") {
        playWaterDrop(380, 850, 0.1, 0.07);
        scheduleNote(() => playWaterDrop(500, 1200, 0.15, 0.08), 90);
      } else if (theme === "haptic") {
        playHapticClick(240, 0.025, 0.1);
        scheduleNote(() => playHapticClick(360, 0.04, 0.12), 70);
      } else {
        playBowl(174.61, 1.8, 0.07);
        scheduleNote(() => playBowl(261.63, 1.8, 0.06), 60);
      }
    } catch (_) {}
  }

  /** @param {string} [theme] */
  function playError(theme = "zen") {
    if (soundVolume <= 0) return;
    clearNoteTimers();
    try {
      if (theme === "rhodes") {
        playRhodes(554.37, 0.35, 0.07);
        scheduleNote(() => playRhodes(440.0, 0.55, 0.07), 120);
      } else if (theme === "scifi") {
        playSciFi(587.33, 293.66, 0.28, 0.06);
      } else if (theme === "classic") {
        playBell(659.25, 0.35, 0.06);
        scheduleNote(() => playBell(440.0, 0.5, 0.06), 120);
      } else if (theme === "bubble") {
        playWaterDrop(650, 280, 0.18, 0.08);
      } else if (theme === "haptic") {
        playHapticClick(120, 0.05, 0.12);
        scheduleNote(() => playHapticClick(90, 0.08, 0.12), 100);
      } else {
        playBowl(174.61, 0.45, 0.07);
      }
    } catch (_) {}
  }

  return {
    setVolume,
    suspend,
    clearNoteTimers,
    playStart,
    playSuccess,
    playError,
  };
}
