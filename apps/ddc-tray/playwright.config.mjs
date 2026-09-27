// Gate 7 (D-2026-09-26-tray-app-8): the popup in demo mode — the bridge's
// in-memory fake, picked because `window.__TAURI__` is absent — served as
// the static files Tauri embeds, in a window the size of the popup, in the
// light and the dark theme.

import { defineConfig } from '@playwright/test';

const PORT = 1420;
const POPUP = Object.freeze({ width: 360, height: 560 });

export default defineConfig({
  testDir: 'tests/e2e',
  fullyParallel: true,
  forbidOnly: true,
  retries: 0,
  reporter: 'list',
  use: {
    browserName: 'chromium',
    baseURL: `http://localhost:${PORT}`,
    viewport: POPUP,
    locale: 'pt-BR',
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
  },
  projects: [
    { name: 'light', use: { colorScheme: 'light' } },
    { name: 'dark', use: { colorScheme: 'dark' } },
  ],
  webServer: {
    command: `python3 -m http.server ${PORT} --directory src`,
    url: `http://localhost:${PORT}/index.html`,
    reuseExistingServer: true,
    stdout: 'ignore',
    stderr: 'ignore',
  },
});
