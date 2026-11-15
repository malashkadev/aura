#!/usr/bin/env node
/**
 * Visual Capture Runner for Aura 2.0
 * 
 * Automates headless browser screenshot capture of design-lab harnesses and UI components
 * in compliance with User Rules 4 (Visual QA Pre-flight) and 5 (Windows Browser Automation).
 *
 * Usage:
 *   node tests/visual-capture.mjs
 *   node tests/visual-capture.mjs --target design-lab/icon-catalog.html --output tests/screenshots/catalog.png
 */

import { existsSync, mkdirSync, mkdtempSync, rmSync, statSync } from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve } from "node:path";
import { pathToFileURL, fileURLToPath } from "node:url";
import { spawnSync } from "node:child_process";

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);
const ROOT_DIR = resolve(__dirname, "..");

// Known Windows browser executable locations
const CANDIDATE_BROWSERS = [
  "C:\\Program Files (x86)\\Microsoft\\Edge\\Application\\msedge.exe",
  "C:\\Program Files\\Microsoft\\Edge\\Application\\msedge.exe",
  "C:\\Program Files\\Google\\Chrome\\Application\\chrome.exe",
  "C:\\Program Files (x86)\\Google\\Chrome\\Application\\chrome.exe",
  join(process.env.LOCALAPPDATA || "", "Microsoft\\Edge\\Application\\msedge.exe"),
  join(process.env.LOCALAPPDATA || "", "Google\\Chrome\\Application\\chrome.exe"),
];

/**
 * Finds the first existing browser executable on the Windows host.
 */
export function findBrowserExecutable() {
  for (const browserPath of CANDIDATE_BROWSERS) {
    if (browserPath && existsSync(browserPath)) {
      return browserPath;
    }
  }
  return null;
}

/**
 * Captures a screenshot of the specified target file using a headless browser.
 */
export async function captureScreenshot({
  targetRelativePath = "design-lab/icon-catalog.html",
  outputRelativePath = "tests/screenshots/icon-catalog-preview.png",
  width = 1280,
  height = 1024,
  timeoutMs = 15000,
} = {}) {
  const browserPath = findBrowserExecutable();
  if (!browserPath) {
    throw new Error("No compatible headless browser (Edge or Chrome) found on host.");
  }

  const [filePathPart, queryPart] = targetRelativePath.split("?");
  const targetFile = resolve(ROOT_DIR, filePathPart);
  if (!existsSync(targetFile)) {
    return {
      success: false,
      skipped: true,
      reason: `Target file does not exist yet: ${filePathPart}`,
      outputPath: null,
    };
  }

  const outputPath = resolve(ROOT_DIR, outputRelativePath);
  mkdirSync(dirname(outputPath), { recursive: true });

  const tempUserDataDir = mkdtempSync(join(tmpdir(), "aura-headless-"));
  let targetUrl = pathToFileURL(targetFile).href;
  if (queryPart) {
    targetUrl += `?${queryPart}`;
  }

  try {
    const args = [
      "--headless=new",
      "--disable-gpu",
      "--no-first-run",
      "--no-default-browser-check",
      "--allow-file-access-from-files",
      "--disable-web-security",
      `--user-data-dir=${tempUserDataDir}`,
      `--window-size=${width},${height}`,
      "--hide-scrollbars",
      `--screenshot=${outputPath}`,
      targetUrl,
    ];

    const result = spawnSync(browserPath, args, {
      timeout: timeoutMs,
      encoding: "utf8",
      windowsHide: true,
    });

    if (result.error) {
      throw result.error;
    }

    if (!existsSync(outputPath)) {
      throw new Error(`Screenshot file was not generated at expected location: ${outputPath}`);
    }

    const stats = statSync(outputPath);
    if (stats.size === 0) {
      throw new Error(`Screenshot file was generated but is 0 bytes: ${outputPath}`);
    }

    return {
      success: true,
      skipped: false,
      browser: browserPath,
      url: targetUrl,
      outputPath,
      sizeBytes: stats.size,
    };
  } finally {
    try {
      rmSync(tempUserDataDir, { recursive: true, force: true });
    } catch {
      // Best-effort temp dir cleanup
    }
  }
}

// CLI execution entrypoint
if (process.argv[1] && resolve(process.argv[1]) === resolve(__filename)) {
  const args = process.argv.slice(2);
  let target = "design-lab/icon-catalog.html";
  let output = "tests/screenshots/icon-catalog-preview.png";

  for (let i = 0; i < args.length; i++) {
    if (args[i] === "--target" && args[i + 1]) {
      target = args[++i];
    } else if (args[i] === "--output" && args[i + 1]) {
      output = args[++i];
    }
  }

  console.log(`[Visual-QA] Starting visual capture runner...`);
  console.log(`[Visual-QA] Target: ${target}`);
  console.log(`[Visual-QA] Destination: ${output}`);

  captureScreenshot({
    targetRelativePath: target,
    outputRelativePath: output,
  })
    .then((res) => {
      if (res.skipped) {
        console.log(`[Visual-QA] Notice: ${res.reason}`);
        process.exit(0);
      } else if (res.success) {
        console.log(`[Visual-QA] SUCCESS: Captured ${res.sizeBytes} bytes`);
        console.log(`[Visual-QA] Browser: ${res.browser}`);
        console.log(`[Visual-QA] Saved to: ${res.outputPath}`);
        process.exit(0);
      } else {
        console.error(`[Visual-QA] FAILED: Screenshot generation failed.`);
        process.exit(1);
      }
    })
    .catch((err) => {
      console.error(`[Visual-QA] ERROR: ${err.message}`);
      process.exit(1);
    });
}
