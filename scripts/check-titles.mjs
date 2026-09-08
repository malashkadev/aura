import { readFileSync } from "node:fs";

const read = (p) => readFileSync(p, "utf8");
const html = read("src/index.html");
const mainJs = read("src/main.js");

console.log("=== CHECKING ALL TITLE ATTRIBUTES IN INDEX.HTML ===");
const htmlTitles = [...html.matchAll(/title="([^"]+)"/g)].map(m => m[1]);
console.log("HTML titles:", htmlTitles);

console.log("=== CHECKING ALL TITLE ASSIGNMENTS IN MAIN.JS ===");
const lines = mainJs.split("\n");
lines.forEach((l, idx) => {
  if (l.includes('.setAttribute("title"') || l.includes('.title =') || l.includes('title="') || l.includes("title='")) {
    if (idx > 20 && idx < 2400) return; // skip dict
    console.log(`Line ${idx+1}: ${l.trim()}`);
  }
});
