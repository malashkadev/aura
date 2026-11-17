// @ts-check

/**
 * Adds the WAI-ARIA tablist keyboard pattern without replacing native buttons.
 * @param {Iterable<HTMLElement>} tabs
 * @param {(tab: HTMLElement) => void} activate
 * @returns {void}
 */
export function bindTabKeyboardNavigation(tabs, activate) {
  const items = Array.from(tabs);
  /** @param {HTMLElement} tab */
  const isVisible = (tab) => tab.offsetParent !== null;

  /**
   * @param {number} index
   * @param {number} step
   * @returns {number}
   */
  const wrap = (index, step) => (((index + step) % items.length) + items.length) % items.length;

  /**
   * @param {number} fromIndex
   * @param {number} step
   * @returns {HTMLElement | null}
   */
  const firstVisible = (fromIndex, step) => {
    for (let count = 0; count < items.length; count += 1) {
      const index = wrap(fromIndex, count * step);
      if (isVisible(items[index])) return items[index];
    }
    return null;
  };

  /**
   * @param {number} fromIndex
   * @param {number} step
   * @returns {HTMLElement | null}
   */
  const nextVisible = (fromIndex, step) => {
    for (let count = 1; count <= items.length; count += 1) {
      const index = wrap(fromIndex, count * step);
      if (isVisible(items[index])) return items[index];
    }
    return null;
  };

  items.forEach((tab, index) => {
    tab.addEventListener("keydown", (event) => {
      if (!isVisible(tab)) return;
      /** @type {HTMLElement | null} */
      let target = null;

      if (event.key === "ArrowDown" || event.key === "ArrowRight") {
        target = nextVisible(index, 1);
      } else if (event.key === "ArrowUp" || event.key === "ArrowLeft") {
        target = nextVisible(index, -1);
      } else if (event.key === "Home") {
        target = firstVisible(0, 1);
      } else if (event.key === "End") {
        target = firstVisible(items.length - 1, -1);
      } else {
        return;
      }

      if (!target) return;
      event.preventDefault();
      target.focus();
      activate(target);
    });
  });
}
