// @ts-check

/**
 * Debounced settings autosave.
 *
 * Owns the three pieces the page used to coordinate by hand: the debounce
 * timer, the dirty/revision counter, and the in-flight/pending handshake.
 * `save` receives the revision that was current when the save started and
 * returns `{ permanentFailure }` when the backend rejected the values in a way
 * that must not be retried automatically.
 */

/**
 * @typedef {Object} AutosaveContext
 * @property {number} revisionAtStart
 */

/**
 * @typedef {Object} AutosaveResult
 * @property {boolean} [permanentFailure]
 */

/**
 * @typedef {Object} SettingsAutosaveOptions
 * @property {(context: AutosaveContext) => Promise<AutosaveResult | void> | AutosaveResult | void} save
 * @property {() => boolean} isEnabled
 * @property {number} [normalDelay]
 * @property {number} [immediateDelay]
 */

/**
 * @typedef {Object} SettingsAutosaveController
 * @property {(immediate?: boolean) => void} mark
 * @property {() => void} markClean
 * @property {() => boolean} isDirty
 * @property {() => boolean} isSaving
 * @property {() => Promise<void>} flush
 */

/**
 * @param {SettingsAutosaveOptions} options
 * @returns {SettingsAutosaveController}
 */
export function createSettingsAutosave({
  save,
  isEnabled,
  normalDelay = 700,
  immediateDelay = 150,
}) {
  /** @type {ReturnType<typeof setTimeout> | null} */
  let timer = null;
  let revision = 0;
  let dirty = false;
  let inFlight = false;
  let pending = false;

  function clearTimer() {
    if (timer !== null) {
      clearTimeout(timer);
      timer = null;
    }
  }

  /**
   * @param {boolean} immediate
   */
  function schedule(immediate) {
    clearTimer();
    timer = setTimeout(() => {
      timer = null;
      if (dirty) {
        void run();
      }
    }, immediate ? immediateDelay : normalDelay);
  }

  /**
   * @param {boolean} [immediate]
   */
  function mark(immediate = false) {
    if (!isEnabled()) return;
    dirty = true;
    revision += 1;
    if (inFlight) {
      pending = true;
      return;
    }
    schedule(immediate);
  }

  async function run() {
    if (inFlight) {
      pending = true;
      return;
    }
    if (!dirty) return;

    inFlight = true;
    const revisionAtStart = revision;
    let permanentFailure = false;
    try {
      const result = await save({ revisionAtStart });
      permanentFailure = Boolean(result && typeof result === "object" && result.permanentFailure);
      if (revision === revisionAtStart) {
        dirty = false;
      }
    } catch (error) {
      console.error("Settings autosave failed", error);
    } finally {
      inFlight = false;
      if (!permanentFailure && (pending || dirty) && isEnabled()) {
        pending = false;
        schedule(true);
      } else {
        pending = false;
      }
    }
  }

  function markClean() {
    dirty = false;
    clearTimer();
  }

  return {
    mark,
    markClean,
    isDirty: () => dirty,
    isSaving: () => inFlight,
    flush: run,
  };
}
