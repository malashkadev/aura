import { readFileSync } from "node:fs";

const read = (p) => readFileSync(p, "utf8");
const html = read("src/index.html");

console.log("=== CHECKING HARDCODED TEXT IN INDEX.HTML ===");

// Regex to find tags that have text content but no data-i18n
// e.g. <span ...>Some Text</span>
// We want to skip <script>, <style>, SVG elements (<svg>, <path>, <circle>, <line>, <polyline>, <polygon>, <rect>), comments, and whitespace-only.

// Let's parse tags
const tagRegex = /<([a-zA-Z0-9_-]+)([^>]*)>([^<]+)<\/\1>/g;
let m;
const unlocalized = [];

const allowedTagsWithoutI18n = new Set([
  "title", "style", "script", "code", "kbd",
  // SVG tags:
  "path", "polyline", "line", "circle", "rect", "polygon"
]);

while ((m = tagRegex.exec(html)) !== null) {
  const tagName = m[1].toLowerCase();
  const attrs = m[2];
  const text = m[3].trim();

  if (allowedTagsWithoutI18n.has(tagName)) continue;
  if (!text) continue;
  // If it has data-i18n attribute, it's localized!
  if (attrs.includes("data-i18n=")) continue;
  // If text is purely punctuation, numbers, or symbols like +, -, /, •, etc.
  if (/^[0-9\s.,/\\+–—:;!?()#%*•→‹›✓🎮]+$/.test(text)) continue;

  // Let's filter out known dynamic placeholders like "v1.1.0" or "0%" or "0:00" or option values that are self-describing
  if (text.startsWith("v1.") || text.startsWith("{") || text === "Aura" || text === "Gemini" || text === "OpenAI" || text === "Groq" || text === "HuggingFace" || text === "Whisper" || text === "Parakeet" || text === "CUDA") continue;

  unlocalized.push({ tag: tagName, attrs: attrs.trim(), text });
}

console.log(`Found ${unlocalized.length} elements with text and without data-i18n:`);
for (const item of unlocalized) {
  console.log(`  <${item.tag} ${item.attrs}> -> "${item.text}"`);
}
