import { readFileSync } from "node:fs";

const read = (p) => readFileSync(p, "utf8");
const html = read("src/index.html");

console.log("=== CHECKING HARDCODED TEXT IN INTERACTIVE ELEMENTS AND HEADINGS ===");

// Options and bare spans are intentionally excluded: language names, provider
// names and model names are proper nouns, while spec values are populated by
// main.js at runtime. The gate focuses on controls and headings a user reads
// as interface copy.
const auditedTags = new Set(["button", "a", "label", "h1", "h2", "h3", "summary"]);
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

const tagRegex = /<([a-zA-Z0-9-]+)([^>]*)>([^<]+)<\/\1>/g;
const unlocalized = [];
let match;

while ((match = tagRegex.exec(html)) !== null) {
  const tagName = match[1].toLowerCase();
  const attrs = match[2];
  const text = match[3].trim();

  if (!auditedTags.has(tagName) || !text) continue;
  if (attrs.includes("data-i18n=")) continue;
  if (allowedExact.has(text)) continue;
  if (/^[0-9\s.,/\\+\u2013\u2014:;!?()#%*\u2022\u2192\u2039\u203A\u2713]+$/.test(text)) continue;

  unlocalized.push({ tag: tagName, attrs: attrs.trim(), text });
}

console.log(`Found ${unlocalized.length} interactive/heading element(s) without data-i18n:`);
for (const item of unlocalized) {
  console.log(`  <${item.tag} ${item.attrs}> -> "${item.text}"`);
}

console.log(`\nHardcoded-text result: ${unlocalized.length === 0 ? "PASS" : "FAIL"}`);
if (unlocalized.length > 0) {
  process.exitCode = 1;
}
