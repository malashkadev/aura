import { readFileSync } from "node:fs";
import vm from "node:vm";

const read = (p) => readFileSync(p, "utf8");
const overlayJs = read("src/overlay.js").replaceAll("\r\n", "\n");

const expectedLocales = ["ru", "en", "de", "es", "fr", "it", "zh", "pt", "tr"];

let warningCount = 0;
function warn(message) {
  warningCount += 1;
  console.warn(message);
}

console.log("=== CHECKING OVERLAY.JS TRANSLATIONS ===");

function extractObjectLiteral(source, varName) {
  const regex = new RegExp(`\\bconst\\s+${varName}\\s*=\\s*\\{`);
  const match = regex.exec(source);
  if (!match) {
    return null;
  }
  const braceStart = source.indexOf("{", match.index);
  let depth = 0;
  let inString = null;
  let escaped = false;

  for (let i = braceStart; i < source.length; i++) {
    const ch = source[i];
    if (inString) {
      if (escaped) {
        escaped = false;
      } else if (ch === "\\") {
        escaped = true;
      } else if (ch === inString) {
        inString = null;
      }
      continue;
    }
    if (ch === '"' || ch === "'" || ch === "`") {
      inString = ch;
      continue;
    }
    if (ch === "{") {
      depth += 1;
    } else if (ch === "}") {
      depth -= 1;
      if (depth === 0) {
        return vm.runInNewContext("(" + source.slice(braceStart, i + 1) + ")");
      }
    }
  }
  return null;
}

function checkNestedDictionary(dictName) {
  const obj = extractObjectLiteral(overlayJs, dictName);
  if (!obj) {
    warn(`[ERROR] src/overlay.js: required dictionary 'const ${dictName}' not found!`);
    return;
  }
  const entries = Object.entries(obj);
  console.log(`${dictName} entries: ${entries.length}`);
  for (const [entryKey, localeMap] of entries) {
    for (const loc of expectedLocales) {
      if (typeof localeMap?.[loc] !== "string" || localeMap[loc].trim() === "") {
        warn(`[WARN] ${dictName}['${entryKey}'] missing or empty locale '${loc}'`);
      }
    }
  }
}

function checkFlatLocaleMap(dictName) {
  const obj = extractObjectLiteral(overlayJs, dictName);
  if (!obj) {
    warn(`[ERROR] src/overlay.js: required locale map 'const ${dictName}' not found!`);
    return;
  }
  console.log(`${dictName} locales:`, Object.keys(obj));
  for (const loc of expectedLocales) {
    if (typeof obj[loc] !== "string" || obj[loc].trim() === "") {
      warn(`[WARN] ${dictName} missing or empty locale '${loc}'`);
    }
  }
}

checkNestedDictionary("noticeTranslations");
checkNestedDictionary("errorTranslations");
checkFlatLocaleMap("netTranslations");
checkFlatLocaleMap("insecureTranslations");

console.log("=== CHECKING TRAY MENU TRANSLATIONS ===");
const trayMenuJs = read("src/tray-menu.js").replaceAll("\r\n", "\n");
const trayI18n = extractObjectLiteral(trayMenuJs, "I18N");
if (!trayI18n) {
  warn("[ERROR] src/tray-menu.js: `const I18N = { ... }` dictionary not found");
} else {
  const branches = Object.keys(trayI18n);
  console.log("tray-menu.js UI languages:", branches);
  const refKeys = Object.keys(trayI18n.ru || {});
  for (const loc of expectedLocales) {
    if (!trayI18n[loc]) {
      warn(`[WARN] tray-menu.js I18N missing locale '${loc}'`);
      continue;
    }
    for (const key of refKeys) {
      if (typeof trayI18n[loc][key] !== "string" || trayI18n[loc][key].trim() === "") {
        warn(`[WARN] tray-menu.js I18N['${loc}']['${key}'] missing or empty`);
      }
    }
  }
}

console.log(`\nOverlay & Tray i18n result: ${warningCount === 0 ? "PASS" : "FAIL"} (${warningCount} issue(s))`);
if (warningCount > 0) {
  process.exitCode = 1;
}
