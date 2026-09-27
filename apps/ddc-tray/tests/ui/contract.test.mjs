import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import { createBridge } from '../../src/bridge.js';

// Written by the Rust side from its fakes (plan A-1/A-2): the RTK, and the
// mute TV with the error its panel fails with. The demo must answer exactly
// the same, so the browser shows what the app would.
const fixture = (name) =>
  JSON.parse(readFileSync(new URL(`../fixtures/${name}`, import.meta.url), 'utf8'));
const golden = fixture('contract-rtk.json');
const muteGolden = fixture('contract-mute.json');

const keys = (value) => Object.keys(value).sort();

// The demo answers only on a local dev server (D-2026-09-27-tray-app-6).
function demo(search) {
  return createBridge({ location: new URL(`http://localhost:1420/${search}`) }, { latencyMs: 0 });
}

async function snapshot(bridge, monitorId) {
  return {
    panel: await bridge.loadPanel(monitorId),
    features: await bridge.loadFeatures(monitorId),
  };
}

test('the demo RTK scenario answers exactly the golden contract', async () => {
  const bridge = demo('?demo=rtk');
  const monitors = await bridge.listMonitors();

  const answered = { monitors, ...(await snapshot(bridge, monitors[0].id)) };

  assert.deepStrictEqual(answered, golden);
});

test('the default demo is the golden RTK scenario', async () => {
  const bridge = demo('');
  const monitors = await bridge.listMonitors();

  assert.deepStrictEqual(monitors, golden.monitors);
  assert.deepStrictEqual(await snapshot(bridge, monitors[0].id), {
    panel: golden.panel,
    features: golden.features,
  });
});

test('the demo mute TV is listed first and fails its panel exactly as the Rust side does', async () => {
  const bridge = demo('?demo=two-monitors');
  const monitors = await bridge.listMonitors();

  assert.deepStrictEqual(monitors[0], muteGolden.monitor);
  await assert.rejects(bridge.loadPanel(muteGolden.monitor.id), (error) => {
    assert.deepStrictEqual(error, muteGolden.loadPanel);
    return true;
  });
  const rtkPanel = await bridge.loadPanel(golden.monitors[0].id);
  assert.deepStrictEqual(rtkPanel.controls, golden.panel.controls);
});

test('two-monitors keeps the golden RTK and shapes the other monitor like the contract', async () => {
  const bridge = demo('?demo=two-monitors');
  const monitors = await bridge.listMonitors();
  const rtk = monitors.find((monitor) => monitor.id === golden.monitors[0].id);
  const other = monitors.find((monitor) => monitor.id.startsWith('DEL-'));
  const { panel, features } = await snapshot(bridge, other.id);
  const [goldenControl] = golden.panel.controls;
  const goldenContinuous = golden.panel.controls.find((c) => c.value.kind === 'continuous').value;
  const goldenChoice = golden.panel.controls.find((c) => c.value.kind === 'nonContinuous').value;

  assert.deepStrictEqual(rtk, golden.monitors[0]);
  for (const monitor of monitors) assert.deepEqual(keys(monitor), keys(golden.monitors[0]));
  assert.deepStrictEqual((await snapshot(bridge, rtk.id)).panel, golden.panel);
  assert.deepEqual(keys(other), keys(golden.monitors[0]));
  assert.deepEqual(keys(panel), keys(golden.panel));
  for (const control of panel.controls) {
    assert.deepEqual(keys(control), keys(goldenControl));
    const expected = control.value.kind === 'continuous' ? goldenContinuous : goldenChoice;
    assert.deepEqual(keys(control.value), keys(expected));
    for (const option of control.value.options ?? []) {
      assert.deepEqual(keys(option), ['name', 'value']);
    }
  }
  for (const feature of features) {
    assert.deepEqual(keys(feature), keys(golden.features[0]));
  }
});
