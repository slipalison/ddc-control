// A dragged slider (D-2026-09-26-tray-app-5): while the thumb moves, the
// debounce holds the writes back and coalesces them; the one write that
// goes out after the release carries the last value, and the slider then
// shows the value the monitor read back. The drag is a real one: the mouse
// presses the thumb, moves along the track in small steps and lets go, so
// a slider that wrote on every `input` fails here, even though the write
// queue alone passes its node tests.
//
// The page's clock is Playwright's and stands still during the drag: each
// step moves the mouse and then lets exactly MOVE_PAUSE_MS pass, firing
// whatever timer falls due. On a busy machine a real pause between two
// moves can outlast the debounce window, and the write it then sends would
// be right, and the test would fail for nothing.

import { readFileSync } from 'node:fs';
import { DEMO_LATENCY_MS } from '../../src/bridge.js';
import { DEBOUNCE_MS } from '../../src/debounce.js';
import { MONITORS, expect, open, serve, slider, t, test, writes } from './support.mjs';

const BRIGHTNESS = 0x10;
/** The RTK demo's brightness, where the drag starts. */
const FROM = 75;
/** Where the drag ends. */
const TO = 20;
/** Width of `.range::-webkit-slider-thumb` in styles.css. */
const THUMB_PX = 18;
const MOVES = 20;
/** The time between two moves, well inside the debounce window. */
const MOVE_PAUSE_MS = 20;
const DRAG_MS_AT_LEAST = 300;

/** Makes the page's clock stand still until the test moves it. */
async function stopTheClock(page) {
  await page.clock.install();
  const now = await page.evaluate(() => Date.now());
  await page.clock.pauseAt(now + 1_000);
}

/**
 * Records, in the page, what happens while `range` is dragged: each value
 * and when it came, how many writes the demo had taken by then, and when
 * the pointer was released and each write landed.
 */
function recordDrag(range) {
  return range.evaluate((input) => {
    const demoWrites = window.__ddcDemo.writes;
    const drag = { inputs: [], releasedAt: null, writesAtRelease: null, landedAt: [] };
    input.addEventListener('input', () => {
      drag.inputs.push({ value: Number(input.value), at: performance.now(), writes: demoWrites.length });
    });
    // Capturing on the window runs before the page's own handlers.
    window.addEventListener(
      'pointerup',
      () => {
        drag.releasedAt = performance.now();
        drag.writesAtRelease = demoWrites.length;
      },
      { capture: true, once: true },
    );
    const push = demoWrites.push.bind(demoWrites);
    demoWrites.push = (...items) => {
      drag.landedAt.push(performance.now());
      return push(...items);
    };
    window.__drag = drag;
  });
}

/**
 * Presses the thumb at `from`, moves it to `to` in MOVES steps, MOVE_PAUSE_MS
 * apart on the page's clock and on the wall clock, and lets go.
 */
async function drag(page, range, from, to) {
  const box = await range.boundingBox();
  // The thumb's centre runs from half a thumb in from each end of the track.
  const x = (value) => box.x + THUMB_PX / 2 + ((box.width - THUMB_PX) * value) / 100;
  const y = box.y + box.height / 2;
  await page.mouse.move(x(from), y);
  await page.mouse.down();
  for (let step = 1; step <= MOVES; step += 1) {
    await page.mouse.move(x(from + ((to - from) * step) / MOVES), y);
    await page.clock.runFor(MOVE_PAUSE_MS);
    await page.waitForTimeout(MOVE_PAUSE_MS);
  }
  await page.mouse.up();
}

test('dragging the brightness slider writes once, after the drag', async ({ page }) => {
  await open(page, '/?demo=rtk');
  const brightness = slider(page, 'brightness');
  await expect(brightness).toHaveValue(String(FROM));
  await stopTheClock(page);
  await recordDrag(brightness);

  await drag(page, brightness, FROM, TO);

  const recorded = await page.evaluate(() => window.__drag);
  const { inputs } = recorded;
  expect(inputs.length, 'input events of the drag').toBeGreaterThanOrEqual(15);
  expect(inputs.at(-1).at - inputs[0].at, 'ms from the first input to the last').toBeGreaterThanOrEqual(
    DRAG_MS_AT_LEAST,
  );
  // The drag never paused for a debounce window, so nothing was due yet.
  const pauses = inputs.slice(1).map(({ at }, index) => at - inputs[index].at);
  pauses.push(recorded.releasedAt - inputs.at(-1).at);
  expect(Math.max(...pauses), 'longest pause of the drag, in ms').toBeLessThan(DEBOUNCE_MS);
  expect(
    inputs.map(({ writes: taken }) => taken),
    'writes taken at each input',
  ).toEqual(inputs.map(() => 0));
  expect(recorded.writesAtRelease, 'writes taken at the release').toBe(0);
  const last = inputs.at(-1).value;
  expect(last).toBe(TO);

  await page.clock.runFor(DEMO_LATENCY_MS);
  const once = [{ monitorId: MONITORS.rtk, code: BRIGHTNESS, value: last, confirmed: false }];
  await expect.poll(() => writes(page)).toEqual(once);
  // Past another debounce window and write, still that one write.
  await page.clock.runFor(2 * DEBOUNCE_MS + DEMO_LATENCY_MS);
  expect(await writes(page)).toEqual(once);
  // It was sent at the release: the demo takes DEMO_LATENCY_MS to answer.
  const { landedAt, releasedAt } = await page.evaluate(() => window.__drag);
  expect(landedAt[0] - releasedAt, 'ms from the release to the write').toBe(DEMO_LATENCY_MS);
  await expect(brightness).toHaveValue(String(last));
  await expect(brightness).toHaveAttribute('aria-valuetext', `${last}%`);
});

// A scaler may keep its brightness. A test-only bridge.js keeps the reading
// of the codes in `window.__ddcKeep`, as dropdown.spec.mjs does for the preset.
const WRITE_LINE = 'entry.reading.current = value;';
const KEEPING_LINE = 'if (!globalThis.__ddcKeep?.includes(code)) entry.reading.current = value;';

test('a dragged slider settles on the brightness the monitor read back', async ({ page }) => {
  const bridge = readFileSync(new URL('../../src/bridge.js', import.meta.url), 'utf8');
  expect(bridge, 'the demo write this test patches').toContain(WRITE_LINE);
  await page.route('**/bridge.js', (route) =>
    serve(route, { patch: (source) => source.replace(WRITE_LINE, KEEPING_LINE) }),
  );
  await open(page, '/?demo=rtk');
  await page.evaluate((code) => {
    window.__ddcKeep = [code];
  }, BRIGHTNESS);
  const brightness = slider(page, 'brightness');
  await stopTheClock(page);

  await drag(page, brightness, FROM, TO);
  await page.clock.resume();

  await expect(page.locator('#announcer')).toHaveText(
    t('announce.readBack', { feature: t('feature.brightness'), value: `${FROM}%` }),
  );
  await expect(brightness).toHaveValue(String(FROM));
  await expect(brightness).toHaveAttribute('aria-valuetext', `${FROM}%`);
  expect(await writes(page)).toEqual([
    { monitorId: MONITORS.rtk, code: BRIGHTNESS, value: TO, confirmed: false },
  ]);
});
