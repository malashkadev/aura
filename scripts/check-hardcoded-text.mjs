import { readdirSync, readFileSync } from "node:fs";

const read = (p) => readFileSync(p, "utf8");

console.log("=== CHECKING HARDCODED TEXT IN INTERACTIVE ELEMENTS AND HEADINGS ===");

// Options and bare spans are intentionally excluded: language names, provider
// names and model names are proper nouns, while spec values are populated by
// main.js at runtime. The gate focuses on controls and headings a user reads
// as interface copy across all three desktop windows.
const auditedTags = ["button", "a", "label", "h1", "h2", "h3", "summary"];
const allowedExact = new Set([
  "AURA",
  "Aura",
  "GitHub",
  "Windows",
  "Whisper",
  "Parakeet",
  "CUDA",
  "NVIDIA",
  "Onest",
  "Gemini",
  "OpenAI",
  "Groq",
  "Hugging Face",
  "Qwen3",
  "Qwen3 0.6B",
  "Qwen3 1.7B",
  "Qwen3 4B",
  "Tiny",
  "Base",
  "Small",
  "Medium",
  "Large v3 Turbo",
  "Large v3 Turbo (Q5)",
  "Parakeet TDT 0.6b v3 (int8)",
]);

const allSrcJs = readdirSync("src")
  .filter((f) => f.endsWith(".js"))
  .sort();
const mainBundleJs = allSrcJs
  .filter((f) => f !== "overlay.js" && f !== "tray-menu.js")
  .map((f) => read(`src/${f}`))
  .join("\n");

const surfaces = [
  { file: "src/index.html", js: mainBundleJs },
  { file: "src/overlay.html", js: read("src/overlay.js") },
  { file: "src/tray-menu.html", js: read("src/tray-menu.js") },
];

const tagRegex = new RegExp(`<(${auditedTags.join("|")})\\b([^>]*)>([\\s\\S]*?)<\\/\\1>`, "gi");
const unlocalized = [];

for (const { file, js } of surfaces) {
  const html = read(file)
    .replace(/<style[\s\S]*?<\/style>/gi, "")
    .replace(/<script[\s\S]*?<\/script>/gi, "");

  let match;
  tagRegex.lastIndex = 0;
  while ((match = tagRegex.exec(html)) !== null) {
    const tagName = match[1].toLowerCase();
    const attrs = match[2];
    const inner = match[3];

    if (attrs.includes("data-i18n=") || inner.includes("data-i18n=")) continue;

    // Strip nested SVGs and child elements that have their own data-i18n or JS ID localization
    const stripped = inner
      .replace(/<svg[\s\S]*?<\/svg>/gi, " ")
      .replace(/<[^>]+>/g, " ")
      .replace(/\s+/g, " ")
      .trim();

    if (!stripped) continue;
    if (allowedExact.has(stripped)) continue;
    if (/^[0-9\s.,/\\+\u2013\u2014:;!?()#%*\u2022\u2192\u2039\u203A\u2713]+$/.test(stripped)) continue;

    const selfId = attrs.match(/\sid="([^"]+)"/)?.[1];
    const childIds = [...inner.matchAll(/\sid="([^"]+)"/g)].map((m) => m[1]);
    const candidateIds = [selfId, ...childIds].filter(Boolean);
    const localizedInJs = candidateIds.some(
      (id) => js.includes(`"${id}"`) || js.includes(`'${id}'`),
    );
    if (localizedInJs) continue;

    unlocalized.push({ file, tag: tagName, attrs: attrs.trim(), text: stripped });
  }
}

console.log(`Found ${unlocalized.length} interactive/heading element(s) without localization across ${surfaces.length} windows:`);
for (const item of unlocalized) {
  console.log(`  ${item.file}: <${item.tag} ${item.attrs}> -> "${item.text}"`);
}

console.log(`\nHardcoded-text result: ${unlocalized.length === 0 ? "PASS" : "FAIL"}`);
if (unlocalized.length > 0) {
  process.exitCode = 1;
}
