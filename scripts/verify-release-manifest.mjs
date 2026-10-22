#!/usr/bin/env node
// Verifies that every release surface advertises the same installer identity.
//
// The manifest is the single source of truth. Website, winget and the
// SourceForge kit are hand-edited, so this script fails the build when they
// drift apart. Pass `--file <installer.exe>` to also compare the real bytes.
//
// Usage:
//   node scripts/verify-release-manifest.mjs
//   node scripts/verify-release-manifest.mjs --file src-tauri/target/release/bundle/nsis/Aura_1.2.0_x64-setup.exe

import { createHash } from "node:crypto";
import { existsSync, readFileSync, readdirSync } from "node:fs";
import { resolve } from "node:path";

const root = resolve(import.meta.dirname, "..");
const failures = [];
const rows = [];

function fail(message) {
  failures.push(message);
}

function check(condition, message) {
  if (!condition) {
    fail(message);
  }
}

function read(relativePath) {
  const absolute = resolve(root, relativePath);
  if (!existsSync(absolute)) {
    fail(`${relativePath}: file is missing`);
    return "";
  }
  return readFileSync(absolute, "utf8");
}

function record(surface, value, expected) {
  const ok = String(value).toUpperCase() === String(expected).toUpperCase();
  rows.push({ surface, value, ok });
  return ok;
}

function parseArgs(argv) {
  const index = argv.indexOf("--file");
  return index === -1 ? null : resolve(argv[index + 1] || "");
}

const artifactPath = parseArgs(process.argv.slice(2));

// ---------------------------------------------------------------------------
// Version sources
// ---------------------------------------------------------------------------

const packageJson = JSON.parse(read("package.json") || "{}");
const version = packageJson.version;
check(typeof version === "string" && version.length > 0, "package.json: version is missing");

const tauriConfig = JSON.parse(read("src-tauri/tauri.conf.json") || "{}");
check(
  tauriConfig.version === version,
  `tauri.conf.json: version ${tauriConfig.version} does not match package.json ${version}`,
);

const cargoToml = read("src-tauri/Cargo.toml");
const cargoVersion = cargoToml.match(/^version\s*=\s*"([^"]+)"/m)?.[1];
check(
  cargoVersion === version,
  `Cargo.toml: version ${cargoVersion} does not match package.json ${version}`,
);

// ---------------------------------------------------------------------------
// Manifest
// ---------------------------------------------------------------------------

const manifestPath = `release-manifest-${version}.json`;
const manifest = JSON.parse(read(manifestPath) || "{}");

check(manifest.schemaVersion === 1, `${manifestPath}: unsupported schemaVersion ${manifest.schemaVersion}`);
check(manifest.version === version, `${manifestPath}: version ${manifest.version} does not match ${version}`);
check(typeof manifest.published === "boolean", `${manifestPath}: published must be a boolean`);
check(
  typeof manifest.authenticodeSigned === "boolean",
  `${manifestPath}: authenticodeSigned must be a boolean`,
);
check(
  Array.isArray(manifest.artifacts) && manifest.artifacts.length === 1,
  `${manifestPath}: exactly one artifact must be described`,
);

const artifact = (manifest.artifacts || [])[0] || {};
const expectedName = `Aura_${version}_x64-setup.exe`;
const expectedBytes = Number(artifact.bytes);
const expectedSha = String(artifact.sha256 || "").toUpperCase();

check(artifact.kind === "nsis", `${manifestPath}: artifact.kind must be "nsis"`);
check(artifact.file === expectedName, `${manifestPath}: artifact.file must be ${expectedName}`);
check(
  Number.isInteger(expectedBytes) && expectedBytes > 0,
  `${manifestPath}: artifact.bytes must be a positive integer`,
);
check(/^[0-9A-F]{64}$/.test(expectedSha), `${manifestPath}: artifact.sha256 must be 64 hex characters`);

// ---------------------------------------------------------------------------
// Website
// ---------------------------------------------------------------------------

for (const page of ["website/index.html", "website/index_ru.html"]) {
  const html = read(page);
  const hashes = [...html.matchAll(/download-modal-checksum-code"[^>]*>([0-9A-Fa-f]{64})</g)].map(
    (match) => match[1].toUpperCase(),
  );
  check(hashes.length > 0, `${page}: no SHA-256 checksum block found`);
  for (const hash of hashes) {
    record(`${page} checksum`, hash, expectedSha);
  }
  check(
    html.includes(`href="${expectedName}"`),
    `${page}: download links do not point at ${expectedName}`,
  );
  check(html.includes(`>v${version}<`), `${page}: about section does not show v${version}`);
}

// ---------------------------------------------------------------------------
// winget
// ---------------------------------------------------------------------------

const wingetDir = `winget/manifests/m/Malashka/Aura/${version}`;
let wingetFiles = [];
if (existsSync(resolve(root, wingetDir))) {
  wingetFiles = readdirSync(resolve(root, wingetDir)).filter((name) => name.endsWith(".installer.yaml"));
} else {
  fail(`${wingetDir}: directory is missing`);
}

check(wingetFiles.length === 1, `${wingetDir}: expected exactly one installer manifest`);
for (const file of wingetFiles) {
  const yaml = read(`${wingetDir}/${file}`);
  const packageVersion = yaml.match(/^PackageVersion:\s*(\S+)/m)?.[1];
  const installerUrl = yaml.match(/^\s*InstallerUrl:\s*(\S+)/m)?.[1] || "";
  const installerSha = yaml.match(/^\s*InstallerSha256:\s*([0-9A-Fa-f]{64})/m)?.[1] || "";

  check(packageVersion === version, `${wingetDir}/${file}: PackageVersion ${packageVersion} != ${version}`);
  check(
    installerUrl.endsWith(`/v${version}/${expectedName}`),
    `${wingetDir}/${file}: InstallerUrl must end with /v${version}/${expectedName}`,
  );
  record(`${wingetDir}/${file} InstallerSha256`, installerSha, expectedSha);
}

// ---------------------------------------------------------------------------
// SourceForge publication kit
// ---------------------------------------------------------------------------

const kit = read("docs/SOURCEFORGE_PUBLICATION_KIT.md");
const kitBytes = kit.match(/File Size:\*\*\s*`?([\d\s]+)`?\s*bytes/)?.[1]?.replace(/\s/g, "");
const kitSha = kit.match(/SHA-256 Checksum:\*\*\s*`([0-9A-Fa-f]{64})`/)?.[1] || "";

check(
  Number(kitBytes) === expectedBytes,
  `SOURCEFORGE_PUBLICATION_KIT.md: file size ${kitBytes} does not match ${expectedBytes}`,
);
record("SOURCEFORGE_PUBLICATION_KIT.md checksum", kitSha, expectedSha);
check(
  kit.includes(expectedName),
  `SOURCEFORGE_PUBLICATION_KIT.md: does not mention ${expectedName}`,
);

// ---------------------------------------------------------------------------
// Optional: the real artifact on disk
// ---------------------------------------------------------------------------

if (artifactPath) {
  if (!existsSync(artifactPath)) {
    fail(`artifact: ${artifactPath} does not exist`);
  } else {
    const bytes = readFileSync(artifactPath);
    check(
      bytes.length === expectedBytes,
      `artifact: ${artifactPath} is ${bytes.length} bytes, manifest says ${expectedBytes}`,
    );
    const digest = createHash("sha256").update(bytes).digest("hex").toUpperCase();
    record(`artifact ${artifactPath}`, digest, expectedSha);
  }
}

// ---------------------------------------------------------------------------
// Report
// ---------------------------------------------------------------------------

for (const row of rows) {
  console.log(`${row.ok ? "ok  " : "FAIL"}  ${row.surface}: ${row.value}`);
}

if (failures.length > 0) {
  console.error(`\nRelease manifest verification failed (${failures.length}):`);
  for (const message of failures) {
    console.error(`  - ${message}`);
  }
  process.exit(1);
}

console.log(`\nRelease manifest ${version} is consistent across all surfaces.`);
