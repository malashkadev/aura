// @ts-check

/**
 * Shared SVG visualizers for the Aura voice recording overlay and Settings preview tiles.
 *
 * Keeps the overlay card surface and Aura brand logo 100% untouched; only the
 * visualization zone (x: 31..93, y: 4..26 inside viewBox="0 0 103 30") changes.
 */

export const OVERLAY_STYLES = /** @type {const} */ ([
  "wave",
  "matrix",
  "scale",
  "tape",
  "beam",
  "ribbon",
]);

/** @typedef {(typeof OVERLAY_STYLES)[number]} OverlayStyleId */

/**
 * @typedef {Object} VisualizerFrameInput
 * @property {number} volume
 * @property {number} energy
 * @property {number} nowMs
 * @property {number} dt
 * @property {number} procFactor
 * @property {string} state
 * @property {boolean} [previewActive]
 */

/**
 * @typedef {Object} OverlayVisualizerController
 * @property {OverlayStyleId} styleId
 * @property {(state: string) => void} reset
 * @property {(color: string) => void} setColor
 * @property {(state: string, volume: number) => void} applyReducedMotion
 * @property {(input: VisualizerFrameInput) => void} update
 */

const SVG_NS = "http://www.w3.org/2000/svg";

const COLOR_OFF = { r: 28, g: 16, b: 12 };
const COLOR_SILENCE = { r: 114, g: 34, b: 5 }; // #722205
const COLOR_SPEECH = { r: 255, g: 66, b: 0 }; // #ff4200
const COLOR_PEAK = { r: 255, g: 132, b: 74 };
const COLOR_NEUTRAL_DIM = { r: 58, g: 58, b: 58 };

/**
 * @param {unknown} value
 * @returns {OverlayStyleId}
 */
export function normalizeOverlayStyle(value) {
  if (typeof value === "string") {
    for (const style of OVERLAY_STYLES) {
      if (style === value) return style;
    }
  }
  return "wave";
}

/**
 * @param {string} tag
 * @param {Record<string, string | number>} [attrs]
 * @returns {Element}
 */
function createSvgEl(tag, attrs = {}) {
  const node =
    typeof document.createElementNS === "function"
      ? document.createElementNS(SVG_NS, tag)
      : document.createElement(tag);
  for (const [key, val] of Object.entries(attrs)) {
    node.setAttribute(key, String(val));
  }
  return node;
}

/**
 * @param {{ r: number, g: number, b: number }} c1
 * @param {{ r: number, g: number, b: number }} c2
 * @param {number} t
 * @returns {string}
 */
function lerpRGB(c1, c2, t) {
  const k = Math.max(0, Math.min(1, t));
  const r = Math.round(c1.r + (c2.r - c1.r) * k);
  const g = Math.round(c1.g + (c2.g - c1.g) * k);
  const b = Math.round(c1.b + (c2.b - c1.b) * k);
  return `rgb(${r}, ${g}, ${b})`;
}

/**
 * @param {number} energy
 * @returns {string}
 */
function getHeatColor(energy) {
  return lerpRGB(COLOR_SILENCE, COLOR_SPEECH, energy);
}

/**
 * @param {number} level
 * @returns {string}
 */
function getLedRamp(level) {
  if (level < 0.28) return lerpRGB(COLOR_OFF, COLOR_SILENCE, level / 0.28);
  if (level < 0.92) return lerpRGB(COLOR_SILENCE, COLOR_SPEECH, (level - 0.28) / 0.64);
  return lerpRGB(COLOR_SPEECH, COLOR_PEAK, (level - 0.92) / 0.28);
}

/**
 * Removes all child nodes from an element safely across browser DOM and MockDOM.
 * @param {Element} node
 */
function clearChildren(node) {
  while (node.firstChild) {
    node.removeChild(node.firstChild);
  }
}

/**
 * Removes style-specific <defs> elements created by a previous visualizer.
 * @param {Element | null} defsEl
 */
function clearVizDefs(defsEl) {
  if (!defsEl) return;
  const toRemove = [];
  for (let i = 0; i < defsEl.childNodes.length; i++) {
    const child = /** @type {Element} */ (defsEl.childNodes[i]);
    if (child && typeof child.getAttribute === "function" && child.getAttribute("data-viz-def") === "1") {
      toRemove.push(child);
    }
  }
  for (const child of toRemove) {
    defsEl.removeChild(child);
  }
}

/**
 * Builds the wave (original 10-bar) visualizer.
 * @param {Element} hostGroup
 * @returns {OverlayVisualizerController}
 */
function createWaveController(hostGroup) {
  const restHeight = 2.5;
  const maxHeight = 16.0;
  /** @type {Array<{ el: Element, currentHeight: number, targetHeight: number }>} */
  const bars = [];
  let angle = 0;

  for (let i = 0; i < 10; i++) {
    const x = (32.4 + i * 6.3).toFixed(1);
    const rect = createSvgEl("rect", {
      class: "sound-bar",
      x,
      y: "13.75",
      width: "3",
      height: "2.5",
      rx: "1.25",
      fill: "#722205",
    });
    hostGroup.appendChild(rect);
    bars.push({ el: rect, currentHeight: restHeight, targetHeight: restHeight });
  }

  /** @param {string} color */
  function setColor(color) {
    for (const bar of bars) {
      bar.el.setAttribute("fill", color);
    }
  }

  /** @param {string} state */
  function reset(state) {
    angle = 0;
    for (const bar of bars) {
      bar.currentHeight = restHeight;
      bar.targetHeight = restHeight;
      bar.el.setAttribute("height", restHeight.toString());
      bar.el.setAttribute("y", (15.0 - restHeight / 2).toString());
      bar.el.setAttribute("rx", (restHeight / 2).toString());
      if (state === "recording") {
        bar.el.setAttribute("fill", "rgb(114, 34, 5)");
      }
    }
  }

  /**
   * @param {string} state
   * @param {number} volume
   */
  function applyReducedMotion(state, volume) {
    if (state === "recording") {
      const nowMs = Date.now();
      for (let index = 0; index < bars.length; index++) {
        const bar = bars[index];
        let h = restHeight;
        if (volume > 0.00005) {
          const boosted = Math.sqrt(volume);
          const time = nowMs * 0.007;
          const wave =
            0.35 +
            Math.sin(time * (0.9 + index * 0.12) + index * 1.6) * 0.4 +
            Math.cos(time * 0.65 - index * 0.95) * 0.25;
          h = Math.min(maxHeight, Math.max(restHeight, restHeight + boosted * 36.0 * Math.max(0.05, wave)));
        }
        bar.targetHeight = h;
        bar.currentHeight = h;
        bar.el.setAttribute("height", h.toFixed(2));
        bar.el.setAttribute("y", (15.0 - h / 2).toFixed(2));
        bar.el.setAttribute("rx", Math.min(1.5, h / 2).toFixed(2));
        const barEnergy = Math.max(0, Math.min(1, (h - restHeight) / 7.0));
        bar.el.setAttribute("fill", getHeatColor(barEnergy));
      }
    } else if (state === "processing") {
      for (const bar of bars) {
        bar.el.setAttribute("height", "4.0");
        bar.el.setAttribute("y", "13.0");
        bar.el.setAttribute("rx", "1.5");
        bar.el.setAttribute("fill", "#ff4200");
      }
    }
  }

  /** @param {VisualizerFrameInput} input */
  function update({ volume, nowMs, procFactor, state, previewActive = true }) {
    const recFactor = 1 - procFactor;
    angle = (angle + 0.05) % (Math.PI * 2);
    const activeVol = previewActive ? volume : 0;

    for (let index = 0; index < bars.length; index++) {
      const bar = bars[index];
      let h = restHeight;
      if (activeVol > 0.00005) {
        const boosted = Math.sqrt(activeVol);
        const time = nowMs * 0.007;
        const wave =
          0.35 +
          Math.sin(time * (0.9 + index * 0.12) + index * 1.6) * 0.4 +
          Math.cos(time * 0.65 - index * 0.95) * 0.25;
        h = restHeight + boosted * 36.0 * Math.max(0.05, wave);
        h = Math.min(maxHeight, Math.max(restHeight, h));
      }
      bar.targetHeight = h;

      const distFromCenter = Math.abs(index - 4.5);
      const targetProcH = 4.0 + Math.sin(angle - distFromCenter * 0.45) * 3.5;
      const blendedTarget = bar.targetHeight * recFactor + targetProcH * procFactor;
      bar.currentHeight += (blendedTarget - bar.currentHeight) * 0.16;

      const y = 15.0 - bar.currentHeight / 2;
      bar.el.setAttribute("height", bar.currentHeight.toFixed(2));
      bar.el.setAttribute("y", y.toFixed(2));
      bar.el.setAttribute("rx", Math.min(1.5, bar.currentHeight / 2).toFixed(2));

      if (state === "recording" || state === "processing") {
        const barEnergy = Math.max(0, Math.min(1, (bar.currentHeight - restHeight) / 7.0));
        const eff = procFactor > 0 ? barEnergy + (1 - barEnergy) * procFactor : barEnergy;
        bar.el.setAttribute("fill", getHeatColor(eff));
      }
    }
  }

  return { styleId: "wave", reset, setColor, applyReducedMotion, update };
}

/**
 * Builds the 10x3 hardware micro-LED matrix visualizer.
 * @param {Element} hostGroup
 * @returns {OverlayVisualizerController}
 */
function createMatrixController(hostGroup) {
  const M_COLS = 10;
  const M_ROWS = 3;
  /** @type {Array<{ c: number, r: number, el: Element, level: number, seed: number }>} */
  const cells = [];

  for (let c = 0; c < M_COLS; c++) {
    for (let r = 0; r < M_ROWS; r++) {
      const rect = createSvgEl("rect", {
        x: (32.8 + c * 6.2).toFixed(2),
        y: (7.7 + r * 5.7).toFixed(2),
        width: "3.2",
        height: "3.2",
        rx: "1.0",
        fill: r === 1 ? "rgb(66, 24, 9)" : "rgb(28, 16, 12)",
      });
      hostGroup.appendChild(rect);
      cells.push({
        c,
        r,
        el: rect,
        level: r === 1 ? 0.16 : 0.02,
        seed: (c * 1.73 + r * 3.17) % 6.283,
      });
    }
  }

  /** @param {string} color */
  function setColor(color) {
    for (const cell of cells) {
      cell.el.setAttribute("fill", cell.r === 1 ? color : "rgb(28, 16, 12)");
    }
  }

  /** @param {string} state */
  function reset(state) {
    for (const cell of cells) {
      cell.level = cell.r === 1 ? 0.16 : 0.02;
      if (state === "recording") {
        cell.el.setAttribute("fill", getLedRamp(cell.level));
      }
    }
  }

  /**
   * @param {string} state
   * @param {number} volume
   */
  function applyReducedMotion(state, volume) {
    if (state === "processing") {
      setColor("#ff4200");
      return;
    }
    const energy = volume > 0.0001 ? Math.min(1, Math.sqrt(volume) * 2.05) : 0;
    for (const cell of cells) {
      const isCenter = cell.r === 1;
      const lv = energy > 0.05 ? Math.min(1, energy * (isCenter ? 1.1 : 0.65)) : isCenter ? 0.18 : 0.02;
      cell.level = lv;
      cell.el.setAttribute("fill", getLedRamp(lv));
    }
  }

  /** @param {VisualizerFrameInput} input */
  function update({ energy, nowMs, procFactor, state, previewActive = true }) {
    const sec = nowMs / 1000;
    const recFactor = 1 - procFactor;
    const activeE = previewActive ? energy : 0;
    const idleScan = ((sec * 2.2) % (M_COLS + 4)) - 2;
    const procScan = ((sec * 8.5) % (M_COLS + 5)) - 2.5;

    for (const cell of cells) {
      const isCenterRow = cell.r === 1;
      let recTarget = isCenterRow ? 0.16 : 0.02;
      if (activeE > 0.04) {
        const cellWave = 0.5 + 0.5 * Math.sin(sec * (5.8 + cell.c * 0.48) + cell.seed + cell.r * 2.1);
        const cellSpark = 0.5 + 0.5 * Math.cos(sec * 9.4 - cell.c * 1.25 + cell.r * 1.9);
        const mix = cellWave * 0.52 + cellSpark * 0.48;
        const threshold = isCenterRow ? 0.1 : 0.28;
        const raw = (activeE * (0.68 + mix * 1.05) - threshold) / (1 - threshold);
        recTarget = Math.max(isCenterRow ? 0.06 : 0.02, Math.min(1.18, raw * 1.38));
      } else if (isCenterRow && previewActive) {
        const dist = Math.abs(cell.c - idleScan);
        recTarget = 0.14 + Math.max(0, 1 - dist / 2.4) * 0.24;
      }

      const chevronOffset = isCenterRow ? 0 : -1.0;
      const pDist = Math.abs(cell.c - (procScan + chevronOffset));
      const procTarget = Math.max(0.12, Math.min(1.0, 1.0 - pDist / 2.2));

      const target = recTarget * recFactor + procTarget * procFactor;
      const speed = target > cell.level ? 0.38 : 0.13;
      cell.level += (target - cell.level) * speed;

      if (state === "recording" || state === "processing") {
        cell.el.setAttribute("fill", getLedRamp(cell.level));
      }
    }
  }

  return { styleId: "matrix", reset, setColor, applyReducedMotion, update };
}

/**
 * Builds the precision vernier rangefinder scale visualizer.
 * @param {Element} hostGroup
 * @returns {OverlayVisualizerController}
 */
function createScaleController(hostGroup) {
  const V_MIN = 35.0;
  const V_MAX = 89.0;
  const V_TICKS = 15;

  hostGroup.appendChild(
    createSvgEl("line", {
      x1: V_MIN,
      y1: "15",
      x2: V_MAX,
      y2: "15",
      stroke: "rgba(255, 255, 255, 0.06)",
      "stroke-width": "0.65",
    }),
  );

  /** @type {Array<{ major: boolean, el: Element }>} */
  const ticks = [];
  for (let i = 0; i < V_TICKS; i++) {
    const x = V_MIN + (i / (V_TICKS - 1)) * (V_MAX - V_MIN);
    const major = i % 2 === 0;
    const h = major ? 5.2 : 2.8;
    const tick = createSvgEl("rect", {
      x: (x - 0.5).toFixed(2),
      y: (15.0 - h / 2).toFixed(2),
      width: "1.0",
      height: h.toFixed(2),
      rx: "0.5",
      fill: major ? "rgba(255, 255, 255, 0.16)" : "rgba(255, 255, 255, 0.10)",
    });
    hostGroup.appendChild(tick);
    ticks.push({ major, el: tick });
  }

  const peak = createSvgEl("rect", {
    x: V_MIN.toFixed(2),
    y: "10.75",
    width: "1.2",
    height: "8.5",
    rx: "0.6",
    fill: "#8c2b06",
    opacity: "0",
  });
  hostGroup.appendChild(peak);

  const needle = createSvgEl("rect", {
    x: (V_MIN - 1.2).toFixed(2),
    y: "8.5",
    width: "2.4",
    height: "13.0",
    rx: "1.2",
    fill: "#722205",
  });
  hostGroup.appendChild(needle);

  let needlePos = 0;
  let peakPos = 0;
  let peakHoldFrames = 0;

  /** @param {string} color */
  function setColor(color) {
    needle.setAttribute("fill", color);
    peak.setAttribute("opacity", "0");
    for (const tk of ticks) {
      tk.el.setAttribute("fill", tk.major ? "rgba(255, 255, 255, 0.16)" : "rgba(255, 255, 255, 0.10)");
    }
  }

  /** @param {string} state */
  function reset(state) {
    needlePos = 0;
    peakPos = 0;
    peakHoldFrames = 0;
    needle.setAttribute("x", (V_MIN - 1.2).toFixed(2));
    peak.setAttribute("opacity", "0");
    if (state === "recording") {
      setColor("#722205");
    }
  }

  /**
   * @param {string} state
   * @param {number} volume
   */
  function applyReducedMotion(state, volume) {
    if (state === "processing") {
      needle.setAttribute("x", ((V_MIN + V_MAX) / 2 - 1.2).toFixed(2));
      needle.setAttribute("fill", "#ff4200");
      peak.setAttribute("opacity", "0");
      return;
    }
    const energy = volume > 0.0001 ? Math.min(1, Math.sqrt(volume) * 2.05) : 0;
    const pos = energy > 0.03 ? Math.min(1, Math.pow(energy, 0.82) * 1.05) : 0;
    needlePos = pos;
    const nx = V_MIN + pos * (V_MAX - V_MIN);
    needle.setAttribute("x", (nx - 1.2).toFixed(2));
    needle.setAttribute("fill", getHeatColor(Math.min(1, pos * 1.8)));
    peak.setAttribute("opacity", "0");
  }

  /** @param {VisualizerFrameInput} input */
  function update({ energy, nowMs, procFactor, state, previewActive = true }) {
    const sec = nowMs / 1000;
    const recFactor = 1 - procFactor;

    const targetRec = previewActive
      ? energy > 0.03
        ? Math.min(1, Math.pow(energy, 0.82) * 1.05)
        : 0
      : 0.22;
    const targetProc = 0.5 + 0.5 * Math.sin(sec * 3.8 - Math.PI / 2);

    const target = targetRec * recFactor + targetProc * procFactor;
    needlePos += (target - needlePos) * (target > needlePos ? 0.3 : 0.12);

    if (recFactor > 0.5 && previewActive) {
      if (needlePos >= peakPos) {
        peakPos = needlePos;
        peakHoldFrames = 18;
      } else if (peakHoldFrames > 0) {
        peakHoldFrames--;
      } else {
        peakPos += (needlePos - peakPos) * 0.07;
      }
    } else {
      peakPos += (needlePos - peakPos) * 0.25;
    }

    const nx = V_MIN + needlePos * (V_MAX - V_MIN);
    const px = V_MIN + peakPos * (V_MAX - V_MIN);

    needle.setAttribute("x", (nx - 1.2).toFixed(2));
    peak.setAttribute("x", (px - 0.6).toFixed(2));
    const showPeak = previewActive
      ? recFactor * Math.max(0, Math.min(1, (peakPos - needlePos - 0.03) * 8))
      : 0;
    peak.setAttribute("opacity", showPeak.toFixed(2));

    if (state === "recording" || state === "processing") {
      const needleHeat = Math.max(procFactor, previewActive ? Math.min(1, needlePos * 1.8) : 0.16);
      needle.setAttribute("fill", getHeatColor(needleHeat));

      for (let i = 0; i < ticks.length; i++) {
        const tk = ticks[i];
        const norm = i / (V_TICKS - 1);
        if (procFactor > 0.5) {
          const dist = Math.abs(norm - needlePos);
          const glow = Math.max(0, 1 - dist * 5.2);
          tk.el.setAttribute(
            "fill",
            glow > 0.05 ? lerpRGB(COLOR_NEUTRAL_DIM, COLOR_SPEECH, glow) : "rgba(255, 255, 255, 0.14)",
          );
        } else if (norm <= needlePos + 0.015 && needlePos > 0.03) {
          const nearTip = Math.max(0, 1 - (needlePos - norm) * 2.4);
          tk.el.setAttribute("fill", lerpRGB(COLOR_SILENCE, COLOR_SPEECH, previewActive ? nearTip * 0.85 : 0.05));
        } else {
          tk.el.setAttribute("fill", tk.major ? "rgba(255, 255, 255, 0.16)" : "rgba(255, 255, 255, 0.10)");
        }
      }
    }
  }

  return { styleId: "scale", reset, setColor, applyReducedMotion, update };
}

/**
 * Builds the 14-slot fixed-height hardware shift-register tape visualizer.
 * @param {Element} hostGroup
 * @returns {OverlayVisualizerController}
 */
function createTapeController(hostGroup) {
  const T_SLOTS = 14;
  const RING_LEN = 96;
  /** @type {Array<{ el: Element, level: number }>} */
  const slots = [];
  const shiftRing = new Float32Array(RING_LEN);
  let ringHead = 0;
  let shiftAccum = 0;

  for (let i = 0; i < T_SLOTS; i++) {
    const rect = createSvgEl("rect", {
      x: (32.8 + i * 4.35).toFixed(2),
      y: "10.0",
      width: "2.8",
      height: "10.0",
      rx: "0.9",
      fill: "rgb(28, 16, 12)",
    });
    hostGroup.appendChild(rect);
    slots.push({ el: rect, level: i === 0 ? 0.2 : 0.02 });
  }

  /** @param {string} color */
  function setColor(color) {
    for (let i = 0; i < slots.length; i++) {
      slots[i].el.setAttribute("fill", i < 4 ? color : "rgb(28, 16, 12)");
    }
  }

  /** @param {string} state */
  function reset(state) {
    shiftRing.fill(0);
    ringHead = 0;
    shiftAccum = 0;
    for (let i = 0; i < slots.length; i++) {
      slots[i].level = i === 0 ? 0.18 : 0.02;
      if (state === "recording") {
        slots[i].el.setAttribute("fill", getLedRamp(slots[i].level));
      }
    }
  }

  /**
   * @param {string} state
   * @param {number} volume
   */
  function applyReducedMotion(state, volume) {
    if (state === "processing") {
      for (const slot of slots) {
        slot.el.setAttribute("fill", "#ff4200");
      }
      return;
    }
    const energy = volume > 0.0001 ? Math.min(1, Math.sqrt(volume) * 2.05) : 0;
    const litCount = Math.round(energy * T_SLOTS);
    for (let i = 0; i < slots.length; i++) {
      const lv = i < litCount ? Math.max(0.35, 1 - i / (T_SLOTS * 1.4)) : i === 0 ? 0.18 : 0.02;
      slots[i].level = lv;
      slots[i].el.setAttribute("fill", getLedRamp(lv));
    }
  }

  /** @param {VisualizerFrameInput} input */
  function update({ energy, nowMs, dt, procFactor, state, previewActive = true }) {
    const sec = nowMs / 1000;
    const recFactor = 1 - procFactor;

    shiftAccum += dt * 60;
    const steps = Math.min(4, Math.floor(shiftAccum));
    if (steps > 0) {
      shiftAccum -= steps;
      for (let s = 0; s < steps; s++) {
        ringHead = (ringHead + 1) % RING_LEN;
        shiftRing[ringHead] =
          previewActive && energy > 0.05 ? Math.min(1.15, Math.pow(energy, 0.85) * 1.18) : 0;
      }
    }

    const procScan = ((sec * 11.5) % (T_SLOTS + 6)) - 3;

    for (let i = 0; i < T_SLOTS; i++) {
      const slot = slots[i];
      const tapIdx = (ringHead - i * 3 + RING_LEN * 4) % RING_LEN;
      const rawSample = shiftRing[tapIdx];
      const cooling = 1 - (i / T_SLOTS) * 0.24;

      let recTarget = previewActive ? rawSample * cooling : i === 0 ? 0.24 : i === 1 ? 0.14 : 0.03;
      if (previewActive && rawSample <= 0.01) {
        recTarget = i === 0 ? 0.14 + 0.14 * (0.5 + 0.5 * Math.sin(sec * 2.8)) : 0.02;
      }

      const pDist = procScan - i;
      let procTarget = 0.08;
      if (pDist >= -0.8 && pDist <= 3.6) {
        procTarget = pDist < 0 ? 1 + pDist : Math.max(0.08, 1 - pDist / 3.8);
      }

      const target = recTarget * recFactor + procTarget * procFactor;
      slot.level += (target - slot.level) * 0.34;

      if (state === "recording" || state === "processing") {
        slot.el.setAttribute("fill", getLedRamp(slot.level));
      }
    }
  }

  return { styleId: "tape", reset, setColor, applyReducedMotion, update };
}

/**
 * Builds the symmetric center-out voice light beam visualizer.
 * @param {Element} hostGroup
 * @param {Element | null} defsEl
 * @param {string} uid
 * @returns {OverlayVisualizerController}
 */
function createBeamController(hostGroup, defsEl, uid) {
  const B_CX = 62.0;
  const B_X = 33.5;
  const B_W = 57.0;
  const gradId = `aura-beam-grad-${uid}`;

  if (defsEl) {
    const lg = createSvgEl("linearGradient", {
      id: gradId,
      x1: "0%",
      y1: "0%",
      x2: "100%",
      y2: "0%",
      "data-viz-def": "1",
    });
    lg.appendChild(createSvgEl("stop", { offset: "0%", "stop-color": "#ff4200" }));
    lg.appendChild(createSvgEl("stop", { offset: "50%", "stop-color": "#ff994d" }));
    lg.appendChild(createSvgEl("stop", { offset: "100%", "stop-color": "#ff4200" }));
    defsEl.appendChild(lg);
  }

  hostGroup.appendChild(
    createSvgEl("rect", {
      x: B_X,
      y: "13.7",
      width: B_W,
      height: "2.6",
      rx: "1.3",
      fill: "rgb(28, 16, 12)",
    }),
  );

  const echo = createSvgEl("rect", {
    x: (B_CX - 5).toFixed(2),
    y: "13.7",
    width: "10",
    height: "2.6",
    rx: "1.3",
    fill: "#722205",
    opacity: "0",
  });
  const core = createSvgEl("rect", {
    x: (B_CX - 4).toFixed(2),
    y: "13.5",
    width: "8",
    height: "3.0",
    rx: "1.5",
    fill: "#722205",
  });
  hostGroup.appendChild(echo);
  hostGroup.appendChild(core);

  let beamNorm = 0;
  let beamEchoNorm = 0;

  /** @param {string} color */
  function setColor(color) {
    core.setAttribute("fill", color);
    core.setAttribute("opacity", "1");
    echo.setAttribute("opacity", "0");
  }

  /** @param {string} state */
  function reset(state) {
    beamNorm = 0;
    beamEchoNorm = 0;
    core.setAttribute("x", (B_CX - 4).toFixed(2));
    core.setAttribute("y", "13.7");
    core.setAttribute("width", "8.0");
    core.setAttribute("height", "2.6");
    core.setAttribute("rx", "1.3");
    echo.setAttribute("opacity", "0");
    if (state === "recording") {
      core.setAttribute("fill", "#722205");
    }
  }

  /**
   * @param {string} state
   * @param {number} volume
   */
  function applyReducedMotion(state, volume) {
    if (state === "processing") {
      core.setAttribute("x", (B_CX - 14).toFixed(2));
      core.setAttribute("y", "13.4");
      core.setAttribute("width", "28.0");
      core.setAttribute("height", "3.2");
      core.setAttribute("rx", "1.6");
      core.setAttribute("fill", "#ff4200");
      echo.setAttribute("opacity", "0");
      return;
    }
    const energy = volume > 0.0001 ? Math.min(1, Math.sqrt(volume) * 2.05) : 0;
    const norm = energy > 0.03 ? Math.min(1, Math.pow(energy, 0.8) * 1.06) : 0;
    const w = 8 + norm * (B_W - 8);
    core.setAttribute("x", (B_CX - w / 2).toFixed(2));
    core.setAttribute("width", w.toFixed(2));
    core.setAttribute("fill", norm > 0.08 && defsEl ? `url(#${gradId})` : getHeatColor(norm * 3));
    echo.setAttribute("opacity", "0");
  }

  /** @param {VisualizerFrameInput} input */
  function update({ energy, nowMs, procFactor, state, previewActive = true }) {
    const sec = nowMs / 1000;
    const targetRec = previewActive && energy > 0.03 ? Math.min(1, Math.pow(energy, 0.8) * 1.06) : 0;
    beamNorm += (targetRec - beamNorm) * (targetRec > beamNorm ? 0.34 : 0.14);

    if (beamNorm > beamEchoNorm) {
      beamEchoNorm = beamNorm;
    } else {
      beamEchoNorm += (beamNorm - beamEchoNorm) * 0.07;
    }

    if (procFactor > 0.5) {
      const scanW = 19.0;
      const pos = 0.5 + 0.5 * Math.sin(sec * 3.8 - Math.PI / 2);
      const x = B_X + pos * (B_W - scanW);
      core.setAttribute("x", x.toFixed(2));
      core.setAttribute("y", "13.4");
      core.setAttribute("width", scanW.toFixed(2));
      core.setAttribute("height", "3.2");
      core.setAttribute("rx", "1.6");
      if (state === "recording" || state === "processing") {
        core.setAttribute("fill", defsEl ? `url(#${gradId})` : "#ff4200");
      }
      core.setAttribute("opacity", "1");
      echo.setAttribute("opacity", "0");
      return;
    }

    const idleW = previewActive ? 7.5 + 1.4 * (0.5 + 0.5 * Math.sin(sec * 2.4)) : 14.0;
    const coreW = Math.max(idleW, idleW + beamNorm * (B_W - idleW));
    const echoW = Math.max(coreW, idleW + beamEchoNorm * (B_W - idleW));
    const coreH = 2.6 + beamNorm * 1.2;

    echo.setAttribute("x", (B_CX - echoW / 2).toFixed(2));
    echo.setAttribute("width", echoW.toFixed(2));
    echo.setAttribute("opacity", (previewActive && echoW > coreW + 1.0 ? 0.72 : 0).toFixed(2));

    core.setAttribute("x", (B_CX - coreW / 2).toFixed(2));
    core.setAttribute("y", (15.0 - coreH / 2).toFixed(2));
    core.setAttribute("width", coreW.toFixed(2));
    core.setAttribute("height", coreH.toFixed(2));
    core.setAttribute("rx", (coreH / 2).toFixed(2));
    if (state === "recording" || state === "processing") {
      core.setAttribute(
        "fill",
        previewActive && beamNorm > 0.08 && defsEl
          ? `url(#${gradId})`
          : getHeatColor(previewActive ? beamNorm * 3 : 0.14),
      );
    }
    core.setAttribute("opacity", (0.78 + beamNorm * 0.22).toFixed(2));
  }

  return { styleId: "beam", reset, setColor, applyReducedMotion, update };
}

/**
 * Builds the crisp multi-layer fluid silk ribbon wave visualizer (no blur).
 * @param {Element} hostGroup
 * @param {Element | null} defsEl
 * @param {string} uid
 * @returns {OverlayVisualizerController}
 */
function createRibbonController(hostGroup, defsEl, uid) {
  const RIB_X0 = 31.5;
  const RIB_X1 = 92.5;
  const RIB_N = 28;
  const backGradId = `aura-rib-back-${uid}`;
  const midGradId = `aura-rib-mid-${uid}`;
  const lineGradId = `aura-rib-line-${uid}`;

  /** @type {Element | null} */
  let stopMid = null;
  /** @type {Element | null} */
  let stopCrest = null;

  if (defsEl) {
    const gBack = createSvgEl("linearGradient", {
      id: backGradId,
      x1: "31",
      y1: "15",
      x2: "93",
      y2: "15",
      gradientUnits: "userSpaceOnUse",
      "data-viz-def": "1",
    });
    [
      { offset: "0%", "stop-color": "#ff4200", "stop-opacity": "0" },
      { offset: "28%", "stop-color": "#d92b00", "stop-opacity": "0.85" },
      { offset: "55%", "stop-color": "#ff6e1a", "stop-opacity": "0.95" },
      { offset: "80%", "stop-color": "#ff4200", "stop-opacity": "0.8" },
      { offset: "100%", "stop-color": "#ff4200", "stop-opacity": "0" },
    ].forEach((s) => gBack.appendChild(createSvgEl("stop", s)));
    defsEl.appendChild(gBack);

    const gMid = createSvgEl("linearGradient", {
      id: midGradId,
      x1: "31",
      y1: "15",
      x2: "93",
      y2: "15",
      gradientUnits: "userSpaceOnUse",
      "data-viz-def": "1",
    });
    gMid.appendChild(createSvgEl("stop", { offset: "0%", "stop-color": "#ff4200", "stop-opacity": "0" }));
    gMid.appendChild(createSvgEl("stop", { offset: "22%", "stop-color": "#ff4200", "stop-opacity": "0.9" }));
    stopMid = createSvgEl("stop", { offset: "50%", "stop-color": "#ffaa52", "stop-opacity": "1" });
    gMid.appendChild(stopMid);
    gMid.appendChild(createSvgEl("stop", { offset: "78%", "stop-color": "#ff4200", "stop-opacity": "0.9" }));
    gMid.appendChild(createSvgEl("stop", { offset: "100%", "stop-color": "#ff4200", "stop-opacity": "0" }));
    defsEl.appendChild(gMid);

    const gLine = createSvgEl("linearGradient", {
      id: lineGradId,
      x1: "31",
      y1: "15",
      x2: "93",
      y2: "15",
      gradientUnits: "userSpaceOnUse",
      "data-viz-def": "1",
    });
    gLine.appendChild(createSvgEl("stop", { offset: "0%", "stop-color": "#722205", "stop-opacity": "0" }));
    gLine.appendChild(createSvgEl("stop", { offset: "20%", "stop-color": "#ff4200", "stop-opacity": "0.95" }));
    stopCrest = createSvgEl("stop", { offset: "50%", "stop-color": "#ffd099", "stop-opacity": "1" });
    gLine.appendChild(stopCrest);
    gLine.appendChild(createSvgEl("stop", { offset: "80%", "stop-color": "#ff4200", "stop-opacity": "0.95" }));
    gLine.appendChild(createSvgEl("stop", { offset: "100%", "stop-color": "#722205", "stop-opacity": "0" }));
    defsEl.appendChild(gLine);
  }

  const ribBack = createSvgEl("path", {
    d: "",
    fill: defsEl ? `url(#${backGradId})` : "#d92b00",
    "fill-opacity": "0.34",
  });
  const ribMid = createSvgEl("path", {
    d: "",
    fill: defsEl ? `url(#${midGradId})` : "#ff4200",
    "fill-opacity": "0.52",
  });
  const ribCrest = createSvgEl("path", {
    d: "",
    fill: "none",
    stroke: defsEl ? `url(#${lineGradId})` : "#ff4200",
    "stroke-width": "1.45",
    "stroke-linecap": "round",
  });
  hostGroup.appendChild(ribBack);
  hostGroup.appendChild(ribMid);
  hostGroup.appendChild(ribCrest);

  let ribAmp = 0.08;
  let ribPhase = 0;

  /**
   * @param {number} phase
   * @param {number} freq
   * @param {number} amp
   * @param {number} asym
   * @returns {string}
   */
  function buildSymmetricRibbon(phase, freq, amp, asym) {
    let top = "";
    let bot = "";
    for (let i = 0; i <= RIB_N; i++) {
      const t = i / RIB_N;
      const x = RIB_X0 + t * (RIB_X1 - RIB_X0);
      const env = Math.pow(Math.sin(Math.PI * t), 1.7);
      const wave = Math.sin(t * Math.PI * freq - phase);
      const halfH = Math.max(0.35, (0.38 + Math.abs(wave) * amp) * env);
      const shift = wave * asym * env;
      const yt = 15.0 - halfH + shift;
      const yb = 15.0 + halfH + shift;
      top += (i === 0 ? "M " : "L ") + `${x.toFixed(2)} ${yt.toFixed(2)} `;
      bot = `L ${x.toFixed(2)} ${yb.toFixed(2)} ` + bot;
    }
    return top + bot + "Z";
  }

  /**
   * @param {number} phase
   * @param {number} freq
   * @param {number} amp
   * @returns {string}
   */
  function buildCrestLine(phase, freq, amp) {
    let d = "";
    for (let i = 0; i <= RIB_N; i++) {
      const t = i / RIB_N;
      const x = RIB_X0 + t * (RIB_X1 - RIB_X0);
      const env = Math.pow(Math.sin(Math.PI * t), 1.65);
      const y = 15.0 + Math.sin(t * Math.PI * freq - phase) * amp * env;
      d += (i === 0 ? "M " : "L ") + `${x.toFixed(2)} ${y.toFixed(2)} `;
    }
    return d;
  }

  function applyStaticPaths(amp, phase, opacityScale = 1) {
    ribBack.setAttribute("d", buildSymmetricRibbon(phase * 0.85, 2.2, amp * 8.4, amp * 1.4));
    ribMid.setAttribute("d", buildSymmetricRibbon(phase * 1.15 + 1.1, 2.8, amp * 6.4, -amp * 1.2));
    ribCrest.setAttribute("d", buildCrestLine(phase * 1.25 + 0.4, 2.5, amp * 6.8));
    ribBack.setAttribute("fill-opacity", ((0.2 + amp * 0.24) * opacityScale).toFixed(2));
    ribMid.setAttribute("fill-opacity", ((0.3 + amp * 0.34) * opacityScale).toFixed(2));
    ribCrest.setAttribute("stroke-opacity", ((0.55 + amp * 0.45) * opacityScale).toFixed(2));
  }

  function restoreGradients() {
    ribBack.setAttribute("fill", defsEl ? `url(#${backGradId})` : "#d92b00");
    ribMid.setAttribute("fill", defsEl ? `url(#${midGradId})` : "#ff4200");
    ribCrest.setAttribute("stroke", defsEl ? `url(#${lineGradId})` : "#ff4200");
  }

  /** @param {string} color */
  function setColor(color) {
    ribBack.setAttribute("fill", color);
    ribMid.setAttribute("fill", color);
    ribCrest.setAttribute("stroke", color);
  }

  /** @param {string} state */
  function reset(state) {
    ribAmp = 0.08;
    ribPhase = 0;
    if (state === "recording") {
      restoreGradients();
    }
    applyStaticPaths(0.08, 0, 0.75);
  }

  /**
   * @param {string} state
   * @param {number} volume
   */
  function applyReducedMotion(state, volume) {
    restoreGradients();
    if (state === "processing") {
      applyStaticPaths(0.38, 0.9, 1);
      return;
    }
    const energy = volume > 0.0001 ? Math.min(1, Math.sqrt(volume) * 2.05) : 0;
    const amp = 0.08 + Math.pow(energy, 0.85) * 0.85;
    applyStaticPaths(amp, 0.6, 1);
  }

  /** @param {VisualizerFrameInput} input */
  function update({ energy, nowMs, dt, procFactor, state, previewActive = true }) {
    const sec = nowMs / 1000;
    const recFactor = 1 - procFactor;

    if (state === "recording" || state === "processing") {
      restoreGradients();
    }

    const targetRec = previewActive ? 0.08 + Math.pow(energy, 0.85) * 0.92 : 0.24;
    const targetProc = 0.42;
    const targetAmp = targetRec * recFactor + targetProc * procFactor;
    ribAmp += (targetAmp - ribAmp) * (targetAmp > ribAmp ? 0.26 : 0.12);

    if (previewActive || procFactor > 0.02) {
      const speed = (2.6 + energy * 4.4) * recFactor + 4.8 * procFactor;
      ribPhase += dt * speed;
    }

    const shimmerPct = 50 + Math.sin(sec * 2.1) * 20;
    if (stopMid) stopMid.setAttribute("offset", `${shimmerPct.toFixed(1)}%`);
    if (stopCrest) stopCrest.setAttribute("offset", `${(100 - shimmerPct).toFixed(1)}%`);

    applyStaticPaths(ribAmp, ribPhase, previewActive || procFactor > 0.02 ? 1 : 0.55);
  }

  applyStaticPaths(0.2, 0, 0.65);

  return { styleId: "ribbon", reset, setColor, applyReducedMotion, update };
}

/**
 * Mounts a visualizer into an existing SVG <g> container (e.g. #bars-group in overlay.html).
 * @param {unknown} styleId
 * @param {Element} hostGroup
 * @param {Element | null} defsEl
 * @param {string} [uid]
 * @returns {OverlayVisualizerController}
 */
export function mountOverlayVisualizer(styleId, hostGroup, defsEl, uid = "main") {
  const normalized = normalizeOverlayStyle(styleId);
  clearChildren(hostGroup);
  clearVizDefs(defsEl);

  switch (normalized) {
    case "matrix":
      return createMatrixController(hostGroup);
    case "scale":
      return createScaleController(hostGroup);
    case "tape":
      return createTapeController(hostGroup);
    case "beam":
      return createBeamController(hostGroup, defsEl, uid);
    case "ribbon":
      return createRibbonController(hostGroup, defsEl, uid);
    case "wave":
    default:
      return createWaveController(hostGroup);
  }
}

/**
 * Creates a complete 1:1 Aura overlay SVG (exact brand logo + visualizer) for settings preview tiles.
 * @param {unknown} styleId
 * @param {string} uid
 * @returns {{ svg: Element, controller: OverlayVisualizerController }}
 */
export function createOverlayPreviewSvg(styleId, uid) {
  const svg = createSvgEl("svg", {
    width: "103",
    height: "30",
    viewBox: "0 0 103 30",
    fill: "none",
    "aria-hidden": "true",
  });
  const defs = createSvgEl("defs");
  const clipId = `aura-prev-logo-${uid}`;
  const clipLogo = createSvgEl("clipPath", { id: clipId });
  clipLogo.appendChild(
    createSvgEl("rect", {
      width: "19.2",
      height: "19.2",
      fill: "white",
      transform: "matrix(-1 0 0 1 27.2 5)",
    }),
  );
  defs.appendChild(clipLogo);
  svg.appendChild(defs);

  const logoG = createSvgEl("g", {
    "clip-path": `url(#${clipId})`,
    transform: "translate(3.0, 4.03) scale(0.75)",
  });
  logoG.appendChild(
    createSvgEl("path", {
      d: "M12.2707 20.7875C10.4925 19.2829 9.39822 17.0943 9.39822 14.6322C9.39822 10.1183 13.0914 6.42505 17.6054 6.42505C22.1193 6.42505 25.8125 10.1183 25.8125 14.6322C25.8125 17.0943 24.7182 19.2829 22.94 20.7875",
      stroke: "#ff4200",
      "stroke-width": "2.7434",
    }),
  );
  logoG.appendChild(
    createSvgEl("path", {
      d: "M20.3763 11.601L25.6238 24.1871H21.844L19.6927 18.2358L17.9033 13.7724C17.796 13.4776 17.6955 13.3301 17.6017 13.3301C17.5213 13.3301 17.4274 13.4776 17.3202 13.7724L15.5308 18.2358L13.3795 24.1871H9.57961L14.8472 11.601C15.1153 10.9577 15.4705 10.522 15.9128 10.2942C16.3418 10.0663 16.9047 9.95239 17.6017 9.95239C18.4193 9.95239 19.0292 10.0797 19.4313 10.3344C19.82 10.5891 20.135 11.0113 20.3763 11.601Z",
      fill: "#ff4200",
    }),
  );
  svg.appendChild(logoG);

  const vizGroup = createSvgEl("g");
  svg.appendChild(vizGroup);
  const controller = mountOverlayVisualizer(styleId, vizGroup, defs, uid);

  return { svg, controller };
}
