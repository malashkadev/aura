import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import vm from "node:vm";

console.log("=== RUNNING FULL PROGRAM INTEGRITY AUDIT ===");

const read = (p) => readFileSync(p, "utf8");

// 1. Check DOM IDs in index.html vs main.js
const html = read("src/index.html");
const mainJs = read("src/main.js");
const htmlIds = new Set([...html.matchAll(/id="([^"]+)"/g)].map(m => m[1]));

// Find getElementById in main.js
const getElementByIdMatches = [...mainJs.matchAll(/getElementById\(["']([^"']+)["']\)/g)].map(m => m[1]);
const missingIds = [];
for (const id of getElementByIdMatches) {
  if (!htmlIds.has(id)) {
    missingIds.push(id);
  }
}
console.log(`Checked ${getElementByIdMatches.length} getElementById calls. Missing IDs:`, missingIds);

// 2. Parse i18n dictionaries in main.js
const normalized = mainJs.replaceAll("\r\n", "\n");
const marker = "const i18nDict =";
const start = normalized.indexOf(marker) + marker.length;
const end = normalized.indexOf("\n};\n\nlet currentLanguage", start) + 2;
const dicts = vm.runInNewContext("(" + normalized.slice(start, end) + ")");

const locales = Object.keys(dicts);
console.log("Found locales in main.js:", locales);

// Check reference keys
const allKeys = new Set();
for (const loc of locales) {
  for (const k of Object.keys(dicts[loc])) {
    allKeys.add(k);
  }
}
console.log(`Total unique keys across all locales: ${allKeys.size}`);

const missingKeysPerLocale = {};
for (const loc of locales) {
  missingKeysPerLocale[loc] = [];
  for (const k of allKeys) {
    if (dicts[loc][k] === undefined) {
      missingKeysPerLocale[loc].push(k);
    }
  }
}
for (const loc of locales) {
  if (missingKeysPerLocale[loc].length > 0) {
    console.error(`Locale ${loc} is missing ${missingKeysPerLocale[loc].length} keys:`, missingKeysPerLocale[loc]);
  } else {
    console.log(`Locale ${loc} has all ${allKeys.size} keys!`);
  }
}

// Check data-i18n in index.html
const dataI18nKeys = [...new Set([...html.matchAll(/data-i18n="([^"]+)"/g)].map(m => m[1]))];
console.log(`Found ${dataI18nKeys.length} data-i18n keys in index.html`);
const missingDataI18n = {};
for (const loc of locales) {
  missingDataI18n[loc] = [];
  for (const k of dataI18nKeys) {
    if (!dicts[loc][k]) {
      missingDataI18n[loc].push(k);
    }
  }
  if (missingDataI18n[loc].length > 0) {
    console.error(`Locale ${loc} missing data-i18n keys:`, missingDataI18n[loc]);
  }
}

// Check getTranslation calls in main.js
const getTranslationCalls = [...new Set([...mainJs.matchAll(/getTranslation\(["']([^"']+)["']/g)].map(m => m[1]))];
console.log(`Found ${getTranslationCalls.length} getTranslation calls in main.js`);
const missingGetTrans = {};
for (const loc of locales) {
  missingGetTrans[loc] = [];
  for (const k of getTranslationCalls) {
    if (!dicts[loc][k]) {
      missingGetTrans[loc].push(k);
    }
  }
  if (missingGetTrans[loc].length > 0) {
    console.error(`Locale ${loc} missing getTranslation keys:`, missingGetTrans[loc]);
  }
}

// Check parameter matching in translations (e.g. {count}, {0}, etc.)
for (const k of allKeys) {
  const ruVal = dicts.ru?.[k];
  if (typeof ruVal === "string") {
    const placeholders = [...ruVal.matchAll(/\{([a-zA-Z0-9_]+)\}/g)].map(m => m[1]);
    if (placeholders.length > 0) {
      for (const loc of locales) {
        const val = dicts[loc]?.[k];
        if (typeof val === "string") {
          for (const p of placeholders) {
            if (!val.includes(`{${p}}`)) {
              console.warn(`[Placeholder Mismatch] key '${k}' in locale '${loc}' missing {${p}}. RU: "${ruVal}" vs ${loc}: "${val}"`);
            }
          }
        }
      }
    }
  }
}

// 3. Overlay inspection
const overlayHtml = read("src/overlay.html");
const overlayJs = read("src/overlay.js");
const overlayHtmlIds = new Set([...overlayHtml.matchAll(/id="([^"]+)"/g)].map(m => m[1]));
const overlayGetElementById = [...overlayJs.matchAll(/getElementById\(["']([^"']+)["']\)/g)].map(m => m[1]);
const overlayMissingIds = overlayGetElementById.filter(id => !overlayHtmlIds.has(id));
console.log("Overlay missing IDs:", overlayMissingIds);

// 4. Tray menu inspection
const trayHtml = read("src/tray-menu.html");
const trayJs = read("src/tray-menu.js");
const trayHtmlIds = new Set([...trayHtml.matchAll(/id="([^"]+)"/g)].map(m => m[1]));
const trayGetElementById = [...trayJs.matchAll(/getElementById\(["']([^"']+)["']\)/g)].map(m => m[1]);
const trayMissingIds = trayGetElementById.filter(id => !trayHtmlIds.has(id));
console.log("Tray menu missing IDs:", trayMissingIds);

// 5. Tauri command invocation audit
const allJsFiles = [
  { name: "main.js", content: mainJs },
  { name: "overlay.js", content: overlayJs },
  { name: "tray-menu.js", content: trayJs }
];

const invokedCommands = [];
for (const f of allJsFiles) {
  const regex = /invoke\(["']([^"']+)["'](?:\s*,\s*(\{[\s\S]*?\}))?\)/g;
  let m;
  while ((m = regex.exec(f.content)) !== null) {
    const cmd = m[1];
    const rawArgs = m[2];
    invokedCommands.push({ file: f.name, cmd, rawArgs });
  }
}
console.log(`Found ${invokedCommands.length} invoke calls across JS files.`);

// Read Rust commands from lib.rs
const libRs = read("src-tauri/src/lib.rs");
const generateHandlerMatch = libRs.match(/generate_handler!\[([\s\S]*?)\]/);
const registeredCommands = new Set(
  generateHandlerMatch ? generateHandlerMatch[1].split(",").map(s => s.trim()).filter(Boolean) : []
);
console.log(`Registered ${registeredCommands.size} commands in lib.rs generate_handler`);

// Check capability files
const mainCap = JSON.parse(read("src-tauri/capabilities/main.json"));
const overlayCap = JSON.parse(read("src-tauri/capabilities/overlay.json"));
const trayCap = JSON.parse(read("src-tauri/capabilities/tray-menu.json"));

const capPermissions = {
  "main.js": new Set(mainCap.permissions),
  "overlay.js": new Set(overlayCap.permissions),
  "tray-menu.js": new Set(trayCap.permissions),
};

for (const inv of invokedCommands) {
  if (!registeredCommands.has(inv.cmd)) {
    console.error(`[ERROR] JS file ${inv.file} invokes unregistered command: '${inv.cmd}'`);
  }
  // Check permission in capability
  const expectedPerm = `allow-${inv.cmd.replaceAll("_", "-")}`;
  const perms = capPermissions[inv.file];
  if (perms && !perms.has(expectedPerm)) {
    console.error(`[ERROR] Capability for ${inv.file} is missing permission: '${expectedPerm}' for command '${inv.cmd}'`);
  }
}

console.log("=== AUDIT SCRIPT FINISHED ===");
