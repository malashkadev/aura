import { readFileSync } from "node:fs";
import vm from "node:vm";

const read = (p) => readFileSync(p, "utf8");
const html = read("src/index.html");
const mainJs = read("src/main.js").replaceAll("\r\n", "\n");

const marker = "const i18nDict =";
const start = mainJs.indexOf(marker) + marker.length;
const end = mainJs.indexOf("\n};\n\nlet currentLanguage", start) + 2;
const dicts = vm.runInNewContext("(" + mainJs.slice(start, end) + ")");
const dictionaryKeys = new Set(Object.keys(dicts.ru || {}));

const issues = [];

console.log("=== CHECKING ALL TITLE ATTRIBUTES IN INDEX.HTML ===");
for (const tag of html.match(/<[^>]+>/g) || []) {
  const titleMatch = tag.match(/\stitle="([^"]*)"/);
  if (titleMatch) {
    const title = titleMatch[1].trim();
    if (/^[a-z][a-z0-9]*(?:_[a-z0-9]+)+$/.test(title) && !/\sdata-i18n-title="/.test(tag)) {
      issues.push(`raw title key "${title}" has no data-i18n-title replacement`);
    }
  }

  const i18nTitleMatch = tag.match(/\sdata-i18n-title="([^"]+)"/);
  if (i18nTitleMatch && !dictionaryKeys.has(i18nTitleMatch[1])) {
    issues.push(`data-i18n-title key "${i18nTitleMatch[1]}" is missing from the dictionaries`);
  }
}

console.log(`Checked ${(html.match(/\stitle="/g) || []).length} title attribute(s).`);
console.log(`\nTitle result: ${issues.length === 0 ? "PASS" : "FAIL"} (${issues.length} issue(s))`);
for (const issue of issues) {
  console.error(`  [TITLE ISSUE] ${issue}`);
}
if (issues.length > 0) {
  process.exitCode = 1;
}
