import assert from "node:assert/strict";
import { existsSync, readFileSync, readdirSync, statSync } from "node:fs";
import test from "node:test";
import vm from "node:vm";

const read = (path) => readFileSync(new URL(`../${path}`, import.meta.url), "utf8");
const json = (path) => JSON.parse(read(path));
const parseI18nDictionaries = () => {
  const normalized = read("src/main.js").replaceAll("\r\n", "\n");
  const marker = "const i18nDict =";
  const start = normalized.indexOf(marker) + marker.length;
  const end = normalized.indexOf("\n};\n\nlet currentLanguage", start) + 2;
  assert.ok(start >= marker.length && end > 1, "i18n dictionary bounds must be discoverable");
  return vm.runInNewContext("(" + normalized.slice(start, end) + ")");
};
const parseOverlayNotices = () => {
  const normalized = read("src/overlay.js").replaceAll("\r\n", "\n");
  const marker = "const noticeTranslations =";
  const start = normalized.indexOf(marker) + marker.length;
  const end = normalized.indexOf("\n};\n\nconst errorTranslations", start) + 2;
  assert.ok(start >= marker.length && end > 1, "overlay notice bounds must be discoverable");
  return vm.runInNewContext("(" + normalized.slice(start, end) + ")");
};


test("Tauri capabilities isolate main and overlay with least privilege", () => {
  assert.equal(existsSync(new URL("../src-tauri/capabilities/default.json", import.meta.url)), false);

  const main = json("src-tauri/capabilities/main.json");
  const overlay = json("src-tauri/capabilities/overlay.json");

  assert.deepEqual(main.windows, ["main"]);
  assert.deepEqual(overlay.windows, ["overlay"]);
  for (const capability of [main, overlay]) {
    assert.equal(capability.permissions.includes("core:default"), false);
    assert.equal(capability.permissions.includes("opener:default"), false);
    assert.equal(capability.permissions.includes("updater:default"), false);
  }

  assert.ok(main.permissions.includes("allow-get-settings"));
  assert.ok(main.permissions.includes("allow-set-provider-key"));
  assert.ok(main.permissions.includes("allow-check-for-app-update"));
  assert.ok(main.permissions.includes("allow-install-app-update"));
  assert.equal(main.permissions.some((permission) => permission.startsWith("updater:")), false);
  assert.deepEqual(
    overlay.permissions.toSorted(),
    ["allow-hide-overlay-window", "core:event:allow-listen", "core:event:allow-unlisten"].toSorted(),
  );

  const build = read("src-tauri/build.rs");
  assert.match(build, /AppManifest::new\(\)\.commands/);
  assert.match(build, /"set_provider_key"/);
  assert.match(build, /"hide_overlay_window"/);
});

test("offline frontend neither loads Google Fonts nor performs an unconditional update request", () => {
  const html = read("src/index.html");
  const config = read("src-tauri/tauri.conf.json");
  const main = read("src/main.js");

  assert.doesNotMatch(html, /fonts\.googleapis|fonts\.gstatic/i);
  assert.doesNotMatch(config, /fonts\.googleapis|fonts\.gstatic/i);
  assert.match(config, /base-uri 'none'/);
  assert.match(html, /id="checkbox-automatic-update-checks"/);
  assert.match(html, /id="btn-check-updates"/);
  assert.match(main, /automatic_update_checks/);
  assert.match(main, /if \(settings\?\.automatic_update_checks\)/);
  assert.doesNotMatch(main, /invoke\("check_for_update"\)/);
});

test("frontend uses redacted key metadata and write-only provider-key IPC", () => {
  const html = read("src/index.html");
  const main = read("src/main.js");
  const settingsLiteral = main.match(/const settings = \{([\s\S]*?)\n\s*\};/u)?.[1] ?? "";

  assert.match(main, /has_api_key_gemini/);
  assert.match(main, /invoke\("set_provider_key"/);
  assert.doesNotMatch(settingsLiteral, /api_key(?:_gemini|_openai|_groq)?\s*:/);
  assert.match(html, /id="checkbox-selection-edit-enabled"/);
  assert.doesNotMatch(html, /cloud_data_desc/);
  assert.doesNotMatch(html, /value="parakeet"[^>]*disabled/);
});

test("settings translations cover every supported locale without fallback gaps", () => {
  const expectedLocales = ["ru", "en", "de", "es", "fr", "it", "zh", "pt", "tr"];
  const dictionaries = parseI18nDictionaries();
  const referenceKeys = Object.keys(dictionaries.ru).toSorted();
  assert.deepEqual(Object.keys(dictionaries), expectedLocales);

  for (const locale of expectedLocales) {
    assert.deepEqual(
      Object.keys(dictionaries[locale]).toSorted(),
      referenceKeys,
      locale + " must define the same translation keys as Russian",
    );
  }

  const html = read("src/index.html");
  const renderedKeys = [...html.matchAll(/data-i18n="([^"]+)"/g)].map((match) => match[1]);
  for (const locale of expectedLocales) {
    for (const key of renderedKeys) {
      assert.equal(typeof dictionaries[locale][key], "string", locale + "." + key + " is missing");
      assert.notEqual(dictionaries[locale][key].trim(), "", locale + "." + key + " is empty");
    }
  }

  const newlyLocalizedSettings = [
    "update_checks_title", "update_checks_desc", "update_checks_checkbox", "update_check_now",
    "local_model_desc", "fallback_title",
    "copy_context_title", "copy_context_desc", "copy_context_checkbox",
    "gpu_accel_label", "gpu_accel_cpu_title", "gpu_accel_cpu_desc",
    "gpu_accel_cuda_title", "gpu_accel_cuda_desc", "gpu_accel_dml_title", "gpu_accel_dml_desc",
    "update_current", "update_available_pattern", "update_check_error_pattern",
    "update_installing", "update_installed_restarting", "update_install_error_open_release",
  ];
  for (const locale of expectedLocales.slice(1)) {
    for (const key of newlyLocalizedSettings) {
      assert.notEqual(
        dictionaries[locale][key],
        dictionaries.ru[key],
        locale + "." + key + " must not silently fall back to Russian",
      );
    }
  }


  const generalPanel = html.match(/id="panel-general"[\s\S]*?id="panel-speech"/)?.[0] ?? "";
  const aboutPanel = html.match(/id="panel-about"[\s\S]*?<\/section>/)?.[0] ?? "";
  assert.doesNotMatch(generalPanel, /cloud_data_desc|Какие данные использует облачный режим/);
  assert.doesNotMatch(aboutPanel, /cloud_data_desc/);
  assert.doesNotMatch(read("src/main.js") + html, /\uFFFD/);
  assert.match(read("src/main.js"), /getTranslation\("update_current"\)/);
  assert.match(read("src/main.js"), /getTranslation\("update_installing"\)/);
});

test("interface language persists in native settings and defaults to a supported system locale", () => {
  const main = read("src/main.js");
  const rust = read("src-tauri/src/lib.rs");
  const settings = read("src-tauri/src/settings_secure.rs");

  assert.match(main, /navigator\.language\.toLowerCase\(\)\.split\(\/\[-_\]\/\)\[0\]/);
  assert.match(main, /supportedLangs\.includes\(browserUiLang\) \? browserUiLang : "en"/);
  assert.match(main, /invoke\("set_ui_language", \{ uiLanguage: selectedLang \}\)/);
  assert.match(rust, /fn tray_translations\(language: &str\)/);
  assert.match(rust, /show_sync\.set_text\(text\.show\)/);
  assert.match(settings, /GetUserDefaultUILanguage/);
});

test("safe clipboard handoff notice is translated in every overlay locale", () => {
  const notices = parseOverlayNotices();
  const translated = notices["final-copied-after-edit"];
  const expectedLocales = ["ru", "en", "de", "fr", "it", "es", "pt", "zh", "ja", "tr"];

  assert.deepEqual(Object.keys(translated), expectedLocales);
  for (const locale of expectedLocales) {
    assert.equal(typeof translated[locale], "string");
    assert.notEqual(translated[locale].trim(), "");
  }
});

test("settings UI exposes native accessible controls and responsive motion-safe behavior", () => {
  const html = read("src/index.html");
  const main = read("src/main.js");
  const css = read("src/style.css");

  assert.match(html, /role="tablist"/);
  assert.match(html, /role="tab"/);
  assert.match(html, /role="tabpanel"/);
  assert.match(html, /<dialog[^>]+id="custom-confirm-modal"/);
  assert.match(html, /aria-live="polite"/);
  assert.match(html, /id="input-history-search"/);
  assert.match(html, /id="btn-clear-history-search"/);
  assert.match(html, /class="history-filter-btn"/);
  assert.match(main, /inputHistorySearch/);
  assert.match(main, /btnClearHistorySearch/);
  assert.match(main, /historyFilterButtons/);
  assert.match(css, /\.select-panel-item\.is-focused/);
  assert.doesNotMatch(main, /select\.style\.display\s*=\s*"none"/);
  assert.doesNotMatch(main, /initCustomSelects/);
  assert.match(main, /document\.documentElement\.lang/);
  assert.match(main, /\.showModal\(\)/);
  assert.match(main, /addEventListener\("cancel"/);
  assert.match(css, /prefers-reduced-motion:\s*reduce/);
  assert.match(css, /@media\s*\(max-width:\s*680px\)/);
  assert.equal(existsSync(new URL("../src/styles.css", import.meta.url)), false);
  assert.equal(existsSync(new URL("../src/ui-accessibility.js", import.meta.url)), true);
});

test("hotkey recorder supports Windows and modifier-only combinations", () => {
  const main = read("src/main.js");

  assert.match(main, /"MetaLeft": "Win"/);
  assert.match(main, /"OSRight": "Win"/);
  assert.match(main, /if \(e\.metaKey\) recordedHotkeyModifiers\.add\("Win"\)/);
  assert.match(main, /selectHotkey\.addEventListener\("keyup"/);
  assert.match(main, /recordedHotkeyModifiers\.size >= 2/);
  assert.match(main, /orderedHotkeyModifiers\(\)\.join\("\+"\)/);
});

test("package scripts provide reproducible lint, type, test and build gates", () => {
  const pkg = json("package.json");
  const lock = json("package-lock.json");

  for (const script of ["lint", "typecheck", "check", "test", "build"]) {
    assert.equal(typeof pkg.scripts[script], "string");
    assert.doesNotMatch(pkg.scripts[script], /^echo\b/);
  }
  assert.equal(pkg.devDependencies.sharp, undefined);
  assert.doesNotMatch(pkg.devDependencies["@tauri-apps/cli"], /^[~^]/);
  assert.equal(typeof pkg.devDependencies.typescript, "string");
  assert.equal(lock.packages[""].version, pkg.version);
  assert.equal(lock.packages[""].license, pkg.license);
});

test("release manifests and localized UI expose one authoritative version", () => {
  const pkg = json("package.json");
  const packageLock = json("package-lock.json");
  const tauri = json("src-tauri/tauri.conf.json");
  const cargoManifest = read("src-tauri/Cargo.toml");
  const cargoLock = read("src-tauri/Cargo.lock");
  const dictionaries = parseI18nDictionaries();

  const cargoVersion = cargoManifest.match(/\[package\][\s\S]*?\nversion\s*=\s*"([^"]+)"/)?.[1];
  const lockedCargoVersion = cargoLock.match(
    /\[\[package\]\]\r?\nname\s*=\s*"aura-app"\r?\nversion\s*=\s*"([^"]+)"/,
  )?.[1];

  assert.equal(tauri.version, pkg.version);
  assert.equal(packageLock.version, pkg.version);
  assert.equal(packageLock.packages[""].version, pkg.version);
  assert.equal(cargoVersion, pkg.version);
  assert.equal(lockedCargoVersion, pkg.version);

  for (const [locale, dictionary] of Object.entries(dictionaries)) {
    assert.equal(dictionary.about_version, `v${pkg.version}`, `${locale} UI version must match`);
  }
  assert.match(read("src/index.html"), new RegExp(`>v${pkg.version.replaceAll(".", "\\.")}<`));
});

test("CI and release workflows pin actions and enforce frontend and dependency gates", () => {
  for (const path of [".github/workflows/ci.yml", ".github/workflows/release.yml"]) {
    const workflow = read(path);
    const refs = [...workflow.matchAll(/uses:\s*[^@\s]+@([^\s#]+)/g)].map((match) => match[1]);
    assert.ok(refs.length > 0);
    refs.forEach((ref) => assert.match(ref, /^[a-f0-9]{40}$/));
    assert.match(workflow, /npm ci/);
    assert.match(workflow, /npm run check/);
    assert.match(workflow, /npm test/);
    assert.match(workflow, /npm audit/);
    assert.match(workflow, /cargo fmt/);
    assert.match(workflow, /cargo clippy/);
    assert.match(workflow, /cargo audit/);
    assert.match(workflow, /osv-scanner/i);
  }
  assert.match(read(".github/workflows/release.yml"), /needs:\s*verify/);
});

test("website describes provider data flow and unsigned installer honestly", () => {
  const en = read("website/index.html");
  const ru = read("website/index_ru.html");

  assert.match(en, /cloud mode[^.]*audio[^.]*transcript[^.]*selected text[^.]*dictionary/is);
  assert.match(en, /optional update check/is);
  assert.match(en, /not digitally signed|not Authenticode-signed/i);

  assert.match(ru, /облачн[^.]*аудио[^.]*транскрипт[^.]*выделенн[^.]*словар/is);
  assert.match(ru, /необязательн[^.]*провер[^.]*обновлен/is);
  assert.match(ru, /не подписан[^.]*Authenticode|нет подписи Authenticode/i);

  const linkedInstallers = [
    ...new Set([...`${en}\n${ru}`.matchAll(/href="(Aura_[\w.]+_x64-setup\.exe)"/g)].map((m) => m[1])),
  ];
  assert.equal(linkedInstallers.length, 1, "both locales must link exactly one installer file");
  const installerFile = linkedInstallers[0];

  const manifestNames = readdirSync(new URL("../", import.meta.url)).filter((name) =>
    /^release-manifest-.+\.json$/.test(name),
  );
  assert.ok(manifestNames.length > 0, "at least one release manifest must exist");
  let publishedArtifact;
  for (const name of manifestNames) {
    const manifest = json(name);
    assert.equal(typeof manifest.version, "string", `${name} must declare its version`);
    assert.equal(manifest.published, false, `${name} must not claim publication from the repo`);
    assert.equal(manifest.authenticodeSigned, false, `${name} must not claim Authenticode`);
    for (const artifact of manifest.artifacts ?? []) {
      if (artifact.kind !== "nsis") continue;
      assert.equal(artifact.file, `Aura_${manifest.version}_x64-setup.exe`, `${name} filename/version mismatch`);
      assert.match(artifact.sha256, /^[A-F0-9]{64}$/, `${name} sha256 must be uppercase hex`);
      assert.ok(artifact.bytes > 0, `${name} bytes must be positive`);
      if (artifact.file === installerFile) publishedArtifact = artifact;
    }
  }

  assert.ok(publishedArtifact, "the linked installer must be described by a real manifest entry");
  assert.ok(en.includes(publishedArtifact.sha256), "EN integrity block must publish the real hash");
  assert.ok(ru.includes(publishedArtifact.sha256), "RU integrity block must publish the real hash");

  const hostedInstaller = new URL(`../website/${installerFile}`, import.meta.url);
  if (existsSync(hostedInstaller)) {
    assert.equal(statSync(hostedInstaller).size, publishedArtifact.bytes, "hosted installer size must match manifest");
  }
});
