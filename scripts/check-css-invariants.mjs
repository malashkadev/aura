import { readFileSync } from "node:fs";

const read = (p) => readFileSync(p, "utf8");

const cssFiles = ["src/style.css", "src/tray-menu.css", "src/overlay.css"];

console.log("=== CHECKING CSS INVARIANTS & ZERO SHADOWS/GLOWS ===");

for (const file of cssFiles) {
  console.log(`\n--- Inspecting ${file} ---`);
  const css = read(file);
  const lines = css.split("\n");

  lines.forEach((line, idx) => {
    const lineNum = idx + 1;
    const trimmed = line.trim();

    // Check box-shadow
    if (trimmed.includes("box-shadow:")) {
      // Check if it's an allowed inset facet highlight: e.g. inset 0 1px 0 rgba(255, 255, 255, ...)
      const isAllowedInset = /box-shadow:\s*inset\s+0\s+1px\s+0\s+rgba\(255,\s*255,\s*255,\s*0\.0[0-9]+\);?/.test(trimmed)
        || trimmed === "box-shadow: none;"
        || trimmed === "box-shadow: none !important;";
      if (!isAllowedInset) {
        console.log(`[BOX-SHADOW] ${file}:${lineNum}: ${trimmed}`);
      }
    }

    // Check text-shadow
    if (trimmed.includes("text-shadow:") && trimmed !== "text-shadow: none;") {
      console.log(`[TEXT-SHADOW] ${file}:${lineNum}: ${trimmed}`);
    }

    // Check filter drop-shadow
    if (trimmed.includes("drop-shadow") && !trimmed.includes("drop-shadow(none)")) {
      console.log(`[DROP-SHADOW] ${file}:${lineNum}: ${trimmed}`);
    }

    // Check for solid orange fills on buttons hover
    if ((trimmed.includes("background: var(--accent-color)") || trimmed.includes("background-color: var(--accent-color)") || trimmed.includes("background: #fe4200") || trimmed.includes("background: #ff4200")) && (line.includes(":hover") || lines[idx-1]?.includes(":hover") || lines[idx-2]?.includes(":hover"))) {
      console.log(`[SOLID ORANGE HOVER] ${file}:${lineNum}: ${trimmed}`);
    }
  });
}
