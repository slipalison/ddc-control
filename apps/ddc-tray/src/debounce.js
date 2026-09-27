// Slider writes (D-2026-09-26-tray-app-5): a burst of values becomes one
// write of the last value once the burst pauses for DEBOUNCE_MS, and a key
// never has two writes in flight — DDC/CI is slow and half-duplex, so a
// value that arrives meanwhile waits and replaces any older waiting one.

export const DEBOUNCE_MS = 80;

const realTimers = Object.freeze({
  setTimeout: (callback, ms) => globalThis.setTimeout(callback, ms),
  clearTimeout: (id) => globalThis.clearTimeout(id),
});

/**
 * A write queue with one lane per key (the popup uses one per VCP code).
 *
 * `write(key, value)` performs a write and resolves with what it answers;
 * `onResult(key, result, value)` and `onError(key, error, value)` report it.
 * The next waiting value of the lane is sent before they run, so a failed
 * write or a throwing callback never blocks the lane.
 *
 * @template K, V, R
 * @param {{
 *   write: (key: K, value: V) => Promise<R> | R,
 *   onResult?: (key: K, result: R, value: V) => void,
 *   onError?: (key: K, error: unknown, value: V) => void,
 *   delayMs?: number,
 *   timers?: { setTimeout: Function, clearTimeout: Function },
 * }} options
 */
export function createWriteQueue({
  write,
  onResult = () => {},
  onError = () => {},
  delayMs = DEBOUNCE_MS,
  timers = realTimers,
}) {
  const lanes = new Map();

  function lane(key) {
    let found = lanes.get(key);
    if (!found) {
      found = { waiting: false, value: undefined, timer: null, inFlight: false };
      lanes.set(key, found);
    }
    return found;
  }

  function stopTimer(state) {
    if (state.timer === null) return;
    timers.clearTimeout(state.timer);
    state.timer = null;
  }

  function send(key, state) {
    if (!state.waiting || state.inFlight || state.timer !== null) return;
    const { value } = state;
    state.waiting = false;
    state.value = undefined;
    state.inFlight = true;
    void run(key, state, value);
  }

  async function run(key, state, value) {
    let outcome;
    try {
      outcome = { ok: true, result: await write(key, value) };
    } catch (error) {
      outcome = { ok: false, error };
    }
    state.inFlight = false;
    send(key, state);
    if (outcome.ok) onResult(key, outcome.result, value);
    else onError(key, outcome.error, value);
  }

  return {
    /** Queues `value` for `key`, replacing a value still waiting there. */
    push(key, value) {
      const state = lane(key);
      state.value = value;
      state.waiting = true;
      stopTimer(state);
      state.timer = timers.setTimeout(() => {
        state.timer = null;
        send(key, state);
      }, delayMs);
    },

    /** Sends the waiting value of `key` now, or right after its write in flight. */
    flush(key) {
      const state = lane(key);
      stopTimer(state);
      send(key, state);
    },

    /** Whether `key` has a value waiting or a write in flight. */
    busy(key) {
      const state = lanes.get(key);
      return Boolean(state && (state.waiting || state.inFlight));
    },
  };
}
