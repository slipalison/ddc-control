// The popup works from the keyboard alone (D-2026-09-26-tray-app-5): Tab
// reaches the quick controls, an arrow key moves a slider and writes once
// after the debounce, and in the input group the arrows only move the focus
// — switching the input is a dangerous change, asked first.

import { DEMO_LATENCY_MS } from '../../src/bridge.js';
import { DEBOUNCE_MS } from '../../src/debounce.js';
import {
  MONITORS,
  cancelButton,
  confirmDialog,
  expect,
  inputChip,
  open,
  slider,
  test,
  writes,
} from './support.mjs';

const BRIGHTNESS = 0x10;

/** Presses Tab until `target` has the focus; fails if it never gets it. */
async function tabTo(page, target, maxPresses = 12) {
  for (let presses = 0; presses < maxPresses; presses += 1) {
    if (await target.evaluate((node) => node === document.activeElement)) return;
    await page.keyboard.press('Tab');
  }
  await expect(target).toBeFocused();
}

test('Tab reaches brightness; ArrowRight moves it and writes 0x10 exactly once', async ({ page }) => {
  await open(page, '/?demo=rtk');
  const brightness = slider(page, 'brightness');
  await tabTo(page, brightness);
  await expect(brightness).toHaveValue('75');

  await page.keyboard.press('ArrowRight');

  await expect(brightness).toHaveValue('76');
  await expect(brightness).toHaveAttribute('aria-valuetext', '76%');
  const once = [{ monitorId: MONITORS.rtk, code: BRIGHTNESS, value: 76, confirmed: false }];
  await expect.poll(() => writes(page)).toEqual(once);
  // Past another debounce window and write, still that one write.
  await page.waitForTimeout(2 * DEBOUNCE_MS + DEMO_LATENCY_MS);
  expect(await writes(page)).toEqual(once);
  await expect(brightness).toHaveAttribute('aria-valuetext', '76%');
});

test('in the input group arrows only move the focus; Space asks before switching', async ({
  page,
}) => {
  await open(page, '/?demo=rtk');
  const current = inputChip(page, 'DisplayPort-1');
  // Roving tabindex: the checked input is the group's only Tab stop.
  await tabTo(page, current);

  await page.keyboard.press('ArrowRight');

  await expect(inputChip(page, 'DisplayPort-2')).toBeFocused();
  await expect(current).toBeChecked();
  await expect(confirmDialog(page)).toBeHidden();

  await page.keyboard.press('Space');

  await expect(confirmDialog(page)).toBeVisible();
  await expect(cancelButton(page)).toBeFocused();
  await page.keyboard.press('Escape');
  await expect(confirmDialog(page)).toBeHidden();
  await expect(current).toBeChecked();
  expect(await writes(page)).toEqual([]);
});
