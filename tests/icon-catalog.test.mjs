import assert from "node:assert/strict";
import { existsSync, readFileSync } from "node:fs";
import test from "node:test";
import { getIcon, ICONS, ICON_DEFS, ACCENT_PALETTE, resolveIconDef } from "../design-lab/icon-system/legacy/icons.js";

const read = (path) => readFileSync(new URL(`../${path}`, import.meta.url), "utf8");

// ============================================================================
// TIER 1: FEATURE COVERAGE (Core Icon Library & SVG Geometry Verification)
// ============================================================================

test("Tier 1: Core icon library exports required structures and accent palette", () => {
  assert.ok(ICON_DEFS, "ICON_DEFS must be exported");
  assert.ok(ICONS, "ICONS proxy must be exported");
  assert.equal(typeof getIcon, "function", "getIcon factory must be exported");
  assert.ok(ACCENT_PALETTE, "ACCENT_PALETTE must be exported");

  // Verify semantic accent tokens matching reference screenshot & PROJECT.md
  assert.equal(ACCENT_PALETTE.orange, "#ff4200", "Warm Tactical Orange accent");
  assert.equal(ACCENT_PALETTE.green, "#10b981", "Emerald Green success accent");
  assert.equal(ACCENT_PALETTE.red, "#f43f5e", "Coral Red critical/error accent");
  assert.equal(ACCENT_PALETTE.blue, "#2563eb", "Royal Blue cloud accent");
  assert.equal(ACCENT_PALETTE.teal, "#0d9488", "Teal hardware/GPU accent");
  assert.equal(ACCENT_PALETTE.magenta, "#db2777", "Magenta text AI accent");
  assert.equal(ACCENT_PALETTE.electric, "#3b82f6", "Electric Blue sound accent");
  assert.equal(ACCENT_PALETTE.gold, "#ffb800", "Amber Gold invariant star accent");
});

test("Tier 1: All 33 redesigned glyphs are registered and resolvable", () => {
  // Exhaustive list of 33 unique glyphs from Survey 1 & PROJECT.md
  const core33Glyphs = [
    // Header & Window Controls
    "logo", "minus", "x",
    // 6 Sidebar Navigation Tabs
    "settings", "mic", "file-text", "keyboard", "history", "info",
    // Engine & Processing Units
    "cloud", "cpu", "monitor",
    // Chevrons & Navigational Directions
    "chevron-down", "chevron-right", "chevron-left", "arrow-right", "arrow-up-right", "rotate-ccw",
    // Form & Input Controls
    "eye", "search",
    // Action Buttons & Badges
    "check", "download", "upload", "trash-2", "copy",
    // Fluent / About Actions
    "play-circle", "alert-circle", "star", "power", "external-link",
    // Media & Hardware
    "audio-lines", "globe", "gamepad-2"
  ];

  for (const glyph of core33Glyphs) {
    const def = resolveIconDef(glyph);
    assert.ok(def, `Glyph '${glyph}' must be resolvable via resolveIconDef`);
    const svg = getIcon(glyph);
    assert.ok(svg && svg.length > 0, `getIcon('${glyph}') must generate non-empty SVG markup`);
  }
});

test("Tier 1: Companion navigation and menu list glyphs are registered (Requirement R1/R3)", () => {
  const companionGlyphs = [
    // Language / Script Glyphs (Surface S4 & S5)
    "lang-auto", "lang-layout", "lang-ru", "lang-en", "lang-de", "lang-es",
    "lang-fr", "lang-it", "lang-zh", "lang-pt", "lang-tr", "lang-nl",
    // Cloud AI Providers (Surface S5.3)
    "provider-gemini", "provider-openai", "provider-groq", "provider-huggingface", "provider-custom",
    // Audio Feedback Sound Themes (Surface S5.10)
    "sound-zen", "sound-rhodes", "sound-scifi", "sound-classic", "sound-bubble", "sound-haptic",
    // Hardware & AI Models (Surface S7)
    "engine-cuda", "engine-whisper", "engine-parakeet", "model-turbo", "model-qwen",
    // Voice Commands Guide (Surface S9)
    "cmd-grammar", "cmd-translate",
    // Accordion Summaries (Surface S8)
    "summary-local-model", "summary-performance", "summary-commands", "summary-edit-engine",
    // History Filters (Surface S10)
    "filter-all", "filter-cloud", "filter-local"
  ];

  for (const glyph of companionGlyphs) {
    const def = resolveIconDef(glyph);
    assert.ok(def, `Companion glyph '${glyph}' must be resolvable`);
    const svg = getIcon(glyph);
    assert.ok(svg.startsWith("<svg"), `Companion glyph '${glyph}' must return valid SVG`);
  }
});

test("Tier 1: Every icon definition adheres strictly to visual canon geometry", () => {
  const allKeys = Object.keys(ICON_DEFS);
  assert.ok(allKeys.length >= 33, "ICON_DEFS must define at least 33 canonical glyphs");

  for (const key of allKeys) {
    const svg = getIcon(key, { size: 24, strokeWidth: 1.75 });

    // Strict SVG structural assertions
    assert.match(svg, /^<svg\b/, `Icon '${key}' must begin with <svg`);
    assert.match(svg, /<\/svg>$/, `Icon '${key}' must terminate with </svg>`);
    assert.match(svg, /viewBox="0 0 24 24"/, `Icon '${key}' must declare viewBox="0 0 24 24"`);
    assert.match(svg, /stroke-width="1.75"/, `Icon '${key}' must respect requested stroke-width 1.75`);
    assert.match(svg, /stroke-linecap="round"/, `Icon '${key}' must use round stroke-linecap`);
    assert.match(svg, /stroke-linejoin="round"/, `Icon '${key}' must use round stroke-linejoin`);

    // Verify non-empty body geometry
    const inner = svg.replace(/<svg[^>]*>/, "").replace(/<\/svg>/, "");
    assert.ok(inner.trim().length > 10, `Icon '${key}' must contain substantive vector geometry`);
  }
});

// ============================================================================
// TIER 2: BOUNDARY & CORNER CASES
// ============================================================================

test("Tier 2: Boundary & Corner Cases — Fallback handling for unknown or empty icon names", () => {
  // Unknown names must return empty string and never throw or crash runtime
  assert.equal(getIcon("unknown-non-existent-icon"), "");
  assert.equal(getIcon(""), "");
  assert.equal(getIcon(null), "");
  assert.equal(getIcon(undefined), "");
  assert.equal(getIcon(123), "");

  // ICONS proxy access for unknown names returns undefined
  assert.equal(ICONS["unknown-name"], undefined);
});

test("Tier 2: Boundary & Corner Cases — Extreme and custom viewport sizing", () => {
  // Size 0
  const svg0 = getIcon("settings", { size: 0 });
  assert.match(svg0, /width="0"\s+height="0"/);

  // Size 11 (minimal external link scale in confirmation dialogs)
  const svg11 = getIcon("external-link", { size: 11 });
  assert.match(svg11, /width="11"\s+height="11"/);

  // Size 14 (standard .btn-icon scale)
  const svg14 = getIcon("download", { size: 14 });
  assert.match(svg14, /width="14"\s+height="14"/);

  // Size 15 (tray menu scale)
  const svg15 = getIcon("globe", { size: 15 });
  assert.match(svg15, /width="15"\s+height="15"/);

  // Size 16 (custom select dropdown scale)
  const svg16 = getIcon("lang-ru", { size: 16 });
  assert.match(svg16, /width="16"\s+height="16"/);

  // Size 18 (navigation tab scale)
  const svg18 = getIcon("mic", { size: 18 });
  assert.match(svg18, /width="18"\s+height="18"/);

  // Size 56 (About hero logo scale)
  const svg56 = getIcon("logo", { size: 56 });
  assert.match(svg56, /width="56"\s+height="56"/);

  // Independent width and height overrides
  const svgCustom = getIcon("settings", { width: 48, height: 24 });
  assert.match(svgCustom, /width="48"\s+height="24"/);
});

test("Tier 2: Boundary & Corner Cases — Custom className and attribute escaping", () => {
  const svgClass = getIcon("settings", { className: "nav-icon active-highlight" });
  assert.match(svgClass, /class="nav-icon active-highlight"/);

  // XSS and attribute breakout safety
  const svgEscaped = getIcon("settings", { className: 'test" onmouseover="alert(1)' });
  assert.doesNotMatch(svgEscaped, /onmouseover="alert/);
  assert.match(svgEscaped, /class="test&quot; onmouseover=&quot;alert\(1\)"/);
});

test("Tier 2: Boundary & Corner Cases — Semantic accent handling", () => {
  // Accent orange on Tier 2 icon (cloud)
  const svgOrange = getIcon("cloud", { accent: "orange" });
  assert.ok(svgOrange.includes("#ff4200"), "Accent orange must resolve to #ff4200");

  // Accent green on Tier 2 badge
  const svgGreen = getIcon("badge-work", { accent: "green" });
  assert.ok(svgGreen.includes("#10b981"), "Accent green must resolve to #10b981");

  // Accent red on Tier 2 badge
  const svgRed = getIcon("badge-cycles", { accent: "red" });
  assert.ok(svgRed.includes("#f43f5e"), "Accent red must resolve to #f43f5e");

  // Accent gold on star
  const svgGold = getIcon("star", { accent: "gold" });
  assert.ok(svgGold.includes("#ffb800"), "Accent gold must resolve to #ffb800");

  // Custom stroke color override
  const svgCustomColor = getIcon("nav-general", { color: "#ff4200" });
  assert.ok(svgCustomColor.includes('stroke="#ff4200"'), "Custom stroke color must pass through");
});

test("Tier 2: Boundary & Corner Cases — Accessibility attributes and title tag", () => {
  // Decorative default: aria-hidden="true"
  const svgDec = getIcon("settings");
  assert.match(svgDec, /aria-hidden="true"/);

  // Accessible icon with aria-label: role="img" aria-label="..." and aria-hidden omitted
  const svgA11y = getIcon("settings", { ariaLabel: "Настройки приложения" });
  assert.match(svgA11y, /role="img"/);
  assert.match(svgA11y, /aria-label="Настройки приложения"/);
  assert.doesNotMatch(svgA11y, /aria-hidden="true"/);

  // Title tag injection
  const svgTitle = getIcon("settings", { title: "Параметры" });
  assert.match(svgTitle, /<title>Параметры<\/title>/);
});

test("Tier 2: Boundary & Corner Cases — Model card action button contract preservation", () => {
  // Model card buttons require exact .btn-icon class and 14x14 dimension to satisfy tests/frontend-static.test.mjs
  const btnDownloadSvg = getIcon("download", { size: 14, className: "btn-icon" });
  assert.match(btnDownloadSvg, /class="btn-icon"/);
  assert.match(btnDownloadSvg, /width="14"/);
  assert.match(btnDownloadSvg, /height="14"/);
});

// ============================================================================
// TIER 3: PAIRWISE COMBINATIONS & APPLICATION LAYOUT CONTRACTS
// ============================================================================

test("Tier 3: Pairwise Combinations — Sidebar navigation tab pairings (Surface S1)", () => {
  const tabs = [
    { id: "tab-btn-general", iconKey: "nav-general", label: "Основные" },
    { id: "tab-btn-speech", iconKey: "nav-speech", label: "Голос" },
    { id: "tab-btn-text", iconKey: "nav-text", label: "Текст" },
    { id: "tab-btn-hotkeys", iconKey: "nav-hotkeys", label: "Управление" },
    { id: "tab-btn-history", iconKey: "nav-history", label: "История" },
    { id: "tab-btn-about", iconKey: "nav-about", label: "О программе" }
  ];

  for (const tab of tabs) {
    const icon = getIcon(tab.iconKey, { size: 18, className: "nav-icon" });
    assert.ok(icon, `Tab '${tab.id}' must have an associated icon`);
    assert.match(icon, /width="18"\s+height="18"/);
    assert.match(icon, /class="nav-icon"/);
  }
});

test("Tier 3: Pairwise Combinations — Tray context menu layout contract (Surface S2 & S3)", () => {
  // Check tray-menu.css layout rules
  const css = read("src/tray-menu.css");

  // Tray window must strictly adhere to 270x196 bounds without letterboxing
  assert.match(css, /\.aura-tray-menu\s*\{[^}]*width:\s*100%;/);
  assert.match(css, /\.aura-tray-menu\s*\{[^}]*height:\s*100%;/);

  // Menu icons must be exactly 15px
  assert.match(css, /\.menu-icon\s*\{[^}]*width:\s*15px;[^}]*height:\s*15px;/);

  // Language list container bounded to 142px
  assert.match(css, /\.lang-list-container\s*\{[^}]*max-height:\s*142px;/);

  // Verify tray root item glyph availability
  const trayItems = ["settings", "audio-lines", "globe", "gamepad-2", "power"];
  for (const item of trayItems) {
    const icon = getIcon(item, { size: 15, className: "menu-icon" });
    assert.ok(icon, `Tray item '${item}' must render at 15px`);
    assert.match(icon, /width="15"\s+height="15"/);
  }
});

test("Tier 3: Pairwise Combinations — Custom select dropdown 3-column grid contract (Surface S5)", () => {
  // Select dropdown option icons must render at 16x16px
  const sampleOptionIcon = getIcon("lang-ru", { size: 16, className: "select-panel-item-icon" });
  assert.match(sampleOptionIcon, /width="16"\s+height="16"/);
  assert.match(sampleOptionIcon, /class="select-panel-item-icon"/);

  // 12 supported languages must all have distinctive 16px glyphs
  const languages = ["auto", "layout", "ru", "en", "de", "es", "fr", "it", "zh", "pt", "tr", "nl"];
  for (const lang of languages) {
    const langIcon = getIcon(`lang-${lang}`, { size: 16 });
    assert.ok(langIcon, `Language '${lang}' must have a dedicated 16px option glyph`);
  }
});

test("Tier 3: Pairwise Combinations — Color invariants and design system quarantine", () => {
  // Invariant 1: GitHub Star icon MUST support and default to #ffb800 (Yellow invariant)
  const starDef = resolveIconDef("star");
  assert.equal(starDef.defaultAccent, "gold", "Star icon must have defaultAccent 'gold'");
  const starSvg = getIcon("star");
  assert.ok(starSvg.includes("#ffb800"), "Star SVG must output #ffb800 invariant color");

  // Invariant 2: Report Issue icon MUST have defaultAccent 'red'
  const issueDef = resolveIconDef("alert-circle");
  assert.equal(issueDef.defaultAccent, "red", "Issue icon must have defaultAccent 'red'");

  // Invariant 3: Zero website dark palette contamination in icon registry
  const iconsSource = read("design-lab/icon-system/legacy/icons.js");
  const bannedWebsiteColors = ["#030305", "#0a0a0c", "#0e0e12", "#111114", "#131317", "#121214"];
  for (const banned of bannedWebsiteColors) {
    assert.equal(
      iconsSource.toLowerCase().includes(banned),
      false,
      `Banned website dark color '${banned}' must NEVER appear in the legacy icon registry`
    );
  }
});

// ============================================================================
// TIER 4: STANDALONE INTERACTIVE CATALOG VERIFICATION (design-lab/icon-catalog.html)
// ============================================================================

test("Tier 4: Standalone catalog file exists and is well-formed HTML5", () => {
  assert.ok(
    existsSync(new URL("../design-lab/icon-catalog.html", import.meta.url)),
    "design-lab/icon-catalog.html must exist (Requirement R2)"
  );

  const html = read("design-lab/icon-catalog.html");
  assert.match(html, /<!DOCTYPE html>/i, "Catalog must be standard HTML5");
  assert.match(html, /<html\b[^>]*>/, "Catalog must have opening html tag");
  assert.match(html, /<\/html>/, "Catalog must have closing html tag");
  assert.match(html, /<title>.*Icon.*Catalog.*<\/title>/i, "Catalog must have descriptive title");
});

test("Tier 4: Standalone catalog includes native 16px, 20px, and 24px scale hierarchy", () => {
  const html = read("design-lab/icon-catalog.html");

  // Must declare and demonstrate 16px, 20px, and 24px scales per Requirement R2
  assert.match(html, /16px/i, "Catalog must exhibit 16px interface scale");
  assert.match(html, /20px/i, "Catalog must exhibit 20px interface scale");
  assert.match(html, /24px/i, "Catalog must exhibit 24px interface scale");

  // Verify scale columns or markers in table/grid
  assert.match(html, /scale-16|scale-col-16|16\s*px/i, "Catalog must contain 16px column/marker");
  assert.match(html, /scale-20|scale-col-20|20\s*px/i, "Catalog must contain 20px column/marker");
  assert.match(html, /scale-24|scale-col-24|24\s*px/i, "Catalog must contain 24px column/marker");
});

test("Tier 4: Standalone catalog contains old vs new visual comparisons", () => {
  const html = read("design-lab/icon-catalog.html");

  // Must display old vs new comparison per Requirement R2
  assert.match(html, /old|legacy|baseline/i, "Catalog must reference old/baseline icons");
  assert.match(html, /new|redesign/i, "Catalog must reference new redesigned icons");
});

test("Tier 4: Standalone catalog demonstrates live menu and button pairings", () => {
  const html = read("design-lab/icon-catalog.html");

  // Live context pairings (Requirement R2)
  assert.match(html, /nav-tab|aura-menu-item|menu-item|preview-pairing/i, "Catalog must demonstrate menu pairings");
  assert.match(html, /Основные|General/i, "Catalog must pair icon with real navigation label");
  assert.match(html, /Голос|Voice|Speech/i, "Catalog must pair speech icon with real label");
});

test("Tier 4: Standalone catalog complies strictly with User Rule 10 (Zero Fluff)", () => {
  const html = read("design-lab/icon-catalog.html");

  // Banned pretentious project titles, marketing tags, and lab banners
  const bannedMarketingPhrases = [
    "Aura Sound Lab",
    "Web Audio API Synthesis",
    "Preview Hub",
    "Showcase Hub",
    "Marketing Showcase",
    "Next-Gen Voice Studio"
  ];

  for (const phrase of bannedMarketingPhrases) {
    assert.equal(
      html.includes(phrase),
      false,
      `Banned marketing phrase '${phrase}' must not appear in catalog (Rule 10)`
    );
  }

  // Banned subjective decorative pills
  const bannedDecorativePills = ["Органический", "Минималистичный", "Рекомендуется", "Чистый звук"];
  for (const pill of bannedDecorativePills) {
    assert.equal(
      html.includes(`>${pill}<`),
      false,
      `Banned subjective badge '${pill}' must not appear in catalog (Rule 10)`
    );
  }
});

test("Tier 4: Standalone catalog supports Dark Glass and Light theme toggles", () => {
  const html = read("design-lab/icon-catalog.html");

  // Theme support
  assert.match(html, /data-theme/i, "Catalog must support data-theme attribute");
  assert.match(html, /--bg-app/i, "Catalog must define --bg-app CSS variable");
  assert.match(html, /theme/i, "Catalog must feature theme toggle control");
});
