import { readFileSync } from "node:fs";

console.log("=== CHECKING IPC INVOCATION SIGNATURES ===");

const read = (p) => readFileSync(p, "utf8");

const libRs = read("src-tauri/src/lib.rs");

// Parse all #[tauri::command] functions in lib.rs and any referenced modules
// Match: #[tauri::command]\s*(?:async\s*)?fn\s+([a-zA-Z0-9_]+)\s*\(([\s\S]*?)\)
const commandRegex = /#\[tauri::command\][\s\S]*?(?:pub\s+)?(?:async\s+)?fn\s+([a-zA-Z0-9_]+)\s*(?:<[^>]+>)?\s*\(([\s\S]*?)\)/g;
const rustCommands = new Map();

let m;
while ((m = commandRegex.exec(libRs)) !== null) {
  const fnName = m[1];
  const paramsRaw = m[2];
  // Parse params, ignoring State, AppHandle, Window, etc.
  const params = paramsRaw
    .split(",")
    .map(p => p.trim())
    .filter(Boolean)
    .filter(p => !p.includes("State<") && !p.includes("AppHandle") && !p.includes("WebviewWindow") && !p.includes("&Window") && !p.includes("Window<") && !p.includes("Channel<"))
    .map(p => {
      const parts = p.split(":");
      return parts[0].trim().replace(/^mut\s+/, "");
    });
  rustCommands.set(fnName, params);
}

console.log(`Parsed ${rustCommands.size} #[tauri::command] definitions from lib.rs:`);
for (const [name, params] of rustCommands.entries()) {
  console.log(`  - ${name}(${params.join(", ")})`);
}

// Now parse invoke calls in JS
function toCamelCase(str) {
  return str.replace(/_([a-z])/g, (_, letter) => letter.toUpperCase());
}
function toSnakeCase(str) {
  return str.replace(/[A-Z]/g, letter => `_${letter.toLowerCase()}`);
}

const jsFiles = ["src/main.js", "src/overlay.js", "src/tray-menu.js"];

for (const file of jsFiles) {
  const content = read(file);
  // Match invoke calls: invoke("cmd_name", { ... }) or invoke('cmd_name')
  // Use regex to find invoke calls
  const invokeRegex = /invoke\(\s*["']([^"']+)["']\s*(?:,\s*(\{[\s\S]*?\}))?\s*\)/g;
  let inv;
  while ((inv = invokeRegex.exec(content)) !== null) {
    const cmd = inv[1];
    const rawArgs = inv[2];
    const rustParams = rustCommands.get(cmd);
    if (!rustParams) {
      console.warn(`[WARN] ${file}: Command '${cmd}' not found in lib.rs #[tauri::command] map.`);
      continue;
    }

    if (rustParams.length === 0) {
      if (rawArgs && rawArgs.trim() !== "{}") {
        console.warn(`[MISMATCH] ${file}: '${cmd}' expects no params, but passed: ${rawArgs}`);
      }
    } else {
      if (!rawArgs || rawArgs.trim() === "{}") {
        console.error(`[MISMATCH] ${file}: '${cmd}' expects params [${rustParams.join(", ")}], but no args passed!`);
      } else {
        // Parse keys from rawArgs
        // e.g. { provider, acceptedNvidiaTerms } or { key: val, ... }
        // Simple key extractor
        const keys = [];
        const cleaned = rawArgs.replace(/\{([\s\S]*)\}/, "$1");
        // Split by top-level commas
        let depth = 0;
        let current = "";
        for (const char of cleaned) {
          if (char === "{" || char === "[" || char === "(") depth++;
          else if (char === "}" || char === "]" || char === ")") depth--;
          else if (char === "," && depth === 0) {
            keys.push(current.trim());
            current = "";
            continue;
          }
          current += char;
        }
        if (current.trim()) keys.push(current.trim());

        const passedKeyNames = keys.map(k => {
          const colonIdx = k.indexOf(":");
          if (colonIdx !== -1) {
            return k.slice(0, colonIdx).trim().replace(/^["']|["']$/g, "");
          }
          return k.trim().replace(/^["']|["']$/g, "");
        }).filter(Boolean);

        // Check if each rust param has a corresponding passed key (in camelCase or snake_case)
        for (const rp of rustParams) {
          const camel = toCamelCase(rp);
          const found = passedKeyNames.some(k => k === rp || k === camel);
          if (!found) {
            console.error(`[PARAM ERROR] ${file}: invoke('${cmd}') missing expected param '${rp}' (or '${camel}'). Passed keys: [${passedKeyNames.join(", ")}]`);
          }
        }
      }
    }
  }
}
console.log("=== IPC SIGNATURE CHECK COMPLETE ===");
