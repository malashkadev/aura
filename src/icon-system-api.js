/**
 * Thin ES-module facade over the classic icon registry in `icon-system.js`.
 *
 * `icon-system.js` is loaded synchronously before this module (see index.html and
 * tray-menu.html) and exposes `window.AuraIcons`. Keeping the registry classic
 * lets the static HTML hydrate without a build step, while this facade gives
 * the application modules a normal `getIcon()` import.
 */

const icons = window.AuraIcons || {
  body: () => "",
  svg: () => "",
  hydrate: () => {},
};

export function getIcon(name, options = {}) {
  try {
    return icons.svg(name, options);
  } catch (err) {
    console.error("Icon render failed", name, err);
    return "";
  }
}

export function getBrandIcon(name, options = {}) {
  try {
    return typeof icons.brandSvg === "function" ? icons.brandSvg(name, options) : "";
  } catch (err) {
    console.error("Brand icon render failed", name, err);
    return "";
  }
}

export function iconBody(name) {
  return icons.body(name);
}

export function hydrateIcons(root = document) {
  icons.hydrate(root);
}

export { icons };
export default icons;
