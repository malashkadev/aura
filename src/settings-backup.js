// @ts-check

export const SETTINGS_BACKUP_FORMAT = "aura-settings";
export const SETTINGS_BACKUP_VERSION = 1;

/** @typedef {Record<string, unknown>} SettingsRecord */

/**
 * @param {unknown} value
 * @returns {value is SettingsRecord}
 */
function isRecord(value) {
  return Boolean(value) && typeof value === "object" && !Array.isArray(value);
}

/**
 * @param {string} message
 * @returns {Error & { code: "invalid-backup" }}
 */
function invalidBackup(message) {
  const error = /** @type {Error & { code: "invalid-backup" }} */ (new Error(message));
  error.code = "invalid-backup";
  return error;
}

/**
 * Builds a portable, secret-free settings backup.
 * @param {SettingsRecord} settingsView
 * @param {Date} [exportedAt]
 * @returns {{ format: string, version: number, exported_at: string, settings: SettingsRecord }}
 */
export function buildSettingsBackup(settingsView, exportedAt = new Date()) {
  if (!isRecord(settingsView)) {
    throw invalidBackup("Settings view must be an object");
  }
  /** @type {SettingsRecord} */
  const settings = {};
  for (const [key, value] of Object.entries(settingsView)) {
    if (key.startsWith("api_key") || key.startsWith("has_api_key_")) continue;
    settings[key] = value;
  }
  return {
    format: SETTINGS_BACKUP_FORMAT,
    version: SETTINGS_BACKUP_VERSION,
    exported_at: exportedAt.toISOString(),
    settings,
  };
}

/**
 * Parses a backup file and returns a settings object safe to send to the
 * backend. Accepts both the wrapped export format and a raw Settings object.
 * @param {string} text
 * @param {Iterable<string>} [expectedKeys] Non-secret fields that must be present.
 * @returns {SettingsRecord}
 */
export function parseSettingsBackup(text, expectedKeys = []) {
  let parsed;
  try {
    parsed = JSON.parse(text);
  } catch {
    throw invalidBackup("Settings backup is not valid JSON");
  }
  if (!isRecord(parsed)) {
    throw invalidBackup("Settings backup must be a JSON object");
  }
  if (parsed.format !== undefined && parsed.format !== SETTINGS_BACKUP_FORMAT) {
    throw invalidBackup("Settings backup has an unknown format");
  }
  if (parsed.version !== undefined && parsed.version !== SETTINGS_BACKUP_VERSION) {
    throw invalidBackup("Settings backup has an unsupported version");
  }

  let source = parsed;
  if (Object.prototype.hasOwnProperty.call(parsed, "settings")) {
    if (!isRecord(parsed.settings)) {
      throw invalidBackup("Settings backup payload must be an object");
    }
    source = parsed.settings;
  }

  /** @type {SettingsRecord} */
  const settings = {};
  for (const [key, value] of Object.entries(source)) {
    if (key.startsWith("api_key") || key.startsWith("has_api_key_")) continue;
    settings[key] = value;
  }

  const requiredKeys = Array.from(expectedKeys).filter(
    (key) => !key.startsWith("api_key") && !key.startsWith("has_api_key_")
  );
  const missing = requiredKeys.filter(
    (key) => !Object.prototype.hasOwnProperty.call(settings, key)
  );
  if (missing.length > 0) {
    throw invalidBackup(`Settings backup is missing: ${missing.join(", ")}`);
  }
  return settings;
}

/**
 * @param {Date} [date]
 * @returns {string}
 */
export function settingsBackupFileName(date = new Date()) {
  return `aura-settings-${date.toISOString().slice(0, 10)}.json`;
}
