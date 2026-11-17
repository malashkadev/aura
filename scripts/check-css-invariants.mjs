import { existsSync, readFileSync } from "node:fs";

const read = (p) => readFileSync(p, "utf8");

const cssFiles = ["src/style.css", "src/tray-menu.css", "src/overlay.html"];
if (existsSync("src/overlay.css")) {
  cssFiles.push("src/overlay.css");
}
const htmlFiles = ["src/index.html", "src/overlay.html", "src/tray-menu.html"];

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

const solidOrangeDeclPattern =
  /^background(?:-color)?:\s*(?:var\(--accent-color\)|#ff4200|#fe4200|rgb\(\s*25[45],\s*66,\s*0\s*\)|rgba\(\s*(?:25[45],\s*66,\s*0|var\(--accent-color-rgb\)),\s*(?:1|1\.0+)\s*\))(\s*!important)?;?$/i;

function isAllowedShadow(trimmed) {
  return allowedShadowPatterns.some((pattern) => pattern.test(trimmed));
}

function lineAtOffset(text, offset) {
  return text.slice(0, offset).split("\n").length;
}

function checkHoverBlocks(file, cssSource, baseLineOffset = 0) {
  // Strip CSS comments while preserving line breaks so line numbers stay accurate
  const stripped = cssSource.replace(/\/\*[\s\S]*?\*\//g, (comment) =>
    comment.replace(/[^\n]/g, " "),
  );

  // Walk balanced braces so @media / @supports wrappers are descended into cleanly
  function scanRules(text, offsetBase) {
    let i = 0;
    while (i < text.length) {
      const openIdx = text.indexOf("{", i);
      if (openIdx === -1) break;
      const selector = text.slice(i, openIdx).trim();
      let depth = 1;
      let closeIdx = openIdx + 1;
      while (closeIdx < text.length && depth > 0) {
        if (text[closeIdx] === "{") depth += 1;
        else if (text[closeIdx] === "}") depth -= 1;
        closeIdx += 1;
      }
      const body = text.slice(openIdx + 1, closeIdx - 1);
      if (selector.startsWith("@media") || selector.startsWith("@supports")) {
        scanRules(body, offsetBase + openIdx + 1);
      } else if (selector.includes(":hover")) {
        const bodyStartOffset = offsetBase + openIdx + 1;
        const bodyLines = body.split("\n");
        let relOffset = 0;
        for (const rawLine of bodyLines) {
          const decl = rawLine.trim();
          if (solidOrangeDeclPattern.test(decl)) {
            const lineNum = baseLineOffset + lineAtOffset(stripped, bodyStartOffset + relOffset);
            const message = `${file}:${lineNum}: solid orange hover fill in '${selector.replace(/\s+/g, " ")}': ${decl}`;
            issues.push(message);
            console.log(`[SOLID ORANGE HOVER] ${message}`);
          }
          relOffset += rawLine.length + 1;
        }
      }
      i = closeIdx;
    }
  }

  scanRules(stripped, 0);
}

for (const file of cssFiles) {
  console.log(`\n--- Inspecting ${file} ---`);
  const content = read(file).replaceAll("\r\n", "\n");
  const lines = content.split("\n");

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
  });

  if (file.endsWith(".html")) {
    for (const styleMatch of content.matchAll(/<style\b[^>]*>([\s\S]*?)<\/style>/gi)) {
      const styleStartLine = lineAtOffset(content, styleMatch.index) - 1;
      checkHoverBlocks(file, styleMatch[1], styleStartLine);
    }
  } else {
    checkHoverBlocks(file, content, 0);
  }
}

for (const htmlFile of htmlFiles) {
  const html = read(htmlFile).replaceAll("\r\n", "\n");
  for (const attrMatch of html.matchAll(/\sstyle="([^"]+)"/g)) {
    const styleVal = attrMatch[1];
    const lineNum = lineAtOffset(html, attrMatch.index);
    if (/(?:box-shadow|text-shadow|drop-shadow)/i.test(styleVal)) {
      const message = `${htmlFile}:${lineNum}: inline shadow/glow style is forbidden: style="${styleVal}"`;
      issues.push(message);
      console.log(`[INLINE-STYLE] ${message}`);
    }
  }
}

console.log(`\nCSS invariant result: ${issues.length === 0 ? "PASS" : "FAIL"} (${issues.length} issue(s))`);
if (issues.length > 0) {
  process.exitCode = 1;
}
