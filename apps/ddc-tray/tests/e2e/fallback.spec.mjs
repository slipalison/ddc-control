// A monitor listed first may never answer DDC/CI — a TV, as on the
// development machine. The popup opens on the first monitor that answers,
// and the mute one stays listed: picking it shows its error, whose "Try
// again" tries that same monitor (D-2026-09-26-tray-app-5).

import {
  MONITORS,
  expect,
  expectAccessible,
  expectPicked,
  inputChip,
  loadEnds,
  monitorPicker,
  open,
  optionsOf,
  pick,
  powerButton,
  retryButton,
  selectedMonitor,
  slider,
  t,
  test,
} from './support.mjs';

const TWO_MONITORS = '/?demo=two-monitors';
const LAST_MONITOR_KEY = 'ddc-tray.last-monitor';
const TV = t('header.silent', { label: 'LG TV SSCR2' });
const DELL = 'DELL U2723QE';

test('the mute TV listed first is skipped: the popup opens on the RTK with no error', async ({
  page,
}) => {
  await open(page, TWO_MONITORS);

  const options = optionsOf(monitorPicker(page));
  await expect(options).toHaveText([TV, 'RTK QHD HDR', DELL]);
  expect(await options.evaluateAll((nodes) => nodes.map((node) => node.dataset.value))).toEqual([
    MONITORS.tv,
    MONITORS.rtk,
    MONITORS.dell,
  ]);
  await expectPicked(monitorPicker(page), MONITORS.rtk);
  await expect(slider(page, 'brightness')).toHaveValue('75');
  await expect(page.locator('#message')).toBeHidden();
  await expect(page.locator('#toast')).toBeHidden();
  expect(await selectedMonitor(page)).toBe(MONITORS.rtk);
});

test('picking the TV shows its error, and "Try again" tries the TV again', async ({ page }) => {
  await open(page, TWO_MONITORS);

  await loadEnds(page, () => pick(monitorPicker(page), TV));

  await expect(page.locator('#app')).toHaveAttribute('data-state', 'error');
  await expect(page.getByRole('heading', { name: t('error.transport'), exact: true })).toBeVisible();
  await expect(page.locator('#message-tip')).toHaveText(t('hint.ddc'));
  await expect(page.locator('#monitor-meta')).toHaveText(t('header.noAnswer'));
  await expect(retryButton(page)).toBeVisible();
  await expectPicked(monitorPicker(page), MONITORS.tv);
  await expect(powerButton(page)).toBeHidden();
  await expectAccessible(page);

  // A refresh would open the RTK again; the retry of the TV fails again.
  await loadEnds(page, () => retryButton(page).click());

  await expect(page.locator('#app')).toHaveAttribute('data-state', 'error');
  await expectPicked(monitorPicker(page), MONITORS.tv);
  // A mute monitor never becomes the tray shortcuts' target.
  expect(await selectedMonitor(page)).toBe(MONITORS.rtk);
});

test('from the TV error, picking the DELL shows its panel and remembers it', async ({ page }) => {
  await open(page, TWO_MONITORS);
  await loadEnds(page, () => pick(monitorPicker(page), TV));

  await loadEnds(page, () => pick(monitorPicker(page), DELL));

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
  await expectPicked(monitorPicker(page), MONITORS.dell);
});
