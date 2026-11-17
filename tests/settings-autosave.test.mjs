import assert from "node:assert/strict";
import test from "node:test";
import { createSettingsAutosave } from "../src/settings-autosave.js";

const flushMicrotasks = async () => {
  await Promise.resolve();
  await Promise.resolve();
};

test("coalesces rapid changes into one save", async (t) => {
  t.mock.timers.enable({ apis: ["setTimeout"] });
  let saves = 0;
  const autosave = createSettingsAutosave({
    save: async () => {
      saves += 1;
      return { permanentFailure: false };
    },
    isEnabled: () => true,
    normalDelay: 10,
    immediateDelay: 5,
  });

  autosave.mark();
  t.mock.timers.tick(6);
  autosave.mark();
  t.mock.timers.tick(6);
  autosave.mark();
  t.mock.timers.tick(9);
  assert.equal(saves, 0);

  t.mock.timers.tick(1);
  await flushMicrotasks();
  assert.equal(saves, 1);

  t.mock.timers.tick(50);
  await flushMicrotasks();
  assert.equal(saves, 1);
});

test("does not save while settings are not loaded", async (t) => {
  t.mock.timers.enable({ apis: ["setTimeout"] });
  let saves = 0;
  const autosave = createSettingsAutosave({
    save: async () => {
      saves += 1;
      return { permanentFailure: false };
    },
    isEnabled: () => false,
    normalDelay: 10,
    immediateDelay: 5,
  });

  autosave.mark();
  t.mock.timers.tick(50);
  await flushMicrotasks();

  assert.equal(saves, 0);
});

test("reschedules one save when a change arrives mid-flight", async (t) => {
  t.mock.timers.enable({ apis: ["setTimeout"] });
  let saves = 0;
  let releaseFirst;
  const autosave = createSettingsAutosave({
    save: () => {
      saves += 1;
      if (saves === 1) {
        return new Promise((resolve) => {
          releaseFirst = resolve;
        });
      }
      return Promise.resolve({ permanentFailure: false });
    },
    isEnabled: () => true,
    normalDelay: 10,
    immediateDelay: 5,
  });

  autosave.mark();
  t.mock.timers.tick(9);
  assert.equal(saves, 0);

  t.mock.timers.tick(1);
  assert.equal(saves, 1);

  autosave.mark();
  releaseFirst({ permanentFailure: false });
  await flushMicrotasks();
  assert.equal(saves, 1);

  t.mock.timers.tick(5);
  await flushMicrotasks();
  assert.equal(saves, 2);
});

test("permanent failure stops the retry loop", async (t) => {
  t.mock.timers.enable({ apis: ["setTimeout"] });
  let saves = 0;
  let releaseFirst;
  const autosave = createSettingsAutosave({
    save: () => {
      saves += 1;
      return new Promise((resolve) => {
        releaseFirst = resolve;
      });
    },
    isEnabled: () => true,
    normalDelay: 10,
    immediateDelay: 5,
  });

  autosave.mark();
  t.mock.timers.tick(10);
  assert.equal(saves, 1);

  // Even if another mark arrived while in-flight, permanentFailure prevents automatic retry
  autosave.mark();
  releaseFirst({ permanentFailure: true });
  await flushMicrotasks();

  t.mock.timers.tick(50);
  await flushMicrotasks();
  assert.equal(saves, 1);
});
