import { readFileSync } from "node:fs";
import vm from "node:vm";

const read = (p) => readFileSync(p, "utf8");
const overlayJs = read("src/overlay.js").replaceAll("\r\n", "\n");

const expectedLocales = ["ru", "en", "de", "es", "fr", "it", "zh", "pt", "tr"];

console.log("=== CHECKING OVERLAY.JS TRANSLATIONS ===");

function extractObject(varName, silent = false) {
  const marker = `const ${varName} =`;
  const start = overlayJs.indexOf(marker);
  if (start === -1) {
    if (!silent) console.error(`Marker ${marker} not found!`);
    return null;
  }
  const end = overlayJs.indexOf("};", start) + 1;
  return vm.runInNewContext("(" + overlayJs.slice(start + marker.length, end) + ")");
}

const contextObj = extractObject("contextIndicatorTranslations", true);
if (contextObj) {
  console.log("contextIndicatorTranslations locales:", Object.keys(contextObj));
  for (const loc of expectedLocales) {
    if (!contextObj[loc]) {
      console.warn(`[WARN] contextIndicatorTranslations missing locale '${loc}'`);
    }
  }
} else {
  console.log("contextIndicatorTranslations not used (edit indicator is handled via amber contour glow).");
}

// Find processingTranslations
const procStart = overlayJs.indexOf("const processingTranslations =");
if (procStart !== -1) {
  const procEnd = overlayJs.indexOf("};", procStart) + 1;
  const procObj = vm.runInNewContext("(" + overlayJs.slice(procStart + "const processingTranslations =".length, procEnd) + ")");
  console.log("processingTranslations locales:", Object.keys(procObj));
  for (const loc of expectedLocales) {
    if (!procObj[loc]) {
      console.warn(`[WARN] processingTranslations missing locale '${loc}'`);
    }
  }
} else {
  console.log("processingTranslations not found or named differently.");
}

// Find noticeTranslations
const noticeStart = overlayJs.indexOf("const noticeTranslations =");
if (noticeStart !== -1) {
  const noticeEnd = overlayJs.indexOf("};\n\nconst errorTranslations", noticeStart) + 1;
  const noticeObj = vm.runInNewContext("(" + overlayJs.slice(noticeStart + "const noticeTranslations =".length, noticeEnd) + ")");
  console.log("noticeTranslations notices:", Object.keys(noticeObj));
  for (const [notice, dict] of Object.entries(noticeObj)) {
    console.log(`Notice '${notice}' locales:`, Object.keys(dict));
    for (const loc of expectedLocales) {
      if (!dict[loc]) {
        console.warn(`[WARN] noticeTranslations['${notice}'] missing locale '${loc}'`);
      }
    }
  }
}

// Find errorTranslations
const errStart = overlayJs.indexOf("const errorTranslations =");
if (errStart !== -1) {
  const errEnd = overlayJs.indexOf("};\n\nfunction translateError", errStart) + 1;
  const errObj = vm.runInNewContext("(" + overlayJs.slice(errStart + "const errorTranslations =".length, errEnd) + ")");
  console.log("errorTranslations count:", Object.keys(errObj).length);
  for (const [err, dict] of Object.entries(errObj)) {
    for (const loc of expectedLocales) {
      if (!dict[loc]) {
        console.warn(`[WARN] errorTranslations['${err}'] missing locale '${loc}'`);
      }
    }
  }
}

console.log("=== CHECKING TRAY MENU TRANSLATIONS ===");
const trayMenuJs = read("src/tray-menu.js");
const trayDictMatch = trayMenuJs.match(/const I18N = \{([\s\S]*?)\n\};/);
if (!trayDictMatch) {
  console.warn("[WARN] src/tray-menu.js: `const I18N = { ... }` dictionary not found");
} else {
  const branches = [...trayDictMatch[1].matchAll(/^ {2}([a-z]{2}):\s*\{/gm)].map((m) => m[1]);
  console.log("tray-menu.js UI languages:", branches);
  for (const loc of expectedLocales) {
    if (!branches.includes(loc)) {
      console.warn(`[WARN] tray-menu.js I18N missing locale '${loc}'`);
    }
  }
}
