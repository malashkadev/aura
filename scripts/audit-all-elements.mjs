import { readdirSync, readFileSync } from "node:fs";

const read = (p) => readFileSync(p, "utf8").replaceAll("\r\n", "\n");

const allSrcJs = readdirSync("src")
  .filter((f) => f.endsWith(".js"))
  .sort();
const mainBundleJs = allSrcJs
  .filter((f) => f !== "overlay.js" && f !== "tray-menu.js")
  .map((f) => read(`src/${f}`))
  .join("\n");

const surfaces = [
  { htmlFile: "src/index.html", controllerJs: mainBundleJs },
  { htmlFile: "src/overlay.html", controllerJs: read("src/overlay.js") },
  { htmlFile: "src/tray-menu.html", controllerJs: read("src/tray-menu.js") },
];

console.log("=========================================================");
console.log("   DEEP ELEMENT-BY-ELEMENT AUDIT OF ALL 3 APP WINDOWS    ");
console.log("=========================================================");

let totalButtons = 0;
let totalInputs = 0;
const buttonIssues = [];
const inputIssues = [];

for (const { htmlFile, controllerJs } of surfaces) {
  const html = read(htmlFile)
    .replace(/<style[\s\S]*?<\/style>/gi, "")
    .replace(/<script[\s\S]*?<\/script>/gi, "");

  // 1. Audit all <button> elements
  const buttonRegex = /<button\b([^>]*)>([\s\S]*?)<\/button>/g;
  let btnMatch;
  let btnIdx = 0;

  while ((btnMatch = buttonRegex.exec(html)) !== null) {
    btnIdx++;
    totalButtons++;
    const attrs = btnMatch[1];
    const inner = btnMatch[2].trim();
    const idMatch = attrs.match(/id="([^"]+)"/);
    const id = idMatch ? idMatch[1] : `${htmlFile}-btn-${btnIdx}`;
    const classMatch = attrs.match(/class="([^"]+)"/);
    const cls = classMatch ? classMatch[1] : "";
    const titleMatch = attrs.match(/title="([^"]+)"/);
    const title = titleMatch ? titleMatch[1] : null;
    const ariaMatch = attrs.match(/aria-label="([^"]+)"/);
    const aria = ariaMatch ? ariaMatch[1] : null;
    const dataI18nMatch = attrs.match(/data-i18n="([^"]+)"/);
    const dataI18n = dataI18nMatch ? dataI18nMatch[1] : null;

    const innerWithoutSvg = inner
      .replace(/<svg[\s\S]*?<\/svg>/g, "")
      .replace(/<[^>]+>/g, "")
      .trim();

    const innerHasI18n = inner.includes("data-i18n=");
    const innerIds = [...inner.matchAll(/\sid="([^"]+)"/g)].map((m) => m[1]);
    const innerLocalizedById = innerIds.some(
      (childId) =>
        controllerJs.includes(`"${childId}"`) || controllerJs.includes(`'${childId}'`),
    );

    if (!dataI18n && !innerHasI18n && !innerLocalizedById) {
      if (innerWithoutSvg) {
        buttonIssues.push({
          file: htmlFile,
          id,
          cls,
          issue: `Has text "${innerWithoutSvg}" without data-i18n or JS localization`,
        });
      } else {
        const isHidden = attrs.includes("hidden");
        if (!title && !aria && !isHidden) {
          buttonIssues.push({
            file: htmlFile,
            id,
            cls,
            issue: "Icon-only button has neither title nor aria-label",
          });
        } else if (!isHidden) {
          const text = title || aria;
          const hasI18nTitle = attrs.includes("data-i18n-title=");
          const varNames = [
            ...controllerJs.matchAll(
              new RegExp(
                `(?:const|let|var|\\b)\\s*([a-zA-Z0-9_$]+)\\s*=\\s*document\\.getElementById\\(["']${id}["']\\)`,
                "g",
              ),
            ),
          ].map((m) => m[1]);
          varNames.push(id);

          const jsSetsTitleOrAria =
            hasI18nTitle ||
            varNames.some(
              (varName) =>
                controllerJs.includes(`${varName}.setAttribute("title"`) ||
                controllerJs.includes(`${varName}.setAttribute("aria-label"`) ||
                controllerJs.includes(`${varName}.title`) ||
                controllerJs.includes(`${varName}.ariaLabel`),
            ) ||
            controllerJs.includes(`getElementById("${id}").setAttribute("title"`) ||
            controllerJs.includes(`getElementById("${id}").setAttribute("aria-label"`);

          if (!jsSetsTitleOrAria) {
            buttonIssues.push({
              file: htmlFile,
              id,
              cls,
              issue: `Icon-only button has static attribute "${text}" never updated when language changes`,
            });
          }
        }
      }
    }
  }

  // 2. Audit all inputs and textareas
  const inputRegex = /<(input|textarea)\b([^>]*)>/g;
  let inpMatch;
  let inpIdx = 0;

  while ((inpMatch = inputRegex.exec(html)) !== null) {
    inpIdx++;
    totalInputs++;
    const tag = inpMatch[1];
    const attrs = inpMatch[2];
    const idMatch = attrs.match(/id="([^"]+)"/);
    const id = idMatch ? idMatch[1] : `${htmlFile}-input-${inpIdx}`;
    const typeMatch = attrs.match(/type="([^"]+)"/);
    const type = typeMatch ? typeMatch[1] : tag === "textarea" ? "textarea" : "text";
    const placeholderMatch = attrs.match(/placeholder="([^"]+)"/);
    const placeholder = placeholderMatch ? placeholderMatch[1] : null;
    const dataI18nMatch = attrs.match(/data-i18n="([^"]+)"/);
    const dataI18n = dataI18nMatch ? dataI18nMatch[1] : null;

    if (type === "checkbox" || type === "radio") {
      continue;
    }

    if (placeholder && (placeholder.startsWith("http") || placeholder.startsWith("openai/"))) {
      continue;
    }

    if (placeholder && !dataI18n) {
      const varNames = [
        ...controllerJs.matchAll(
          new RegExp(
            `(?:const|let|var|\\b)\\s*([a-zA-Z0-9_$]+)\\s*=\\s*document\\.getElementById\\(["']${id}["']\\)`,
            "g",
          ),
        ),
      ].map((m) => m[1]);
      varNames.push(id);

      const isUpdatedInJs =
        varNames.some((varName) => controllerJs.includes(`${varName}.placeholder`)) ||
        controllerJs.includes(`getElementById("${id}").placeholder`);

      if (!isUpdatedInJs) {
        inputIssues.push({
          file: htmlFile,
          id,
          type,
          issue: `Placeholder "${placeholder}" is static and not in data-i18n or controller JS`,
        });
      }
    }
  }
}

console.log(`Audited ${totalButtons} buttons across ${surfaces.length} windows. Issues found: ${buttonIssues.length}`);
for (const bi of buttonIssues) {
  console.log(`  [BUTTON ISSUE] ${bi.file} #${bi.id} (.${bi.cls}): ${bi.issue}`);
}

console.log(`\nAudited ${totalInputs} inputs/textareas across ${surfaces.length} windows. Issues found: ${inputIssues.length}`);
for (const ii of inputIssues) {
  console.log(`  [INPUT ISSUE] ${ii.file} #${ii.id} (${ii.type}): ${ii.issue}`);
}

console.log("\n=========================================================");
const elementIssueCount = buttonIssues.length + inputIssues.length;
console.log(` Element audit result: ${elementIssueCount === 0 ? "PASS" : "FAIL"} (${elementIssueCount} issue(s))`);
if (elementIssueCount > 0) {
  process.exitCode = 1;
}
