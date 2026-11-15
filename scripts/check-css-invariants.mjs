import { existsSync, readFileSync } from "node:fs";

const read = (p) => readFileSync(p, "utf8");

const cssFiles = ["src/style.css", "src/tray-menu.css", "src/overlay.html"];
if (existsSync("src/overlay.css")) {
  cssFiles.push("src/overlay.css");
}

console.log("=== CHECKING CSS INVARIANTS & ZERO SHADOWS/GLOWS ===");

const issues = [];

// The design system allows only white top-edge specular facets, the two
// canonical facet tokens, the spotlight cutout, and explicit `none` values.
const allowedShadowPatterns = [
  /^box-shadow:\s*none(\s*!important)?;?$/,
  /^box-shadow:\s*var\(--facet-(?:card|elevated)\);?$/,
  /^box-shadow:\s*inset\s+0\s+1px\s+(?:0|1px)\s+0?\s*rgba\(\s*255,\s*255,\s*255,\s*0?\.(?:0\d|1[0-5]|\d)\s*\)(\s*!important)?;?$/,
  /^box-shadow:\s*0\s+0\s+0\s+9999px\s+rgba\(\s*0,\s*0,\s*0,\s*0?\.5\d?\s*\);?$/,
];

function isAllowedShadow(trimmed) {
  return allowedShadowPatterns.some((pattern) => pattern.test(trimmed));
}

for (const file of cssFiles) {
  console.log(`\n--- Inspecting ${file} ---`);
  const lines = read(file).split("\n");

  lines.forEach((line, idx) => {
    const lineNum = idx + 1;
    const trimmed = line.trim();

    if (trimmed.includes("box-shadow:")) {
      if (!isAllowedShadow(trimmed)) {
        const message = `${file}:${lineNum}: non-canonical box-shadow: ${trimmed}`;
        issues.push(message);
        console.log(`[BOX-SHADOW] ${message}`);
      }
    }

    if (trimmed.includes("text-shadow:") && trimmed !== "text-shadow: none;") {
      const message = `${file}:${lineNum}: text-shadow is forbidden: ${trimmed}`;
      issues.push(message);
      console.log(`[TEXT-SHADOW] ${message}`);
    }

    if (trimmed.includes("drop-shadow") && !trimmed.includes("drop-shadow(none)")) {
      const message = `${file}:${lineNum}: drop-shadow is forbidden: ${trimmed}`;
      issues.push(message);
      console.log(`[DROP-SHADOW] ${message}`);
    }

    const solidOrangeFill =
      trimmed.includes("background: var(--accent-color)") ||
      trimmed.includes("background-color: var(--accent-color)") ||
      trimmed.includes("background: #fe4200") ||
      trimmed.includes("background: #ff4200");
    const nearHover =
      line.includes(":hover") ||
      lines[idx - 1]?.includes(":hover") ||
      lines[idx - 2]?.includes(":hover");
    if (solidOrangeFill && nearHover) {
      const message = `${file}:${lineNum}: solid orange hover fill: ${trimmed}`;
      issues.push(message);
      console.log(`[SOLID ORANGE HOVER] ${message}`);
    }
  });
}

console.log(`\nCSS invariant result: ${issues.length === 0 ? "PASS" : "FAIL"} (${issues.length} issue(s))`);
if (issues.length > 0) {
  process.exitCode = 1;
}
