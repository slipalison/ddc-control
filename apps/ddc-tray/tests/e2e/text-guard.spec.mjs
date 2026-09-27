// The dev guard (D-2026-09-27-tray-app-11) runs on the demo, where this
// whole suite runs: a text that reaches the page from no translation, and
// not as the monitor's data under `translate="no"`, is a console error,
// and support.mjs fails any test whose page logs one. So every path a test
// walks — the keyboard, a drag, the dialogs, the failures — checks the
// texts it shows. These tests prove the guard is on there, whatever writes
// the text, and off at the app's origin, where the demo never runs. Each
// uses a page of its own, served as the fixture serves it (same files,
// same CSP): the fixture's page would fail on the error provoked here.

import { CSP, expect, serve, test } from './support.mjs';

const RTK = '/?demo=rtk';
const APP_ORIGIN = 'http://tauri.localhost';

/** A page of the test's own, and the console errors and uncaught errors it logs. */
async function watchedPage(context) {
  const page = await context.newPage();
  const errors = [];
  page.on('console', (message) => {
    if (message.type() === 'error') errors.push(message.text());
  });
  page.on('pageerror', (error) => errors.push(`pageerror: ${error.message}`));
  await page.route('**/*', (route) => serve(route));
  return { page, errors };
}

/** Lets the page end the task it is in, and whatever it queued after it. */
function settle(page) {
  return page.evaluate(() => new Promise((resolve) => setTimeout(resolve, 50)));
}

test('on the demo, the guard reports a text no translation produced, and only that', async ({ context }) => {
  const { page, errors } = await watchedPage(context);
  await page.goto(RTK);
  await expect(page.locator('#app')).toHaveAttribute('data-state', 'ready');

  // A translation, the monitor's data, a text without a letter.
  await page.evaluate(() => {
    const translated = document.querySelector('.more-title').textContent;
    document.getElementById('announcer').textContent = translated;
    document.getElementById('message-detail').textContent = 'monitor did not respond in time';
    document.getElementById('toast-text').textContent = '75%';
  });
  await settle(page);
  expect(errors, 'reports of translations, data and numbers').toEqual([]);

  // A literal as the announcer's text, as a readable attribute, and inside
  // an element added to the page.
  await page.evaluate(() => {
    document.getElementById('announcer').textContent = 'Saved';
    document.getElementById('toast-retry').setAttribute('title', 'Reload');
    const note = document.createElement('p');
    note.append('Done');
    document.getElementById('toast').append(note);
  });

  await expect
    .poll(() => errors)
    .toEqual([
      'ddc-tray: untranslated text: "Saved" in <p#announcer.visually-hidden>',
      'ddc-tray: untranslated text: "Reload" in <button#toast-retry.link-button>',
      'ddc-tray: untranslated text: "Done" in <p>',
    ]);
});

test('at the app origin the guard is off: a literal on the page is not reported', async ({ context, baseURL }) => {
  const { page, errors } = await watchedPage(context);
  // Registered after the catch-all, so it answers first.
  await page.route(`${APP_ORIGIN}/**`, async (route) => {
    const { pathname, search } = new URL(route.request().url());
    const response = await route.fetch({ url: `${baseURL}${pathname}${search}` });
    await route.fulfill({ response, headers: { ...response.headers(), 'content-security-policy': CSP } });
  });
  await page.goto(`${APP_ORIGIN}${RTK}`);
  await expect(page.locator('#app')).toHaveAttribute('data-state', 'error');
  expect(await page.evaluate(() => typeof window.__ddcDemo)).toBe('undefined');

  await page.evaluate(() => {
    document.getElementById('announcer').textContent = 'Saved';
    document.getElementById('toast-retry').setAttribute('title', 'Reload');
  });
  await settle(page);

  expect(errors).toEqual([]);
});
