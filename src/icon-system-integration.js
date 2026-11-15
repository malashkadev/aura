/**
 * Replaces the application's legacy inline SVGs with the Aura monochrome set.
 *
 * The markup keeps the existing classes and dimensions, so the current CSS
 * keeps controlling size, color and alignment. Only the glyph body and the
 * presentation attributes are swapped.
 */

import { icons } from "./icon-system-api.js";

const SINGLE_ICONS = [
  // Window chrome
  ["#btn-window-minimize .control-icon", "window-minimize"],
  ["#btn-window-close .control-icon", "window-close"],

  // Settings navigation
  ["#tab-btn-general .nav-icon", "nav-general"],
  ["#tab-btn-speech .nav-icon", "nav-speech"],
  ["#tab-btn-text .nav-icon", "nav-text"],
  ["#tab-btn-hotkeys .nav-icon", "nav-hotkeys"],
  ["#tab-btn-history .nav-icon", "nav-history"],
  ["#tab-btn-about .nav-icon", "nav-about"],

  // Speech / engine
  ["#label-cloud .engine-segment-icon", "engine-cloud"],
  ["#label-local .engine-segment-icon", "engine-local"],
  [".eye-icon", "eye"],

  // Accordions and actions
  [".summary-chevron", "chevron-down"],
  ["#btn-reset-hotkey svg", "reset"],
  [".search-icon", "search"],
  ["#btn-clear-history-search svg", "close"],
  ["#toast-close svg", "close"],
  ["#btn-first-run-download .btn-icon", "download"],
  ["#btn-export-settings .btn-icon", "upload"],
  ["#btn-import-settings .btn-icon", "download"],
  ["#update-badge svg", "download"],
  ["#btn-exit-app .btn-tour-icon", "power"],

  // About rows
  ["#btn-replay-onboarding .about-fluent-icon svg", "play"],
  ["#btn-changelog .about-fluent-icon svg", "doc"],
  ["#btn-report-issue .about-fluent-icon svg", "alert"],
  ["#btn-github-star .about-fluent-icon svg", "star"],
  ["#btn-replay-onboarding .about-fluent-chevron", "chevron-right"],
  ["#btn-changelog .about-fluent-chevron", "external"],
  ["#btn-report-issue .about-fluent-chevron", "external"],
  ["#btn-github-star .about-fluent-chevron", "external"],

  // History filters and model actions
  [".status-ready-icon", "check", 2.2],
  ["#btn-history-filter-all .history-filter-icon", "filter-all"],
  ["#btn-history-filter-cloud .history-filter-icon", "filter-cloud"],
  ["#btn-history-filter-local .history-filter-icon", "filter-local"],

  // GPU cards
  ["[data-gpu='cpu'] .model-glyph", "cpu"],
  ["[data-gpu='cuda'] .model-glyph", "cuda"],

  // Whisper size cards
  ["[data-model='tiny'] .model-glyph", "bars-1"],
  ["[data-model='base'] .model-glyph", "bars-2"],
  ["[data-model='small'] .model-glyph", "bars-3"],
  ["[data-model='medium'] .model-glyph", "bars-4"],
  ["[data-model='large-v3-turbo-q5_0'] .model-glyph", "bolt"],
  ["[data-model='large-v3-turbo'] .model-glyph", "bolt"],
  ["[data-model='parakeet-v3'] .model-glyph", "cuda"],

  // Qwen editing models
  ["[data-model='qwen3-0.6b'] .model-glyph", "qwen"],
  ["[data-model='qwen3-1.7b'] .model-glyph", "qwen"],
  ["[data-model='qwen3-4b'] .model-glyph", "qwen"],

  // Tray menu
  ["#item-open-settings .menu-icon", "tray-settings"],
  ["#item-recog-trigger .menu-icon", "tray-recognition"],
  ["#recog-status-icon", "cloud"],
  ["#item-recog-trigger .menu-arrow", "chevron-right"],
  ["#item-lang-trigger .menu-icon", "globe"],
  ["#item-lang-trigger .menu-arrow", "chevron-right"],
  ["#item-gaming .menu-icon", "gamepad"],
  ["#item-exit .menu-icon", "power"],
  ["#btn-back-header svg", "chevron-left"],
  ["#btn-back-lang-header svg", "chevron-left"],
  ["#sub-cloud .menu-icon", "cloud"],
  ["#sub-local .menu-icon", "device"],
  ["#sub-cloud .check-mark", "check"],
  ["#sub-local .check-mark", "check"],
];

const SUMMARY_ICONS = [
  "section-local-model",
  "section-performance",
  "section-voice-commands",
  "section-edit-engine",
];

// One icon per settings card, prepended to the card heading at hydration time.
// Utility glyphs are intentionally reused where the meaning matches.
const CARD_TITLES = [
  ["#first-run-card", "download"],
  ["#card-autostart", "power"],
  ["#card-ui-language", "globe"],
  ["#card-mic-meter", "mic"],
  ["#card-engine", "waveform"],
  ["#card-api-keys", "key"],
  ["#card-cloud-fallback", "reset"],
  ["#card-language", "globe"],
  ["#card-streaming", "doc"],
  ["#card-vocabulary", "list"],
  ["#card-replacements", "arrow-right"],
  ["#card-smart-context", "sparkles"],
  ["#card-selection-edit", "pencil"],
  ["#card-hotkey", "keyboard"],
  ["#card-overlay-timer", "clock"],
  ["#card-sounds", "volume-2"],
  [".card-history", "clock"],
  ["#card-update-checks", "download"],
  ["#card-diagnostics", "terminal"],
  ["#card-backup", "database"],
];

function applyCardTitleIcon(card, key) {
  if (!card || card.querySelector(".card-title-row")) return;
  const title = card.querySelector(".card-title");
  if (!title) return;

  const row = document.createElement("div");
  row.className = "card-title-row";

  const icon = document.createElement("span");
  icon.className = "card-title-icon";
  icon.setAttribute("aria-hidden", "true");
  icon.innerHTML = icons.svg(key, { size: 20 });

  title.parentNode.insertBefore(row, title);
  row.appendChild(icon);
  row.appendChild(title);
}

function applyToSvg(svg, key, strokeWidth = 2) {
  if (!svg || !icons || typeof icons.body !== "function") return;
  const markup = icons.body(key);
  if (!markup) return;

  svg.innerHTML = markup;
  svg.setAttribute("viewBox", "0 0 24 24");
  svg.setAttribute("fill", "none");
  svg.setAttribute("stroke", "currentColor");
  svg.setAttribute("stroke-width", String(strokeWidth));
  svg.setAttribute("stroke-linecap", "round");
  svg.setAttribute("stroke-linejoin", "round");
}

/**
 * Hydrates the static icon slots in the settings window and the tray menu.
 * Safe to call on either document: unknown selectors simply match nothing.
 */
export function hydrateAppIcons(root = document) {
  if (!icons || !icons.body) return;

  for (const [selector, key, strokeWidth] of SINGLE_ICONS) {
    root.querySelectorAll(selector).forEach((svg) => applyToSvg(svg, key, strokeWidth));
  }

  root.querySelectorAll(".summary-category-icon").forEach((svg, index) => {
    applyToSvg(svg, SUMMARY_ICONS[index] || "layers");
  });

  for (const [selector, key] of CARD_TITLES) {
    const card = root.querySelector(selector);
    if (card) applyCardTitleIcon(card, key);
  }
}
