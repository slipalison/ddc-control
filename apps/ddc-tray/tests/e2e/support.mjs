// What every popup spec shares (Gate 7, D-2026-09-26-tray-app-8): each
// response carries the CSP of the Tauri window (D-2026-09-26-tray-app-7),
// so what the webview would refuse fails here too; console errors and
// uncaught errors fail the test; a native <select> anywhere on the page
// fails it too, in every state checked (D-2026-09-27-tray-app-1); the axe
// check; and the texts, read from the locale files the page uses (the
// suite runs in pt-BR, see the config).

import { readFileSync } from 'node:fs';
import AxeBuilder from '@axe-core/playwright';
import { test as base, expect } from '@playwright/test';
import { translator } from '../../src/i18n/index.js';

export { expect };

const TAURI_CONF = new URL('../../src-tauri/tauri.conf.json', import.meta.url);

/** The Content-Security-Policy of the popup window, as Tauri applies it. */
export const CSP = JSON.parse(readFileSync(TAURI_CONF, 'utf8')).app.security.csp;

/** The page's texts, by key, in the locale the suite runs in. */
export const t = translator('pt-BR');

/** The demo's monitor ids (`src/demo-data.js`). */
export const MONITORS = Object.freeze({
  rtk: 'RTK-RTK-QHD-HDR-01010101',
  tv: 'GSM-LG-TV-SSCR2-01010101',
  dell: 'DEL-DELL-U2723QE-7X9K2L3',
});

const AXE_TAGS = ['wcag2a', 'wcag2aa', 'wcag21aa'];
const BLOCKING_IMPACTS = new Set(['critical', 'serious']);

/**
 * Answers `route` with what the static server sent, under the popup's CSP;
 * `patch` rewrites the body of a test-only variant of a file.
 * @param {import('@playwright/test').Route} route
 * @param {{ patch?: (source: string) => string }} [options]
 */
export async function serve(route, { patch = null } = {}) {
  const response = await route.fetch();
  const headers = { ...response.headers(), 'content-security-policy': CSP };
  if (!patch) return route.fulfill({ response, headers });
  return route.fulfill({ response, headers, body: patch(await response.text()) });
}

export const test = base.extend({
  /** Console errors and uncaught errors of the page, in order. */
  pageErrors: async ({}, use) => {
    await use([]);
  },
  page: async ({ page, pageErrors }, use) => {
    page.on('console', (message) => {
      if (message.type() === 'error') pageErrors.push(`console.error: ${message.text()}`);
    });
    page.on('pageerror', (error) => pageErrors.push(`pageerror: ${error.message}`));
    await page.route('**/*', (route) => serve(route));
    await use(page);
    await expectNoNativeSelect(page);
    expect(pageErrors, 'console errors and uncaught page errors').toEqual([]);
  },
});

/**
 * Fails when the page has a native `<select>` (D-2026-09-27-tray-app-1):
 * its menu opens in a window of its own, the popup loses the focus and
 * hides. Run on every state a spec settles in, checks and ends in.
 */
export async function expectNoNativeSelect(page) {
  await expect(page.locator('select'), 'native <select> elements on the page').toHaveCount(0);
}

/**
 * Opens `path` and waits for the popup to settle in `state`
 * (`ready`, `empty` or `error`).
 */
export async function open(page, path, state = 'ready') {
  await page.goto(path);
  await expect(page.locator('#app')).toHaveAttribute('data-state', state);
  await expectNoNativeSelect(page);
}

/**
 * Runs `action` and waits for the load it starts to end: the header's
 * refresh button is busy while any panel load runs.
 */
export async function loadEnds(page, action) {
  await page.evaluate(() => {
    const refresh = document.getElementById('refresh');
    window.__loadEnded = new Promise((resolve) => {
      let started = false;
      new MutationObserver((_, observer) => {
        if (refresh.classList.contains('is-busy')) started = true;
        else if (started) {
          observer.disconnect();
          resolve();
        }
      }).observe(refresh, { attributes: true, attributeFilter: ['class'] });
    });
  });
  await action();
  await page.evaluate(() => window.__loadEnded);
}

/** The writes the demo accepted, in order: `{ monitorId, code, value, confirmed }`. */
export function writes(page) {
  return page.evaluate(() => window.__ddcDemo.writes);
}

/** The monitor the demo was told to target (`select_monitor`). */
export function selectedMonitor(page) {
  return page.evaluate(() => window.__ddcDemo.selected);
}

/** How many times the popup asked to be hidden (`hide_popup`). */
export function hides(page) {
  return page.evaluate(() => window.__ddcDemo.hides);
}

/**
 * Axe on the page as it is now, with the WCAG 2.0 A/AA and 2.1 AA rules:
 * a critical or serious violation fails the test; moderate and minor ones
 * are reported as an annotation and on stdout. A check that passed leaves
 * an `axe` annotation, so the report shows which tests ran it
 * (D-2026-10-01-input-switch-autostart-1).
 */
export async function expectAccessible(page) {
  await expectNoNativeSelect(page);
  const { violations } = await new AxeBuilder({ page }).withTags(AXE_TAGS).analyze();
  const found = violations.map(({ id, impact, help, nodes }) => ({
    id,
    impact,
    help,
    targets: nodes.map((node) => node.target.join(' ')),
  }));
  const minor = found.filter((violation) => !BLOCKING_IMPACTS.has(violation.impact));
  if (minor.length > 0) {
    const description = minor.map(({ impact, id }) => `${impact} ${id}`).join(', ');
    test.info().annotations.push({ type: 'axe moderate/minor', description });
    console.log(`axe moderate/minor: ${JSON.stringify(minor)}`);
  }
  const blocking = found.filter((violation) => BLOCKING_IMPACTS.has(violation.impact));
  expect(blocking, 'axe critical/serious violations').toEqual([]);
  test.info().annotations.push({ type: 'axe', description: 'no critical or serious violation' });
}

// --------------------------------------------------------------- locators

/** A quick-panel slider by its feature key (`brightness`, `contrast`, `volume`). */
export function slider(page, key) {
  return page.getByRole('slider', { name: t(`feature.${key}`), exact: true });
}

/** The input chip named `name` (the core's name, e.g. `HDMI-1`). */
export function inputChip(page, name) {
  return page
    .getByRole('radiogroup', { name: t('feature.input'), exact: true })
    .getByRole('radio', { name, exact: true });
}

export function monitorPicker(page) {
  return dropdown(page, t('header.monitor'));
}

/** An in-page dropdown (D-2026-09-27-tray-app-1) by its label. */
export function dropdown(page, name) {
  return page.getByRole('combobox', { name, exact: true });
}

/** The list of a dropdown, open or not. */
export function listOf(combobox) {
  return combobox.locator('xpath=..').getByRole('listbox', { includeHidden: true });
}

/** The options of a dropdown, open or not, in order. */
export function optionsOf(combobox) {
  return listOf(combobox).locator('[role="option"]');
}

/** Opens a dropdown with a click and clicks its option named `name`. */
export async function pick(combobox, name) {
  await combobox.click();
  await listOf(combobox).getByRole('option', { name, exact: true }).click();
}

/** Asserts the value a dropdown shows, as the string of its value. */
export async function expectPicked(combobox, value) {
  await expect(combobox).toHaveAttribute('data-value', String(value));
}

export function powerButton(page) {
  return page.getByRole('button', { name: t('power.changeLabel'), exact: true });
}

export function confirmDialog(page) {
  return page.getByRole('dialog', { name: t('confirm.title'), exact: true });
}

export function cancelButton(page) {
  return confirmDialog(page).getByRole('button', { name: t('confirm.cancel'), exact: true });
}

export function acceptButton(page) {
  return confirmDialog(page).getByRole('button', { name: t('confirm.accept'), exact: true });
}

export function retryButton(page) {
  return page.getByRole('button', { name: t('action.retry'), exact: true });
}
