---
name: Aura Design System
description: Premium dark-mode glassmorphic utility interface for voice dictation
colors:
  bg: "#000000"
  surface: "#0d0d0d"
  surface-card: "#14141499"
  border: "#29292980"
  text-primary: "#f5f7fa"
  text-secondary: "#9ba6b3"
  text-muted: "#64748b"
  accent: "#ff4200"
  accent-hover: "#e63b00"
  accent-glow: "none"
typography:
  display:
    fontFamily: "'Onest', 'Segoe UI Variable Display', system-ui, sans-serif"
    fontWeight: 700
    letterSpacing: "0.08em"
  body:
    fontFamily: "'Onest', 'Segoe UI Variable Text', system-ui, sans-serif"
    fontWeight: 400
rounded:
  xs: "4px"
  sm: "8px"
  md: "12px"
  lg: "20px"
  full: "9999px"
spacing:
  sm: "8px"
  md: "12px"
  lg: "20px"
  xl: "24px"
components:
  container:
    backgroundColor: "{colors.surface-card}"
    rounded: "{rounded.lg}"
    padding: "24px"
  tab-active:
    backgroundColor: "rgba(255, 66, 0, 0.08)"
    textColor: "{colors.accent}"
    rounded: "{rounded.md}"
---

# Design System: Aura

## 1. Overview

**Creative North Star: "The Stealth Command Deck"**

Aura is designed as a tactical dark-mode dashboard and helper overlay. Because it functions as a global Windows utility, it must feel precise, fast, and unintrusive. Visual style is characterized by deep black surfaces, glassmorphic panel blurs, and sharp orange accents. The design focuses attention on active states while keeping the background static and low-contrast to avoid distraction during typing and editing tasks.

**Key Characteristics:**
- Deep black body backgrounds (`#000000`) paired with semi-transparent frosted card overlays (`#14141499`).
- Single vibrant accent color (`#ff4200`) representing activity and focus.
- Modern sans-serif typography for clean digital readability.
- **Strict Zero Shadows & Zero Glows Rule**: Complete absence of outer drop shadows (`box-shadow`, `drop-shadow`) and neon glows (`glow`, `--accent-glow`, `text-shadow`) across all elements and states.

## 2. Colors

Aura uses a highly restricted palette to minimize visual noise while running in the background.

### Primary
- **Tactical Orange** (`#ff4200`): Used for primary active states, selected navigation, buttons, progress indicators, and recording indicators.

### Neutral
- **Pure Black** (`#000000`): Application and window background.
- **Deep Obsidian** (`#0d0d0d`): Body layouts and structural containers.
- **Obsidian Glass** (`#14141499`): Interactive cards and settings panels.
- **Stealth Border** (`#29292980`): Container outlines and divider lines.
- **Near White** (`#f5f7fa`): Title text and primary body copy.
- **Muted Slate** (`#9ba6b3`): Subtext, settings descriptions, and inactive states.

### Named Rules
**The Rarity Rule.** The Tactical Orange accent (`#ff4200`) should be used on no more than 10% of any screen layout. Its rarity keeps it highly effective at drawing focus to the active state.

## 3. Typography

Aura pairs modern geometric sans-serif typography with native system fallbacks to convey clean technical precision, completely packaged offline in local WOFF2 files (zero network tracking or external requests).

**Primary Typeface:** Onest (packaged locally via `src/assets/fonts/*.woff2`)
**Fallbacks:** "Segoe UI Variable Display", "Segoe UI Variable Text", system-ui, sans-serif

### Hierarchy
- **Display / Logo** (800, 1.1rem / 1.5rem, letter-spacing: 0.08em): App branding logo text (all-caps).
- **Headline** (600, 1.0rem, letter-spacing: 0.05em): Upper headers and section headings.
- **Title** (600, 1.1rem, line-height: 1.4): Card names and primary setting categories.
- **Body** (400, 0.95rem, line-height: 1.5): Standard settings descriptions, inputs, history items.
- **Label** (500, 0.85rem, letter-spacing: 0.02em): Small metadata tags, model information.

## 4. Elevation & Shadow Invariant

The system relies on glassmorphism and physical specular facet lines rather than shadow hierarchies, keeping the application flush with the Windows desktop experience.

### Named Rules
**Strict Zero Shadows & Zero Glows Rule.** Layouts remain flush and embedded in the dark canvas.
- ❌ **NO outer shadows**: `box-shadow: none !important;` (no black, dark, or floating shadows under cards, windows, popovers, or buttons).
- ❌ **NO glows**: `filter: none !important;` (no colored halos, neon glows, `--accent-glow`, or `text-shadow` in any state).
- ✅ **Physical Specular Top-Edge Facet Line Only**: Only subtle top border glass highlight is permitted to define the top bevel of glass cards:
  - Default card rest: `box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08);`
  - Active / modal / popover: `box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.12);`

## 5. Components

### Buttons
- **Shape:** rounded-sm (`8px`) for standard buttons, rounded-md (`12px`) for large cards.
- **Secondary / Standard Actions (Add, Reset, Clear, Test):** Background `transparent`, border `1px solid var(--border-color)` (`#29292980`), text `var(--text-secondary)`. Hover: `border-color: var(--accent-color)` and `color: var(--text-primary)`. Zero solid orange fills.
- **Primary / Save (Settings):** Inverted outline style — background `none`, border `1px solid var(--border-color)`, text `var(--text-primary)`. Hover: `background-color: var(--text-primary)`, text `#0d0d0d`.
- **Ghost / Icon buttons:** Transparent background, `color: var(--text-muted)`. Hover: `color: var(--text-primary)` or `var(--status-error)` for delete.

### Cards / Containers
- **Corner Style:** rounded-lg (`20px`) for outer boundaries, rounded-md (`12px`) for nested cards.
- **Background:** `#14141499` with `backdrop-filter: blur(20px)`.
- **Border:** `1px solid #29292980`.
- **Shadow / Facet:** `box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.08);` (Zero outer shadows, specular top edge).

### Inputs / Fields
- **Style:** Background `#0d0d0d`, border `1px solid #29292980`, rounded-sm (`8px`).
- **Focus:** Border changes to `#ff4200` without outer glow (`outline: none; box-shadow: none;`).
- **API Credentials:** Synchronized between speech and text tabs via Windows DPAPI. Clean label without intrusive status badges; security is indicated by masked placeholder and dedicated helper caption.

### Sound Settings
- **Audio Feedback:** Volume slider and theme selector trigger instant sound preview on interaction. No redundant "Test" buttons.

### Navigation (Sidebar)
- **Active tab (Achromatic Frosted Plate):** Background `rgba(255, 255, 255, 0.06)`, border `rgba(255, 255, 255, 0.16)`, text `var(--text-primary)`, rounded-md (`12px`). Active icon retains Tactical Orange (`#ff4200`).
- **Hover tab:** Background `hsla(0, 0%, 18%, 0.4)`, text `var(--text-primary)`.

### Buttons & Interactive Controls (Achromatic Frosted Plate Pattern)
- **Primary Action Buttons (.btn-primary):** High-priority call to action (e.g. Save, Confirm). Rest: `background: rgba(var(--accent-color-rgb), 0.12); border: 1px solid var(--accent-color); color: var(--text-primary);`. Hover: `background: rgba(var(--accent-color-rgb), 0.22);`. Active: `transform: scale(0.97);`.
- **Secondary & Utility Buttons (.btn-secondary):** Rest: `background: transparent; border: 1px solid var(--border-color); color: var(--text-secondary);`. Hover: `background-color: rgba(255, 255, 255, 0.06); border-color: rgba(255, 255, 255, 0.18); color: var(--text-primary);` (quiet frosted lift, no orange flash). Active: `background-color: rgba(255, 255, 255, 0.09); border-color: rgba(255, 255, 255, 0.22); transform: scale(0.98);`.
- **Inner Button Isolation:** Container cards with child buttons (e.g. `.model-card` containing delete/download/cancel buttons) MUST decouple container `:active` via `.model-card:not(:has(button:active)):active` and `.model-card:has(button:active) { transform: none !important; }`. Inner action buttons handle their own tactile compression (`scale(0.90..0.94)`).

### Notification Toasts (4-Tier Semantic Matrix)
- **Form factor:** Floating capsule pill with `border-radius: var(--radius-full)` (`9999px`) and uniform 1px perimeter stroke (`border: 1px solid rgba(255, 255, 255, 0.16)`). *Note: `box-shadow: inset 0 1px 0` is strictly banned on capsule pills to prevent double-border artifacts on curved caps.*
- **Typography:** Local `var(--font-display)` (`Onest`), 600 weight, `0.82rem`, `letter-spacing: 0.01em`.
- **Base Surface:** 100% achromatic deep obsidian `hsla(0, 0%, 9%, 0.92)` with `backdrop-filter: blur(16px)`. Never tint the base surface with red or green.
- **4 Semantic States:**
  - `info`: Tactical Orange `(i)` for in-progress operations and announcements (e.g. "Downloading model...", "Update available").
  - `success`: Emerald Green `✓` (`#4ade80`) for target goals achieved (e.g. "Model downloaded", "Settings file saved", "Diagnostics copied").
  - `neutral`: Muted Gray trash icon (`var(--text-muted)`) for deletions and cancellations (e.g. "Model deleted", "Download cancelled") — quiet confirmation without celebratory green checkmarks.
  - `error`: Ruby Red `!` (`#f87171`) for failures.

## 6. Information Architecture & Tab Organization

Aura structures settings into 6 dedicated, non-overlapping domains:
1. **Основные (General)**: Minimal system-level configuration — Windows Autostart and UI Language.
2. **Голос и движок (Speech & Engine)**: Unified speech recognition cockpit — Microphone selector with live VU meter, primary engine switcher, Cloud API credentials (Gemini, OpenAI, Groq, HF, Custom), Cloud Fallback, and Local Engine Section (offline Whisper/Parakeet model selectors, hardware acceleration mode, VRAM Standby Unloader, Gaming Mode).
3. **Текст (Text)**: Contextual formatting and AI text modification — Smart Cursor Context Insertion, Selection Edit Master Toggle with inline phrase chips, Default Translation Language, AI Editing Mode (Auto/Local/Cloud), and synchronized API credentials.
4. **Горячие клавиши (Hotkeys)**: Interaction triggers and sensory feedback — Global recording hotkey recorder with Win32 LL hook suppression, HUD Overlay screen position selector, and Sound Effects (master toggle, volume slider, sound themes). *Previews trigger automatically on volume adjustment and theme selection.*
5. **История (History)**: Local transcription memory — Instant search, filter chips, copy with tactile feedback, and history wipe.
6. **О программе (About)**: Windows 11 Fluent interactive rows — Interactive onboarding walkthrough, Version changelog, Issue reporter, GitHub repository link (with permanent golden star), Automatic Update Checks card with manual check button, and clean application exit.

## 7. Do's and Don'ts

### Do:
- **Do** stick to a pure black and obsidian background (`#000000` / `#0d0d0d`) to maintain the Tactical Deck mood.
- **Do** use `backdrop-filter: blur(20px)` on primary window containers for a premium desktop feel.
- **Do** use uppercase and tracking on brand and section labels to preserve technical style.
- **Do** keep cards and elements strictly flat without floating drop shadows or neon glows, using specular facet top-edges (`inset 0 1px 0 rgba(255, 255, 255, 0.08)`).
- **Do** maintain bi-directional synchronization between common settings across tabs.

### Don't:
- **Don't** add drop shadows or box shadows to cards, buttons, or dialogs in any state (`:hover`, `:active`, `:focus`).
- **Don't** add neon glows, colored shadow halos, or text shadows to active tokens or buttons.
- **Don't** use standard white, light grey, or saturated blue/purple SaaS cards.
- **Don't** apply gradients to buttons or text labels (keep Tactical Orange `#ff4200` flat).
- **Don't** add borders thicker than 1px for cards or containers (no side-stripe borders).
- **Don't** add redundant test/preview buttons when an input control already provides direct interactive feedback.
- **Don't** display internal technical jargon (like "Windows DPAPI") as user-facing status badges.
