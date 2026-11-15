import assert from "node:assert/strict";
import test from "node:test";
import { createSettingsAutosave } from "../src/settings-autosave.js";

const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));

test("coalesces rapid changes into one save", async () => {
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
  autosave.mark();
  autosave.mark();
  await sleep(40);

  assert.equal(saves, 1);
});

test("does not save while settings are not loaded", async () => {
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
  await sleep(30);

  assert.equal(saves, 0);
});

test("reschedules one save when a change arrives mid-flight", async () => {
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
  await sleep(25);
  assert.equal(saves, 1);

  autosave.mark();
  releaseFirst({ permanentFailure: false });
  await sleep(40);

  assert.equal(saves, 2);
});

test("permanent failure stops the retry loop", async () => {
  let saves = 0;
  const autosave = createSettingsAutosave({
    save: async () => {
      saves += 1;
      return { permanentFailure: true };
    },
    isEnabled: () => true,
    normalDelay: 10,
    immediateDelay: 5,
  });

  autosave.mark();
  await sleep(30);

  assert.equal(saves, 1);
});
