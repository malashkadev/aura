// Codemod: replace numeric z-index literals with the named layering scale.
// Usage: node design-lab/scale-z.mjs [--dry]

import { readFileSync, writeFileSync } from "node:fs";

const MAP = new Map([
  [1, "var(--z-base)"],
  [2, "var(--z-raised)"],
  [5, "var(--z-raised)"],
  [50, "var(--z-dropdown-host)"],
  [60, "var(--z-dropdown)"],
  [100, "var(--z-header)"],
  [1000, "var(--z-tour-backdrop)"],
  [1001, "var(--z-tour-highlight)"],
  [1002, "var(--z-tour-tooltip)"],
  [9999, "var(--z-modal)"],
]);

const FILE = "src/style.css";
const dry = process.argv.includes("--dry");
const src = readFileSync(FILE, "utf8");
let n = 0;
const out = src.replace(/z-index:\s*(\d+)\s*;/g, (whole, num) => {
  const next = MAP.get(Number(num));
  if (!next) return whole;
  n++;
  return `z-index: ${next};`;
});

if (!dry) writeFileSync(FILE, out);
const leftovers = [...out.matchAll(/z-index:\s*(\d+)\s*;/g)].map((m) => m[1]);
console.log(`replaced: ${n}`);
console.log(`remaining numeric z-index: ${leftovers.length ? leftovers.join(", ") : "none"}`);
