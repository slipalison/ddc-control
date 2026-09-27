import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import { createBridge } from '../../src/bridge.js';

// Written by the Rust side from the fake RTK (plan A-1/A-2); the demo must
// answer exactly the same, so the browser shows what the app would.
const golden = JSON.parse(
  readFileSync(new URL('../fixtures/contract-rtk.json', import.meta.url), 'utf8'),
);

const keys = (value) => Object.keys(value).sort();

function demo(search) {
  return createBridge({ location: { search } }, { latencyMs: 0 });
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
