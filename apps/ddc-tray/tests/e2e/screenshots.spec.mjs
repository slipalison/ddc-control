// The popup's pictures for the README and the PR reviewer, every one in
// each theme (plan A-7): only with SCREENSHOTS=1, so a Gate 7 run never
// rewrites the committed files, and then none is skipped. English, like the
// README; motion off, so a frame never catches a transition.

import { fileURLToPath } from 'node:url';
import { expect, open, test } from './support.mjs';

const SHOTS = new URL('../../../../docs/screenshots/', import.meta.url);
const HDMI_1 = 0x11;

test.skip(process.env.SCREENSHOTS !== '1', 'SCREENSHOTS=1 rewrites docs/screenshots');
test.use({ locale: 'en-US', reducedMotion: 'reduce', deviceScaleFactor: 2 });

function shotPath(name) {
  return fileURLToPath(new URL(`${name}.png`, SHOTS));
}

async function settled(page) {
  await page.evaluate(() => document.fonts.ready);
  await page.evaluate(() => document.activeElement?.blur());
}

test('the RTK popup', async ({ page }, testInfo) => {
  await open(page, '/?demo=rtk');
  await settled(page);

  await page.screenshot({
    path: shotPath(`tray-popup-${testInfo.project.name}`),
    animations: 'disabled',
  });
});

test('the RTK popup asking before switching the input', async ({ page }, testInfo) => {
  await open(page, '/?demo=rtk');
  await settled(page);

  await page.locator(`#input-chips [data-value="${HDMI_1}"]`).click();
  await expect(page.locator('#confirm')).toBeVisible();

  await page.screenshot({
    path: shotPath(`tray-popup-dialog-${testInfo.project.name}`),
    animations: 'disabled',
  });
});

test('the RTK popup with the color preset list open', async ({ page }, testInfo) => {
  await open(page, '/?demo=rtk');
  await settled(page);

  await page.getByRole('combobox', { name: 'Color preset', exact: true }).click();
  await expect(page.getByRole('listbox', { name: 'Color preset', exact: true })).toBeVisible();

  await page.screenshot({
    path: shotPath(`tray-popup-list-${testInfo.project.name}`),
    animations: 'disabled',
  });
});
