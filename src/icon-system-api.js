// @ts-check

/**
 * Thin ES-module facade over the classic icon registry in `icon-system.js`.
 *
 * `icon-system.js` is loaded synchronously before this module (see index.html and
 * tray-menu.html) and exposes `window.AuraIcons`. Keeping the registry classic
 * lets the static HTML hydrate without a build step, while this facade gives
 * the application modules a normal `getIcon()` import.
 */

/**
 * @typedef {Object} AuraIconOptions
 * @property {number} [size]
 * @property {number} [strokeWidth]
 * @property {string} [className]
 * @property {string} [title]
 * @property {boolean} [accent]
 */

/**
 * @typedef {Object} AuraIconRegistry
 * @property {(name: string, options?: AuraIconOptions) => string} body
 * @property {(name: string, options?: AuraIconOptions) => string} svg
 * @property {(name: string, options?: AuraIconOptions) => string} [brandSvg]
 * @property {(root?: ParentNode) => void} hydrate
 * @property {(name: string) => string} [resolve]
 */

const globalWithIcons = /** @type {Window & typeof globalThis & { AuraIcons?: AuraIconRegistry }} */ (window);

/** @type {AuraIconRegistry} */
const icons = globalWithIcons.AuraIcons || {
  body: () => "",
  svg: () => "",
  hydrate: () => {},
};

/**
 * @param {string} name
 * @param {AuraIconOptions} [options]
 * @returns {string}
 */
export function getIcon(name, options = {}) {
  try {
    return icons.svg(name, options);
  } catch (err) {
    console.error("Icon render failed", name, err);
    return "";
  }
}

/**
 * @param {string} name
 * @param {AuraIconOptions} [options]
 * @returns {string}
 */
export function getBrandIcon(name, options = {}) {
  try {
    return typeof icons.brandSvg === "function" ? icons.brandSvg(name, options) : "";
  } catch (err) {
    console.error("Brand icon render failed", name, err);
    return "";
  }
}

/**
 * @param {string} name
 * @param {AuraIconOptions} [options]
 * @returns {string}
 */
export function iconBody(name, options) {
  return icons.body(name, options);
}

/**
 * @param {ParentNode} [root]
 * @returns {void}
 */
export function hydrateIcons(root = document) {
  icons.hydrate(root);
}

export { icons };
export default icons;
