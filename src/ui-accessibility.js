/**
 * Adds the WAI-ARIA tablist keyboard pattern without replacing native buttons.
 * @param {Iterable<HTMLElement>} tabs
 * @param {(tab: HTMLElement) => void} activate
 */
export function bindTabKeyboardNavigation(tabs, activate) {
  const items = Array.from(tabs);
  const isVisible = (tab) => tab.offsetParent !== null;

  const wrap = (index, step) => (((index + step) % items.length) + items.length) % items.length;
  const firstVisible = (fromIndex, step) => {
    for (let count = 0; count < items.length; count += 1) {
      const index = wrap(fromIndex, count * step);
      if (isVisible(items[index])) return items[index];
    }
    return null;
  };
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
