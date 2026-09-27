// A dangerous change waits for the in-app dialog (D-2026-09-26-tray-app-4,
// -5): nothing reaches the bridge before "Apply", the write that follows is
// flagged `confirmed`, and the UI shows the value the monitor reads back.

import { readFileSync } from 'node:fs';
import {
  MONITORS,
  acceptButton,
  cancelButton,
  confirmDialog,
  expect,
  inputChip,
  open,
  powerButton,
  serve,
  t,
  test,
  writes,
} from './support.mjs';

const INPUT = 0x60;
const POWER = 0xd6;
const HDMI_1 = 0x11;
const OFF_WRITE_ONLY = 0x05;

test('another input opens the dialog, and nothing is written yet', async ({ page }) => {
  await open(page, '/?demo=rtk');

  await inputChip(page, 'HDMI-1').click();

  await expect(confirmDialog(page)).toBeVisible();
  await expect(page.locator('#confirm-body')).toHaveText(t('confirm.body.input', { to: 'HDMI-1' }));
  await expect(page.locator('#confirm-note')).toHaveText(t('confirm.recover.input'));
  await expect(cancelButton(page)).toBeFocused();
  expect(await writes(page)).toEqual([]);
});

test('Cancel, or Esc, writes nothing and keeps the input', async ({ page }) => {
  await open(page, '/?demo=rtk');

  await inputChip(page, 'HDMI-1').click();
  await cancelButton(page).click();
  await expect(confirmDialog(page)).toBeHidden();
  await inputChip(page, 'HDMI-2').click();
  await expect(confirmDialog(page)).toBeVisible();
  await page.keyboard.press('Escape');
  await expect(confirmDialog(page)).toBeHidden();

  await expect(inputChip(page, 'DisplayPort-1')).toBeChecked();
  await expect(inputChip(page, 'HDMI-1')).not.toBeChecked();
  await expect(inputChip(page, 'HDMI-2')).not.toBeChecked();
  expect(await writes(page)).toEqual([]);
});

test('Apply writes 0x60 once, confirmed, and the UI shows the read-back', async ({ page }) => {
  await open(page, '/?demo=rtk');

  await inputChip(page, 'HDMI-1').click();
  await acceptButton(page).click();

  await expect(confirmDialog(page)).toBeHidden();
  // The chip is checked only when the read-back arrives; until then it is pending.
  await expect(inputChip(page, 'HDMI-1')).toBeChecked();
  await expect(inputChip(page, 'HDMI-1')).not.toHaveClass(/is-pending/);
  await expect(inputChip(page, 'DisplayPort-1')).not.toBeChecked();
  expect(await writes(page)).toEqual([
    { monitorId: MONITORS.rtk, code: INPUT, value: HDMI_1, confirmed: true },
  ]);
});

// A scaler may ignore a switch to an input with no signal. A test-only
// bridge.js keeps the reading of the codes in `window.__ddcKeep`.
const WRITE_LINE = 'entry.reading.current = value;';
const KEEPING_LINE = 'if (!globalThis.__ddcKeep?.includes(code)) entry.reading.current = value;';

test('a monitor that keeps its input: the UI shows the input read back, not the one asked for', async ({
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
  }, INPUT);

  await inputChip(page, 'HDMI-1').click();
  await acceptButton(page).click();

  await expect(page.locator('#announcer')).toHaveText(
    t('announce.readBack', { feature: t('feature.input'), value: 'DisplayPort-1' }),
  );
  await expect(inputChip(page, 'DisplayPort-1')).toBeChecked();
  await expect(inputChip(page, 'HDMI-1')).not.toBeChecked();
  await expect(inputChip(page, 'HDMI-1')).not.toHaveClass(/is-pending/);
  expect(await writes(page)).toEqual([
    { monitorId: MONITORS.rtk, code: INPUT, value: HDMI_1, confirmed: true },
  ]);
});

test('power asks with the other modes and writes the one picked, confirmed', async ({ page }) => {
  await open(page, '/?demo=rtk');
  const dialog = confirmDialog(page);

  await powerButton(page).click();
  await expect(dialog.getByRole('radio')).toHaveCount(2);
  await expect(dialog.getByRole('radio', { name: t('value.off-dpm'), exact: true })).toBeChecked();
  await cancelButton(page).click();
  await expect(dialog).toBeHidden();
  expect(await writes(page)).toEqual([]);

  await powerButton(page).click();
  await dialog.getByRole('radio', { name: t('value.off-write-only'), exact: true }).check();
  await expect(page.locator('#confirm-body')).toHaveText(
    t('confirm.body.power', { to: t('value.off-write-only') }),
  );
  await acceptButton(page).click();

  await expect
    .poll(() => writes(page))
    .toEqual([{ monitorId: MONITORS.rtk, code: POWER, value: OFF_WRITE_ONLY, confirmed: true }]);
});
