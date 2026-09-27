// The popup's pictures for the README and the PR reviewer, in each theme
// (plan A-7): only with SCREENSHOTS=1, so a Gate 7 run never rewrites the
// committed files. English, like the README; motion off, so a frame never
// catches a transition.

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
  test.skip(testInfo.project.name !== 'light', 'one dialog picture is enough for the README');
  await open(page, '/?demo=rtk');
  await settled(page);

  await page.locator(`#input-chips [data-value="${HDMI_1}"]`).click();
  await expect(page.locator('#confirm')).toBeVisible();

  await page.screenshot({
    path: shotPath(`tray-popup-dialog-${testInfo.project.name}`),
    animations: 'disabled',
  });
});
