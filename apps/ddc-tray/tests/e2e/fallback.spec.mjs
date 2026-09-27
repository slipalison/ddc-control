// A monitor listed first may never answer DDC/CI — a TV, as on the
// development machine. The popup opens on the first monitor that answers,
// and the mute one stays listed: picking it shows its error, whose "Try
// again" tries that same monitor (D-2026-09-26-tray-app-5).

import {
  MONITORS,
  expect,
  expectAccessible,
  inputChip,
  loadEnds,
  monitorPicker,
  open,
  powerButton,
  retryButton,
  selectedMonitor,
  slider,
  t,
  test,
} from './support.mjs';

const TWO_MONITORS = '/?demo=two-monitors';
const LAST_MONITOR_KEY = 'ddc-tray.last-monitor';

test('the mute TV listed first is skipped: the popup opens on the RTK with no error', async ({
  page,
}) => {
  await open(page, TWO_MONITORS);

  const options = monitorPicker(page).locator('option');
  await expect(options).toHaveText([
    t('header.silent', { label: 'LG TV SSCR2' }),
    'RTK QHD HDR',
    'DELL U2723QE',
  ]);
  expect(await options.evaluateAll((nodes) => nodes.map((node) => node.value))).toEqual([
    MONITORS.tv,
    MONITORS.rtk,
    MONITORS.dell,
  ]);
  await expect(monitorPicker(page)).toHaveValue(MONITORS.rtk);
  await expect(slider(page, 'brightness')).toHaveValue('75');
  await expect(page.locator('#message')).toBeHidden();
  await expect(page.locator('#toast')).toBeHidden();
  expect(await selectedMonitor(page)).toBe(MONITORS.rtk);
});

test('picking the TV shows its error, and "Try again" tries the TV again', async ({ page }) => {
  await open(page, TWO_MONITORS);

  await loadEnds(page, () => monitorPicker(page).selectOption(MONITORS.tv));

  await expect(page.locator('#app')).toHaveAttribute('data-state', 'error');
  await expect(page.getByRole('heading', { name: t('error.timeout'), exact: true })).toBeVisible();
  await expect(page.locator('#message-tip')).toHaveText(t('hint.ddc'));
  await expect(page.locator('#monitor-meta')).toHaveText(t('header.noAnswer'));
  await expect(retryButton(page)).toBeVisible();
  await expect(monitorPicker(page)).toHaveValue(MONITORS.tv);
  await expect(powerButton(page)).toBeHidden();
  await expectAccessible(page);

  // A refresh would open the RTK again; the retry of the TV fails again.
  await loadEnds(page, () => retryButton(page).click());

  await expect(page.locator('#app')).toHaveAttribute('data-state', 'error');
  await expect(monitorPicker(page)).toHaveValue(MONITORS.tv);
  // A mute monitor never becomes the tray shortcuts' target.
  expect(await selectedMonitor(page)).toBe(MONITORS.rtk);
});

test('from the TV error, picking the DELL shows its panel and remembers it', async ({ page }) => {
  await open(page, TWO_MONITORS);
  await loadEnds(page, () => monitorPicker(page).selectOption(MONITORS.tv));

  await loadEnds(page, () => monitorPicker(page).selectOption(MONITORS.dell));

  await expect(page.locator('#app')).toHaveAttribute('data-state', 'ready');
  await expect(slider(page, 'brightness')).toHaveValue('40');
  await expect(slider(page, 'volume')).toHaveCount(0);
  await expect(inputChip(page, t('format.unnamed', { hex: '0x1B' }))).toBeVisible();
  expect(await selectedMonitor(page)).toBe(MONITORS.dell);
  expect(await page.evaluate((key) => localStorage.getItem(key), LAST_MONITOR_KEY)).toBe(
    MONITORS.dell,
  );

  await page.reload();

  await expect(page.locator('#app')).toHaveAttribute('data-state', 'ready');
  await expect(monitorPicker(page)).toHaveValue(MONITORS.dell);
});
