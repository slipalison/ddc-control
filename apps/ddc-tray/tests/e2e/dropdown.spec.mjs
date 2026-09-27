// The in-page dropdown (D-2026-09-27-tray-app-1): a list the popup draws
// itself instead of a native select element, whose menu was a window of
// its own and made the popup hide on blur. Picking writes once and shows
// the value read back; Esc, a second click or a click outside write
// nothing; the list always fits the popup's 360×560 window; and a
// dangerous list still asks first.

import { readFileSync } from 'node:fs';
import { DEMO_LATENCY_MS } from '../../src/bridge.js';
import { DEBOUNCE_MS } from '../../src/debounce.js';
import {
  MONITORS,
  cancelButton,
  confirmDialog,
  dropdown,
  expect,
  expectAccessible,
  expectPicked,
  hides,
  listOf,
  open,
  optionsOf,
  pick,
  serve,
  t,
  test,
  writes,
} from './support.mjs';

const PRESET = 0x14;
const OSD_LANGUAGE = 0xcc;
const SRGB = 0x01;
const NATIVE = 0x02;
const USER_1 = 0x0b;
const OSD_ENABLED = 0x02;
const FRENCH = 0x03;

const WINDOW = Object.freeze({ width: 360, height: 560 });

function presetList(page) {
  return dropdown(page, t('feature.preset'));
}

/** Waits past one more debounce window and write, for a write that should not come. */
async function settle(page) {
  await page.waitForTimeout(2 * DEBOUNCE_MS + DEMO_LATENCY_MS);
}

/** The list's box lies inside the popup's window. */
async function expectInsideWindow(list) {
  const box = await list.boundingBox();
  expect(box, 'the list is laid out').not.toBeNull();
  expect(box.x).toBeGreaterThanOrEqual(0);
  expect(box.y).toBeGreaterThanOrEqual(0);
  expect(box.x + box.width).toBeLessThanOrEqual(WINDOW.width);
  expect(box.y + box.height).toBeLessThanOrEqual(WINDOW.height);
}

test('picking another preset in the list writes 0x14 once and shows the value read back', async ({
  page,
}) => {
  await open(page, '/?demo=rtk');
  const preset = presetList(page);

  await preset.click();

  const list = listOf(preset);
  await expect(list).toBeVisible();
  await expect(preset).toHaveAttribute('aria-expanded', 'true');
  await expect(optionsOf(preset)).toHaveCount(7);
  await expect(list.getByRole('option', { name: 'sRGB', exact: true })).toHaveAttribute('aria-selected', 'true');
  await expectInsideWindow(list);

  await list.getByRole('option', { name: t('value.display-native'), exact: true }).click();

  await expect(list).toBeHidden();
  await expect(preset).toHaveAttribute('aria-expanded', 'false');
  await expect(preset).toBeFocused();
  const once = [{ monitorId: MONITORS.rtk, code: PRESET, value: NATIVE, confirmed: false }];
  await expect.poll(() => writes(page)).toEqual(once);
  await expectPicked(preset, NATIVE);
  await expect(preset).toHaveText(t('value.display-native'));
  await settle(page);
  expect(await writes(page)).toEqual(once);
  expect(await hides(page)).toBe(0);
});

// A scaler may keep its preset. A test-only bridge.js keeps the reading of
// the codes in `window.__ddcKeep`, as confirm.spec.mjs does for the input.
const WRITE_LINE = 'entry.reading.current = value;';
const KEEPING_LINE = 'if (!globalThis.__ddcKeep?.includes(code)) entry.reading.current = value;';

test('a monitor that keeps its preset: the list shows the preset read back, not the one picked', async ({
  page,
}) => {
  const bridge = readFileSync(new URL('../../src/bridge.js', import.meta.url), 'utf8');
  expect(bridge, 'the demo write this test patches').toContain(WRITE_LINE);
  await page.route('**/bridge.js', (route) =>
    serve(route, { patch: (source) => source.replace(WRITE_LINE, KEEPING_LINE) }),
  );
  await open(page, '/?demo=rtk');
  await page.evaluate((code) => {
    window.__ddcKeep = [code];
  }, PRESET);
  const preset = presetList(page);

  await pick(preset, t('value.user-1'));

  await expect(page.locator('#announcer')).toHaveText(
    t('announce.readBack', { feature: t('feature.preset'), value: 'sRGB' }),
  );
  await expectPicked(preset, SRGB);
  await expect(preset).toHaveText('sRGB');
  expect(await writes(page)).toEqual([
    { monitorId: MONITORS.rtk, code: PRESET, value: USER_1, confirmed: false },
  ]);
});

test('Esc closes the list with no write and keeps the popup; a second Esc hides the popup', async ({
  page,
}) => {
  await open(page, '/?demo=rtk');
  const preset = presetList(page);
  await preset.focus();

  await page.keyboard.press('Enter');
  await expect(listOf(preset)).toBeVisible();
  await page.keyboard.press('ArrowDown');
  await page.keyboard.press('ArrowDown');
  await expect(preset).toHaveAttribute('aria-activedescendant', `${await preset.getAttribute('id')}-option-2`);
  await page.keyboard.press('Escape');

  await expect(listOf(preset)).toBeHidden();
  await expect(preset).toBeFocused();
  await expectPicked(preset, SRGB);
  await settle(page);
  expect(await writes(page)).toEqual([]);
  expect(await hides(page)).toBe(0);

  await page.keyboard.press('Escape');

  await expect.poll(() => hides(page)).toBe(1);
  expect(await writes(page)).toEqual([]);
});

test('from the keyboard: Alt+ArrowDown opens, End and Enter write the last preset once', async ({
  page,
}) => {
  await open(page, '/?demo=rtk');
  const preset = presetList(page);
  await preset.focus();

  await page.keyboard.press('Alt+ArrowDown');
  await page.keyboard.press('End');
  await page.keyboard.press('Enter');

  const once = [{ monitorId: MONITORS.rtk, code: PRESET, value: USER_1, confirmed: false }];
  await expect.poll(() => writes(page)).toEqual(once);
  await expect(listOf(preset)).toBeHidden();
  await expectPicked(preset, USER_1);
  await expect(preset).toHaveText(t('value.user-1'));
  await settle(page);
  expect(await writes(page)).toEqual(once);
});

test('with the list open: no serious axe violation; a click outside closes it with no write', async ({
  page,
  pageErrors,
}) => {
  await open(page, '/?demo=rtk');
  const preset = presetList(page);
  await preset.click();
  await expect(listOf(preset)).toBeVisible();

  await expectAccessible(page);

  await page.getByRole('heading', { name: t('feature.input'), exact: true }).click();
  await expect(listOf(preset)).toBeHidden();
  await preset.click();
  await expect(listOf(preset)).toBeVisible();
  await preset.click();
  await expect(listOf(preset)).toBeHidden();
  await settle(page);
  expect(await writes(page)).toEqual([]);
  expect(pageErrors).toEqual([]);
});

test('"All settings": a list near the bottom opens above its button, inside the window, and writes once', async ({
  page,
}) => {
  await open(page, '/?demo=rtk');
  await page.getByText(t('more.title'), { exact: true }).click();
  const language = dropdown(page, t('feature.osd-language'));
  await language.scrollIntoViewIfNeeded();

  await language.click();

  const list = listOf(language);
  await expect(list).toBeVisible();
  await expect(language.locator('xpath=..')).toHaveAttribute('data-side', 'above');
  await expectInsideWindow(list);
  await list.getByRole('option', { name: t('value.french'), exact: true }).click();

  const once = [{ monitorId: MONITORS.rtk, code: OSD_LANGUAGE, value: FRENCH, confirmed: false }];
  await expect.poll(() => writes(page)).toEqual(once);
  await expectPicked(language, FRENCH);
});

test('"All settings": a dangerous list asks first, and Cancel writes nothing and restores it', async ({
  page,
}) => {
  await open(page, '/?demo=rtk');
  await page.getByText(t('more.title'), { exact: true }).click();
  const osdLock = dropdown(page, t('feature.osd-lock'));

  await pick(osdLock, t('value.osd-disabled'));

  await expect(confirmDialog(page)).toBeVisible();
  await expect(page.locator('#confirm-body')).toHaveText(
    t('confirm.body.generic', { feature: t('feature.osd-lock'), to: t('value.osd-disabled') }),
  );
  await cancelButton(page).click();
  await expect(confirmDialog(page)).toBeHidden();
  await expectPicked(osdLock, OSD_ENABLED);
  await settle(page);
  expect(await writes(page)).toEqual([]);
});

test('a scroll moves the open list with its button, and closes it once the button is out of sight', async ({
  page,
}) => {
  await open(page, '/?demo=rtk');
  await page.getByText(t('more.title'), { exact: true }).click();
  await expect(page.locator('#feature-list > li')).toHaveCount(7);
  const preset = presetList(page);
  await preset.click();
  const list = listOf(preset);
  await expect(list).toBeVisible();
  const before = await list.boundingBox();

  await page.mouse.move(40, 400);
  await page.mouse.wheel(0, 40);

  await expect.poll(async () => (await list.boundingBox())?.y).toBeLessThan(before.y);
  await expect(list).toBeVisible();

  await page.mouse.wheel(0, 600);

  await expect(list).toBeHidden();
  await expect(preset).toHaveAttribute('aria-expanded', 'false');
  await settle(page);
  expect(await writes(page)).toEqual([]);
});
