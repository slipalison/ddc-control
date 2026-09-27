// A dangerous change waits for the in-app dialog (D-2026-09-26-tray-app-4,
// -5): nothing reaches the bridge before "Apply", the write that follows is
// flagged `confirmed`, and the UI shows the value the monitor reads back.
// Each variant of the dialog shows its own texts: the input's has a note on
// how to go back, power's lists the other modes, and the generic one, for
// any other dangerous setting, has neither.

import { readFileSync } from 'node:fs';
import {
  MONITORS,
  acceptButton,
  cancelButton,
  confirmDialog,
  dropdown,
  expect,
  expectAccessible,
  expectPicked,
  inputChip,
  open,
  pick,
  powerButton,
  serve,
  t,
  test,
  writes,
} from './support.mjs';

const INPUT = 0x60;
const POWER = 0xd6;
const OSD_LOCK = 0xca;
const HDMI_1 = 0x11;
const OFF_WRITE_ONLY = 0x05;
const OSD_DISABLED = 0x01;
/** A continuous code the catalog does not name (see the slider test). */
const UNNAMED = 0xe9;

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

/** Expects the generic dialog: its title, `body`, both buttons, and no note nor choice. */
async function expectGenericDialog(page, body) {
  const dialog = confirmDialog(page);
  await expect(dialog).toBeVisible();
  await expect(dialog).toHaveAttribute('data-tone', 'danger');
  await expect(page.locator('#confirm-title')).toHaveText(t('confirm.title'));
  await expect(page.locator('#confirm-body')).toHaveText(body);
  await expect(page.locator('#confirm-note')).toBeHidden();
  await expect(page.locator('#confirm-note')).toHaveText('');
  await expect(page.locator('#confirm-choices')).toBeHidden();
  await expect(acceptButton(page)).toHaveText(t('confirm.accept'));
  await expect(cancelButton(page)).toHaveText(t('confirm.cancel'));
  await expect(cancelButton(page)).toBeFocused();
}

test('a dangerous list of All settings asks with the generic dialog, then writes confirmed', async ({ page }) => {
  await open(page, '/?demo=rtk');
  await page.getByText(t('more.title'), { exact: true }).click();
  const osdLock = dropdown(page, t('feature.osd-lock'));

  await pick(osdLock, t('value.osd-disabled'));

  await expectGenericDialog(
    page,
    t('confirm.body.generic', { feature: t('feature.osd-lock'), to: t('value.osd-disabled') }),
  );
  await expectAccessible(page);
  expect(await writes(page)).toEqual([]);

  await acceptButton(page).click();

  await expect
    .poll(() => writes(page))
    .toEqual([{ monitorId: MONITORS.rtk, code: OSD_LOCK, value: OSD_DISABLED, confirmed: true }]);
  await expectPicked(osdLock, OSD_DISABLED);
});

// A monitor may declare a continuous code the catalog does not name: it is
// labelled by its code and is dangerous, as any unknown code is (the
// core's `risk_for_code`). A test-only bridge.js adds one to the RTK's
// settings, where the demo's writes find it too.
const MONITORS_LINE = 'const monitors = scenarioMonitors(scenario);';
const UNNAMED_SETTING = {
  code: UNNAMED,
  alias: null,
  name: null,
  dangerous: true,
  origin: 'caps',
  status: 'ok',
  reading: { current: 20, max: 40 },
  options: null,
};

test('a dangerous slider of All settings asks on release: Cancel puts it back, Apply writes it', async ({
  page,
}) => {
  const bridge = readFileSync(new URL('../../src/bridge.js', import.meta.url), 'utf8');
  expect(bridge, 'the demo line this test patches').toContain(MONITORS_LINE);
  const adding = `for (const monitor of monitors) monitor.features.push(${JSON.stringify(UNNAMED_SETTING)});`;
  await page.route('**/bridge.js', (route) =>
    serve(route, { patch: (source) => source.replace(MONITORS_LINE, `${MONITORS_LINE} ${adding}`) }),
  );
  await open(page, '/?demo=rtk');
  await page.getByText(t('more.title'), { exact: true }).click();
  const label = t('format.code', { hex: '0xE9' });
  const range = page.getByRole('slider', { name: label, exact: true });

  await range.press('ArrowRight');

  await expectGenericDialog(
    page,
    t('confirm.body.generic', { feature: label, to: t('format.fraction', { current: 21, max: 40 }) }),
  );
  await expectAccessible(page);
  await cancelButton(page).click();
  await expect(confirmDialog(page)).toBeHidden();
  await expect(range).toHaveValue('20');
  await expect(range).toHaveAttribute('aria-valuetext', t('format.fraction', { current: 20, max: 40 }));
  expect(await writes(page)).toEqual([]);

  await range.press('ArrowRight');
  await acceptButton(page).click();

  await expect
    .poll(() => writes(page))
    .toEqual([{ monitorId: MONITORS.rtk, code: UNNAMED, value: 21, confirmed: true }]);
  await expect(range).toHaveValue('21');
});
