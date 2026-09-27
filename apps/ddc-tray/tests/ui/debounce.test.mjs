import { test } from 'node:test';
import assert from 'node:assert/strict';

import { DEBOUNCE_MS, createWriteQueue } from '../../src/debounce.js';

const BRIGHTNESS = 0x10;
const CONTRAST = 0x12;

// setImmediate is left real, so this drains the promise jobs of a write
// that just settled while setTimeout stays mocked.
const settle = () => new Promise((resolve) => setImmediate(resolve));

// Writes that stay in flight until the test settles them by hand.
function manualWrites() {
  const calls = [];
  const write = (key, value) =>
    new Promise((resolve, reject) => {
      calls.push({ key, value, resolve, reject });
    });
  return { calls, write, sent: () => calls.map(({ key, value }) => [key, value]) };
}

function mockTimers(t) {
  t.mock.timers.enable({ apis: ['setTimeout'] });
  return (ms) => t.mock.timers.tick(ms);
}

test('the debounce window is 80 ms', () => {
  assert.equal(DEBOUNCE_MS, 80);
});

test('five inputs within 80 ms become one write of the last value', (t) => {
  const tick = mockTimers(t);
  const writes = [];
  const queue = createWriteQueue({ write: (key, value) => writes.push([key, value]) });

  for (const value of [10, 20, 30, 40, 50]) {
    queue.push(BRIGHTNESS, value);
    tick(15);
  }
  assert.deepEqual(writes, []);
  tick(DEBOUNCE_MS);

  assert.deepEqual(writes, [[BRIGHTNESS, 50]]);
});

test('a value is sent exactly when the burst has paused for 80 ms', (t) => {
  const tick = mockTimers(t);
  const { write, sent } = manualWrites();
  const queue = createWriteQueue({ write });

  queue.push(BRIGHTNESS, 60);
  tick(DEBOUNCE_MS - 1);
  assert.deepEqual(sent(), []);
  tick(1);

  assert.deepEqual(sent(), [[BRIGHTNESS, 60]]);
});

// A slider dragged for 400 ms: every push restarts the window, so nothing
// is written while the drag lasts — a throttle would write every 80 ms.
test('a burst longer than the debounce window writes once, after it ends', async (t) => {
  const tick = mockTimers(t);
  const writes = [];
  const queue = createWriteQueue({ write: async (key, value) => writes.push([key, value]) });
  const burst = Array.from({ length: 20 }, (_, index) => index * 5);
  const every = 20;
  assert.ok(burst.length * every > DEBOUNCE_MS * 4);

  for (const [index, value] of burst.entries()) {
    queue.push(BRIGHTNESS, value);
    if (index === burst.length - 1) break;
    tick(every);
    await settle();
    assert.deepEqual(writes, [], `no write ${(index + 1) * every} ms into the burst`);
  }
  tick(DEBOUNCE_MS - 1);
  await settle();
  assert.deepEqual(writes, [], `no write ${DEBOUNCE_MS - 1} ms after the last push`);
  tick(1);
  await settle();

  assert.deepEqual(writes, [[BRIGHTNESS, burst.at(-1)]]);
});

test('the second write waits for the first and carries only the last value', async (t) => {
  const tick = mockTimers(t);
  const { calls, write, sent } = manualWrites();
  const queue = createWriteQueue({ write });

  queue.push(BRIGHTNESS, 10);
  tick(DEBOUNCE_MS);
  queue.push(BRIGHTNESS, 20);
  queue.push(BRIGHTNESS, 30);
  tick(DEBOUNCE_MS);
  assert.deepEqual(sent(), [[BRIGHTNESS, 10]]);

  calls[0].resolve({ current: 10, max: 100 });
  await settle();

  assert.deepEqual(sent(), [
    [BRIGHTNESS, 10],
    [BRIGHTNESS, 30],
  ]);
});

test('a value still in its debounce window when a write ends waits for its own timer', async (t) => {
  const tick = mockTimers(t);
  const { calls, write, sent } = manualWrites();
  const queue = createWriteQueue({ write });

  queue.push(BRIGHTNESS, 10);
  tick(DEBOUNCE_MS);
  queue.push(BRIGHTNESS, 20);
  calls[0].resolve({ current: 10, max: 100 });
  await settle();
  assert.deepEqual(sent(), [[BRIGHTNESS, 10]]);
  tick(DEBOUNCE_MS);

  assert.deepEqual(sent(), [
    [BRIGHTNESS, 10],
    [BRIGHTNESS, 20],
  ]);
});

test('flush sends the final value at once and cancels its timer', (t) => {
  const tick = mockTimers(t);
  const { write, sent } = manualWrites();
  const queue = createWriteQueue({ write });

  queue.push(BRIGHTNESS, 10);
  queue.push(BRIGHTNESS, 70);
  queue.flush(BRIGHTNESS);
  assert.deepEqual(sent(), [[BRIGHTNESS, 70]]);
  tick(DEBOUNCE_MS * 3);

  assert.deepEqual(sent(), [[BRIGHTNESS, 70]]);
});

test('flush during a write in flight sends the final value right after it', async (t) => {
  const tick = mockTimers(t);
  const { calls, write, sent } = manualWrites();
  const queue = createWriteQueue({ write });

  queue.push(BRIGHTNESS, 10);
  tick(DEBOUNCE_MS);
  queue.push(BRIGHTNESS, 90);
  queue.flush(BRIGHTNESS);
  assert.deepEqual(sent(), [[BRIGHTNESS, 10]]);

  calls[0].resolve({ current: 10, max: 100 });
  await settle();

  assert.deepEqual(sent(), [
    [BRIGHTNESS, 10],
    [BRIGHTNESS, 90],
  ]);
});

test('flush with nothing waiting sends nothing', () => {
  const { write, sent } = manualWrites();
  const queue = createWriteQueue({ write });

  queue.flush(BRIGHTNESS);

  assert.deepEqual(sent(), []);
});

test('each code has its own lane with one write in flight', (t) => {
  const tick = mockTimers(t);
  const { write, sent } = manualWrites();
  const queue = createWriteQueue({ write });

  queue.push(BRIGHTNESS, 40);
  queue.push(CONTRAST, 60);
  tick(DEBOUNCE_MS);
  queue.push(BRIGHTNESS, 41);
  tick(DEBOUNCE_MS);

  assert.deepEqual(sent(), [
    [BRIGHTNESS, 40],
    [CONTRAST, 60],
  ]);
});

test('the read-back of a write is reported with the value asked for', async (t) => {
  const tick = mockTimers(t);
  const results = [];
  const queue = createWriteQueue({
    write: async () => ({ current: 48, max: 100 }),
    onResult: (key, result, value) => results.push([key, result, value]),
  });

  queue.push(BRIGHTNESS, 50);
  tick(DEBOUNCE_MS);
  await settle();

  assert.deepEqual(results, [[BRIGHTNESS, { current: 48, max: 100 }, 50]]);
});

test('a failed write is reported and the lane keeps working', async (t) => {
  const tick = mockTimers(t);
  const { calls, write, sent } = manualWrites();
  const errors = [];
  const queue = createWriteQueue({
    write,
    onError: (key, error, value) => errors.push([key, error, value]),
  });
  const timeout = { kind: 'timeout', message: 'monitor did not respond in time' };

  queue.push(BRIGHTNESS, 10);
  tick(DEBOUNCE_MS);
  calls[0].reject(timeout);
  await settle();
  queue.push(BRIGHTNESS, 20);
  tick(DEBOUNCE_MS);

  assert.deepEqual(errors, [[BRIGHTNESS, timeout, 10]]);
  assert.deepEqual(sent(), [
    [BRIGHTNESS, 10],
    [BRIGHTNESS, 20],
  ]);
});

test('a write that throws at once is reported like a rejected one', async (t) => {
  const tick = mockTimers(t);
  const errors = [];
  const failure = new Error('boom');
  const queue = createWriteQueue({
    write: () => {
      throw failure;
    },
    onError: (key, error, value) => errors.push([key, error, value]),
  });

  queue.push(CONTRAST, 5);
  tick(DEBOUNCE_MS);
  await settle();

  assert.deepEqual(errors, [[CONTRAST, failure, 5]]);
});

test('busy covers a waiting value and a write in flight', async (t) => {
  const tick = mockTimers(t);
  const { calls, write } = manualWrites();
  const queue = createWriteQueue({ write });
  const states = [queue.busy(BRIGHTNESS)];

  queue.push(BRIGHTNESS, 10);
  states.push(queue.busy(BRIGHTNESS));
  tick(DEBOUNCE_MS);
  states.push(queue.busy(BRIGHTNESS));
  calls[0].resolve({ current: 10, max: 100 });
  await settle();
  states.push(queue.busy(BRIGHTNESS));

  assert.deepEqual(states, [false, true, true, false]);
});
