---
name: Aura Design System
description: Achromatic stealth dark-mode desktop utility interface for voice dictation
colors:
  bg: "#090909"
  surface: "#0d0d0d"
  surface-card: "rgba(18, 18, 18, 0.7)"
  surface-modal: "#0f0f0f"
  border: "rgba(255, 255, 255, 0.08)"
  text-primary: "#f5f5f5"
  text-secondary: "#a3a3a3"
  text-muted: "#808080"
  accent: "#ff4200"
  accent-hover: "#e63b00"
  status-error: "hsl(0, 85%, 65%)"
  status-success: "hsl(142, 70%, 45%)"
typography:
  display:
    fontFamily: "\"Onest\", \"Segoe UI Variable Display\", \"Segoe UI\", -apple-system, BlinkMacSystemFont, system-ui, sans-serif"
    fontWeight: 700
    letterSpacing: "0.08em"
  body:
    fontFamily: "\"Onest\", \"Segoe UI Variable Text\", \"Segoe UI\", -apple-system, BlinkMacSystemFont, system-ui, sans-serif"
    fontWeight: 400
rounded:
  xs: "4px"
  xs-sm: "6px"
  sm: "8px"
  sm-md: "10px"
  md: "12px"
  lg: "20px"
  full: "9999px"
components:
  container:
    backgroundColor: "{colors.surface-card}"
    rounded: "{rounded.lg}"
    padding: "24px 26px"
  tab-active:
    backgroundColor: "rgba(255, 255, 255, 0.05)"
    borderColor: "rgba(255, 255, 255, 0.12)"
    iconColor: "{colors.accent}"
    rounded: "{rounded.sm-md}"
---

# Design System: Aura (Desktop Application)

## 1. Overview

**Creative North Star: "The Stealth Command Deck"**

Aura is designed as a tactical dark-mode dashboard and helper overlay. Because it functions as a global Windows utility, it must feel precise, fast, and unintrusive. Visual style is characterized by 100% achromatic dark obsidian surfaces, clean hairline borders with top-edge specular facets, and sharp Tactical Orange accents strictly reserved for focus, active states, and status. The background stays static and low-contrast to avoid distraction during typing and editing tasks.

**Key Characteristics:**
- 100% achromatic deep obsidian backgrounds (`#090909` / `#0d0d0d`) paired with frosted card surfaces (`rgba(18, 18, 18, 0.7)` and `#0f0f0f` for modals) — strictly zero blue/indigo/slate undertones.
- Single vibrant accent color (`#ff4200`) representing activity and focus.
- Offline `Onest` variable typography (bundled WOFF2) for crisp geometric readability across all supported locales.
- **Strict Zero Shadows & Zero Glows Rule**: Complete absence of outer drop shadows (`box-shadow`, `drop-shadow`) and neon glows (`glow`, `text-shadow`) across all elements and states.

## 2. Colors

Aura uses a strictly achromatic neutral ramp paired with a single Tactical Orange accent to minimize visual noise while running in the background.

### Primary & Status
- **Tactical Orange** (`#ff4200` / `rgb(255, 66, 0)`): Used for primary active states, active navigation icon, progress indicators, update download icon, and recording waveforms. Hover highlight: `#e63b00`.
- **Status Success** (`hsl(142, 70%, 45%)`): Emerald green indicator for target goals completed in toast notifications.
- **Status Error** (`hsl(0, 85%, 65%)`, RGB `239, 68, 68`): Destructive hover actions, download cancellation, close button hover, and error toasts/overlay messages.

### Achromatic Neutrals
- **Background Canvas** (`--bg-color: #090909`): Application frame, tray menu canvas, and HUD overlay surface.
- **Obsidian Surface** (`--surface-color: #0d0d0d`): Structural sidebar/header surfaces.
- **Card Surface** (`--surface-card: rgba(18, 18, 18, 0.7)`): Settings cards and grouped panels.
- **Modal Surface** (`#0f0f0f`): Confirmation dialogs (`#custom-confirm-modal`) and Spotlight onboarding dock (`#spotlight-popover`).
- **Stealth Border** (`--border-color: rgba(255, 255, 255, 0.08)`): Card perimeters and hairline dividers.
- **Primary Text** (`--text-primary: #f5f5f5`): Headings and primary body copy (WCAG AAA contrast ~18.2:1).
- **Secondary Text** (`--text-secondary: #a3a3a3`): Descriptions and inactive tab/option labels (~7.9:1).
- **Muted Text** (`--text-muted: #808080`): Field labels, timestamps, metadata, and utility icons (~5.1:1).

### Named Rules
**The Rarity Rule.** The Tactical Orange accent (`#ff4200`) should occupy no more than ~10% of any viewport. Rarity preserves visual focus for active recording and primary confirmations.

## 3. Typography

Aura bundles the variable geometric sans-serif **Onest** 100% offline inside `src/assets/fonts/` across all 4 required Unicode subsets (`onest-cyrillic-ext.woff2`, `onest-cyrillic.woff2`, `onest-latin-ext.woff2`, `onest-latin.woff2`) and loads them identically in all three desktop windows (`index.html`, `overlay.html`, `tray-menu.css`). No Google Fonts or external CDN font requests are permitted.

**Primary Typeface:** `'Onest'` (`--font-display` and `--font-sans`)
**Fallbacks:** `"Segoe UI Variable Display"` / `"Segoe UI Variable Text"`, `"Segoe UI"`, system-ui, sans-serif

### Hierarchy & Numeric Discipline
- **Brand / Logo** (`800`, `18px`, `letter-spacing: 0.08em`): Sidebar header / About logo text (all-caps).
- **Section Label** (`600`, `12px`, `letter-spacing: 0.14em`–`0.15em`, uppercase): Navigation title and `.field-label` headers.
- **Card Title** (`700`, `19px`, `letter-spacing: -0.018em`, `line-height: 1.25`): Settings card headings with inline 20px monochrome lead icons.
- **Body / Control Text** (`400`–`500`, `14px`, `line-height: 1.55`): Card descriptions, inputs, select options, history items.
- **Metadata / Hints** (`400`–`600`, `12px`–`13px`): Hint lines, model metadata, status badges.
- **Tabular Numerals (`font-variant-numeric: tabular-nums`)**: Enforced on all live counters, percentages, durations, and timestamps (`#overlay-status`, `.progress-pct`, `.history-item-time`, `.history-item-duration`, `.spec-value`, `.volume-value-text`, `.spotlight-step-text`, `.select-panel-item-code`) to prevent horizontal layout jitter.

## 4. Elevation & Shadow Invariant

The system relies on tonal layering and physical top-edge specular facet lines rather than shadow hierarchies, keeping the application flush with Windows.

### Named Rules
**Strict Zero Shadows & Zero Glows Rule.**
- ❌ **NO outer shadows**: Zero floating drop shadows on cards, windows, popovers, dropdowns, or buttons.
- ❌ **NO glows**: Zero neon halos, `drop-shadow()`, or `text-shadow` in any state.
- ✅ **Physical Specular Top-Edge Facet Line Only**:
  - Card rest (`--facet-card`): `box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08);`
  - Modal / Popover / Dropdown (`--facet-elevated`): `box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.12);`
  - Exception: `.spotlight-cutout` uses `box-shadow: 0 0 0 9999px rgba(0, 0, 0, 0.50);` exclusively for viewport dimming.

## 5. Components & Radius Scale

### Corner Radius Scale
- `--radius-xs` (`4px`): Inline chips, scrollbar thumbs, switch thumbs.
- `--radius-xs-sm` (`6px`): Toggle switch tracks (`.toggle-track`), Fluent row icon tiles (`.about-fluent-icon`), replacement trigger chips (`.replacement-trigger`).
- `--radius-sm` (`8px`): History items, history filter buttons inside segmented rail, dropdown rows, search input, card delete buttons.
- `--radius-sm-md` (`10px`): Sidebar nav tabs & sliding `.nav-indicator`, `.btn-primary`, `.btn-secondary`, `.modal-actions .btn`, `.spotlight-dock-actions .btn`, compact utility buttons (`.btn-clear-history`, `.btn-download-card-model`, `.btn-reset-icon`, `.update-badge`), and `.history-filters` segmented rail container.
- `--radius-md` (`12px`): Form inputs (`.custom-select`, `.custom-input`, `.custom-textarea`), `.model-card`, `.summary-block`, `.overlay-card`, tray container.
- `--radius-lg` (`20px`): Outer `.card` containers and `.modal-card`.
- `--radius-full` (`9999px`): Floating toast pills (`.app-toast`) and progress bar tracks.

### Buttons & Interactive Controls (Single Unified Pattern)
- **Primary CTA / Confirm (`.btn-primary`)**: Achromatic elevated graphite key — crisp monochrome separation without orange border noise (`10px` `--radius-sm-md` across cards, modals, and spotlight dock).
  - Rest: `background-color: #272727; border: 1px solid rgba(255, 255, 255, 0.24); box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.15); color: #ffffff;` (white `.btn-icon` / SVG glyphs).
  - Hover: `background-color: #303030; border-color: rgba(255, 255, 255, 0.34); box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.15); color: #ffffff;`
  - Active: `background-color: #1f1f1f; box-shadow: none; transform: scale(0.975) translateY(1px);` (`scale(0.97)` inside modals).
- **Secondary & Utility Buttons (`.btn-secondary`, `.btn-clear-history`, `.btn-download-card-model`, `.btn-reset-icon`, modal/spotlight secondary actions)**: Borderless tonal plates with hairline top specular facet for quiet secondary actions (`Add`, `Clear`, `Test`, `Export`, `Import`, modal cancel).
  - Rest: `background-color: rgba(255, 255, 255, 0.055); border: 1px solid transparent; box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.07); color: #d6d6d6;`
  - Hover: `background-color: rgba(255, 255, 255, 0.105); border-color: transparent; box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.13); color: #ffffff;`
  - Active: `background-color: rgba(255, 255, 255, 0.035); border-color: transparent; box-shadow: none;`
  - Mic Test Active (`.btn-mic-test.active`): Borderless orange-tinted plate `background-color: rgba(var(--accent-color-rgb), 0.18); box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.10); color: #ff6933;`.
- **History Filter Rail (`.history-filters` / `.history-filter-btn`)**: Recessed dark segmented track (`background-color: #070707; border: 1px solid rgba(255, 255, 255, 0.06); padding: 3px; gap: 3px;`), borderless muted tabs transitioning to elevated `#212121` active state (`box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.12); color: #ffffff;`).
- **Update Badge (`.update-badge`)**: Borderless translucent surface `rgba(255, 255, 255, 0.08)` with top specular facet `0.11`, neutral text, and Tactical Orange download glyph (`var(--accent-color)`).
- **Ghost / Destructive Icon Buttons**: Transparent and borderless at rest (`color: var(--text-muted)`), lifting to `rgba(255, 255, 255, 0.08)` / `var(--text-primary)` for neutral icons (`.btn-copy-history`) or `var(--status-error)` with `rgba(var(--status-error-rgb), 0.14)` borderless fill on destructive actions (`.btn-delete-card-model`, `.btn-cancel-download`, `.replacement-delete`).
- **Inner Button Isolation**: Container cards with child action buttons (`.model-card`) decouple container press feedback via `.model-card:not(:has(button:active)):active` and `.model-card:has(button:active) { transform: none !important; }`.

### Cards & Collapsible Summary Blocks
- **Outer Cards (`.card`)**: `border-radius: var(--radius-lg)` (`20px`), `padding: 24px 26px`, `background-color: var(--surface-card)`, `border: 1px solid var(--border-color)`, `box-shadow: var(--facet-card)`.
- **Nested Disclosures (`.summary-block`)**: `border-radius: var(--radius-md)` (`12px`), `background: hsla(0, 0%, 7%, 0.8)`, `box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.07)`.

### Inputs & Select Fields
- **Style**: `background-color: hsl(0, 0%, 8%)`, `border: 1px solid var(--border-color)`, `border-radius: var(--radius-md)` (`12px`) (`var(--radius-sm)` `8px` for `.history-search-input`).
- **Caret & Focus**: `caret-color: var(--accent-color)` on text fields, `border-color: rgba(255, 255, 255, 0.28)` on `:focus` without outer outline/glow (only `.hotkey-input.recording` switches border to `var(--accent-color)`).
- **API Credentials**: Synchronized between Speech and Text tabs via Windows DPAPI, without technical status badges or extra wrapper clutter.

### Navigation (Sidebar & Segmented Switch)
- **Sliding Indicators**: `.nav-indicator`, `#engine-indicator`, and `#history-filter-indicator` share composite-only `transform 0.22s cubic-bezier(0.16, 1, 0.3, 1)` animation via `positionSlidingIndicator`.
- **Active Nav Tab**: `background-color: rgba(255, 255, 255, 0.05); border-color: rgba(255, 255, 255, 0.12); color: var(--text-primary);`, icon in Tactical Orange (`#ff4200`).

### Notification Toasts (4-Tier Semantic Matrix)
- **Form factor**: Floating capsule pill with `border-radius: var(--radius-full)` (`9999px`) and uniform `1px` stroke. *Note: `box-shadow: inset 0 1px 0` is omitted (`box-shadow: none`) on capsule pills to prevent double-border artifacts on rounded caps.*
- **Base Surface**: 100% achromatic `hsla(0, 0%, 9%, 0.92)` with `backdrop-filter: blur(16px)` + auto-dismiss hover pause.
- **4 Semantic States**: `info` (orange icon), `success` (emerald border/icon), `neutral` (muted gray icon for deletions/cancellations), `error` (`rgba(var(--status-error-rgb), 0.35)` border + `--status-error` icon).

### Monochrome Icon System
- All application and tray icons render monochrome via `src/icon-system.js` (`stroke="currentColor"` / `fill="currentColor"`), except official multi-color cloud provider logos (Gemini, Groq, Hugging Face) and the fixed yellow GitHub star / red issue tile on the About tab.

## 6. Information Architecture & Tab Organization

Aura structures settings into 6 dedicated tabs:
1. **Основные (General)**: Minimal system-level configuration — First-Run Offline Prompt (`#first-run-card`), Windows Autostart (`#card-autostart`), and UI Language (`#card-ui-language`).
2. **Голос (Speech)**: Unified speech recognition cockpit — Microphone selector with collapsible live VU meter (`#card-mic-meter`), Recognition Mode segmented control (`#card-engine`), Cloud API credentials (`#card-api-keys`), Cloud Fallback (`#card-cloud-fallback`), Local Model & GPU Accelerator cards (`#card-local-model`), and Performance (`#performance-block`: VRAM Standby & Gaming Mode).
3. **Текст (Text)**: Contextual formatting and AI editing — Conditional Whisper Recognition Language (`#card-language`), Experimental Streaming Input (`#card-streaming`), Custom Dictionary (`#card-vocabulary`), Text Auto-Replacements with `|` caret snippets (`#card-replacements`), Smart Cursor Context (`#card-smart-context`), and Selection Editing (`#card-selection-edit` with Voice Commands Guide, Target Translation Language, and AI Edit Mode Auto/Local/Cloud + shared DPAPI credentials).
4. **Управление (Hotkeys)**: Input trigger & sensory feedback — Global Recording Hotkey recorder (`#card-hotkey` with Toggle Mode switch), Overlay appearance & recording timer (`#card-overlay` with 6 visualizer styles: `wave`, `matrix`, `scale`, `tape`, `beam`, `ribbon`), and Overlay Sound Effects (`#card-sounds` with 6 procedural themes — Zen, Xylophone, Pebble, Classic Bell, Bubble, Haptic Click — plus instant volume/theme preview).
5. **История (History)**: Local transcription memory (`#panel-history`) — Grouped daily timeline, instant search, Cloud/Local filters, copy action, and clear history modal.
6. **О программе (About)**: App identity header, Fluent interactive rows (Spotlight Tour replay, Releases changelog, GitHub Issue reporter, GitHub Star), Clean App Exit, Automatic/Manual Update Checks (`#card-update-checks`), Privacy Diagnostics (`#card-diagnostics`), and Settings JSON Backup/Restore (`#card-backup`).

## 7. Do's and Don'ts

### Do:
- **Do** stick strictly to pure achromatic obsidian backgrounds (`#090909` / `#0d0d0d` / `rgba(18, 18, 18, 0.7)`) and achromatic text (`#f5f5f5`, `#a3a3a3`, `#808080`).
- **Do** bundle fonts offline (`Onest` WOFF2 across all 4 Unicode subsets) across every window.
- **Do** enforce `font-variant-numeric: tabular-nums` on every live timer, percentage, duration, or numeric code badge.
- **Do** keep cards and elements flat with zero outer shadows/glows, relying solely on white top-edge specular facets (`--facet-card` / `--facet-elevated`).
- **Do** maintain `@media (prefers-reduced-motion: reduce)` parity across all 3 windows.

### Don't:
- **Don't** leak cold blue/indigo/slate website tones (`#0a0a0c`, `#9ba6b3`, `#64748b`, etc.) into desktop `src/` files.
- **Don't** add outer `box-shadow`, `drop-shadow`, or `text-shadow` in any state.
- **Don't** apply gradients to buttons or fill secondary buttons with solid orange on hover (keep Tactical Orange flat and translucent).
- **Don't** create ad-hoc button styles when `.btn.btn-secondary` or `.btn.btn-primary` applies.
- **Don't** display internal technical jargon (like "Windows DPAPI") as user-facing status badges.
