import assert from "node:assert/strict";
import { readFileSync, readdirSync } from "node:fs";
import { join } from "node:path";
import vm from "node:vm";

const read = (p) => readFileSync(p, "utf8");

console.log("=================================================");
console.log("     AURA DEEP INTEGRITY AUDIT SUITE             ");
console.log("=================================================");

const html = read("src/index.html");
const mainJs = read("src/main.js");
const overlayHtml = read("src/overlay.html");
const overlayJs = read("src/overlay.js");
const trayHtml = read("src/tray-menu.html");
const trayJs = read("src/tray-menu.js");
const libRs = read("src-tauri/src/lib.rs");
const settingsRs = read("src-tauri/src/settings_secure.rs");
const normalizerRs = read("src-tauri/src/text_normalizer.rs");

let totalIssues = 0;
function reportIssue(severity, category, message) {
  totalIssues++;
  console.log(`[${severity}] [${category}] ${message}`);
}

// -------------------------------------------------------------
// CHECK 1: Sound themes consistency across all layers
// -------------------------------------------------------------
console.log("\n--- Checking Sound Themes Consistency ---");
const soundThemeHtmlOptions = [...html.match(/<select[^>]*id="select-sound-theme"[^>]*>([\s\S]*?)<\/select>/)?.[1].matchAll(/value="([^"]+)"/g)].map(m => m[1]);
console.log("HTML Sound themes:", soundThemeHtmlOptions);

const allowedThemesMatch = settingsRs.match(/ALLOWED_SOUND_THEMES:[^=]*=\s*&\[([\s\S]*?)\];/);
const rustAllowedThemes = allowedThemesMatch
  ? [...allowedThemesMatch[1].matchAll(/"([^"]+)"/g)].map(m => m[1])
  : [];
console.log("Rust ALLOWED_SOUND_THEMES:", rustAllowedThemes);

for (const theme of soundThemeHtmlOptions) {
  if (!rustAllowedThemes.includes(theme)) {
    reportIssue("ERROR", "SOUND_THEME", `Theme '${theme}' in index.html is NOT in Rust ALLOWED_SOUND_THEMES!`);
  }
  if (!overlayJs.includes(`"${theme}"`) && !overlayJs.includes(`'${theme}'`)) {
    reportIssue("ERROR", "SOUND_THEME", `Theme '${theme}' not referenced in overlay.js!`);
  }
}
for (const theme of rustAllowedThemes) {
  if (!soundThemeHtmlOptions.includes(theme)) {
    reportIssue("WARN", "SOUND_THEME", `Theme '${theme}' in Rust ALLOWED_SOUND_THEMES but missing from index.html!`);
  }
}

// -------------------------------------------------------------
// CHECK 2: Recognition languages across all layers
// -------------------------------------------------------------
console.log("\n--- Checking Recognition Languages Consistency ---");
// In index.html: <select id="select-language"> is the recognition language selector!
const recogLangsHtml = [...html.match(/<select[^>]*id="select-language"[^>]*>([\s\S]*?)<\/select>/)?.[1].matchAll(/value="([^"]+)"/g)].map(m => m[1]);
console.log("HTML Recognition languages:", recogLangsHtml);

// In tray-menu.js: SUPPORTED_LANGS
const trayLangs = [...trayJs.matchAll(/code:\s*["']([^"']+)["']/g)].map(m => m[1]);
console.log("Tray menu Recognition languages:", trayLangs);

// In settings_secure.rs: ALLOWED_LANGUAGES
const allowedLangsMatch = settingsRs.match(/ALLOWED_LANGUAGES:[^=]*=\s*&\[([\s\S]*?)\];/);
const rustAllowedLangs = allowedLangsMatch
  ? [...allowedLangsMatch[1].matchAll(/"([^"]+)"/g)].map(m => m[1])
  : [];
console.log("Rust ALLOWED_LANGUAGES:", rustAllowedLangs);

for (const lang of recogLangsHtml) {
  if (!rustAllowedLangs.includes(lang)) {
    reportIssue("ERROR", "RECOG_LANG", `Language '${lang}' in index.html is NOT in Rust ALLOWED_LANGUAGES!`);
  }
  if (!trayLangs.includes(lang)) {
    reportIssue("ERROR", "RECOG_LANG", `Language '${lang}' in index.html is NOT in tray-menu SUPPORTED_LANGS!`);
  }
}
for (const lang of trayLangs) {
  if (!recogLangsHtml.includes(lang)) {
    reportIssue("ERROR", "RECOG_LANG", `Language '${lang}' in tray-menu is NOT in index.html!`);
  }
}

// -------------------------------------------------------------
// CHECK 3: UI Languages across all layers
// -------------------------------------------------------------
console.log("\n--- Checking UI Interface Languages Consistency ---");
// In index.html: <select id="select-ui-lang">
const uiLangsHtml = [...html.match(/<select[^>]*id="select-ui-lang"[^>]*>([\s\S]*?)<\/select>/)?.[1].matchAll(/value="([^"]+)"/g)].map(m => m[1]);
console.log("HTML UI Languages:", uiLangsHtml);

// In main.js dictionaries
const marker = "const i18nDict =";
const normMain = mainJs.replaceAll("\r\n", "\n");
const start = normMain.indexOf(marker) + marker.length;
const end = normMain.indexOf("\n};\n\nlet currentLanguage", start) + 2;
const mainDicts = vm.runInNewContext("(" + normMain.slice(start, end) + ")");
const mainUiLangs = Object.keys(mainDicts);
console.log("main.js UI Languages:", mainUiLangs);

// In tray-menu.js I18N
const trayI18nMarker = "const I18N =";
const normTray = trayJs.replaceAll("\r\n", "\n");
const trayStart = normTray.indexOf(trayI18nMarker) + trayI18nMarker.length;
const trayEnd = normTray.indexOf("\n};\n\nconst SUPPORTED_LANGS", trayStart) + 2;
const trayDicts = vm.runInNewContext("(" + normTray.slice(trayStart, trayEnd) + ")");
const trayUiLangs = Object.keys(trayDicts);
console.log("tray-menu.js UI Languages:", trayUiLangs);

// In settings_secure.rs: ALLOWED_UI_LANGUAGES
const allowedUiLangsMatch = settingsRs.match(/ALLOWED_UI_LANGUAGES:[^=]*=\s*&\[([\s\S]*?)\];/);
const rustAllowedUiLangs = allowedUiLangsMatch
  ? [...allowedUiLangsMatch[1].matchAll(/"([^"]+)"/g)].map(m => m[1])
  : [];
console.log("Rust ALLOWED_UI_LANGUAGES:", rustAllowedUiLangs);

for (const lang of uiLangsHtml) {
  if (!mainUiLangs.includes(lang)) reportIssue("ERROR", "UI_LANG", `UI lang '${lang}' missing in main.js dicts`);
  if (!trayUiLangs.includes(lang)) reportIssue("ERROR", "UI_LANG", `UI lang '${lang}' missing in tray-menu.js dicts`);
  if (!rustAllowedUiLangs.includes(lang)) reportIssue("ERROR", "UI_LANG", `UI lang '${lang}' missing in Rust ALLOWED_UI_LANGUAGES`);
}

// -------------------------------------------------------------
// CHECK 4: Event Emitters & Listeners across all files
// -------------------------------------------------------------
console.log("\n--- Checking Events and Listeners across all files ---");
const rustSrcDir = "src-tauri/src";
const rsFiles = readdirSync(rustSrcDir).filter(f => f.endsWith(".rs"));
const rustEmittedEvents = new Set();
for (const rf of rsFiles) {
  const content = read(join(rustSrcDir, rf));
  for (const m of content.matchAll(/\.emit(?:_to)?\(\s*["']([^"']+)["']/g)) {
    rustEmittedEvents.add(m[1]);
  }
}
console.log("Rust emitted events across all .rs files:", [...rustEmittedEvents]);

// Find all listen( in main.js, overlay.js, tray-menu.js
const mainListeners = new Set([...normMain.matchAll(/listen\(\s*["']([^"']+)["']/g)].map(m => m[1]));
const overlayListeners = new Set([...overlayJs.replaceAll("\r\n", "\n").matchAll(/listen\(\s*["']([^"']+)["']/g)].map(m => m[1]));
const trayListeners = new Set([...normTray.matchAll(/listen\(\s*["']([^"']+)["']/g)].map(m => m[1]));

console.log("main.js listens to:", [...mainListeners]);
console.log("overlay.js listens to:", [...overlayListeners]);
console.log("tray-menu.js listens to:", [...trayListeners]);

const allJsListeners = new Set([...mainListeners, ...overlayListeners, ...trayListeners]);
for (const re of rustEmittedEvents) {
  if (!allJsListeners.has(re)) {
    reportIssue("INFO", "EVENT", `Rust emits event '${re}' which has no JS listener.`);
  }
}
for (const jl of allJsListeners) {
  if (!rustEmittedEvents.has(jl)) {
    reportIssue("WARN", "EVENT", `JS listens to event '${jl}' which is not emitted in Rust .rs files!`);
  }
}

// -------------------------------------------------------------
// CHECK 5: Untranslated or Suspicious Content in Dictionaries
// -------------------------------------------------------------
console.log("\n--- Checking Untranslated / Suspicious Strings in Dictionaries ---");
const cyrillicRegex = /[\u0400-\u04FF]/;
for (const [locale, dict] of Object.entries(mainDicts)) {
  if (locale === "ru") continue;
  for (const [key, val] of Object.entries(dict)) {
    if (typeof val === "string" && cyrillicRegex.test(val)) {
      if (val.includes("Русский") || key.includes("lang_ru") || key.includes("recog_lang_ru")) {
        continue;
      }
      reportIssue("WARN", "I18N_LEAK", `Possible Russian leak in locale '${locale}', key '${key}': "${val}"`);
    }
  }
}

// Check tray-menu.js keys
const trayKeys = Object.keys(trayDicts.ru);
for (const [loc, dict] of Object.entries(trayDicts)) {
  for (const k of trayKeys) {
    if (!dict[k]) {
      reportIssue("ERROR", "TRAY_I18N", `tray-menu.js locale '${loc}' missing key '${k}'`);
    }
  }
}

// -------------------------------------------------------------
// CHECK 6: Cloud Providers & Models
// -------------------------------------------------------------
console.log("\n--- Checking Cloud Providers & Models ---");
const cloudProvidersHtml = [...html.match(/<select[^>]*id="select-provider"[^>]*>([\s\S]*?)<\/select>/)?.[1].matchAll(/value="([^"]+)"/g)].map(m => m[1]);
console.log("HTML Cloud providers:", cloudProvidersHtml);

const allowedProvidersMatch = settingsRs.match(/ALLOWED_PROVIDERS:[^=]*=\s*&\[([\s\S]*?)\];/);
const rustAllowedProviders = allowedProvidersMatch
  ? [...allowedProvidersMatch[1].matchAll(/"([^"]+)"/g)].map(m => m[1])
  : [];
console.log("Rust ALLOWED_PROVIDERS:", rustAllowedProviders);

for (const prov of cloudProvidersHtml) {
  if (!rustAllowedProviders.includes(prov)) {
    reportIssue("ERROR", "CLOUD_PROVIDER", `Provider '${prov}' in index.html is NOT in Rust ALLOWED_PROVIDERS!`);
  }
}

// -------------------------------------------------------------
// CHECK 7: VRAM Standby Timeout Values
// -------------------------------------------------------------
console.log("\n--- Checking VRAM Standby Timeout Options ---");
const standbyOptions = [...html.match(/<select[^>]*id="select-vram-standby"[^>]*>([\s\S]*?)<\/select>/)?.[1].matchAll(/value="([^"]+)"/g)].map(m => parseInt(m[1], 10));
console.log("HTML Standby timeout options (minutes):", standbyOptions);
for (const opt of standbyOptions) {
  if (opt > 1440) {
    reportIssue("ERROR", "STANDBY_TIMEOUT", `Timeout '${opt}' in index.html exceeds 1440 max!`);
  }
}

// -------------------------------------------------------------
// CHECK 8: Local Whisper Models
// -------------------------------------------------------------
console.log("\n--- Checking Local Whisper Models ---");
const whisperModelsMatch = settingsRs.match(/ALLOWED_MODELS:[^=]*=\s*&\[([\s\S]*?)\];/);
const rustWhisperModels = whisperModelsMatch
  ? [...whisperModelsMatch[1].matchAll(/"([^"]+)"/g)].map(m => m[1])
  : [];
console.log("Rust ALLOWED_MODELS:", rustWhisperModels);

for (const model of rustWhisperModels) {
  if (model === "parakeet-v3") continue; // Parakeet has its own section
  const modelCard = html.includes(`data-model="${model}"`) || html.includes(`id="card-model-${model}"`);
  if (!modelCard) {
    reportIssue("WARN", "WHISPER_MODEL", `Model '${model}' defined in Rust but no data-model/card found in index.html!`);
  }
}

console.log("\n=================================================");
console.log(` AUDIT FINISHED: Total issues found = ${totalIssues}`);
console.log("=================================================");
