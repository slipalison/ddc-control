// Gate 7's critical paths (D-2026-09-26-tray-app-8): every demo scenario,
// and the states a dangerous change opens, show what they should with no
// console error, no uncaught error and no critical or serious axe violation.

import {
  MONITORS,
  cancelButton,
  confirmDialog,
  dropdown,
  expect,
  expectAccessible,
  expectPicked,
  inputChip,
  monitorPicker,
  open,
  optionsOf,
  powerButton,
  retryButton,
  slider,
  t,
  test,
  writes,
} from './support.mjs';

const I2C_DOC = 'docs/linux-ddc-setup.md';

async function showsRtkPanel(page) {
  await expect(page.locator('#monitor-name')).toHaveText('RTK QHD HDR');
  await expect(page.locator('#monitor-meta')).toHaveText(t('header.meta', { manufacturer: 'RTK' }));
  for (const [key, value] of [
    ['brightness', 75],
    ['contrast', 50],
    ['volume', 30],
  ]) {
    await expect(slider(page, key)).toHaveValue(String(value));
    await expect(slider(page, key)).toHaveAttribute('aria-valuetext', `${value}%`);
  }
  await expect(page.getByRole('radiogroup').getByRole('radio')).toHaveCount(7);
  await expect(inputChip(page, 'DisplayPort-1')).toBeChecked();
  await expectPicked(dropdown(page, t('feature.preset')), 0x01);
  await expect(dropdown(page, t('feature.preset'))).toHaveText('sRGB');
  await expect(powerButton(page)).toBeVisible();
  await expect(monitorPicker(page)).toBeHidden();
  await expect(page.locator('#message')).toBeHidden();
}

async function showsPickerOnRtk(page) {
  await expect(monitorPicker(page)).toBeVisible();
  await expectPicked(monitorPicker(page), MONITORS.rtk);
  await expect(monitorPicker(page)).toHaveText('RTK QHD HDR');
  await expect(optionsOf(monitorPicker(page))).toHaveCount(3);
  await expect(slider(page, 'brightness')).toHaveValue('75');
  await expect(inputChip(page, 'DisplayPort-1')).toBeChecked();
  await expect(page.locator('#message')).toBeHidden();
}

// On Linux the empty and unavailable states also point at the i2c guide.
async function showsI2cHintOnLinux(page) {
  const hint = page.locator('#message-hint');
  if (process.platform === 'linux') await expect(hint.locator('code')).toHaveText(I2C_DOC);
  else await expect(hint).toBeHidden();
}

async function showsEmpty(page) {
  await expect(page.getByRole('heading', { name: t('state.empty'), exact: true })).toBeVisible();
  await expect(page.locator('#message-tip')).toHaveText(t('hint.ddc'));
  await expect(retryButton(page)).toBeVisible();
  await showsI2cHintOnLinux(page);
  await expect(page.locator('#panel')).toBeHidden();
  await expect(powerButton(page)).toBeHidden();
}

async function showsBackendError(page) {
  await expect(
    page.getByRole('heading', { name: t('error.backend_unavailable'), exact: true }),
  ).toBeVisible();
  await expect(page.locator('#message-detail')).toHaveText('no DDC/CI backend could be started (demo)');
  await expect(retryButton(page)).toBeVisible();
  await showsI2cHintOnLinux(page);
  await expect(page.locator('#monitor-meta')).toHaveText(t('header.tagline'));
  await expect(page.locator('#panel')).toBeHidden();
  await expect(powerButton(page)).toBeHidden();
}

// PROJECT.md `frontend.critical_paths`: `/` is the demo's default, `rtk`.
const CRITICAL_PATHS = [
  { path: '/', state: 'ready', shows: showsRtkPanel },
  { path: '/?demo=rtk', state: 'ready', shows: showsRtkPanel },
  { path: '/?demo=two-monitors', state: 'ready', shows: showsPickerOnRtk },
  { path: '/?demo=empty', state: 'empty', shows: showsEmpty },
  { path: '/?demo=error', state: 'error', shows: showsBackendError },
];

for (const { path, state, shows } of CRITICAL_PATHS) {
  test(`${path} settles ${state}, clean console, no serious axe violation`, async ({
    page,
    pageErrors,
  }) => {
    await open(page, path, state);
    await shows(page);
    await expectAccessible(page);
    expect(pageErrors).toEqual([]);
  });
}

// A modal dialog makes the rest of the page inert, so each is its own state.
const DIALOGS = [
  {
    name: 'the input confirmation',
    ask: (page) => inputChip(page, 'HDMI-1').click(),
    body: t('confirm.body.input', { to: 'HDMI-1' }),
  },
  {
    name: 'the power confirmation',
    ask: (page) => powerButton(page).click(),
    body: t('confirm.body.power', { to: t('value.off-dpm') }),
  },
];

for (const { name, ask, body } of DIALOGS) {
  test(`/?demo=rtk with ${name} open, clean console, no serious axe violation`, async ({
    page,
    pageErrors,
  }) => {
    await open(page, '/?demo=rtk');
    await ask(page);
    await expect(confirmDialog(page)).toBeVisible();
    await expect(page.locator('#confirm-body')).toHaveText(body);
    await expect(cancelButton(page)).toBeFocused();
    await expectAccessible(page);
    expect(await writes(page)).toEqual([]);
    expect(pageErrors).toEqual([]);
  });
}

test('/?demo=rtk with "all settings" open and probed, clean console, no serious axe violation', async ({
  page,
  pageErrors,
}) => {
  await open(page, '/?demo=rtk');
  await page.getByText(t('more.title'), { exact: true }).click();
  await expect(page.locator('#feature-list > li')).toHaveCount(7);
  await page.getByRole('button', { name: t('more.probe'), exact: true }).click();
  await expect(page.locator('#probe-list > li')).toHaveCount(2);
  await expect(page.locator('#probe-note')).toHaveText(t('more.probeSilent', { count: 7 }));
  await expectAccessible(page);
  expect(await writes(page)).toEqual([]);
  expect(pageErrors).toEqual([]);
});
