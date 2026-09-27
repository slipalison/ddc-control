// Every text of the popup comes from a translation or is the monitor's own
// data (D-2026-09-26-tray-app-6, D-2026-09-27-tray-app-7). With `?pseudo=1`
// the demo wraps each translated text in ⟦…⟧, and each text the monitor or
// the core wrote (a monitor's name, a value name no locale translates, a
// VCP code, a backend's message) sits under `translate="no"`. So, in every
// state below, a letter of a visible text outside both reached the page
// from no key — a literal, whatever const, helper or template carried it,
// alone or glued to a translation — and a text under `translate="no"` that
// carries the marks is a translation passed off as data. Readable
// attributes follow the same rule. The states include the failures the
// demo's `&fail=` brings about (a write, "all settings", the probe timing
// out; listening to the tray, hiding the popup refused) and the waits a
// user sees, because their texts show nowhere else: every toast the
// popup's scripts show has a state here (`TOAST_STATES`, held to the
// scripts by tests/ui/toast-states.test.mjs). So does every variant of the
// confirmation dialog — the input's, with its note; power's, before and
// after another mode is picked; the generic one of any other dangerous
// setting, from a list and from a slider — and what only a monitor unlike
// the demo's shows: a code the catalog does not name, a setting that gave
// no value, a list whose current value is not one of its choices.

import { readFileSync } from 'node:fs';
import {
  CSP,
  expect,
  expectAccessible,
  loadEnds,
  open,
  pick,
  serve,
  t,
  test,
} from './support.mjs';

// Written out here rather than imported: a pseudo-locale that stopped
// marking must fail this spec, not redefine what it looks for.
const OPEN_MARK = '⟦';
const CLOSE_MARK = '⟧';

/** The page's text of `key` under the pseudo-locale. */
const pseudo = (key, params) => `${OPEN_MARK}${t(key, params)}${CLOSE_MARK}`;

/** Attributes a person reads or hears. */
const READABLE_ATTRIBUTES = [
  'aria-label',
  'aria-description',
  'aria-roledescription',
  'aria-valuetext',
  'aria-placeholder',
  'title',
  'alt',
  'placeholder',
  'label',
];

/** What `textsOf` reads the page with. */
const READING = Object.freeze({ open: OPEN_MARK, close: CLOSE_MARK, attributes: READABLE_ATTRIBUTES });

const RTK = '/?demo=rtk';
const TWO_MONITORS = '/?demo=two-monitors';
const TV = 'LG TV SSCR2';
const DELL = 'DELL U2723QE';

/**
 * The RTK, with the demo failing the commands `what` names: `write`,
 * `features`, `probe` time out; `events`, `hide` are refused.
 */
const rtkFailing = (what) => `${RTK}&fail=${what}`;

function withPseudo(path) {
  return `${path}${path.includes('?') ? '&' : '?'}pseudo=1`;
}

// Test-only variants of the demo's bridge.js, as slider.spec.mjs serves
// them: the line each replaces must still be there, or the state would
// check a popup that never reached it.
const BRIDGE = readFileSync(new URL('../../src/bridge.js', import.meta.url), 'utf8');
const WAIT_LINE = "await wait(command === 'probe_features' ? latencyMs * PROBE_LATENCY_FACTOR : latencyMs);";
const FEATURES_LINE = 'return features.map(featureDto);';
const MONITORS_LINE = 'const monitors = scenarioMonitors(scenario);';

/**
 * A demo in which `command` never answers, so the popup stays waiting on it
 * with the page's clock running: axe needs its timers.
 */
function holding(command) {
  return { line: WAIT_LINE, by: `if (command === '${command}') await new Promise(() => {}); ${WAIT_LINE}` };
}

/** A demo whose monitor declares nothing besides the quick controls. */
const NOTHING_ELSE_DECLARED = { line: FEATURES_LINE, by: 'return [];' };

/** A demo whose monitors `change` (a function's source) alters before any command. */
function altering(change) {
  return { line: MONITORS_LINE, by: `${MONITORS_LINE} for (const monitor of monitors) (${change})(monitor);` };
}

const PRESET = 0x14;
/** A color preset the RTK does not list. */
const UNLISTED_PRESET = 0x03;

/**
 * Codes the catalog does not name, declared as a monitor unlike the RTK
 * may: the Rust side lists them without alias or name, and dangerous, as
 * any unknown code (`risk_for_code`). One is a slider, one is not
 * supported, one does not answer.
 */
const UNNAMED_CODES = [
  { code: 0xe9, status: 'ok', reading: { current: 20, max: 40 } },
  { code: 0xea, status: 'unsupported', reading: null },
  { code: 0xeb, status: 'unresponsive', reading: null },
].map((entry) => ({ alias: null, name: null, dangerous: true, origin: 'caps', options: null, ...entry }));

const WITH_UNNAMED_CODES = altering(`(monitor) => monitor.features.push(...${JSON.stringify(UNNAMED_CODES)})`);

/** The RTK reading a color preset that is not one of its choices. */
const PRESET_OUTSIDE_ITS_LIST = altering(
  `(monitor) => { const preset = monitor.controls.find(({ code }) => code === ${PRESET}); ` +
    `if (preset) preset.reading.current = ${UNLISTED_PRESET}; }`,
);

async function serveBridge(page, { line, by }) {
  expect(BRIDGE, 'the demo line this state patches').toContain(line);
  await page.route('**/bridge.js', (route) => serve(route, { patch: (source) => source.replace(line, by) }));
}

function combobox(page, key) {
  return page.getByRole('combobox', { name: pseudo(key), exact: true });
}

async function openAllSettings(page, features) {
  await page.getByText(pseudo('more.title'), { exact: true }).click();
  await expect(page.locator('#feature-list > li')).toHaveCount(features);
}

/** Opens "all settings" and waits for its status line to show `kind`. */
async function openAllSettingsTo(page, kind) {
  await page.getByText(pseudo('more.title'), { exact: true }).click();
  await expect(page.locator('#more-status')).toHaveAttribute('data-kind', kind);
  await expect(page.locator('#more-status')).toBeVisible();
}

function probeButton(page) {
  return page.getByRole('button', { name: pseudo('more.probe'), exact: true });
}

async function probe(page) {
  await probeButton(page).click();
  await expect(page.locator('#probe-results')).toBeVisible();
  await expect(page.locator('#probe')).toHaveAttribute('aria-disabled', 'false');
}

async function toastShows(page) {
  await expect(page.locator('#toast')).toBeVisible();
}

/** Waits for the confirmation dialog, its title, body and both buttons. */
async function dialogShows(page) {
  for (const id of ['confirm', 'confirm-title', 'confirm-body', 'confirm-cancel', 'confirm-accept']) {
    await expect(page.locator(`#${id}`)).toBeVisible();
  }
}

/**
 * The states checked, in pseudo-locale: `path` settles in `state`, then
 * `reach` takes the popup where the check runs; `bridge`, when there, is
 * the test-only variant of the demo the state needs. `reach` finds
 * elements and waits for them but never asserts a text: telling a
 * translated text from a literal is the check's job alone. The
 * backend-unavailable state of the app itself is `/?demo=error`'s (same
 * kind, same nodes, less the detail line): outside the demo the
 * pseudo-locale is off, as the last test shows.
 */
const STATES = [
  { name: '/ ready', path: '/', state: 'ready' },
  { name: 'rtk ready', path: RTK, state: 'ready' },
  { name: 'two monitors ready', path: TWO_MONITORS, state: 'ready' },
  { name: 'empty', path: '/?demo=empty', state: 'empty' },
  { name: 'error', path: '/?demo=error', state: 'error' },
  {
    name: 'rtk with the input confirmation open',
    path: RTK,
    state: 'ready',
    reach: async (page) => {
      await page
        .getByRole('radiogroup', { name: pseudo('feature.input'), exact: true })
        .getByRole('radio', { name: 'HDMI-1', exact: true })
        .click();
      await expect(page.locator('#confirm')).toBeVisible();
      await expect(page.locator('#confirm-note')).toBeVisible();
    },
  },
  {
    name: 'rtk with the power confirmation open',
    path: RTK,
    state: 'ready',
    reach: async (page) => {
      await page.getByRole('button', { name: pseudo('power.changeLabel'), exact: true }).click();
      await expect(page.locator('#confirm')).toBeVisible();
      await expect(page.locator('#confirm-choices')).toBeVisible();
    },
  },
  {
    name: 'rtk with the power confirmation open and another mode picked',
    path: RTK,
    state: 'ready',
    reach: async (page) => {
      await page.getByRole('button', { name: pseudo('power.changeLabel'), exact: true }).click();
      await dialogShows(page);
      const other = page.locator('#confirm-choice-list input[type="radio"]').nth(1);
      await other.check();
      await expect(other).toBeChecked();
    },
  },
  {
    name: 'rtk with the generic confirmation of a dangerous list open',
    path: RTK,
    state: 'ready',
    reach: async (page) => {
      await openAllSettings(page, 7);
      await pick(combobox(page, 'feature.osd-lock'), pseudo('value.osd-disabled'));
      await dialogShows(page);
    },
  },
  {
    name: 'rtk with all settings listing codes the catalog does not name',
    path: RTK,
    state: 'ready',
    bridge: WITH_UNNAMED_CODES,
    reach: async (page) => {
      await openAllSettings(page, 7 + UNNAMED_CODES.length);
      await expect(page.locator('#feature-list .feature-status')).toHaveCount(2);
    },
  },
  {
    name: 'rtk with the generic confirmation of a dangerous slider open',
    path: RTK,
    state: 'ready',
    bridge: WITH_UNNAMED_CODES,
    reach: async (page) => {
      await openAllSettings(page, 7 + UNNAMED_CODES.length);
      await page.getByRole('slider', { name: pseudo('format.code', { hex: '0xE9' }), exact: true }).press('ArrowRight');
      await dialogShows(page);
    },
  },
  {
    name: 'rtk with a color preset outside its list, the list open',
    path: RTK,
    state: 'ready',
    bridge: PRESET_OUTSIDE_ITS_LIST,
    reach: async (page) => {
      await combobox(page, 'feature.preset').click();
      const list = page.getByRole('listbox', { name: pseudo('feature.preset'), exact: true });
      await expect(list).toBeVisible();
      await expect(list.locator('[role="option"][aria-disabled="true"]')).toHaveCount(1);
    },
  },
  {
    name: 'rtk with all settings open and probed',
    path: RTK,
    state: 'ready',
    reach: async (page) => {
      await openAllSettings(page, 7);
      await probe(page);
      await expect(page.locator('#probe-list > li')).toHaveCount(2);
      await expect(page.locator('#probe-note')).toBeVisible();
    },
  },
  {
    name: 'rtk with the color preset list open',
    path: RTK,
    state: 'ready',
    reach: async (page) => {
      await combobox(page, 'feature.preset').click();
      await expect(page.getByRole('listbox', { name: pseudo('feature.preset'), exact: true })).toBeVisible();
    },
  },
  {
    name: 'two monitors with the monitor list open',
    path: TWO_MONITORS,
    state: 'ready',
    reach: async (page) => {
      await combobox(page, 'header.monitor').click();
      await expect(page.getByRole('listbox', { name: pseudo('header.monitor'), exact: true })).toBeVisible();
    },
  },
  {
    name: 'two monitors with the mute TV picked',
    path: TWO_MONITORS,
    state: 'ready',
    reach: async (page) => {
      await loadEnds(page, () => pick(combobox(page, 'header.monitor'), pseudo('header.silent', { label: TV })));
      await expect(page.locator('#app')).toHaveAttribute('data-state', 'error');
      await expect(page.locator('#message-detail')).toBeVisible();
    },
  },
  {
    name: 'two monitors with the DELL probed and nothing answering',
    path: TWO_MONITORS,
    state: 'ready',
    reach: async (page) => {
      await loadEnds(page, () => pick(combobox(page, 'header.monitor'), DELL));
      await expect(page.locator('#app')).toHaveAttribute('data-state', 'ready');
      await openAllSettings(page, 1);
      await probe(page);
      await expect(page.locator('#probe-list')).toBeHidden();
      await expect(page.locator('#probe-note')).toBeVisible();
    },
  },
  {
    name: 'rtk after a brightness write that timed out',
    path: rtkFailing('write'),
    state: 'ready',
    reach: async (page) => {
      await page.getByRole('slider', { name: pseudo('feature.brightness'), exact: true }).press('ArrowRight');
      await toastShows(page);
    },
  },
  {
    name: 'rtk after a color preset change that timed out',
    path: rtkFailing('write'),
    state: 'ready',
    reach: async (page) => {
      await pick(combobox(page, 'feature.preset'), pseudo('value.display-native'));
      await toastShows(page);
    },
  },
  {
    name: 'rtk with all settings still loading',
    path: RTK,
    state: 'ready',
    bridge: holding('load_features'),
    reach: (page) => openAllSettingsTo(page, 'loading'),
  },
  {
    name: 'rtk with all settings failing to load',
    path: rtkFailing('features'),
    state: 'ready',
    reach: async (page) => {
      await openAllSettingsTo(page, 'error');
      await expect(page.locator('#more-retry')).toBeVisible();
    },
  },
  {
    name: 'rtk with all settings declaring nothing else',
    path: RTK,
    state: 'ready',
    bridge: NOTHING_ELSE_DECLARED,
    reach: async (page) => {
      await openAllSettingsTo(page, 'empty');
      await expect(page.locator('#feature-list > li')).toHaveCount(0);
    },
  },
  {
    name: 'rtk while probing',
    path: RTK,
    state: 'ready',
    bridge: holding('probe_features'),
    reach: async (page) => {
      await openAllSettings(page, 7);
      await probeButton(page).click();
      await expect(page.locator('#probe')).toHaveAttribute('aria-disabled', 'true');
    },
  },
  {
    name: 'rtk after a probe that timed out',
    path: rtkFailing('probe'),
    state: 'ready',
    reach: async (page) => {
      await openAllSettings(page, 7);
      await probeButton(page).click();
      await toastShows(page);
      await expect(page.locator('#probe')).toHaveAttribute('aria-disabled', 'false');
    },
  },
  {
    name: 'rtk after listening to the tray was refused',
    path: rtkFailing('events'),
    state: 'ready',
    reach: toastShows,
  },
  {
    name: 'rtk after hiding the popup was refused',
    path: rtkFailing('hide'),
    state: 'ready',
    reach: async (page) => {
      await page.keyboard.press('Escape');
      await toastShows(page);
    },
  },
];

/**
 * Every mention of `showToast` in the popup's scripts but its definition,
 * as `file › function it sits in`, with the state above that shows its
 * toast: no other state shows that toast's text. Each state here must show
 * the toast. tests/ui/toast-states.test.mjs reads this list and fails a
 * mention the scripts gain without a state of its own.
 */
const TOAST_STATES = [
  { site: 'app.js › listen', state: 'rtk after listening to the tray was refused' },
  { site: 'app.js › hideOnEscape', state: 'rtk after hiding the popup was refused' },
  { site: 'app.js › writeFailed', state: 'rtk after a brightness write that timed out' },
  { site: 'app.js › probe', state: 'rtk after a probe that timed out' },
];

const SHOWS_A_TOAST = new Set(TOAST_STATES.map(({ state }) => state));

/**
 * What `document` shows that is neither translated nor data, as problems:
 * visible text nodes (outside script and style), readable attributes of
 * any element, and text a style generates (`::before`/`::after`), which
 * no key can reach. Outside `translate="no"` a text must carry the marks,
 * and no letter may be left once every ⟦…⟧ (nested ones too) is taken out:
 * a translation only vouches for what it wraps. The counts keep an empty
 * page from passing.
 */
function textsOf({ open: mark, close, attributes }) {
  // The nearest `translate` attribute decides, as in HTML.
  const isData = (element) =>
    element.closest('[translate]')?.getAttribute('translate').trim().toLowerCase() === 'no';
  const where = (element) =>
    `<${element.localName}${element.id ? `#${element.id}` : ''}${[...element.classList].map((name) => `.${name}`).join('')}>`;
  const report = { texts: 0, marked: 0, data: 0, problems: [] };
  const problem = (what, value, element, why) =>
    report.problems.push(`${what} ${JSON.stringify(value)} in ${where(element)} ${why}`);
  // Innermost first, so a translation inside another one goes too, and a
  // mark left without its pair keeps what follows it outside.
  const escape = (text) => text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
  const innermost = new RegExp(`${escape(mark)}[^${escape(mark)}${escape(close)}]*${escape(close)}`, 'gu');
  const outsideMarks = (value) => {
    let rest = value;
    let before;
    do {
      before = rest;
      rest = rest.replace(innermost, '');
    } while (rest !== before);
    return rest;
  };
  const judge = (what, value, element) => {
    const marked = value.includes(mark);
    const data = isData(element);
    if (marked) report.marked += 1;
    if (data) report.data += 1;
    if (data) {
      if (marked) problem(what, value, element, 'is a translation under translate="no"');
      return;
    }
    if (!marked) {
      problem(what, value, element, 'is neither translated nor under translate="no"');
      return;
    }
    const outside = outsideMarks(value);
    if (/\p{L}/u.test(outside)) {
      problem(what, value, element, `has text outside the marks: ${JSON.stringify(outside)}`);
    }
  };
  const walker = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT);
  for (let node = walker.nextNode(); node; node = walker.nextNode()) {
    const text = node.nodeValue.trim();
    const parent = node.parentElement;
    if (!text || !parent || parent.closest('script, style, template, noscript')) continue;
    if (!parent.checkVisibility({ visibilityProperty: true })) continue;
    report.texts += 1;
    judge('text', text, parent);
  }
  for (const element of document.querySelectorAll('*')) {
    for (const name of attributes) {
      const value = element.getAttribute(name)?.trim();
      if (value) judge(`[${name}]`, value, element);
    }
    for (const pseudoElement of ['::before', '::after']) {
      const { content } = getComputedStyle(element, pseudoElement);
      if (content.startsWith('"') && /\p{L}/u.test(content)) {
        problem(`${pseudoElement} content`, content, element, 'is text a style wrote');
      }
    }
  }
  return report;
}

async function expectEveryTextTranslatedOrData(page) {
  await expect(page).toHaveTitle(pseudo('app.title'));
  const report = await page.evaluate(textsOf, READING);
  expect(report.problems, 'texts that came from no key').toEqual([]);
  expect(report.texts, 'visible text nodes read').toBeGreaterThanOrEqual(3);
  expect(report.marked, 'translated texts read').toBeGreaterThanOrEqual(3);
}

for (const { name, path, state, bridge, reach } of STATES) {
  test(`every visible text is translated or marked as data: ${name}`, async ({ page }) => {
    if (bridge) await serveBridge(page, bridge);
    await open(page, withPseudo(path), state);
    await reach?.(page);
    if (SHOWS_A_TOAST.has(name)) await toastShows(page);
    await expectEveryTextTranslatedOrData(page);
    await expectAccessible(page);
  });
}

test('every toast state is a state checked here, and each has its own', () => {
  const names = STATES.map(({ name }) => name);

  expect([...SHOWS_A_TOAST].filter((state) => !names.includes(state))).toEqual([]);
  expect(SHOWS_A_TOAST.size).toBe(TOAST_STATES.length);
});

// The page's clock stands still, so the demo never answers and the popup
// stays loading. (Axe waits on timers too, so it has no run here.)
test('every visible text is translated or marked as data: loading', async ({ page }) => {
  await page.clock.install({ time: new Date('2026-09-27T12:00:00Z') });
  await page.clock.pauseAt(new Date('2026-09-27T12:00:01Z'));

  await page.goto(withPseudo(RTK));

  await expect(page.locator('#app')).toHaveAttribute('data-state', 'loading');
  await expect(page.locator('#skeleton')).toBeVisible();
  await expectEveryTextTranslatedOrData(page);
});

// Inside the app the bridge is never the demo, and `?pseudo=1` is ignored:
// the popup a user sees is never pseudo-localized.
test('?pseudo=1 is ignored at the app origin: no text is wrapped', async ({ page, baseURL }) => {
  const origin = 'http://tauri.localhost';
  await page.route(`${origin}/**`, async (route) => {
    const { pathname, search } = new URL(route.request().url());
    const response = await route.fetch({ url: `${baseURL}${pathname}${search}` });
    await route.fulfill({ response, headers: { ...response.headers(), 'content-security-policy': CSP } });
  });

  await open(page, `${origin}${withPseudo(RTK)}`, 'error');

  await expect(page).toHaveTitle(t('app.title'));
  const report = await page.evaluate(textsOf, READING);
  expect(report.marked, 'texts wrapped by the pseudo-locale').toBe(0);
  expect(report.texts).toBeGreaterThanOrEqual(3);
});
