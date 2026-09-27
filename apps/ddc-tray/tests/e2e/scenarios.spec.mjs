// Gate 7's critical paths (D-2026-09-26-tray-app-8): every demo scenario,
// the states a dangerous change opens and the failures the demo's `fail=`
// brings about show what they should with no console error, no uncaught
// error and no critical or serious axe violation.

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
  pick,
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

/** What copying all of `locator` gives: its text as the page selects it. */
function copiedText(locator) {
  return locator.evaluate((node) => {
    const selection = getSelection();
    selection.selectAllChildren(node);
    const text = selection.toString();
    selection.removeAllRanges();
    return text;
  });
}

// On Linux the empty and unavailable states also point at the i2c guide.
// The space before its path is text, so a copied hint keeps it.
async function showsI2cHintOnLinux(page) {
  const hint = page.locator('#message-hint');
  if (process.platform !== 'linux') {
    await expect(hint).toBeHidden();
    return;
  }
  await expect(hint.locator('code')).toHaveText(I2C_DOC);
  expect(await copiedText(hint), 'the hint as copied').toBe(`${t('hint.i2c')} ${I2C_DOC}`);
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

// `fail=` makes the demo time out the commands it names, as the backend
// does when a monitor stops answering. Each failure is told in words, the
// control shows the monitor's value again, and nothing was written.
const TIMED_OUT = t('error.timeout');

async function toastTells(page, text) {
  const toast = page.locator('#toast');
  await expect(toast).toBeVisible();
  await expect(page.locator('#toast-text')).toHaveText(text);
  await expect(page.locator('#announcer')).toHaveText(text);
  await expect(toast.getByRole('button', { name: t('action.retry'), exact: true })).toBeVisible();
}

async function openAllSettings(page) {
  await page.getByText(t('more.title'), { exact: true }).click();
}

function probeButton(page) {
  return page.getByRole('button', { name: t('more.probe'), exact: true });
}

const FAILURES = [
  {
    name: 'a brightness write that timed out',
    fail: 'write',
    act: (page) => slider(page, 'brightness').press('ArrowRight'),
    shows: async (page) => {
      await toastTells(page, TIMED_OUT);
      await expect(slider(page, 'brightness')).toHaveValue('75');
      await expect(slider(page, 'brightness')).toHaveAttribute('aria-valuetext', '75%');
    },
  },
  {
    name: 'a color preset change that timed out',
    fail: 'write',
    act: (page) => pick(dropdown(page, t('feature.preset')), t('value.display-native')),
    shows: async (page) => {
      await toastTells(page, TIMED_OUT);
      await expectPicked(dropdown(page, t('feature.preset')), 0x01);
      await expect(dropdown(page, t('feature.preset'))).toHaveText('sRGB');
    },
  },
  {
    name: '"all settings" failing to load',
    fail: 'features',
    act: openAllSettings,
    shows: async (page) => {
      const status = page.locator('#more-status');
      await expect(status).toHaveAttribute('data-kind', 'error');
      await expect(page.locator('#more-status-text')).toHaveText(TIMED_OUT);
      await expect(status.getByRole('button', { name: t('action.retry'), exact: true })).toBeVisible();
      await expect(page.locator('#feature-list > li')).toHaveCount(0);
      await expect(page.locator('#toast')).toBeHidden();
    },
  },
  {
    name: 'a probe that timed out',
    fail: 'probe',
    act: async (page) => {
      await openAllSettings(page);
      await expect(page.locator('#feature-list > li')).toHaveCount(7);
      await probeButton(page).click();
    },
    shows: async (page) => {
      await toastTells(page, TIMED_OUT);
      await expect(probeButton(page)).toHaveAttribute('aria-disabled', 'false');
      await expect(page.locator('#probe-results')).toBeHidden();
    },
  },
];

for (const { name, fail, act, shows } of FAILURES) {
  test(`/?demo=rtk&fail=${fail} after ${name}, clean console, no serious axe violation`, async ({
    page,
    pageErrors,
  }) => {
    await open(page, `/?demo=rtk&fail=${fail}`);
    await act(page);
    await shows(page);
    await expectAccessible(page);
    expect(await writes(page)).toEqual([]);
    expect(pageErrors).toEqual([]);
  });
}
