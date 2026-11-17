import fs from "node:fs";
import { syncBuiltinESMExports } from "node:module";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import vm from "node:vm";

const startedAt = performance.now();
const repoRoot = fileURLToPath(new URL("..", import.meta.url));
process.chdir(repoRoot);

const rawReadFileSync = fs.readFileSync.bind(fs);
const rawReaddirSync = fs.readdirSync.bind(fs);
const rawRunInNewContext = vm.runInNewContext.bind(vm);

const utf8FileCache = new Map();
const dirCache = new Map();
const vmObjectCache = new Map();

fs.readFileSync = function cachedReadFileSync(filePath, options) {
  if (typeof filePath === "string" && (options === "utf8" || options === "utf-8")) {
    const key = resolve(filePath);
    if (utf8FileCache.has(key)) {
      return utf8FileCache.get(key);
    }
    const content = rawReadFileSync(filePath, options);
    utf8FileCache.set(key, content);
    return content;
  }
  return rawReadFileSync(filePath, options);
};

fs.readdirSync = function cachedReaddirSync(dirPath, options) {
  if (typeof dirPath === "string" && options === undefined) {
    const key = resolve(dirPath);
    if (dirCache.has(key)) {
      return [...dirCache.get(key)];
    }
    const entries = rawReaddirSync(dirPath);
    dirCache.set(key, entries);
    return [...entries];
  }
  return rawReaddirSync(dirPath, options);
};

vm.runInNewContext = function cachedRunInNewContext(code, contextObject, options) {
  if (
    typeof code === "string" &&
    contextObject === undefined &&
    options === undefined &&
    code.length > 1024
  ) {
    if (vmObjectCache.has(code)) {
      return vmObjectCache.get(code);
    }
    const evaluated = rawRunInNewContext(code);
    vmObjectCache.set(code, evaluated);
    return evaluated;
  }
  return rawRunInNewContext(code, contextObject, options);
};

syncBuiltinESMExports();

const AUDIT_SCRIPTS = [
  "audit-all-elements.mjs",
  "check-aria.mjs",
  "check-css-invariants.mjs",
  "check-dynamic-strings.mjs",
  "check-hardcoded-text.mjs",
  "check-ipc.mjs",
  "check-overlay-i18n.mjs",
  "check-titles.mjs",
  "comprehensive-audit.mjs",
  "deep-audit.mjs",
];

const failedScripts = [];

for (const scriptName of AUDIT_SCRIPTS) {
  process.exitCode = 0;
  try {
    await import(new URL(`./${scriptName}`, import.meta.url).href);
  } catch (error) {
    console.error(`[AUDIT CRASH] ${scriptName}:`, error);
    process.exitCode = 1;
  }
  if (process.exitCode && process.exitCode !== 0) {
    failedScripts.push(scriptName);
  }
}

const elapsedMs = (performance.now() - startedAt).toFixed(1);
if (failedScripts.length > 0) {
  console.error(
    `\n[AUDIT SUITE FAILED] ${failedScripts.length}/${AUDIT_SCRIPTS.length} gate(s) failed in ${elapsedMs} ms: ${failedScripts.join(", ")}`,
  );
  process.exitCode = 1;
} else {
  process.exitCode = 0;
  console.log(
    `\n[AUDIT SUITE PASSED] ${AUDIT_SCRIPTS.length}/${AUDIT_SCRIPTS.length} gates passed in ${elapsedMs} ms`,
  );
}
