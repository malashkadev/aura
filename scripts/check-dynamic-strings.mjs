import { readFileSync } from "node:fs";

const read = (p) => readFileSync(p, "utf8");
const mainJs = read("src/main.js");

console.log("=== CHECKING DYNAMIC / HARDCODED STRINGS IN MAIN.JS ===");

// Check calls to showStatus, alert, confirm, showConfirm, console.error/warn, or innerText/textContent assignments
const lines = mainJs.split("\n");
const suspicious = [];

lines.forEach((line, idx) => {
  const lineNum = idx + 1;
  const trimmed = line.trim();

  // Skip lines inside i18nDict
  if (lineNum > 20 && lineNum < 3900) return;

  // Check for showStatus("...") where string doesn't come from dict or getTranslation
  if (/showStatus\(\s*["'`][^"'`]+["'`]/.test(trimmed)) {
    suspicious.push({ lineNum, type: "showStatus hardcoded", text: trimmed });
  }

  // Check for textContent = "..." or innerHTML = "..." with Cyrillic or hardcoded English sentences
  if (/(?:textContent|innerText)\s*=\s*["'][А-Яа-яA-Za-z]{4,}/.test(trimmed)) {
    suspicious.push({ lineNum, type: "textContent hardcoded", text: trimmed });
  }

  // Check for setAttribute("title", "...") or setAttribute("aria-label", "...")
  if (/setAttribute\(\s*["'](?:title|aria-label)["']\s*,\s*["'][А-Яа-яA-Za-z]/.test(trimmed)) {
    suspicious.push({ lineNum, type: "attribute hardcoded", text: trimmed });
  }
});

console.log(`Found ${suspicious.length} suspicious hardcoded lines outside dictionary:`);
for (const s of suspicious) {
  console.log(`  Line ${s.lineNum} [${s.type}]: ${s.text}`);
}
