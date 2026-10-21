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


test("Tauri capabilities isolate main, overlay, and tray-menu with least privilege", () => {
  assert.equal(existsSync(new URL("../src-tauri/capabilities/default.json", import.meta.url)), false);

  const main = json("src-tauri/capabilities/main.json");
  const overlay = json("src-tauri/capabilities/overlay.json");
  const trayMenu = json("src-tauri/capabilities/tray-menu.json");

  assert.deepEqual(main.windows, ["main"]);
  assert.deepEqual(overlay.windows, ["overlay"]);
  assert.deepEqual(trayMenu.windows, ["tray-menu"]);
  for (const capability of [main, overlay, trayMenu]) {
    assert.equal(capability.permissions.includes("core:default"), false);
    assert.equal(capability.permissions.includes("opener:default"), false);
    assert.equal(capability.permissions.includes("updater:default"), false);
  }

  assert.ok(main.permissions.includes("allow-get-settings"));
  assert.ok(main.permissions.includes("allow-set-provider-key"));
  assert.ok(main.permissions.includes("allow-check-for-app-update"));
  assert.ok(main.permissions.includes("allow-install-app-update"));
  assert.ok(main.permissions.includes("allow-complete-onboarding"));
  assert.ok(main.permissions.includes("allow-exit-app"));
  assert.equal(main.permissions.some((permission) => permission.startsWith("updater:")), false);
  assert.deepEqual(
    overlay.permissions.toSorted(),
    ["allow-hide-overlay-window", "core:event:allow-listen", "core:event:allow-unlisten"].toSorted(),
  );
  assert.deepEqual(
    trayMenu.permissions.toSorted(),
    [
      "allow-close-window",
      "allow-exit-app",
      "allow-get-gaming-mode-state",
      "allow-get-settings",
      "allow-set-settings",
      "allow-show-settings-window",
      "allow-toggle-gaming-mode",
      "core:event:allow-listen",
      "core:event:allow-unlisten",
    ].toSorted(),
  );

  const build = read("src-tauri/build.rs");
  assert.match(build, /AppManifest::new\(\)\.commands/);
  assert.match(build, /"set_provider_key"/);
  assert.match(build, /"hide_overlay_window"/);
  assert.match(build, /"show_settings_window"/);
});

test("tray menu geometry and CSS prevent letterboxing and black bars", () => {
  const config = json("src-tauri/tauri.conf.json");
  const trayWin = config.app.windows.find((w) => w.label === "tray-menu");
  assert.ok(trayWin, "tray-menu window must be declared in tauri.conf.json");
  assert.equal(trayWin.width, 270);
  assert.equal(trayWin.height, 196);
  assert.equal(trayWin.transparent, true);
  assert.equal(trayWin.decorations, false);
  assert.equal(trayWin.shadow, false);

  const rustLib = read("src-tauri/src/lib.rs");
  assert.match(rustLib, /let logical_w = 270\.0;/);
  assert.match(rustLib, /let logical_h = 196\.0;/);

  const css = read("src/tray-menu.css");
  assert.match(css, /html,\s*body\s*\{[^}]*align-items:\s*stretch;/);
  assert.match(css, /\.aura-tray-menu\s*\{[^}]*width:\s*100%;/);
  assert.match(css, /\.aura-tray-menu\s*\{[^}]*height:\s*100%;/);
  assert.match(css, /\.lang-list-container\s*\{[^}]*max-height:\s*142px;/);
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

test("CUDA setup requires NVIDIA terms only when the system runtime is missing", () => {
  const html = read("src/index.html");
  const main = read("src/main.js");

  assert.match(html, /id="confirm-modal-links"/);
  assert.match(main, /nvidiaRuntimeOnPath = await invoke\("check_nvidia_runtime_on_path"\)/);
  assert.match(main, /if \(!nvidiaRuntimeOnPath\) \{/);
  assert.match(main, /acceptedNvidiaTerms = await showConfirm/);
  assert.match(main, /Compatible NVIDIA components already installed on this PC are reused/);
  assert.match(main, /if \(!acceptedNvidiaTerms\) return/);
  assert.match(main, /invoke\("download_gpu_binaries", \{ provider, acceptedNvidiaTerms \}\)/);
  assert.match(main, /https:\/\/docs\.nvidia\.com\/cuda\/archive\/11\.8\.0\/eula\/index\.html/);
  assert.match(
    main,
    /https:\/\/docs\.nvidia\.com\/deeplearning\/cudnn\/archives\/cudnn-850\/sla\/index\.html/,
  );
});

test("GPU installation progress uses a compact label", () => {
  const main = read("src/main.js");

  assert.match(main, /"gpu_status_installing": "Installing\.\.\."/);
  assert.doesNotMatch(main, /Checking and installing/);
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

test("every non-secret native setting is round-tripped by the settings form", () => {
  const rust = read("src-tauri/src/settings_secure.rs");
  const main = read("src/main.js");

  const structBlock = rust.match(/pub struct Settings \{([\s\S]*?)\n\}/)?.[1] ?? "";
  assert.ok(structBlock, "Settings struct must be parseable");
  const backendFields = [...structBlock.matchAll(/pub\s+([a-z0-9_]+)\s*:/g)]
    .map((match) => match[1])
    .filter((name) => !name.startsWith("api_key"));
  assert.ok(backendFields.length > 20, "Settings struct must expose its fields");

  const settingsLiteral = main.match(/const settings = \{([\s\S]*?)\n\s*\};/u)?.[1] ?? "";
  assert.ok(settingsLiteral, "saveSettings payload must be parseable");
  const frontendFields = [...settingsLiteral.matchAll(/^\s*([a-z][a-z0-9_]*):/gm)]
    .map((match) => match[1]);

  assert.equal(new Set(frontendFields).size, frontendFields.length, "payload keys must be unique");
  assert.deepEqual(
    frontendFields.toSorted(),
    backendFields.toSorted(),
    "the settings form must send every non-secret Settings field and no stale fields",
  );
});

test("settings backup exports no secrets and validates imported files", async () => {
  const {
    buildSettingsBackup,
    parseSettingsBackup,
    settingsBackupFileName,
    SETTINGS_BACKUP_FORMAT,
  } = await import("../src/settings-backup.js");

  const exportedAt = new Date("2026-10-22T12:34:56.000Z");
  const backup = buildSettingsBackup({
    transcription_mode: "local",
    dictionary: "Aura",
    has_api_key_gemini: true,
    api_key_gemini: "super-secret",
  }, exportedAt);

  assert.equal(backup.format, SETTINGS_BACKUP_FORMAT);
  assert.equal(backup.exported_at, exportedAt.toISOString());
  assert.deepEqual(backup.settings, {
    transcription_mode: "local",
    dictionary: "Aura",
  });
  assert.equal(settingsBackupFileName(exportedAt), "aura-settings-2026-10-22.json");

  assert.deepEqual(
    parseSettingsBackup(JSON.stringify(backup), ["transcription_mode", "dictionary"]),
    { transcription_mode: "local", dictionary: "Aura" },
  );
  assert.deepEqual(
    parseSettingsBackup(JSON.stringify({ settings: { dictionary: "Aura" } }), ["dictionary"]),
    { dictionary: "Aura" },
  );

  assert.throws(
    () => parseSettingsBackup("{", ["dictionary"]),
    (error) => error.code === "invalid-backup",
  );
  assert.throws(
    () => parseSettingsBackup(JSON.stringify({ format: "other" }), []),
    (error) => error.code === "invalid-backup",
  );
  assert.throws(
    () => parseSettingsBackup(JSON.stringify({ settings: [] }), []),
    (error) => error.code === "invalid-backup",
  );
  assert.throws(
    () => parseSettingsBackup(JSON.stringify({ settings: { transcription_mode: "local" } }), [
      "transcription_mode",
      "dictionary",
    ]),
    (error) => error.code === "invalid-backup",
  );
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
    "gpu_accel_cuda_title", "gpu_accel_cuda_desc",
    "update_current", "update_available_pattern", "update_check_error_pattern",
    "update_installing", "update_installed_restarting", "update_install_error_open_release",
    "window_minimize", "window_close", "history_copy_title", "history_search_clear",
    "api_key_toggle_visibility", "history_filters_label",
    "smart_context_title", "smart_context_desc", "smart_context_checkbox", "smart_context_subdesc",
    "smart_context_trailing_period_checkbox", "smart_context_trailing_period_subdesc",
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
  const tray = read("src/tray-menu.js");
  const settings = read("src-tauri/src/settings_secure.rs");

  assert.match(main, /navigator\.language\.toLowerCase\(\)\.split\(\/\[-_\]\/\)\[0\]/);
  assert.match(main, /supportedLangs\.includes\(browserUiLang\) \? browserUiLang : "en"/);
  assert.match(main, /invoke\("set_ui_language", \{ uiLanguage: selectedLang \}\)/);
  assert.match(tray, /const I18N =/);
  assert.match(tray, /settings: "Открыть настройки"/);
  assert.match(settings, /GetUserDefaultUILanguage/);
});

test("safe clipboard handoff notice is translated in every overlay locale", () => {
  const notices = parseOverlayNotices();
  const translated = notices["final-copied-after-edit"];
  const expectedLocales = ["ru", "en", "de", "fr", "it", "es", "pt", "zh", "ja", "tr", "nl"];

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
  assert.match(html, /id="commands-info-guide"/);
  assert.match(html, /id="spotlight-tour-overlay"/);
  assert.match(main, /initSpotlightTour/);
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

test("tab keyboard navigation advances through tabs on arrow keys and wraps correctly", async () => {
  const { bindTabKeyboardNavigation } = await import("../src/ui-accessibility.js");

  class MockTab {
    constructor(id, visible = true) {
      this.id = id;
      this.offsetParent = visible ? {} : null;
      this.listeners = {};
      this.focused = false;
    }
    addEventListener(event, fn) {
      this.listeners[event] = fn;
    }
    focus() {
      this.focused = true;
    }
    dispatchKey(key) {
      let defaultPrevented = false;
      const event = {
        key,
        preventDefault: () => {
          defaultPrevented = true;
        },
      };
      if (this.listeners.keydown) {
        this.listeners.keydown(event);
      }
      return defaultPrevented;
    }
  }

  const tab0 = new MockTab("tab0", true);
  const tab1 = new MockTab("tab1", true);
  const tab2 = new MockTab("tab2", false); // hidden
  const tab3 = new MockTab("tab3", true);
  const tabs = [tab0, tab1, tab2, tab3];

  let activatedTab = null;
  bindTabKeyboardNavigation(tabs, (tab) => {
    activatedTab = tab;
  });

  // ArrowRight from tab0 must advance to tab1 (not stay on tab0)
  assert.ok(tab0.dispatchKey("ArrowRight"));
  assert.equal(activatedTab, tab1);
  assert.equal(tab1.focused, true);

  // ArrowRight from tab1 must skip hidden tab2 and activate tab3
  activatedTab = null;
  assert.ok(tab1.dispatchKey("ArrowRight"));
  assert.equal(activatedTab, tab3);

  // ArrowRight from tab3 wraps back to tab0
  activatedTab = null;
  assert.ok(tab3.dispatchKey("ArrowRight"));
  assert.equal(activatedTab, tab0);

  // ArrowLeft from tab0 wraps back to tab3 (skipping hidden tab2)
  activatedTab = null;
  assert.ok(tab0.dispatchKey("ArrowLeft"));
  assert.equal(activatedTab, tab3);

  // Home key activates tab0
  activatedTab = null;
  assert.ok(tab3.dispatchKey("Home"));
  assert.equal(activatedTab, tab0);

  // End key activates tab3
  activatedTab = null;
  assert.ok(tab0.dispatchKey("End"));
  assert.equal(activatedTab, tab3);
});

test("model card actions render visible download icons and delete buttons with defined dimensions", () => {
  const main = read("src/main.js");
  const css = read("src/style.css");

  // CSS must explicitly define .btn-icon and button SVG sizing so flex containers never collapse icons to 0x0
  assert.match(css, /\.btn-icon\s*\{[^}]*width:\s*14px;[^}]*height:\s*14px;/);
  assert.match(css, /\.btn-delete-card-model\s*\{[^}]*width:\s*28px;[^}]*height:\s*28px;/);
  assert.match(css, /\.btn-download-card-model\s*\{[^}]*display:\s*inline-flex;/);

  // main.js must include explicit SVG dimensions and aria-labels for download and delete buttons
  assert.match(main, /class="btn-delete-card-model"[^>]*title="[^"]+"[^>]*aria-label="[^"]+"/);
  assert.match(main, /<svg class="btn-icon" width="14" height="14"/);
  assert.match(main, /actionEl\.querySelector\("\.btn-delete-card-model"\)\.addEventListener\("click",\s*\(e\)\s*=>/);
});

test("typography reset and button font consistency across application UI", () => {
  const css = read("src/style.css");
  const trayCss = read("src/tray-menu.css");
  const overlayHtml = read("src/overlay.html");

  // Form controls must inherit font family to avoid browser default Arial/MS Sans Serif
  assert.match(
    css,
    /button,\s*input,\s*select,\s*textarea\s*\{[^}]*font-family:\s*inherit;/
  );
  assert.match(trayCss, /button\s*\{[^}]*font-family:\s*inherit;/);

  // Model card download button must explicitly use var(--font-display) with semi-bold weight
  assert.match(
    css,
    /\.btn-download-card-model\s*\{[^}]*font-family:\s*var\(--font-display\);[^}]*font-weight:\s*600;/
  );

  // Secondary buttons, star button, gaming button must use var(--font-display)
  assert.match(
    css,
    /\.btn-secondary\s*\{[^}]*font-family:\s*var\(--font-display\);[^}]*font-weight:\s*600;/
  );
  assert.match(
    css,
    /\.btn-github-star\s*\{[^}]*font-family:\s*var\(--font-display\);[^}]*font-weight:\s*600;/
  );

  // Overlay status must use Segoe UI Variable font stack
  assert.match(overlayHtml, /#overlay-status\s*\{[^}]*font-family:\s*"Segoe UI Variable Text"/);
});

test("smart context insertion settings controls are declared and wired in main.js and index.html", () => {
  const html = read("src/index.html");
  const main = read("src/main.js");

  assert.match(html, /id="card-smart-context"/);
  assert.match(html, /id="checkbox-smart-context-enabled"/);
  assert.match(html, /id="smart-context-suboptions"/);
  assert.match(html, /id="checkbox-smart-context-trailing-period"/);

  assert.match(main, /checkboxSmartContext\.checked\s*=\s*settings\.smart_context_enabled !== false/);
  assert.match(main, /checkboxSmartContextTrailingPeriod\.checked\s*=\s*settings\.smart_context_trailing_period !== false/);
  assert.match(main, /smart_context_enabled:\s*\(\(\)\s*=>/);
  assert.match(main, /smart_context_trailing_period:\s*\(\(\)\s*=>/);
});

test("desktop app maintains strict design-system isolation from website palette and enforces user-select rules", () => {
  const styleCss = read("src/style.css");
  const trayCss = read("src/tray-menu.css");
  const html = read("src/index.html");

  // Banned website colors (blue/indigo/velvet tinted dark tones)
  const bannedWebsiteColors = [
    "#030305", "#0a0a0c", "#0e0e12", "#111114", "#131317", "#121214", "rgba(20, 20, 26",
  ];
  for (const banned of bannedWebsiteColors) {
    assert.equal(
      styleCss.toLowerCase().includes(banned.toLowerCase()),
      false,
      `src/style.css must not leak website color ${banned}`,
    );
    assert.equal(
      trayCss.toLowerCase().includes(banned.toLowerCase()),
      false,
      `src/tray-menu.css must not leak website color ${banned}`,
    );
  }

  // Achromatic modal backgrounds
  assert.match(styleCss, /\.modal-card\s*\{[^}]*background-color:\s*#0f0f0f;/);
  assert.match(styleCss, /\.spotlight-popover\s*\{[^}]*background-color:\s*#0f0f0f;/);

  // Global user-select reset
  assert.match(styleCss, /\*,\s*\*::before,\s*\*::after\s*\{[^}]*user-select:\s*none;/);

  // Commands guide button or group is placed near the top of the selection edit section (before select-ai-edit-mode)
  const commandsGuideIdx = Math.max(
    html.indexOf('id="btn-open-commands-guide"'),
    html.indexOf('id="group-supported-commands"'),
  );
  const aiEditModeIdx = html.indexOf('id="select-ai-edit-mode"');
  assert.ok(commandsGuideIdx > 0 && aiEditModeIdx > 0, "Both controls must exist in index.html");
  assert.ok(
    commandsGuideIdx < aiEditModeIdx,
    "Voice commands guide entry must appear above AI edit mode selector for immediate discoverability",
  );
});

