import { readFileSync } from "node:fs";
import vm from "node:vm";

const read = (p) => readFileSync(p, "utf8");
const overlayJs = read("src/overlay.js").replaceAll("\r\n", "\n");

const expectedLocales = ["ru", "en", "de", "es", "fr", "it", "zh", "pt", "tr"];

console.log("=== CHECKING OVERLAY.JS TRANSLATIONS ===");

function extractObject(varName) {
  const marker = `const ${varName} =`;
  const start = overlayJs.indexOf(marker);
  if (start === -1) {
    console.error(`Marker ${marker} not found!`);
    return null;
  }
  const end = overlayJs.indexOf("};", start) + 1;
  return vm.runInNewContext("(" + overlayJs.slice(start + marker.length, end) + ")");
}

const contextObj = extractObject("contextIndicatorTranslations");
console.log("contextIndicatorTranslations locales:", Object.keys(contextObj));
for (const loc of expectedLocales) {
  if (!contextObj[loc]) {
    console.warn(`[WARN] contextIndicatorTranslations missing locale '${loc}'`);
  }
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

console.log("=== CHECKING LIB.RS TRAY TRANSLATIONS ===");
const libRs = read("src-tauri/src/lib.rs");
const trayTransMatch = libRs.match(/fn tray_translations\(language: &str\)[^{]*\{([\s\S]*?)\n\}/);
if (trayTransMatch) {
  const branches = [...trayTransMatch[1].matchAll(/"([a-z]{2})"\s*=>/g)].map(m => m[1]);
  console.log("lib.rs tray_translations language branches:", branches);
  for (const loc of expectedLocales) {
    if (!branches.includes(loc)) {
      console.warn(`[WARN] lib.rs tray_translations missing branch for '${loc}'`);
    }
  }
}
