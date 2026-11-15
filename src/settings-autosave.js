/**
 * Debounced settings autosave.
 *
 * Owns the three pieces the page used to coordinate by hand: the debounce
 * timer, the dirty/revision counter, and the in-flight/pending handshake.
 * `save` receives the revision that was current when the save started and
 * returns `{ permanentFailure }` when the backend rejected the values in a way
 * that must not be retried automatically.
 */
export function createSettingsAutosave({
  save,
  isEnabled,
  normalDelay = 700,
  immediateDelay = 150,
}) {
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

  function schedule(immediate) {
    clearTimer();
    timer = setTimeout(() => {
      timer = null;
      if (dirty) {
        void run();
      }
    }, immediate ? immediateDelay : normalDelay);
  }

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
      permanentFailure = Boolean(result?.permanentFailure);
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
