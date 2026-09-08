import { readFileSync } from "node:fs";

const read = (p) => readFileSync(p, "utf8");

const files = ["src/index.html", "src/overlay.html", "src/tray-menu.html"];

for (const file of files) {
  console.log(`=== CHECKING ARIA-LABELS IN ${file} ===`);
  const content = read(file);
  const labels = [...content.matchAll(/aria-label="([^"]+)"/g)].map(m => m[1]);
  for (const l of labels) {
    console.log(`  ${l}`);
  }
}
