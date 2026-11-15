// Codemod: collapse font-size values onto a 9-step scale.
// Usage: node design-lab/scale-type.mjs [--dry]

import { readFileSync, writeFileSync } from "node:fs";

const MAP = new Map([
  ["0.7rem", "11px"],
  ["0.72rem", "12px"],
  ["0.75rem", "12px"],
  ["0.76rem", "12px"],
  ["0.8rem", "13px"],
  ["0.82rem", "13px"],
  ["0.84rem", "13px"],
  ["0.85rem", "14px"],
  ["0.86rem", "14px"],
  ["0.875rem", "14px"],
  ["0.88rem", "14px"],
  ["0.9rem", "14px"],
  ["0.92rem", "15px"],
  ["0.95rem", "15px"],
  ["1rem", "16px"],
  ["1.1rem", "18px"],
  ["1.2rem", "19px"],
  ["1.3rem", "21px"],
  ["10px", "11px"],
  ["10.5px", "11px"],
  ["12.5px", "13px"],
]);

const FILES = ["src/style.css", "src/tray-menu.css", "src/overlay.html"];
const dry = process.argv.includes("--dry");
const seen = new Map();
let changed = 0;

for (const file of FILES) {
  const src = readFileSync(file, "utf8");
  const out = src.replace(/font-size:\s*([^;!]+)(\s*!important)?;/g, (whole, value, bang = "") => {
    const v = value.trim();
    if (v === "inherit" || v.startsWith("var(")) return whole;
    const next = MAP.get(v);
    if (!next) {
      seen.set(v, (seen.get(v) ?? 0) + 1);
      return whole;
    }
    changed++;
    return `font-size: ${next}${bang};`;
  });
  if (!dry) writeFileSync(file, out);
}

const leftover = [...seen.entries()];
console.log(`rewritten: ${changed}`);
if (leftover.length) {
  console.log("not in map:");
  for (const [v, n] of leftover) console.log(`  ${v} ×${n}`);
}
