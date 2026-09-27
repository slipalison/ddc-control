// Inside the app the popup never shows simulated values
// (D-2026-09-27-tray-app-6): at the origin the Windows webview gives it,
// `http://tauri.localhost`, a page without `window.__TAURI__` reports the
// backend as unavailable instead of falling back to the demo, in the
// user's language alone: no backend answered, so there is no message to
// show as data (D-2026-09-27-tray-app-9). The files are the ones the
// suite's server sends, under the same CSP.

import { CSP, expect, expectAccessible, open, powerButton, retryButton, t, test } from './support.mjs';

const APP_ORIGIN = 'http://tauri.localhost';

// Registered after the fixture's catch-all, so it answers first.
async function serveAsTheApp(page, baseURL) {
  await page.route(`${APP_ORIGIN}/**`, async (route) => {
    const { pathname, search } = new URL(route.request().url());
    const response = await route.fetch({ url: `${baseURL}${pathname}${search}` });
    await route.fulfill({ response, headers: { ...response.headers(), 'content-security-policy': CSP } });
  });
}

for (const path of ['/', '/?demo=rtk']) {
  test(`at the app's origin ${path} settles error, never on the demo`, async ({ page, baseURL }) => {
    await serveAsTheApp(page, baseURL);

    await open(page, `${APP_ORIGIN}${path}`, 'error');

    expect(page.url()).toBe(`${APP_ORIGIN}${path}`);
    expect(await page.evaluate(() => typeof window.__ddcDemo)).toBe('undefined');
    await expect(
      page.getByRole('heading', { name: t('error.backend_unavailable'), exact: true }),
    ).toBeVisible();
    await expect(page.locator('#message-detail')).toBeHidden();
    await expect(page.locator('#message-detail')).toHaveText('');
    await expect(retryButton(page)).toBeVisible();
    await expect(page.locator('#panel')).toBeHidden();
    await expect(powerButton(page)).toBeHidden();
    await expectAccessible(page);
  });
}
