// What the popup tells when the monitor does not do as asked
// (D-2026-09-30-input-switch-autostart-5, -6): a value read back that is not
// the one asked gets a toast naming both — for the input, adding that the
// asked one may have no signal — and an input change whose follow-up fails
// says the monitor may have switched away. A read-back equal to the request
// shows nothing. The demo's writes never reach a monitor; a test-only
// bridge.js keeps the reading of the codes in `window.__ddcKeep`, as
// confirm.spec.mjs does. Each toast's whole text is also checked against
// the sentence written out in the test, so a template that swaps the value
// kept and the value asked fails (D-2026-10-01-input-switch-autostart-3).

import { readFileSync } from 'node:fs';
import { translator } from '../../src/i18n/index.js';
import { MONITORS, expect, expectAccessible, inputChip, open, serve, slider, t, test, writes } from './support.mjs';

const en = translator('en');

const INPUT = 0x60;
const BRIGHTNESS = 0x10;
const HDMI_1 = 0x11;

const WRITE_LINE = 'entry.reading.current = value;';
const KEEPING_LINE = 'if (!globalThis.__ddcKeep?.includes(code)) entry.reading.current = value;';

/** Opens the RTK with a monitor that keeps the reading of `codes` whatever is written to them. */
async function openKeeping(page, codes) {
  const bridge = readFileSync(new URL('../../src/bridge.js', import.meta.url), 'utf8');
  expect(bridge, 'the demo write this test patches').toContain(WRITE_LINE);
  await page.route('**/bridge.js', (route) =>
    serve(route, { patch: (source) => source.replace(WRITE_LINE, KEEPING_LINE) }),
  );
  await open(page, '/?demo=rtk');
  await page.evaluate((kept) => {
    window.__ddcKeep = kept;
  }, codes);
}

/** Picks HDMI-1 in the input group of the page's locale and accepts the confirmation. */
async function switchToHdmi1(page, texts) {
  await page
    .getByRole('radiogroup', { name: texts('feature.input'), exact: true })
    .getByRole('radio', { name: 'HDMI-1', exact: true })
    .click();
  const dialog = page.getByRole('dialog', { name: texts('confirm.title'), exact: true });
  await dialog.getByRole('button', { name: texts('confirm.accept'), exact: true }).click();
  await expect(dialog).toBeHidden();
}

/** The toast is visible and tells exactly `text`. */
async function expectToast(page, text) {
  await expect(page.locator('#toast')).toBeVisible();
  await expect(page.locator('#toast-text')).toHaveText(text);
}

/**
 * The toast names each of `literals`, written out here rather than drawn
 * from the template under test, so a template that drops a name is caught.
 */
async function expectToastNaming(page, literals) {
  for (const literal of literals) {
    await expect(page.locator('#toast-text')).toContainText(literal);
  }
}

const KEPT_AND_ASKED = { kept: 'DisplayPort-1', asked: 'HDMI-1' };
const HDMI_1_WRITE = { monitorId: MONITORS.rtk, code: INPUT, value: HDMI_1, confirmed: true };

test.describe('in English', () => {
  test.use({ locale: 'en-US' });

  test('the toast names the kept and the asked input (en)', async ({ page }) => {
    await openKeeping(page, [INPUT]);

    await switchToHdmi1(page, en);

    await expectToast(page, en('notice.inputKept', KEPT_AND_ASKED));
    await expectToastNaming(page, ['DisplayPort-1', 'HDMI-1']);
    await expect(page.locator('#toast-text')).toContainText('may have no signal');
    await expect(page.locator('#toast-text')).toHaveText(
      'The monitor is still on DisplayPort-1, not HDMI-1. HDMI-1 may have no signal; the monitor goes back to an input that has one.',
    );
    expect(await writes(page)).toEqual([HDMI_1_WRITE]);
    await expectAccessible(page);
  });
});

test('the toast names the kept and the asked input (pt-BR)', async ({ page }) => {
  await openKeeping(page, [INPUT]);

  await switchToHdmi1(page, t);

  await expectToast(page, t('notice.inputKept', KEPT_AND_ASKED));
  await expectToastNaming(page, ['DisplayPort-1', 'HDMI-1']);
  await expect(page.locator('#toast-text')).toContainText('pode estar sem sinal');
  await expect(page.locator('#toast-text')).toHaveText(
    'O monitor continua em DisplayPort-1, não em HDMI-1. HDMI-1 pode estar sem sinal; o monitor volta para uma entrada que tenha.',
  );
  await expect(inputChip(page, 'DisplayPort-1')).toBeChecked();
  expect(await writes(page)).toEqual([HDMI_1_WRITE]);
  await expectAccessible(page);
});

test('an input read back as asked shows no notice', async ({ page }) => {
  await open(page, '/?demo=rtk');

  await switchToHdmi1(page, t);

  await expect(inputChip(page, 'HDMI-1')).toBeChecked();
  await expect(inputChip(page, 'HDMI-1')).not.toHaveClass(/is-pending/);
  await expect(page.locator('#toast')).toBeHidden();
  expect(await writes(page)).toEqual([HDMI_1_WRITE]);
  await expectAccessible(page);
});

test('a failed read after an input write says the monitor may have switched away', async ({
  page,
}) => {
  await open(page, '/?demo=rtk&fail=write');

  await switchToHdmi1(page, t);

  await expectToast(page, t('notice.inputUnread'));
  await expectToastNaming(page, ['troca de entrada', 'pode ter comutado']);
  await expect(page.locator('#toast-text')).toHaveText(
    'O monitor não respondeu depois da troca de entrada. Ele pode ter comutado para uma entrada que este computador não alcança.',
  );
  await expect(inputChip(page, 'DisplayPort-1')).toBeChecked();
  await expect(inputChip(page, 'HDMI-1')).not.toBeChecked();
  expect(await writes(page)).toEqual([]);
  await expectAccessible(page);
});

test('a setting other than the input that the monitor did not apply shows the generic notice', async ({
  page,
}) => {
  await openKeeping(page, [BRIGHTNESS]);

  await slider(page, 'brightness').press('ArrowRight');

  await expectToast(
    page,
    t('notice.kept', {
      feature: t('feature.brightness'),
      kept: t('format.percent', { value: 75 }),
      asked: t('format.percent', { value: 76 }),
    }),
  );
  await expectToastNaming(page, ['Brilho', '75%', '76%']);
  await expect(page.locator('#toast-text')).toHaveText('Brilho: o monitor manteve 75% em vez de 76%.');
  await expect(page.locator('#toast-text')).not.toContainText('sinal');
  expect(await writes(page)).toEqual([
    { monitorId: MONITORS.rtk, code: BRIGHTNESS, value: 76, confirmed: false },
  ]);
  await expectAccessible(page);
});
