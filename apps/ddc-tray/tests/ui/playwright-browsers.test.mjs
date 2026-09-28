import { test } from 'node:test';
import assert from 'node:assert/strict';

import { installArgs } from '../../scripts/playwright-browsers.mjs';

test('on a developer machine only the browser is installed, never with sudo', () => {
  assert.deepEqual(installArgs({}), ['install', 'chromium']);
});

test('on a self-hosted runner only the browser is installed, never with sudo', () => {
  assert.deepEqual(
    installArgs({ GITHUB_ACTIONS: 'true', RUNNER_ENVIRONMENT: 'self-hosted' }),
    ['install', 'chromium'],
  );
});

test('on a GitHub-hosted runner the system libraries come with the browser', () => {
  assert.deepEqual(
    installArgs({ GITHUB_ACTIONS: 'true', RUNNER_ENVIRONMENT: 'github-hosted' }),
    ['install', '--with-deps', 'chromium'],
  );
});

test('a hosted runner name outside GitHub Actions installs no system library', () => {
  assert.deepEqual(installArgs({ RUNNER_ENVIRONMENT: 'github-hosted' }), [
    'install',
    'chromium',
  ]);
});
