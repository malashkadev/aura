// UI audit against the Aura design-system norms.
// Run: node design-lab/audit-ui.mjs
// Checks the shipped front-end files for palette, radius, typography,
// shadow and focus discipline. Prints an inventory, not a verdict.

import { readFileSync } from "node:fs";

const FILES = [
  "src/style.css",
  "src/index.html",
  "src/overlay.html",
  "src/tray-menu.css",
  "src/tray-menu.html",
];

const RADIUS_SCALE = new Set(["2px", "4px", "6px", "8px", "10px", "11px", "12px", "14px", "20px", "999px", "9999px", "50%", "0", "0px", "var(--radius-xs)", "var(--radius-sm)", "var(--radius-md)", "var(--radius-lg)", "var(--radius-full)", "var(--radius-menu)", "var(--radius-item)"]);

const tokens = {
  "--bg-color": "#090909",
  "--surface-color": "#0d0d0d",
  "--text-primary": "#f5f7fa",
  "--text-secondary": "#9ba6b3",
  "--text-muted": "#64748b",
  "--accent-color": "#ff4200",
  "--accent-hover": "#e63b00",
};

const known = new Set([
  ...Object.values(tokens).map((v) => v.toLowerCase()),
  "#fff", "#ffffff", "#000", "#000000",
  // extended palette
  "#737f91", "rgba(18, 18, 18, 0.7)", "rgba(18, 18, 18, 0.72)", "rgba(9, 9, 9, 0.68)",
  // button gradient stops (website .btn-primary)
  "#ff4e10", "#eb3b00", "#ff5c22", "#f04000", "#e63e05", "#d83500",
  // status ramp: error / warning / health
  "#ff5c5c", "#ff7070", "#f87171", "#4ade80", "#ff5a1f", "hsl(0, 85%, 65%)",
  "hsl(0, 85%, 60%)", "hsl(0, 85%, 72%)", "hsla(0, 85%, 65%, 0.08)", "hsla(0, 85%, 65%, 0.4)",
  "hsla(0, 85%, 60%, 0.12)", "hsla(0, 85%, 60%, 0.2)", "hsla(0, 85%, 60%, 0.22)",
  "hsla(0, 85%, 60%, 0.35)", "hsl(40, 95%, 55%)",
  "rgba(34, 197, 94, 0.3)", "rgba(239, 68, 68, 0.35)",
  "rgba(255, 92, 92, 0.15)", "rgba(255, 92, 92, 0.28)",
  "rgba(255, 90, 31, 0.12)", "rgba(255, 90, 31, 0.22)",
  "rgba(255, 68, 68, 0.12)", "rgba(255, 68, 68, 0.2)",
  // overlay HUD: VU ramp and edit beam
  "#722205", "#fe4200", "#666666", "#8a8a8a",
  "rgba(254, 66, 0, 0.6)", "rgba(254, 66, 0, 0.4)", "rgba(254, 66, 0, 0.25)",
  "rgba(255, 120, 50, 0.85)",
  // documented exception: rating star is always yellow
  "#ffb800",
]);

function lines(text) {
  return text.split(/\r?\n/);
}

function collect(text, re, map) {
  const ls = lines(text);
  ls.forEach((line, i) => {
    for (const m of line.matchAll(re)) {
      const raw = (m[1] ?? m[0]).trim().toLowerCase().replace(/\s+/g, " ");
      if (!raw) continue;
      if (!map.has(raw)) map.set(raw, []);
      if (map.get(raw).length < 4) map.get(raw).push(i + 1);
    }
  });
}

const colors = new Map();
const radii = new Map();
const shadows = new Map();
const fontSizes = new Map();
const zIndex = new Map();
let bang = 0;

const textByFile = new Map();
for (const f of FILES) {
  let t;
  try {
    t = readFileSync(f, "utf8");
  } catch {
    continue;
  }
  textByFile.set(f, t);
  collect(t, /(#[0-9a-fA-F]{3,8})/g, colors);
  collect(t, /rgba?\([^)]*\)/g, colors);
  collect(t, /hsla?\([^)]*\)/g, colors);
  collect(t, /border-radius:\s*([^;]+);/g, radii);
  collect(t, /box-shadow:\s*([^;]+);/g, shadows);
  collect(t, /font-size:\s*([^;]+);/g, fontSizes);
  collect(t, /z-index:\s*([^;]+);/g, zIndex);
  bang += (t.match(/!important/g) ?? []).length;
}

function isBlueTint(value) {
  const m = value.match(/^rgba?\(\s*(\d+)[,\s]+(\d+)[,\s]+(\d+)/);
  if (!m) return false;
  const [r, g, b] = [Number(m[1]), Number(m[2]), Number(m[3])];
  return Math.abs(r - g) <= 2 && b - r >= 3;
}

function isPaletteCompatible(value) {
  if (value.startsWith("rgba(255, 255, 255,")) return true;
  if (value.startsWith("rgba(0, 0, 0,")) return true;
  if (value.startsWith("rgba(var(--accent-color-rgb)")) return true;
  if (value.startsWith("rgba(255, 66, 0,")) return true;
  if (/^hsla?\(\s*0\s*,\s*0%\s*,/.test(value)) return true; // neutral grey ramp
  return false;
}

const allColors = [...colors.entries()].sort((a, b) => b[1].length - a[1].length);
const offPalette = allColors.filter(([v]) => !known.has(v) && !isPaletteCompatible(v));
const blueTints = allColors.filter(([v]) => isBlueTint(v));

const badRadii = [...radii.entries()].filter(([v]) => !RADIUS_SCALE.has(v));
const dropShadows = [...shadows.entries()].filter(
  ([v]) => !v.startsWith("inset") && v !== "none" && !v.startsWith("0 0 0 9999px")
);
const textShadows = [];
for (const [f, t] of textByFile) {
  lines(t).forEach((line, i) => {
    if (/text-shadow:/.test(line) && !/none/.test(line)) textShadows.push(`${f}:${i + 1} ${line.trim()}`);
  });
}

const out = [];
out.push("# Aura UI audit\n");
out.push(`Files: ${[...textByFile.keys()].join(", ")}\n`);

out.push("## 1. Цвета вне палитры\n");
if (offPalette.length === 0) out.push("Нет.\n");
else {
  for (const [v, at] of offPalette) out.push(`- \`${v}\` — ${at.length} вхожд., строки ${at.join(", ")}`);
  out.push("");
}

out.push("## 2. Синие оттенки в серых заливках\n");
if (blueTints.length === 0) out.push("Нет.\n");
else {
  for (const [v, at] of blueTints) out.push(`- \`${v}\` — строки ${at.join(", ")}`);
  out.push("");
}

out.push("## 3. Радиусы вне шкалы\n");
if (badRadii.length === 0) out.push("Нет.\n");
else {
  for (const [v, at] of badRadii) out.push(`- \`${v}\` — строки ${at.join(", ")}`);
  out.push("");
}

out.push("## 4. Тени и свечения\n");
if (dropShadows.length === 0 && textShadows.length === 0) out.push("Нет.\n");
else {
  for (const [v, at] of dropShadows) out.push(`- box-shadow \`${v}\` — строки ${at.join(", ")}`);
  for (const s of textShadows) out.push(`- ${s}`);
  out.push("");
}

out.push("## 5. Шкала кеглей\n");
const sizes = [...fontSizes.entries()].sort((a, b) => b[1].length - a[1].length);
for (const [v, at] of sizes) out.push(`- \`${v}\` — ${at.length} вхожд.`);
out.push("");

out.push("## 6. z-index\n");
const zs = [...zIndex.entries()].sort((a, b) => Number(b[0]) - Number(a[0]));
for (const [v, at] of zs) out.push(`- \`${v}\` — строки ${at.join(", ")}`);
out.push("");

out.push(`## 7. !important: ${bang}\n`);

const css = textByFile.get("src/style.css") ?? "";
const focusRules = (css.match(/:focus-visible/g) ?? []).length;
const hoverRules = (css.match(/:hover/g) ?? []).length;
const activeRules = (css.match(/:active/g) ?? []).length;
out.push("## 8. Состояния\n");
out.push(`- :hover — ${hoverRules}`);
out.push(`- :active — ${activeRules}`);
out.push(`- :focus-visible — ${focusRules}\n`);

out.push("## 9. Палитра: полная инвентаризация\n");
for (const [v, at] of allColors) out.push(`- \`${v}\` — ${at.length} вхожд.`);

console.log(out.join("\n"));
