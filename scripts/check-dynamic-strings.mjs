import { readdirSync, readFileSync } from "node:fs";

const read = (p) => readFileSync(p, "utf8").replaceAll("\r\n", "\n");

console.log("=== CHECKING DYNAMIC / HARDCODED STRINGS ACROSS SRC/*.JS ===");

const jsFiles = readdirSync("src")
  .filter((f) => f.endsWith(".js"))
  .sort();

const suspicious = [];

function lineAtOffset(text, offset) {
  return text.slice(0, offset).split("\n").length;
}

function computeSkippedRanges(fileName, content) {
  const ranges = [];
  if (fileName === "main.js") {
    const startIdx = content.indexOf("const i18nDict =");
    const endMarker = "\n};\n\nlet currentLanguage";
    const endIdx = content.indexOf(endMarker, startIdx);
    if (startIdx === -1 || endIdx === -1) {
      suspicious.push({
        file: fileName,
        lineNum: 1,
        type: "dictionary marker missing",
        text: "Could not locate dynamic i18nDict..currentLanguage bounds in src/main.js",
      });
    } else {
      ranges.push([lineAtOffset(content, startIdx), lineAtOffset(content, endIdx + 3)]);
    }
  } else if (fileName === "tray-menu.js") {
    const i18nStart = content.indexOf("const I18N = {");
    const i18nEnd = content.indexOf("\n};", i18nStart);
    if (i18nStart !== -1 && i18nEnd !== -1) {
      ranges.push([lineAtOffset(content, i18nStart), lineAtOffset(content, i18nEnd + 3)]);
    }
    const langsStart = content.indexOf("const SUPPORTED_LANGS = [");
    const langsEnd = content.indexOf("\n];", langsStart);
    if (langsStart !== -1 && langsEnd !== -1) {
      ranges.push([lineAtOffset(content, langsStart), lineAtOffset(content, langsEnd + 3)]);
    }
  }
  return ranges;
}

for (const fileName of jsFiles) {
  const content = read(`src/${fileName}`);
  const skippedRanges = computeSkippedRanges(fileName, content);
  const lines = content.split("\n");

  lines.forEach((line, idx) => {
    const lineNum = idx + 1;
    if (skippedRanges.some(([from, to]) => lineNum >= from && lineNum <= to)) {
      return;
    }

    const trimmed = line.trim();
    if (!trimmed || trimmed.startsWith("//") || trimmed.startsWith("*")) {
      return;
    }

    // Skip locale table key-value entries in overlay.js (or shared i18n objects)
    if (/^(?:ru|en|de|es|fr|it|zh|pt|tr|ja|nl):\s*["']/.test(trimmed)) {
      return;
    }

    // A template literal that starts with a translation lookup is localized;
    // only literal strings and untranslated templates are actionable.
    if (
      /showStatus\(\s*["'`]/.test(trimmed) &&
      !/showStatus\(\s*`\$\{getTranslation\(/.test(trimmed)
    ) {
      suspicious.push({ file: fileName, lineNum, type: "showStatus hardcoded", text: trimmed });
    }

    if (/(?:textContent|innerText)\s*=\s*["'][\u0410-\u042f\u0430-\u044fA-Za-z]{4,}/.test(trimmed)) {
      suspicious.push({ file: fileName, lineNum, type: "textContent hardcoded", text: trimmed });
    }

    if (
      /setAttribute\(\s*["'](?:title|aria-label)["']\s*,\s*["'][\u0410-\u042f\u0430-\u044fA-Za-z]/.test(
        trimmed,
      )
    ) {
      suspicious.push({ file: fileName, lineNum, type: "attribute hardcoded", text: trimmed });
    }
  });
}

console.log(`Scanned ${jsFiles.length} JS module(s). Found ${suspicious.length} suspicious hardcoded line(s):`);
for (const item of suspicious) {
  console.log(`  src/${item.file}:${item.lineNum} [${item.type}]: ${item.text}`);
}

console.log(`\nDynamic-string result: ${suspicious.length === 0 ? "PASS" : "FAIL"}`);
if (suspicious.length > 0) {
  process.exitCode = 1;
}
