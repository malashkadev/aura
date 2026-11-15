import { readFileSync } from "node:fs";

const read = (p) => readFileSync(p, "utf8");

const files = ["src/index.html", "src/overlay.html", "src/tray-menu.html"];
const issues = [];

for (const file of files) {
  console.log(`=== CHECKING ARIA-LABELS IN ${file} ===`);
  const content = read(file);
  const ids = new Set([...content.matchAll(/\sid="([^"]+)"/g)].map((m) => m[1]));

  const labels = [...content.matchAll(/aria-label="([^"]*)"/g)].map((m) => m[1].trim());
  for (const label of labels) {
    console.log(`  ${label}`);
    if (!label) {
      issues.push(`${file}: empty aria-label`);
    } else if (/^[a-z][a-z0-9]*(?:_[a-z0-9]+)+$/.test(label)) {
      issues.push(`${file}: aria-label looks like a raw i18n key: "${label}"`);
    }
  }

  for (const match of content.matchAll(/<label\b[^>]*\sfor="([^"]+)"/g)) {
    if (!ids.has(match[1])) {
      issues.push(`${file}: <label for="${match[1]}"> has no matching id`);
    }
  }

  const buttonRegex = /<button\b([^>]*)>([\s\S]*?)<\/button>/g;
  let buttonMatch;
  while ((buttonMatch = buttonRegex.exec(content)) !== null) {
    const attrs = buttonMatch[1];
    const inner = buttonMatch[2];
    const text = inner
      .replace(/<svg[\s\S]*?<\/svg>/g, "")
      .replace(/<[^>]+>/g, "")
      .trim();
    const hasName = /\saria-label="[^"]+"/.test(attrs) || /\stitle="[^"]+"/.test(attrs);
    const isHidden = /\shidden\b/.test(attrs) || /aria-hidden="true"/.test(attrs);
    if (!text && !hasName && !isHidden) {
      const id = attrs.match(/\sid="([^"]+)"/)?.[1] || "(anonymous)";
      issues.push(`${file}: icon-only button #${id} has no accessible name`);
    }
  }
}

console.log(`\nARIA result: ${issues.length === 0 ? "PASS" : "FAIL"} (${issues.length} issue(s))`);
for (const issue of issues) {
  console.error(`  [ARIA ISSUE] ${issue}`);
}
if (issues.length > 0) {
  process.exitCode = 1;
}
