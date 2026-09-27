// Every text of the popup comes from a translation or is the monitor's own
// data (D-2026-09-26-tray-app-6, D-2026-09-27-tray-app-7). With `?pseudo=1`
// the demo wraps each translated text in ⟦…⟧, and each text the monitor or
// the core wrote (a monitor's name, a value name no locale translates, a
// VCP code, a backend's message) sits under `translate="no"`. So, in every
// state below, a letter of a visible text outside both reached the page
// from no key — a literal, whatever const, helper or template carried it,
// alone or glued to a translation — and a text under `translate="no"` that
// carries the marks is a translation passed off as data. Readable
// attributes follow the same rule.

import {
  CSP,
  expect,
  expectAccessible,
  loadEnds,
  open,
  pick,
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

function withPseudo(path) {
  return `${path}${path.includes('?') ? '&' : '?'}pseudo=1`;
}

function combobox(page, key) {
  return page.getByRole('combobox', { name: pseudo(key), exact: true });
}

async function openAllSettings(page, features) {
  await page.getByText(pseudo('more.title'), { exact: true }).click();
  await expect(page.locator('#feature-list > li')).toHaveCount(features);
}

async function probe(page) {
  await page.getByRole('button', { name: pseudo('more.probe'), exact: true }).click();
  await expect(page.locator('#probe-results')).toBeVisible();
  await expect(page.locator('#probe')).toHaveAttribute('aria-disabled', 'false');
}

/**
 * The states checked, in pseudo-locale: `path` settles in `state`, then
 * `reach` takes the popup where the check runs. `reach` finds elements and
 * waits for them but never asserts a text: telling a translated text from
 * a literal is the check's job alone. The backend-unavailable state of the
 * app itself is `/?demo=error`'s (same kind, same nodes): outside the demo
 * the pseudo-locale is off, as the last test shows.
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
];

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

for (const { name, path, state, reach } of STATES) {
  test(`every visible text is translated or marked as data: ${name}`, async ({ page }) => {
    await open(page, withPseudo(path), state);
    await reach?.(page);
    await expectEveryTextTranslatedOrData(page);
    await expectAccessible(page);
  });
}

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
