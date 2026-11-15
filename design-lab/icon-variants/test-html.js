import fs from "fs";

const content = fs.readFileSync("G:/Aura/2.0/design-lab/icon-variants/settings-icons.html", "utf8");

// Check script execution and DOM structure
console.log("Checking HTML content...");
if (!content.includes("<!DOCTYPE html>")) throw new Error("Missing DOCTYPE");
if (!content.includes("</html>")) throw new Error("Missing </html>");

// Extract ITEMS JSON
const jsonMatch = content.match(/const ITEMS = (\[.*?\]);/s);
if (!jsonMatch) throw new Error("Could not find ITEMS JSON in script");

const items = JSON.parse(jsonMatch[1]);
console.log("Extracted items count:", items.length);
if (items.length !== 24) throw new Error(`Expected 24 items, got ${items.length}`);

// Verify that all IDs 1..24 are present in correct order
for (let i = 0; i < 24; i++) {
  const item = items[i];
  if (item.id !== i + 1) throw new Error(`Bad ID at ${i}: ${item.id}`);
}

console.log("Validation passed! All 24 items verified.");
