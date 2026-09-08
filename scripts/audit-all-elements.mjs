import { readFileSync } from "node:fs";
import vm from "node:vm";

const read = (p) => readFileSync(p, "utf8");
const html = read("src/index.html");
const mainJs = read("src/main.js").replaceAll("\r\n", "\n");

// Parse dictionaries
const marker = "const i18nDict =";
const start = mainJs.indexOf(marker) + marker.length;
const end = mainJs.indexOf("\n};\n\nlet currentLanguage", start) + 2;
const dicts = vm.runInNewContext("(" + mainJs.slice(start, end) + ")");
const locales = Object.keys(dicts);

console.log("=========================================================");
console.log("   DEEP ELEMENT-BY-ELEMENT AUDIT OF SRC/INDEX.HTML      ");
console.log("=========================================================");

// 1. Audit all buttons
const buttonRegex = /<button\b([^>]*)>([\s\S]*?)<\/button>/g;
let btnMatch;
let btnIdx = 0;
const buttonIssues = [];

while ((btnMatch = buttonRegex.exec(html)) !== null) {
  btnIdx++;
  const attrs = btnMatch[1];
  const inner = btnMatch[2].trim();
  const idMatch = attrs.match(/id="([^"]+)"/);
  const id = idMatch ? idMatch[1] : `btn-${btnIdx}`;
  const classMatch = attrs.match(/class="([^"]+)"/);
  const cls = classMatch ? classMatch[1] : "";
  const titleMatch = attrs.match(/title="([^"]+)"/);
  const title = titleMatch ? titleMatch[1] : null;
  const ariaMatch = attrs.match(/aria-label="([^"]+)"/);
  const aria = ariaMatch ? ariaMatch[1] : null;
  const dataI18nMatch = attrs.match(/data-i18n="([^"]+)"/);
  const dataI18n = dataI18nMatch ? dataI18nMatch[1] : null;

  // Check if inner content has text or if it's purely an SVG
  const innerWithoutSvg = inner.replace(/<svg[\s\S]*?<\/svg>/g, "").replace(/<[^>]+>/g, "").trim();

  // Check how this button is localized:
  // Option A: data-i18n on the button itself
  // Option B: inner element with data-i18n
  const innerHasI18n = inner.includes("data-i18n=");

  // Option C: dynamically updated in applyLanguage
  const updatedInMain = mainJs.includes(`"${id}"`) || mainJs.includes(`'${id}'`);

  if (!dataI18n && !innerHasI18n) {
    if (innerWithoutSvg) {
      buttonIssues.push({ id, cls, issue: `Has text "${innerWithoutSvg}" without data-i18n attribute` });
    } else {
      // Icon-only button
      if (!title && !aria) {
        buttonIssues.push({ id, cls, issue: `Icon-only button has neither title nor aria-label` });
      } else {
        // Has title or aria
        const text = title || aria;
        // Is it localized in main.js?
        const mainSetsTitleOrAria = mainJs.includes(id) && (mainJs.includes(`${id}.setAttribute("title"`) || mainJs.includes(`${id}.setAttribute("aria-label"`) || mainJs.includes(`${id}.title`) || mainJs.includes(`${id}.ariaLabel`));
        if (!mainSetsTitleOrAria) {
          buttonIssues.push({ id, cls, issue: `Icon-only button has static attribute "${text}" never updated when language changes` });
        }
      }
    }
  }
}

console.log(`Audited ${btnIdx} buttons. Issues found: ${buttonIssues.length}`);
for (const bi of buttonIssues) {
  console.log(`  [BUTTON ISSUE] #${bi.id} (.${bi.cls}): ${bi.issue}`);
}

// 2. Audit all inputs and textareas
const inputRegex = /<(input|textarea)\b([^>]*)>/g;
let inpMatch;
let inpIdx = 0;
const inputIssues = [];

while ((inpMatch = inputRegex.exec(html)) !== null) {
  inpIdx++;
  const tag = inpMatch[1];
  const attrs = inpMatch[2];
  const idMatch = attrs.match(/id="([^"]+)"/);
  const id = idMatch ? idMatch[1] : `input-${inpIdx}`;
  const typeMatch = attrs.match(/type="([^"]+)"/);
  const type = typeMatch ? typeMatch[1] : (tag === "textarea" ? "textarea" : "text");
  const placeholderMatch = attrs.match(/placeholder="([^"]+)"/);
  const placeholder = placeholderMatch ? placeholderMatch[1] : null;
  const dataI18nMatch = attrs.match(/data-i18n="([^"]+)"/);
  const dataI18n = dataI18nMatch ? dataI18nMatch[1] : null;

  if (type === "checkbox" || type === "radio") {
    // Check if there's a label for this id or parent label
    // If inside <label>, it's fine
    continue;
  }

  if (placeholder && !dataI18n) {
    // Check if updated in main.js
    const isUpdatedInMain = mainJs.includes(id) && mainJs.includes(`${id}.placeholder`);
    if (!isUpdatedInMain) {
      inputIssues.push({ id, type, issue: `Placeholder "${placeholder}" is static and not in data-i18n or main.js` });
    }
  }
}

console.log(`\nAudited ${inpIdx} inputs/textareas. Issues found: ${inputIssues.length}`);
for (const ii of inputIssues) {
  console.log(`  [INPUT ISSUE] #${ii.id} (${ii.type}): ${ii.issue}`);
}

console.log("\n=========================================================");
