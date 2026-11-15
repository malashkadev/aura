/**
 * Aura 2.0 — Production-Grade Vector Icon System
 *
 * Visual Canon:
 * - Geometric contours with multiples-of-3 grid snapping (0, 3, 6, 9, 12, 15, 18, 21, 24)
 *   providing razor-sharp integer pixel rendering at 16px (2:3 scale), 20px, and 24px.
 * - Standardized 1.75px uniform stroke weight with round caps & joins.
 * - Two-tier iconography system:
 *     Tier 1: Neutral monochrome (currentColor / slate)
 *     Tier 2: Purposeful semantic accents (Orange, Emerald, Coral, Royal Blue, Teal, Magenta, Amber)
 */

export const ACCENT_PALETTE = {
  neutral: "currentColor",
  orange: "#ff4200",  // Warm Tactical Orange (Brand / Active)
  green: "#10b981",   // Emerald Green (Success / Verified / Ready)
  red: "#f43f5e",     // Coral Red (Critical / Delete / Errors)
  blue: "#2563eb",    // Royal Blue (Cloud / Remote / Modules)
  teal: "#0d9488",    // Teal (Local GPU / Hardware / Neural Engine)
  magenta: "#db2777", // Magenta (Text AI / Grammar / Formatting)
  electric: "#3b82f6",// Electric Blue (Audio / Sound / Input Channels)
  gold: "#ffb800"     // Amber Gold (Star / Rating / Sponsor Invariant)
};

function escapeAttr(str) {
  return String(str)
    .replace(/&/g, "&amp;")
    .replace(/"/g, "&quot;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

function escapeXml(str) {
  return String(str)
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;");
}

function resolveAccentColor(accent, defaultAccent) {
  if (!accent || accent === "none") {
    return defaultAccent ? (ACCENT_PALETTE[defaultAccent] || defaultAccent) : "currentColor";
  }
  return ACCENT_PALETTE[accent] || accent;
}

/**
 * Registry of SVG icon definitions.
 * Every icon uses a 24x24 viewBox with multiples-of-3 grid alignment.
 */
export const ICON_DEFS = {
  // ==========================================
  // CATEGORY: MAIN NAVIGATION TABS (SURFACE S1)
  // ==========================================
  "nav-general": {
    category: "navigation",
    title: "General Settings",
    defaultAccent: "orange",
    body: () => `
      <circle cx="12" cy="12" r="3" />
      <path d="M12 3v3M12 18v3M3 12h3M18 12h3M5.64 5.64l2.12 2.12M16.24 16.24l2.12 2.12M5.64 18.36l2.12-2.12M16.24 7.76l2.12-2.12" />
      <circle cx="12" cy="12" r="6" stroke-dasharray="1.5 3" />
    `
  },
  "nav-speech": {
    category: "navigation",
    title: "Voice & Speech",
    defaultAccent: "orange",
    body: () => `
      <rect x="9" y="3" width="6" height="11" rx="3" />
      <path d="M6 10v1a6 6 0 0012 0v-1M12 17v4M9 21h6" />
      <line x1="12" y1="6" x2="12" y2="8" />
    `
  },
  "nav-text": {
    category: "navigation",
    title: "Text Formatting",
    defaultAccent: "orange",
    body: () => `
      <path d="M5 3h10l4 4v14H5z" />
      <path d="M15 3v4h4" />
      <path d="M9 11h6M9 15h4" />
    `
  },
  "nav-hotkeys": {
    category: "navigation",
    title: "Global Shortcuts",
    defaultAccent: "orange",
    body: () => `
      <rect x="3" y="6" width="18" height="12" rx="3" />
      <circle cx="7" cy="10" r="0.75" fill="currentColor" />
      <circle cx="12" cy="10" r="0.75" fill="currentColor" />
      <circle cx="17" cy="10" r="0.75" fill="currentColor" />
      <path d="M7 14h10" />
    `
  },
  "nav-history": {
    category: "navigation",
    title: "Transcription History",
    defaultAccent: "orange",
    body: () => `
      <path d="M3 12a9 9 0 109-9 9.75 9.75 0 00-6.74 2.74L3 8" />
      <path d="M3 3v5h5" />
      <path d="M12 7v5l3 3" />
    `
  },
  "nav-about": {
    category: "navigation",
    title: "About Application",
    defaultAccent: "orange",
    body: () => `
      <circle cx="12" cy="12" r="9" />
      <circle cx="12" cy="8" r="0.75" fill="currentColor" />
      <path d="M12 11v5M11 11h1" />
    `
  },

  // ==========================================
  // CATEGORY: WINDOW CONTROLS & BRAND (SURFACE S1)
  // ==========================================
  "window-minimize": {
    category: "window",
    title: "Minimize Window",
    body: () => `<line x1="5" y1="12" x2="19" y2="12" stroke-width="2" />`
  },
  "window-close": {
    category: "window",
    title: "Close Window",
    defaultAccent: "red",
    body: () => `<path d="M6 6l12 12M18 6L6 18" stroke-width="2" />`
  },
  "logo": {
    category: "brand",
    title: "Aura Mark",
    defaultAccent: "orange",
    body: ({ accent }) => `
      <rect x="3" y="3" width="18" height="18" rx="6" stroke="${accent}" stroke-width="1.75" />
      <circle cx="12" cy="12" r="4" stroke="${accent}" stroke-width="1.75" />
      <circle cx="12" cy="12" r="1.5" fill="${accent}" stroke="none" />
      <path d="M12 3v3M12 18v3M3 12h3M18 12h3" stroke="${accent}" stroke-width="1.5" />
    `
  },

  // ==========================================
  // CATEGORY: SYSTEM TRAY MENU (SURFACE S2 & S3)
  // ==========================================
  "tray-settings": {
    category: "tray",
    title: "Open Settings",
    body: () => `
      <circle cx="12" cy="12" r="3" />
      <path d="M12 3v3M12 18v3M3 12h3M18 12h3M5.64 5.64l2.12 2.12M16.24 16.24l2.12 2.12M5.64 18.36l2.12-2.12M16.24 7.76l2.12-2.12" />
    `
  },
  "audio-lines": {
    category: "tray",
    title: "Recognition Waveform",
    defaultAccent: "electric",
    body: () => `
      <line x1="4" y1="10" x2="4" y2="14" />
      <line x1="8" y1="6" x2="8" y2="18" />
      <line x1="12" y1="3" x2="12" y2="21" />
      <line x1="16" y1="6" x2="16" y2="18" />
      <line x1="20" y1="10" x2="20" y2="14" />
    `
  },
  "globe": {
    category: "tray",
    title: "Recognition Language",
    body: () => `
      <circle cx="12" cy="12" r="9" />
      <line x1="3" y1="12" x2="21" y2="12" />
      <path d="M12 3a13.5 13.5 0 010 18M12 3a13.5 13.5 0 000 18" />
    `
  },
  "gamepad-2": {
    category: "tray",
    title: "Gaming Mode",
    defaultAccent: "orange",
    body: () => `
      <path d="M6 11h12a4 4 0 014 4v2a2 2 0 01-3.4 1.4L16 16H8l-2.6 2.4A2 2 0 012 17v-2a4 4 0 014-4z" />
      <path d="M6 13v2M5 14h2" />
      <circle cx="16.5" cy="13.5" r="0.75" fill="currentColor" />
      <circle cx="18.5" cy="15" r="0.75" fill="currentColor" />
    `
  },
  "power": {
    category: "tray",
    title: "Quit Aura",
    defaultAccent: "red",
    body: () => `
      <path d="M18.36 6.64A9 9 0 115.64 6.64" />
      <line x1="12" y1="3" x2="12" y2="11" />
    `
  },

  // ==========================================
  // CATEGORY: ENGINE MODES & ACCELERATION (S3 & S6)
  // ==========================================
  "cloud": {
    category: "engine",
    title: "Cloud AI Engine",
    defaultAccent: "blue",
    body: ({ accent }) => `
      <path d="M18 10h-1.26A6 6 0 107.5 15.5h10.5a4 4 0 000-8z" />
      <circle cx="12" cy="13" r="1.5" fill="${accent}" stroke="none" />
    `
  },
  "cpu": {
    category: "engine",
    title: "Local CPU Engine",
    defaultAccent: "teal",
    body: () => `
      <rect x="5" y="5" width="14" height="14" rx="2" />
      <rect x="9" y="9" width="6" height="6" />
      <path d="M9 2v3M15 2v3M9 19v3M15 19v3M2 9h3M2 15h3M19 9h3M19 15h3" />
    `
  },
  "monitor": {
    category: "engine",
    title: "Local Hardware Display",
    defaultAccent: "teal",
    body: () => `
      <rect x="3" y="4" width="18" height="12" rx="2" />
      <line x1="12" y1="16" x2="12" y2="20" />
      <line x1="8" y1="20" x2="16" y2="20" />
    `
  },

  // ==========================================
  // CATEGORY: NAVIGATION CHEVRONS & ARROWS
  // ==========================================
  "chevron-down": {
    category: "direction",
    title: "Expand Menu",
    body: () => `<path d="M6 9l6 6 6-6" stroke-width="2" />`
  },
  "chevron-right": {
    category: "direction",
    title: "Submenu Drilldown",
    body: () => `<path d="M9 6l6 6-6 6" stroke-width="2" />`
  },
  "chevron-left": {
    category: "direction",
    title: "Back Navigation",
    body: () => `<path d="M15 6l-6 6 6 6" stroke-width="2" />`
  },
  "arrow-right": {
    category: "direction",
    title: "Replacement Mapping",
    body: () => `<path d="M5 12h14M12 5l7 7-7 7" />`
  },
  "arrow-up-right": {
    category: "direction",
    title: "External Web Link",
    body: () => `<path d="M7 17L17 7M7 7h10v10" stroke-width="2" />`
  },

  // ==========================================
  // CATEGORY: UTILITY, ACTIONS & FEEDBACK
  // ==========================================
  "check": {
    category: "status",
    title: "Success Checkmark",
    defaultAccent: "green",
    body: () => `<path d="M5 12l5 5L19 7" stroke-width="2.2" />`
  },
  "eye": {
    category: "control",
    title: "Show Password",
    body: () => `
      <path d="M2 12s3.6-7 10-7 10 7 10 7-3.6 7-10 7-10-7-10-7z" />
      <circle cx="12" cy="12" r="3" />
    `
  },
  "eye-off": {
    category: "control",
    title: "Hide Password",
    body: () => `
      <path d="M17.94 17.94A10.07 10.07 0 0112 19c-6.4 0-10-7-10-7a18.45 18.45 0 015.06-5.94M9.9 4.24A9.12 9.12 0 0112 5c6.4 0 10 7 10 7a18.5 18.5 0 01-2.16 3.19M1 1l22 22M9.88 9.88a3 3 0 104.24 4.24" />
    `
  },
  "rotate-ccw": {
    category: "action",
    title: "Reset Hotkey",
    body: () => `
      <path d="M3 12a9 9 0 103-6.7L2 9" />
      <path d="M2 3v6h6" />
    `
  },
  "search": {
    category: "action",
    title: "Search Records",
    body: () => `
      <circle cx="11" cy="11" r="7" />
      <line x1="21" y1="21" x2="16.5" y2="16.5" stroke-width="2" />
    `
  },
  "download": {
    category: "action",
    title: "Download Component",
    body: () => `
      <path d="M12 3v12M7 10l5 5 5-5M4 17v2a2 2 0 002 2h12a2 2 0 002-2v-2" />
    `
  },
  "upload": {
    category: "action",
    title: "Export Settings",
    body: () => `
      <path d="M12 15V3M7 8l5-5 5 5M4 17v2a2 2 0 002 2h12a2 2 0 002-2v-2" />
    `
  },
  "trash-2": {
    category: "action",
    title: "Delete Model",
    defaultAccent: "red",
    body: () => `
      <path d="M3 6h18M8 6V4a2 2 0 012-2h4a2 2 0 012 2v2M19 6l-1 14a2 2 0 01-2 2H8a2 2 0 01-2-2L5 6M10 11v6M14 11v6" />
    `
  },
  "copy": {
    category: "action",
    title: "Copy to Clipboard",
    body: () => `
      <rect x="9" y="9" width="12" height="12" rx="2" />
      <path d="M5 15H4a2 2 0 01-2-2V4a2 2 0 012-2h9a2 2 0 012 2v1" />
    `
  },
  "play-circle": {
    category: "action",
    title: "Interactive Tour",
    defaultAccent: "orange",
    body: () => `
      <circle cx="12" cy="12" r="9" />
      <polygon points="10 8 16 12 10 16 10 8" fill="currentColor" stroke="none" />
    `
  },
  "alert-circle": {
    category: "status",
    title: "Report Issue / Alert",
    defaultAccent: "red",
    body: () => `
      <circle cx="12" cy="12" r="9" />
      <line x1="12" y1="8" x2="12" y2="12" stroke-width="2" />
      <circle cx="12" cy="16" r="0.75" fill="currentColor" />
    `
  },
  "star": {
    category: "status",
    title: "GitHub Star",
    defaultAccent: "gold",
    body: ({ accent }) => `
      <polygon points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2" fill="${accent}" stroke="${accent}" stroke-width="1.5" />
    `
  },
  "sparkles": {
    category: "ai",
    title: "AI Sparkles",
    defaultAccent: "orange",
    body: ({ accent }) => `
      <path d="M12 3l1.8 4.2L18 9l-4.2 1.8L12 15l-1.8-4.2L6 9l4.2-1.8L12 3z" stroke="${accent}" />
      <path d="M19 16l.9 2.1L22 19l-2.1.9L19 22l-.9-2.1L16 19l2.1-.9L19 16z" fill="${accent}" stroke="none" />
    `
  },
  "key": {
    category: "control",
    title: "API Authorization Key",
    defaultAccent: "orange",
    body: () => `
      <circle cx="8" cy="15" r="4" />
      <path d="M10.85 12.15L19 4M18 5l2 2M15 8l2 2" />
    `
  },
  "terminal": {
    category: "control",
    title: "System Diagnostics",
    body: () => `
      <polyline points="4 17 10 11 4 5" />
      <line x1="12" y1="19" x2="20" y2="19" stroke-width="2" />
    `
  },
  "database": {
    category: "control",
    title: "Backup & Restore",
    body: () => `
      <ellipse cx="12" cy="5" rx="9" ry="3" />
      <path d="M21 12c0 1.66-4 3-9 3s-9-1.34-9-3M3 5v14c0 1.66 4 3 9 3s9-1.34 9-3V5" />
    `
  },
  "volume-2": {
    category: "control",
    title: "Audio Sound Themes",
    defaultAccent: "electric",
    body: () => `
      <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5" />
      <path d="M15.54 8.46a5 5 0 010 7.07M19.07 4.93a10 10 0 010 14.14" />
    `
  },

  // ==========================================
  // CATEGORY: RECOGNITION LANGUAGES (SURFACE S4 & S5)
  // ==========================================
  "lang-auto": {
    category: "language",
    title: "Auto-detect Language",
    defaultAccent: "orange",
    body: ({ accent }) => `
      <rect x="3" y="3" width="18" height="18" rx="4" />
      <path d="M12 6l1.2 2.8L16 10l-2.8 1.2L12 14l-1.2-2.8L8 10l2.8-1.2L12 6z" fill="${accent}" stroke="none" />
      <circle cx="16" cy="16" r="1" fill="${accent}" stroke="none" />
    `
  },
  "lang-layout": {
    category: "language",
    title: "Follow Keyboard Layout",
    defaultAccent: "orange",
    body: () => `
      <rect x="3" y="5" width="18" height="14" rx="3" />
      <path d="M7 9h2M11 9h2M15 9h2M8 13h8" stroke-width="1.75" />
    `
  },
  "lang-ru": {
    category: "language",
    title: "Russian (Русский)",
    body: () => `
      <rect x="3" y="3" width="18" height="18" rx="4" />
      <path d="M15 7v10M15 12h-3a2.5 2.5 0 010-5h3M12 12l-3 5" stroke-width="1.75" />
    `
  },
  "lang-en": {
    category: "language",
    title: "English",
    body: () => `
      <rect x="3" y="3" width="18" height="18" rx="4" />
      <path d="M8 17l4-10 4 10M9.5 13h5" stroke-width="1.75" />
    `
  },
  "lang-de": {
    category: "language",
    title: "German (Deutsch)",
    body: () => `
      <rect x="3" y="3" width="18" height="18" rx="4" />
      <path d="M8.5 7h4a4.5 4.5 0 010 10H8.5V7z" stroke-width="1.75" />
    `
  },
  "lang-fr": {
    category: "language",
    title: "French (Français)",
    body: () => `
      <rect x="3" y="3" width="18" height="18" rx="4" />
      <path d="M8 7h5M8 11.5h3.5M8 7v10M14 11.5h2a2 2 0 000-4h-2v9.5M16 11.5l2 5" stroke-width="1.6" />
    `
  },
  "lang-es": {
    category: "language",
    title: "Spanish (Español)",
    body: () => `
      <rect x="3" y="3" width="18" height="18" rx="4" />
      <path d="M8 8v9M16 8v9M8 8l8 9M9 5c1-.8 2-.8 3 0s2 .8 3 0" stroke-width="1.75" />
    `
  },
  "lang-it": {
    category: "language",
    title: "Italian (Italiano)",
    body: () => `
      <rect x="3" y="3" width="18" height="18" rx="4" />
      <path d="M8 7v10M13 7h6M16 7v10" stroke-width="1.75" />
    `
  },
  "lang-zh": {
    category: "language",
    title: "Chinese (中文)",
    body: () => `
      <rect x="3" y="3" width="18" height="18" rx="4" />
      <path d="M12 6v2M7 9h10M14 11l-5 6M10 11l5 6" stroke-width="1.75" />
    `
  },
  "lang-ja": {
    category: "language",
    title: "Japanese (日本語)",
    body: () => `
      <rect x="3" y="3" width="18" height="18" rx="4" />
      <path d="M7 8h10M12 6v12M9 13.5c3 0 6 1 6 3" stroke-width="1.75" />
    `
  },
  "lang-ko": {
    category: "language",
    title: "Korean (한국어)",
    body: () => `
      <rect x="3" y="3" width="18" height="18" rx="4" />
      <path d="M8 7h8M12 5v4M8 12h8M12 12v5M9 17h6" stroke-width="1.75" />
    `
  },
  "lang-pt": {
    category: "language",
    title: "Portuguese (Português)",
    body: () => `
      <rect x="3" y="3" width="18" height="18" rx="4" />
      <path d="M8 7v10M8 7h3.5a2.5 2.5 0 010 5H8M14 7h5M16.5 7v10" stroke-width="1.6" />
    `
  },
  "lang-ar": {
    category: "language",
    title: "Arabic (العربية)",
    body: () => `
      <rect x="3" y="3" width="18" height="18" rx="4" />
      <path d="M16 10c-2-2-6-1-6 2s4 4 6 2M10 14c-1 3-3 4-5 3" stroke-width="1.75" />
    `
  },
  "lang-tr": {
    category: "language",
    title: "Turkish (Türkçe)",
    body: () => `
      <rect x="3" y="3" width="18" height="18" rx="4" />
      <path d="M7 7h6M10 7v10M14 11.5h2a2 2 0 000-4h-2v9.5M16 11.5l2 5" stroke-width="1.6" />
    `
  },
  "lang-nl": {
    category: "language",
    title: "Dutch (Nederlands)",
    body: () => `
      <rect x="3" y="3" width="18" height="18" rx="4" />
      <path d="M7 7v10M13 7v10M7 7l6 10M16 7v10h4" stroke-width="1.6" />
    `
  },

  // ==========================================
  // CATEGORY: CLOUD AI PROVIDERS (SURFACE S5.3)
  // ==========================================
  "provider-groq": {
    category: "provider",
    title: "Groq LPU Inference",
    defaultAccent: "orange",
    body: ({ accent }) => `
      <path d="M13 2L4 14h7l-2 8 11-12h-7l2-8z" stroke="${accent}" stroke-width="1.75" />
    `
  },
  "provider-cerebras": {
    category: "provider",
    title: "Cerebras Wafer Core",
    defaultAccent: "blue",
    body: ({ accent }) => `
      <rect x="3" y="3" width="18" height="18" rx="3" stroke="${accent}" />
      <rect x="7" y="7" width="10" height="10" rx="1" />
      <circle cx="12" cy="12" r="2" fill="${accent}" stroke="none" />
    `
  },
  "provider-mistral": {
    category: "provider",
    title: "Mistral AI",
    defaultAccent: "orange",
    body: ({ accent }) => `
      <path d="M4 19V5h4l4 5 4-5h4v14" stroke="${accent}" stroke-width="1.75" />
      <path d="M8 14l4-3 4 3" stroke="${accent}" stroke-width="1.75" />
    `
  },
  "provider-deepseek": {
    category: "provider",
    title: "DeepSeek AI",
    defaultAccent: "blue",
    body: ({ accent }) => `
      <path d="M4 14c4-6 10-8 16-4-2 3-5 5-8 5-3 0-5 2-8-1z" stroke="${accent}" stroke-width="1.75" />
      <path d="M12 9c2-3 5-4 8-3" stroke="${accent}" stroke-width="1.5" />
    `
  },
  "provider-openrouter": {
    category: "provider",
    title: "OpenRouter Gateway",
    defaultAccent: "electric",
    body: ({ accent }) => `
      <circle cx="12" cy="12" r="3" stroke="${accent}" />
      <path d="M12 3v6M12 15v6M3 12h6M15 12h6" stroke="${accent}" />
      <circle cx="12" cy="3" r="1" fill="${accent}" stroke="none" />
      <circle cx="12" cy="21" r="1" fill="${accent}" stroke="none" />
      <circle cx="3" cy="12" r="1" fill="${accent}" stroke="none" />
      <circle cx="21" cy="12" r="1" fill="${accent}" stroke="none" />
    `
  },
  "provider-gemini": {
    category: "provider",
    title: "Google Gemini",
    defaultAccent: "blue",
    body: ({ accent }) => `
      <path d="M12 3c0 4.97-4.03 9-9 9 4.97 0 9 4.03 9 9 0-4.97 4.03-9 9-9-4.97 0-9-4.03-9-9z" stroke="${accent}" stroke-width="1.75" />
      <circle cx="12" cy="12" r="1.5" fill="${accent}" stroke="none" />
    `
  },
  "provider-openai": {
    category: "provider",
    title: "OpenAI",
    defaultAccent: "teal",
    body: ({ accent }) => `
      <circle cx="12" cy="12" r="3" stroke="${accent}" />
      <path d="M12 3a9 9 0 016.36 15.36L12 15M12 21a9 9 0 01-6.36-15.36L12 9" stroke="${accent}" stroke-width="1.75" />
    `
  },
  "provider-huggingface": {
    category: "provider",
    title: "Hugging Face",
    defaultAccent: "gold",
    body: ({ accent }) => `
      <circle cx="12" cy="12" r="9" stroke="${accent}" />
      <path d="M8 13a4 4 0 008 0" stroke="${accent}" />
      <circle cx="9" cy="9.5" r="1" fill="${accent}" stroke="none" />
      <circle cx="15" cy="9.5" r="1" fill="${accent}" stroke="none" />
    `
  },
  "provider-custom": {
    category: "provider",
    title: "Custom Self-Hosted Server",
    defaultAccent: "magenta",
    body: () => `
      <rect x="3" y="4" width="18" height="6" rx="2" />
      <rect x="3" y="14" width="18" height="6" rx="2" />
      <circle cx="6" cy="7" r="0.75" fill="currentColor" />
      <circle cx="6" cy="17" r="0.75" fill="currentColor" />
      <path d="M14 7h4M14 17h4" />
    `
  },

  // ==========================================
  // CATEGORY: LOCAL ENGINES & HARDWARE ACCELERATION (S5.4 & S7)
  // ==========================================
  "engine-whisper": {
    category: "hardware",
    title: "Whisper.cpp Engine",
    defaultAccent: "teal",
    body: ({ accent }) => `
      <path d="M3 12h2M7 9v6M11 5v14M15 7v10M19 10v4M21 12h1" stroke="${accent}" stroke-width="1.75" />
    `
  },
  "engine-parakeet": {
    category: "hardware",
    title: "NVIDIA Parakeet Engine",
    defaultAccent: "green",
    body: ({ accent }) => `
      <path d="M3 15c4 0 7-3 9-7 2 4 5 7 9 7M12 8v11" stroke="${accent}" stroke-width="1.75" />
      <path d="M8 12c2 1 3 3 4 5M16 12c-2 1-3 3-4 5" stroke="${accent}" stroke-width="1.5" />
    `
  },
  "engine-cuda": {
    category: "hardware",
    title: "NVIDIA CUDA Acceleration",
    defaultAccent: "green",
    body: ({ accent }) => `
      <path d="M12 3C7.03 3 3 7.03 3 12s4.03 9 9 9c3.2 0 6.02-1.68 7.62-4.22" stroke="${accent}" stroke-width="1.75" />
      <path d="M12 7c-2.76 0-5 2.24-5 5s2.24 5 5 5c1.8 0 3.37-.96 4.24-2.4" stroke="${accent}" stroke-width="1.5" />
      <circle cx="12" cy="12" r="2" fill="${accent}" stroke="none" />
    `
  },
  "engine-cpu": {
    category: "hardware",
    title: "Universal CPU Processing",
    body: () => `
      <rect x="5" y="5" width="14" height="14" rx="2" />
      <rect x="9" y="9" width="6" height="6" />
      <path d="M9 2v3M15 2v3M9 19v3M15 19v3M2 9h3M2 15h3M19 9h3M19 15h3" />
    `
  },

  // ==========================================
  // CATEGORY: MODEL CARD BADGES (SURFACE S7)
  // ==========================================
  "model-tiny": {
    category: "model",
    title: "Tiny Model Gauge",
    body: () => `
      <circle cx="12" cy="12" r="9" />
      <path d="M9 14h6" stroke-width="2" />
    `
  },
  "model-base": {
    category: "model",
    title: "Base Balanced Model",
    defaultAccent: "orange",
    body: ({ accent }) => `
      <circle cx="12" cy="12" r="9" />
      <path d="M7 10h10M7 14h10" stroke="${accent}" stroke-width="2" />
    `
  },
  "model-small": {
    category: "model",
    title: "Small Tier Model",
    body: () => `
      <circle cx="12" cy="12" r="9" />
      <path d="M6 9h12M6 12h12M6 15h12" stroke-width="1.75" />
    `
  },
  "model-medium": {
    category: "model",
    title: "Medium High-Accuracy Model",
    body: () => `
      <rect x="4" y="4" width="16" height="16" rx="3" />
      <path d="M8 8h8M8 11h8M8 14h8M8 17h8" stroke-width="1.5" />
    `
  },
  "model-large": {
    category: "model",
    title: "Large Hexagonal Model",
    defaultAccent: "magenta",
    body: ({ accent }) => `
      <path d="M12 3l8 4.5v9L12 21l-8-4.5v-9L12 3z" stroke="${accent}" stroke-width="1.75" />
      <circle cx="12" cy="12" r="3" stroke="${accent}" />
    `
  },
  "model-turbo": {
    category: "model",
    title: "Turbo Lightning Flagship",
    defaultAccent: "orange",
    body: ({ accent }) => `
      <ellipse cx="12" cy="18" rx="8" ry="3" />
      <ellipse cx="12" cy="14" rx="8" ry="3" />
      <path d="M13 2L7 9h5l-2 5 7-7h-5l2-5z" fill="${accent}" stroke="none" />
    `
  },
  "model-qwen": {
    category: "model",
    title: "Qwen Reasoning Diamond",
    defaultAccent: "magenta",
    body: ({ accent }) => `
      <polygon points="12 3 21 12 12 21 3 12" stroke="${accent}" stroke-width="1.75" />
      <circle cx="12" cy="12" r="3" fill="${accent}" stroke="none" />
    `
  },

  // ==========================================
  // CATEGORY: ACCORDION SUMMARY HEADERS (SURFACE S8)
  // ==========================================
  "summary-local-model": {
    category: "accordion",
    title: "Local Engine & Weights",
    defaultAccent: "teal",
    body: () => `
      <ellipse cx="12" cy="6" rx="8" ry="3" />
      <path d="M4 6v6c0 1.66 3.58 3 8 3s8-1.34 8-3V6" />
      <path d="M4 12v6c0 1.66 3.58 3 8 3s8-1.34 8-3v-6" />
    `
  },
  "summary-performance": {
    category: "accordion",
    title: "Performance & Gaming Dial",
    defaultAccent: "orange",
    body: () => `
      <path d="M4.93 19.07A10 10 0 1119.07 19.07" />
      <path d="M12 12l4-4" stroke-width="2" />
      <circle cx="12" cy="12" r="2" fill="currentColor" stroke="none" />
    `
  },
  "summary-commands": {
    category: "accordion",
    title: "Voice Commands Prompt",
    defaultAccent: "magenta",
    body: ({ accent }) => `
      <path d="M21 15a4 4 0 01-4 4H7l-4 4V7a4 4 0 014-4h10a4 4 0 014 4v8z" />
      <path d="M11 9l1 2 2 1-2 1-1 2-1-2-2-1 2-1 1-2z" fill="${accent}" stroke="none" />
    `
  },
  "summary-edit-engine": {
    category: "accordion",
    title: "AI Editing Pencil",
    defaultAccent: "magenta",
    body: ({ accent }) => `
      <path d="M14 4l6 6L7 23H1v-6L14 4z" />
      <path d="M17 2l2 2-2 2-2-2 2-2z" fill="${accent}" stroke="none" />
      <circle cx="5" cy="19" r="1" fill="${accent}" stroke="none" />
    `
  },

  // ==========================================
  // CATEGORY: VOICE COMMANDS & HISTORY FILTERS (S9 & S10)
  // ==========================================
  "cmd-grammar": {
    category: "command",
    title: "Grammar & Typo Correction",
    defaultAccent: "green",
    body: ({ accent }) => `
      <path d="M13 18l6-6 3 3-6 6-4 1 1-4z" />
      <path d="M3 14l3 3 6-6" stroke="${accent}" stroke-width="2" />
    `
  },
  "cmd-translate": {
    category: "command",
    title: "Instant Translation",
    defaultAccent: "blue",
    body: () => `
      <path d="M4 6h6M7 6v8M5 14c1 0 3-1 4-4" />
      <path d="M15 13l3-7 3 7M16 11h4" />
    `
  },
  "filter-all": {
    category: "filter",
    title: "Filter All Records",
    body: () => `<path d="M4 6h16M4 12h16M4 18h16" stroke-width="1.75" />`
  },
  "filter-cloud": {
    category: "filter",
    title: "Filter Cloud Records",
    defaultAccent: "blue",
    body: ({ accent }) => `
      <path d="M18 10h-1.26A6 6 0 107.5 15.5h10.5a4 4 0 000-8z" stroke="${accent}" stroke-width="1.75" />
    `
  },
  "filter-local": {
    category: "filter",
    title: "Filter Local Records",
    defaultAccent: "teal",
    body: ({ accent }) => `
      <rect x="5" y="5" width="14" height="14" rx="2" stroke="${accent}" stroke-width="1.75" />
      <circle cx="12" cy="12" r="2" fill="${accent}" stroke="none" />
    `
  },

  // ==========================================
  // CATEGORY: SOUND THEMES (SURFACE S5.10)
  // ==========================================
  "sound-zen": {
    category: "sound",
    title: "Zen Theme",
    defaultAccent: "teal",
    body: ({ accent }) => `
      <path d="M12 3c-3 4-3 9 0 14 3-5 3-10 0-14z" stroke="${accent}" stroke-width="1.75" />
      <path d="M12 17c-4-2-8-1-9 2 3 1 7 0 9-2zM12 17c4-2 8-1 9 2-3 1-7 0-9-2z" stroke="${accent}" stroke-width="1.5" />
    `
  },
  "sound-rhodes": {
    category: "sound",
    title: "Rhodes Chime Theme",
    defaultAccent: "electric",
    body: () => `
      <path d="M9 18V5l10-2v13M9 9l10-2" />
      <circle cx="6" cy="18" r="3" fill="currentColor" />
      <circle cx="16" cy="16" r="3" fill="currentColor" />
    `
  },
  "sound-scifi": {
    category: "sound",
    title: "Sci-Fi Laser Theme",
    defaultAccent: "magenta",
    body: ({ accent }) => `
      <circle cx="12" cy="12" r="2.5" fill="${accent}" stroke="none" />
      <ellipse cx="12" cy="12" rx="9" ry="4" transform="rotate(30 12 12)" stroke="${accent}" />
      <ellipse cx="12" cy="12" rx="9" ry="4" transform="rotate(-30 12 12)" stroke="${accent}" />
    `
  },
  "sound-classic": {
    category: "sound",
    title: "Classic Bell Theme",
    defaultAccent: "gold",
    body: () => `
      <path d="M18 8A6 6 0 006 8c0 7-3 9-3 9h18s-3-2-3-9" />
      <path d="M13.73 21a2 2 0 01-3.46 0" />
    `
  },
  "sound-bubble": {
    category: "sound",
    title: "Bubble Droplet Theme",
    defaultAccent: "electric",
    body: ({ accent }) => `
      <path d="M12 3a6 6 0 00-6 6c0 4 6 11 6 11s6-7 6-11a6 6 0 00-6-6z" stroke="${accent}" stroke-width="1.75" />
      <circle cx="12" cy="9" r="2" fill="${accent}" stroke="none" />
    `
  },
  "sound-haptic": {
    category: "sound",
    title: "Haptic Pulse Theme",
    defaultAccent: "orange",
    body: ({ accent }) => `
      <path d="M3 12h3l3-7 6 14 3-7h3" stroke="${accent}" stroke-width="2" />
    `
  },

  // ==========================================
  // CATEGORY: REFERENCE SEMANTIC BADGES (TESSERA STYLE)
  // ==========================================
  "badge-overview": {
    category: "reference",
    title: "Overview Badge",
    defaultAccent: "orange",
    body: ({ accent }) => `
      <circle cx="12" cy="12" r="9" fill="${accent}" stroke="none" />
      <path d="M12 6v12M6 12h12" stroke="#ffffff" stroke-width="2" stroke-linecap="round" />
      <path d="M12 6a6 6 0 016 6h-6V6z" fill="#ffffff" stroke="none" />
    `
  },
  "badge-work": {
    category: "reference",
    title: "Work Items Badge",
    defaultAccent: "green",
    body: ({ accent }) => `
      <circle cx="12" cy="12" r="9" fill="${accent}" stroke="none" />
      <path d="M8 12l3 3 5-5" stroke="#ffffff" stroke-width="2.2" stroke-linecap="round" stroke-linejoin="round" fill="none" />
    `
  },
  "badge-cycles": {
    category: "reference",
    title: "Cycles Dashed Loop",
    defaultAccent: "red",
    body: ({ accent }) => `
      <circle cx="12" cy="12" r="8" stroke="${accent}" stroke-width="2.2" stroke-dasharray="3.2 2.5" fill="none" />
    `
  },
  "badge-modules": {
    category: "reference",
    title: "Modules Network Nodes",
    defaultAccent: "blue",
    body: ({ accent }) => `
      <circle cx="12" cy="6" r="2.5" fill="${accent}" stroke="none" />
      <circle cx="7" cy="17" r="2.5" fill="${accent}" stroke="none" />
      <circle cx="17" cy="17" r="2.5" fill="${accent}" stroke="none" />
      <path d="M12 8.5v3M12 11.5l-3.5 3M12 11.5l3.5 3" stroke="${accent}" stroke-width="1.75" />
    `
  },
  "badge-views": {
    category: "reference",
    title: "Views Pill Lens",
    defaultAccent: "teal",
    body: ({ accent }) => `
      <path d="M3 12c3-4 6-6 9-6s6 2 9 6c-3 4-6 6-9 6s-6-2-9-6z" fill="${accent}" stroke="none" />
      <circle cx="12" cy="12" r="2.5" fill="#ffffff" stroke="none" />
    `
  },
  "badge-pages": {
    category: "reference",
    title: "Pages Folded Document",
    defaultAccent: "magenta",
    body: ({ accent }) => `
      <path d="M6 3h8l5 5v13H6z" fill="${accent}" stroke="none" />
      <path d="M14 3v5h5" fill="rgba(255,255,255,0.4)" stroke="none" />
      <path d="M9 12h6M9 16h4" stroke="#ffffff" stroke-width="1.5" stroke-linecap="round" />
    `
  },
  "badge-intake": {
    category: "reference",
    title: "Intake U-Shield",
    defaultAccent: "electric",
    body: ({ accent }) => `
      <path d="M6 5v8a6 6 0 0012 0V5" stroke="${accent}" stroke-width="2.5" stroke-linecap="round" fill="none" />
      <circle cx="12" cy="13" r="2" fill="${accent}" stroke="none" />
    `
  },
  "badge-trial": {
    category: "reference",
    title: "Trial Countdown Square",
    defaultAccent: "red",
    body: ({ accent }) => `
      <rect x="3" y="3" width="18" height="18" rx="4" fill="${accent}" stroke="none" />
      <path d="M13 5L8 13h4l-1 6 5-8h-4l1-6z" fill="#ffffff" stroke="none" />
    `
  }
};

/**
 * Common Aliases to guarantee backwards compatibility with legacy Lucide / Feather icon names
 */
export const ICON_ALIASES = {
  // Navigation & Settings
  "settings": "nav-general",
  "gear": "nav-general",
  "mic": "nav-speech",
  "microphone": "nav-speech",
  "file-text": "nav-text",
  "keyboard": "nav-hotkeys",
  "history": "nav-history",
  "clock": "nav-history",
  "info": "nav-about",

  // Window Controls
  "minus": "window-minimize",
  "x": "window-close",
  "close": "window-close",

  // Directional
  "chevron-up": "chevron-down", // Can be flipped via transform: rotate(180deg)
  "external-link": "arrow-up-right",

  // Status & Actions
  "delete": "trash-2",
  "trash": "trash-2",
  "clipboard": "copy",
  "reset": "rotate-ccw",
  "wand": "sparkles",
  "auto": "lang-auto",
  "layout": "lang-layout",

  // Language codes without prefix
  "ru": "lang-ru",
  "en": "lang-en",
  "de": "lang-de",
  "fr": "lang-fr",
  "es": "lang-es",
  "it": "lang-it",
  "zh": "lang-zh",
  "ja": "lang-ja",
  "ko": "lang-ko",
  "pt": "lang-pt",
  "ar": "lang-ar",
  "tr": "lang-tr",
  "nl": "lang-nl",

  // Providers without prefix
  "groq": "provider-groq",
  "cerebras": "provider-cerebras",
  "mistral": "provider-mistral",
  "deepseek": "provider-deepseek",
  "openrouter": "provider-openrouter",
  "gemini": "provider-gemini",
  "openai": "provider-openai",
  "huggingface": "provider-huggingface",
  "custom": "provider-custom",

  // Engines without prefix
  "whisper": "engine-whisper",
  "parakeet": "engine-parakeet",
  "cuda": "engine-cuda",

  // Models without prefix
  "tiny": "model-tiny",
  "base": "model-base",
  "small": "model-small",
  "medium": "model-medium",
  "large": "model-large",
  "large-v3-turbo": "model-turbo",
  "large-v3-turbo-q5_0": "model-turbo",
  "turbo": "model-turbo",
  "qwen": "model-qwen",
  "qwen3-0.6b": "model-qwen",
  "qwen3-1.7b": "model-qwen",
  "qwen3-4b": "model-qwen",

  // Accordion summaries without prefix
  "local-model": "summary-local-model",
  "performance": "summary-performance",
  "commands": "summary-commands",
  "edit-engine": "summary-edit-engine",

  // Voice commands without prefix
  "fix_grammar": "cmd-grammar",
  "translate": "cmd-translate"
};

/**
 * Resolves an icon key or alias to its canon definition
 */
export function resolveIconDef(name) {
  if (!name) return null;
  const normalized = String(name).trim().toLowerCase();
  if (ICON_DEFS[normalized]) {
    return { name: normalized, ...ICON_DEFS[normalized] };
  }
  const aliased = ICON_ALIASES[normalized];
  if (aliased && ICON_DEFS[aliased]) {
    return { name: aliased, ...ICON_DEFS[aliased] };
  }
  return null;
}

/**
 * Generates clean, production-grade vector SVG markup.
 *
 * @param {string} name - Icon identifier or alias
 * @param {Object} [options]
 * @param {number} [options.size=18] - Viewport dimension in px
 * @param {number} [options.width] - Overrides size for width
 * @param {number} [options.height] - Overrides size for height
 * @param {string} [options.className=""] - CSS class names
 * @param {number} [options.strokeWidth=1.75] - Stroke weight
 * @param {string} [options.color="currentColor"] - Base stroke color
 * @param {string} [options.accent="none"] - Semantic accent name or hex
 * @param {string} [options.ariaLabel=null] - Accessibility label
 * @param {boolean} [options.ariaHidden=true] - Hide from screen readers if true
 * @param {string} [options.title=null] - Inner title element
 * @returns {string} Raw SVG markup string
 */
export function getIcon(name, options = {}) {
  const {
    size = 18,
    width,
    height,
    className = "",
    strokeWidth = 1.75,
    color = "currentColor",
    accent = "none",
    ariaLabel = null,
    ariaHidden = true,
    title = null
  } = options;

  const def = resolveIconDef(name);
  if (!def) {
    return "";
  }

  const w = width ?? size;
  const h = height ?? size;

  const resolvedAccent = resolveAccentColor(accent, def.defaultAccent);
  const bodyContent = typeof def.body === "function"
    ? def.body({ color, accent: resolvedAccent, strokeWidth })
    : def.body;

  const classAttr = className ? ` class="${escapeAttr(className)}"` : "";
  const titleTag = title ? `<title>${escapeXml(title)}</title>` : "";
  const a11yAttr = ariaLabel
    ? ` role="img" aria-label="${escapeAttr(ariaLabel)}"`
    : (ariaHidden ? ' aria-hidden="true"' : "");

  return `<svg${classAttr} width="${w}" height="${h}" viewBox="0 0 24 24" fill="none" stroke="${color}" stroke-width="${strokeWidth}" stroke-linecap="round" stroke-linejoin="round"${a11yAttr} xmlns="http://www.w3.org/2000/svg">${titleTag}${bodyContent.trim()}</svg>`;
}

/**
 * Dictionary of pre-rendered default SVG strings for direct property interpolation:
 * Example: `ICONS.settings` or `ICONS["nav-general"]`
 */
export const ICONS = new Proxy({}, {
  get(target, prop) {
    if (typeof prop !== "string") return undefined;
    if (prop in target) return target[prop];
    const rendered = getIcon(prop);
    if (rendered) {
      target[prop] = rendered;
      return rendered;
    }
    return undefined;
  },
  has(target, prop) {
    if (typeof prop !== "string") return false;
    return !!resolveIconDef(prop);
  },
  ownKeys() {
    return Array.from(new Set([...Object.keys(ICON_DEFS), ...Object.keys(ICON_ALIASES)]));
  },
  getOwnPropertyDescriptor(target, prop) {
    if (this.has(target, prop)) {
      return {
        enumerable: true,
        configurable: true,
        value: this.get(target, prop)
      };
    }
    return undefined;
  }
});
