import { readFileSync } from "node:fs";

const read = (p) => readFileSync(p, "utf8");
const mainJs = read("src/main.js");

console.log("=== CHECKING DYNAMIC / HARDCODED STRINGS IN MAIN.JS ===");

const lines = mainJs.split("\n");
const suspicious = [];

lines.forEach((line, idx) => {
  const lineNum = idx + 1;
  const trimmed = line.trim();

  // Skip lines inside i18nDict.
  if (lineNum > 20 && lineNum < 3900) return;

  // A template literal that starts with a translation lookup is localized;
  // only literal strings and untranslated templates are actionable.
  if (/showStatus\(\s*["'`]/.test(trimmed) && !/showStatus\(\s*`\$\{getTranslation\(/.test(trimmed)) {
    suspicious.push({ lineNum, type: "showStatus hardcoded", text: trimmed });
  }

  if (/(?:textContent|innerText)\s*=\s*["'][\u0410-\u042f\u0430-\u044fA-Za-z]{4,}/.test(trimmed)) {
    suspicious.push({ lineNum, type: "textContent hardcoded", text: trimmed });
  }

  if (/setAttribute\(\s*["'](?:title|aria-label)["']\s*,\s*["'][\u0410-\u042f\u0430-\u044fA-Za-z]/.test(trimmed)) {
    suspicious.push({ lineNum, type: "attribute hardcoded", text: trimmed });
  }
});

console.log(`Found ${suspicious.length} suspicious hardcoded line(s) outside the dictionary:`);
for (const item of suspicious) {
  console.log(`  Line ${item.lineNum} [${item.type}]: ${item.text}`);
}

console.log(`\nDynamic-string result: ${suspicious.length === 0 ? "PASS" : "FAIL"}`);
if (suspicious.length > 0) {
  process.exitCode = 1;
}
