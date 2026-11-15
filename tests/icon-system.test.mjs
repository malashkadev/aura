import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";
import vm from "node:vm";

const source = readFileSync(new URL("../src/icon-system.js", import.meta.url), "utf8");
const sandbox = { window: {} };
vm.runInNewContext(source, sandbox);
const icons = sandbox.window.AuraIcons;

const CORE_GLYPHS = [
  "general",
  "settings",
  "mic",
  "doc",
  "keyboard",
  "clock",
  "info",
  "minimize",
  "close",
  "waveform",
  "globe",
  "gamepad",
  "power",
  "cloud",
  "device",
  "eye",
  "search",
  "reset",
  "copy",
  "check",
  "alert",
  "trash",
  "download",
  "upload",
  "external",
  "chevron-down",
  "chevron-right",
  "chevron-left",
  "arrow-right",
  "sparkles",
  "pencil",
  "key",
  "server",
  "filter",
  "volume-2",
  "list",
  "database",
  "terminal",
  "play",
  "star",
];

test("production icon registry exposes the shipped API", () => {
  assert.ok(icons, "window.AuraIcons must exist");
  for (const name of ["defs", "aliases", "categories", "resolve", "body", "svg", "brandSvg", "brandDefs", "hydrate", "names"]) {
    assert.ok(icons[name], `AuraIcons.${name} must be exposed`);
  }
  assert.ok(icons.names.length >= 60, `expected at least 60 glyphs, got ${icons.names.length}`);
});

test("every core glyph resolves and renders at the shipped sizes", () => {
  for (const glyph of CORE_GLYPHS) {
    assert.equal(icons.resolve(glyph), glyph, `${glyph} must resolve to itself`);
    assert.ok(icons.body(glyph).length > 0, `${glyph} must have a body`);
    for (const size of [16, 20, 24]) {
      const svg = icons.svg(glyph, { size });
      assert.match(svg, new RegExp(`width="${size}"\\s+height="${size}"`), `${glyph} at ${size}px`);
      assert.match(svg, /viewBox="0 0 24 24"/);
      assert.match(svg, /stroke="currentColor"/);
      assert.match(svg, /aria-hidden="true"/);
    }
  }
});

test("surface aliases resolve to canonical glyphs", () => {
  const aliases = {
    "nav-general": "general",
    "nav-speech": "mic",
    "nav-about": "info",
    "provider-custom": "server",
    "filter-all": "filter",
    "section-voice-commands": "command-bubble",
    "section-edit-engine": "pencil",
    "engine-whisper": "provider-openai",
    "model-qwen": "qwen",
  };
  for (const [alias, canonical] of Object.entries(aliases)) {
    assert.equal(icons.resolve(alias), canonical, `${alias} -> ${canonical}`);
  }
});

test("unknown and empty names fail closed", () => {
  for (const name of ["", " ", "not-a-real-icon", null, undefined]) {
    assert.equal(icons.resolve(name), null);
    assert.equal(icons.body(name), "");
    assert.equal(icons.svg(name), "");
  }
});

test("svg rendering preserves className, title, and stroke width", () => {
  const svg = icons.svg("mic", {
    size: 18,
    strokeWidth: 1.75,
    className: "test-icon",
    title: "Microphone",
  });
  assert.match(svg, /class="test-icon"/);
  assert.match(svg, /width="18"\s+height="18"/);
  assert.match(svg, /stroke-width="1\.75"/);
  assert.match(svg, /<title>Microphone<\/title>/);
  assert.doesNotMatch(svg, /undefined|null/);
});

test("every definition is non-empty and stays on the achromatic palette", () => {
  const banned = ["#030305", "#0a0a0c", "#0e0e12", "#111114", "#131317", "#121214"];
  for (const [name, body] of Object.entries(icons.defs)) {
    assert.ok(body.trim().length > 0, `${name} must not be empty`);
    assert.doesNotMatch(body, /https?:\/\//, `${name} must not embed a remote URL`);
    for (const color of banned) {
      assert.equal(body.toLowerCase().includes(color), false, `${name} uses banned ${color}`);
    }
  }
});

test("every alias points at a defined glyph", () => {
  for (const [alias, target] of Object.entries(icons.aliases)) {
    assert.ok(icons.defs[target], `${alias} points at missing glyph ${target}`);
    assert.equal(icons.resolve(alias), target);
  }
});

test("categories reference real glyphs", () => {
  assert.ok(Object.keys(icons.categories).length > 0);
  for (const name of Object.keys(icons.categories)) {
    assert.ok(icons.defs[name], `category references missing glyph ${name}`);
  }
});

test("official brand marks keep their identity and unique ids", () => {
  const gemini = icons.brandSvg("provider-gemini", { size: 16 });
  assert.match(gemini, /<mask/);
  assert.match(gemini, /feGaussianBlur/);
  assert.match(gemini, /#FFE432/);
  assert.doesNotMatch(gemini, /__ID__/);

  const openAi = icons.brandSvg("provider-openai", { size: 16 });
  assert.match(openAi, /#FFFFFF/);
  assert.doesNotMatch(openAi, /currentColor/);

  const again = icons.brandSvg("provider-gemini", { size: 16 });
  const firstId = gemini.match(/id="([^"]+)"/)?.[1];
  const secondId = again.match(/id="([^"]+)"/)?.[1];
  assert.ok(firstId && secondId && firstId !== secondId, "brand ids must be unique per render");
});

test("hydrate is wired for declarative icon slots", () => {
  assert.equal(typeof icons.hydrate, "function");
});
