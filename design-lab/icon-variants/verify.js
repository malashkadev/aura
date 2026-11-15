import fs from "fs";

const g1 = JSON.parse(fs.readFileSync("G:/Aura/2.0/design-lab/icon-variants/group1.json", "utf8"));
const g2 = JSON.parse(fs.readFileSync("G:/Aura/2.0/design-lab/icon-variants/group2.json", "utf8"));
const g3 = JSON.parse(fs.readFileSync("G:/Aura/2.0/design-lab/icon-variants/group3.json", "utf8"));

const all = [...g1, ...g2, ...g3];
console.log("Total items loaded:", all.length);

let errors = 0;

if (all.length !== 24) {
  console.error("FAIL: Expected 24 items, got " + all.length);
  errors++;
}

// Regex XML validator for self-closing and closed tags in SVG
function validateSvgSnippet(svgBody, label) {
  const fullSvg = `<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">${svgBody}</svg>`;
  
  // Check that every opening tag has matching closing or is self-closing
  const tagRegex = /<\/?([a-zA-Z0-9]+)([^>]*?)(\/?)>/g;
  let match;
  const stack = [];
  
  while ((match = tagRegex.exec(fullSvg)) !== null) {
    const isClosing = match[0].startsWith("</");
    const tagName = match[1];
    const isSelfClosing = match[3] === "/" || match[0].endsWith("/>");

    if (isClosing) {
      if (stack.length === 0) {
        console.error(`XML Error in ${label}: unexpected closing tag </${tagName}>`);
        errors++;
        return false;
      }
      const top = stack.pop();
      if (top !== tagName) {
        console.error(`XML Error in ${label}: mismatched closing tag </${tagName}>, expected </${top}>`);
        errors++;
        return false;
      }
    } else if (!isSelfClosing) {
      stack.push(tagName);
    }
  }

  if (stack.length > 0) {
    console.error(`XML Error in ${label}: unclosed tags: ${stack.join(", ")}`);
    errors++;
    return false;
  }
  return true;
}

for (let i = 0; i < 24; i++) {
  const item = all[i];
  const expectedId = i + 1;
  if (!item) {
    console.error(`Item ${expectedId} is missing!`);
    errors++;
    continue;
  }
  if (item.id !== expectedId) {
    console.error(`Item at index ${i} has id ${item.id}, expected ${expectedId}`);
    errors++;
  }
  if (item.codeA !== `${expectedId}A`) {
    console.error(`Item ${expectedId} codeA is ${item.codeA}, expected ${expectedId}A`);
    errors++;
  }
  if (item.codeB !== `${expectedId}B`) {
    console.error(`Item ${expectedId} codeB is ${item.codeB}, expected ${expectedId}B`);
    errors++;
  }
  if (!item.title || item.title.trim() === "") {
    console.error(`Item ${expectedId} missing title`);
    errors++;
  }
  if (!item.currentSvg || item.currentSvg.trim() === "") {
    console.error(`Item ${expectedId} missing currentSvg`);
    errors++;
  }
  if (!item.varA || item.varA.trim() === "") {
    console.error(`Item ${expectedId} missing varA`);
    errors++;
  }
  if (!item.varB || item.varB.trim() === "") {
    console.error(`Item ${expectedId} missing varB`);
    errors++;
  }

  validateSvgSnippet(item.currentSvg, `Item ${expectedId} (Current)`);
  validateSvgSnippet(item.varA, `Item ${expectedId} (VarA: ${item.codeA})`);
  validateSvgSnippet(item.varB, `Item ${expectedId} (VarB: ${item.codeB})`);
}

if (errors === 0) {
  console.log("SUCCESS: All 24 items and 72 SVG definitions (Current, VarA, VarB) verified perfectly!");
} else {
  console.error(`FAILED: ${errors} errors found.`);
  process.exit(1);
}
