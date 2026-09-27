import { test } from 'node:test';
import assert from 'node:assert/strict';

import {
  DEMO_LATENCY_MS,
  UNAVAILABLE_MESSAGE,
  createBridge,
  isLocalDevServer,
  normalizeError,
} from '../../src/bridge.js';
import { SCENARIOS, scenarioName } from '../../src/demo-data.js';

const RTK = 'RTK-RTK-QHD-HDR-01010101';
const DELL = 'DEL-DELL-U2723QE-7X9K2L3';
const TV = 'GSM-LG-TV-SSCR2-01010101';
const INPUT = 0x60;
const BRIGHTNESS = 0x10;
const POWER = 0xd6;

// Where the Playwright suite serves the popup: the only kind of page the
// demo runs on (D-2026-09-27-tray-app-6).
const DEV_SERVER = 'http://localhost:1420/';

// A window without Tauri at `href`, with the browser's parsing of it.
const windowAt = (href) => ({ location: new URL(href) });

function demo(search = '') {
  const win = windowAt(`${DEV_SERVER}${search}`);
  const bridge = createBridge(win, { latencyMs: 0 });
  return { win, bridge };
}

async function rejection(promise) {
  try {
    await promise;
  } catch (error) {
    return error;
  }
  return assert.fail('expected a rejection');
}

const control = (panel, code) => panel.controls.find((c) => c.code === code);

// Every command of the contract, as the popup calls it.
const COMMANDS = Object.freeze({
  list_monitors: (bridge) => bridge.listMonitors(),
  select_monitor: (bridge) => bridge.selectMonitor(RTK),
  load_panel: (bridge) => bridge.loadPanel(RTK),
  load_features: (bridge) => bridge.loadFeatures(RTK),
  probe_features: (bridge) => bridge.probeFeatures(RTK),
  set_feature: (bridge) => bridge.setFeature(RTK, BRIGHTNESS, 40),
  set_feature_confirmed: (bridge) => bridge.setFeature(RTK, INPUT, 0x11, { confirmed: true }),
  hide_popup: (bridge) => bridge.hidePopup(),
});

async function refusals(bridge) {
  const answers = {};
  for (const [name, call] of Object.entries(COMMANDS)) answers[name] = await rejection(call(bridge));
  return answers;
}

const UNAVAILABLE = Object.freeze({ kind: 'backend_unavailable', message: UNAVAILABLE_MESSAGE });

test('outside a local dev server the bridge never falls back to the demo', async () => {
  const pages = [
    'tauri://localhost/index.html',
    'http://tauri.localhost/index.html',
    'https://tauri.localhost/index.html',
    'tauri://localhost/?demo=rtk',
    'http://tauri.localhost/?demo=two-monitors',
  ];

  for (const href of pages) {
    const win = windowAt(href);
    const bridge = createBridge(win, { latencyMs: 0 });

    assert.equal(bridge.mode, 'unavailable', href);
    assert.equal(win.__ddcDemo, undefined, href);
    assert.deepEqual(
      await refusals(bridge),
      Object.fromEntries(Object.keys(COMMANDS).map((name) => [name, UNAVAILABLE])),
      href,
    );
  }
});

test('the demo needs http or https on localhost or 127.0.0.1', () => {
  const dev = [
    'http://localhost:1420/',
    'http://localhost/?demo=error',
    'https://localhost:8443/',
    'http://127.0.0.1:1420/?demo=empty',
  ];
  const elsewhere = [
    'tauri://localhost/',
    'http://tauri.localhost/',
    'https://tauri.localhost/',
    'file:///home/user/ddc-tray/src/index.html',
    'http://192.168.0.10:1420/',
    'http://localhost.example.com/',
    'http://[::1]:1420/',
    'ftp://localhost/',
  ];

  assert.deepEqual(
    dev.filter((href) => !isLocalDevServer(new URL(href))),
    [],
  );
  assert.deepEqual(
    elsewhere.filter((href) => isLocalDevServer(new URL(href))),
    [],
  );
  assert.equal(isLocalDevServer(undefined), false);
  assert.equal(createBridge({}).mode, 'unavailable');
  assert.equal(createBridge(undefined).mode, 'unavailable');
  assert.equal(createBridge(windowAt('http://127.0.0.1:1420/')).mode, 'demo');
});

test('a __TAURI__ without invoke on a dev server is not a demo either', async () => {
  const win = { ...windowAt(DEV_SERVER), __TAURI__: { event: {} } };
  const bridge = createBridge(win, { latencyMs: 0 });

  assert.equal(bridge.mode, 'unavailable');
  assert.equal(win.__ddcDemo, undefined);
  assert.deepEqual(await rejection(bridge.listMonitors()), UNAVAILABLE);
});

test('the unavailable bridge accepts listeners that never hear anything', async () => {
  const bridge = createBridge(windowAt('tauri://localhost/'));
  const heard = [];

  const unlisten = await bridge.onPanelChanged((payload) => heard.push(payload));
  await bridge.onPopupShown((payload) => heard.push(payload));
  unlisten();

  assert.equal(typeof unlisten, 'function');
  assert.deepEqual(heard, []);
});

test('without Tauri the bridge runs the demo, RTK by default', async () => {
  const { win, bridge } = demo();

  assert.equal(bridge.mode, 'demo');
  assert.equal(win.__ddcDemo.scenario, 'rtk');
  assert.deepEqual(
    (await bridge.listMonitors()).map((monitor) => monitor.id),
    [RTK],
  );
});

test('the demo counts the times the popup asked to be hidden', async () => {
  const { win, bridge } = demo();
  assert.equal(win.__ddcDemo.hides, 0);

  assert.equal(await bridge.hidePopup(), null);
  await bridge.hidePopup();

  assert.equal(win.__ddcDemo.hides, 2);
  assert.deepEqual(win.__ddcDemo.writes, []);
});

test('?demo= picks a scenario and falls back to rtk when unknown', () => {
  assert.deepEqual(SCENARIOS, ['rtk', 'two-monitors', 'empty', 'error']);
  assert.equal(scenarioName('?demo=two-monitors'), 'two-monitors');
  assert.equal(scenarioName('?demo=empty&x=1'), 'empty');
  assert.equal(scenarioName('?demo=error'), 'error');
  assert.equal(scenarioName('?demo=nope'), 'rtk');
  assert.equal(scenarioName(undefined), 'rtk');
});

test('two-monitors lists a mute TV first, then both monitors; the DELL has no volume and an unnamed input', async () => {
  const { bridge } = demo('?demo=two-monitors');

  const monitors = await bridge.listMonitors();
  const panel = await bridge.loadPanel(DELL);

  assert.deepEqual(
    monitors.map(({ id, label }) => [id, label]),
    [
      [TV, 'LG TV SSCR2'],
      [RTK, 'RTK QHD HDR'],
      [DELL, 'DELL U2723QE'],
    ],
  );
  assert.deepEqual(
    panel.controls.map((c) => c.key),
    ['brightness', 'contrast', 'input', 'preset', 'power'],
  );
  assert.deepEqual(control(panel, INPUT).value, {
    kind: 'nonContinuous',
    current: 0x11,
    options: [
      { value: 0x0f, name: 'DisplayPort-1' },
      { value: 0x11, name: 'HDMI-1' },
      { value: 0x1b, name: null },
    ],
  });
});

test('the mute TV is listed but every DDC/CI call to it fails over the transport', async () => {
  const { win, bridge } = demo('?demo=two-monitors');
  const mute = {
    kind: 'transport',
    message:
      'transport error: DDC/CI I2C error: Input/output error (os error 5) (gave up after attempt 3 of 3)',
  };

  assert.deepEqual(await rejection(bridge.loadPanel(TV)), mute);
  assert.deepEqual(await rejection(bridge.loadFeatures(TV)), mute);
  assert.deepEqual(await rejection(bridge.probeFeatures(TV)), mute);
  assert.deepEqual(await rejection(bridge.setFeature(TV, BRIGHTNESS, 10)), mute);
  assert.equal(await bridge.selectMonitor(TV), null);
  assert.deepEqual(win.__ddcDemo.writes, []);
  assert.equal((await bridge.loadPanel(RTK)).monitorId, RTK);
});

test('empty lists no monitor', async () => {
  const { win, bridge } = demo('?demo=empty');

  assert.deepEqual(await bridge.listMonitors(), []);
  assert.deepEqual(win.__ddcDemo.writes, []);
});

test('error answers backend_unavailable to every command', async () => {
  const { bridge } = demo('?demo=error');

  const listed = await rejection(bridge.listMonitors());
  const panel = await rejection(bridge.loadPanel(RTK));

  assert.deepEqual(listed, {
    kind: 'backend_unavailable',
    message: 'no DDC/CI backend could be started (demo)',
  });
  assert.deepEqual(panel, listed);
});

test('a dangerous write without confirmation is refused and writes nothing', async () => {
  const { win, bridge } = demo('?demo=rtk');

  const error = await rejection(bridge.setFeature(RTK, INPUT, 0x11));

  assert.deepEqual(error, {
    kind: 'needs_confirmation',
    message: 'writing feature 0x60 is dangerous and was not confirmed',
  });
  assert.deepEqual(win.__ddcDemo.writes, []);
  assert.equal(control(await bridge.loadPanel(RTK), INPUT).value.current, 0x0f);
});

test('a confirmed dangerous write is recorded and answers the read-back', async () => {
  const { win, bridge } = demo('?demo=rtk');

  const readBack = await bridge.setFeature(RTK, INPUT, 0x11, { confirmed: true });

  assert.deepEqual(readBack, { current: 0x11, max: 0x03 });
  assert.deepEqual(win.__ddcDemo.writes, [{ monitorId: RTK, code: INPUT, value: 0x11, confirmed: true }]);
  assert.equal(control(await bridge.loadPanel(RTK), INPUT).value.current, 0x11);
});

test('a safe write needs no confirmation and the panel shows the new value', async () => {
  const { win, bridge } = demo('?demo=rtk');

  const readBack = await bridge.setFeature(RTK, BRIGHTNESS, 42);

  assert.deepEqual(readBack, { current: 42, max: 100 });
  assert.deepEqual(win.__ddcDemo.writes, [{ monitorId: RTK, code: BRIGHTNESS, value: 42, confirmed: false }]);
  assert.deepEqual(control(await bridge.loadPanel(RTK), BRIGHTNESS).value, {
    kind: 'continuous',
    current: 42,
    max: 100,
  });
});

test('values the feature does not accept are refused as invalid_value', async () => {
  const { win, bridge } = demo('?demo=rtk');

  const tooHigh = await rejection(bridge.setFeature(RTK, BRIGHTNESS, 101));
  const notListed = await rejection(bridge.setFeature(RTK, POWER, 0x02, { confirmed: true }));

  assert.deepEqual(tooHigh, {
    kind: 'invalid_value',
    message: 'value 101 for feature 0x10 exceeds its maximum 100',
  });
  assert.deepEqual(notListed, {
    kind: 'invalid_value',
    message: 'value 2 is not an allowed value for feature 0xD6',
  });
  assert.deepEqual(win.__ddcDemo.writes, []);
});

test('an unknown monitor is not_found and an unknown code is unsupported', async () => {
  const { win, bridge } = demo('?demo=rtk');

  assert.deepEqual(await rejection(bridge.loadPanel('gone')), {
    kind: 'not_found',
    message: 'monitor gone not found',
  });
  assert.deepEqual(await rejection(bridge.selectMonitor('gone')), {
    kind: 'not_found',
    message: 'monitor gone not found',
  });
  assert.deepEqual(await rejection(bridge.setFeature(RTK, 0x20, 1, { confirmed: true })), {
    kind: 'unsupported',
    message: 'feature 0x20 is not supported',
  });
  assert.equal(await bridge.selectMonitor(RTK), null);
  assert.equal(win.__ddcDemo.selected, RTK);
});

test('the RTK probe lists the undeclared codes with their status', async () => {
  const { bridge } = demo('?demo=rtk');

  const probed = await bridge.probeFeatures(RTK);

  assert.deepEqual(
    probed.map(({ code, origin, status }) => [code, origin, status]),
    [
      [0x1e, 'probe', 'ok'],
      [0x20, 'probe', 'unsupported'],
      [0x30, 'probe', 'unsupported'],
      [0x6c, 'probe', 'ok'],
      [0x6e, 'probe', 'unsupported'],
      [0x70, 'probe', 'unsupported'],
      [0x7e, 'probe', 'unresponsive'],
      [0xe6, 'probe', 'unsupported'],
      [0xf1, 'probe', 'unsupported'],
    ],
  );
  assert.deepEqual(probed[3].value, { kind: 'continuous', current: 50, max: 100 });
  assert.equal(probed[1].value, null);
});

test('answers are copies the page cannot use to change the demo', async () => {
  const { bridge } = demo('?demo=rtk');

  const first = await bridge.loadPanel(RTK);
  first.controls[0].value.current = 0;
  first.controls[3].value.options.length = 0;
  const second = await bridge.loadPanel(RTK);

  assert.equal(second.controls[0].value.current, 75);
  assert.equal(second.controls[3].value.options.length, 7);
});

test('demo events reach their listeners until they unlisten', async () => {
  const { win, bridge } = demo('?demo=rtk');
  const changed = [];
  const shown = [];

  const unlisten = await bridge.onPanelChanged((payload) => changed.push(payload));
  await bridge.onPopupShown((payload) => shown.push(payload));
  win.__ddcDemo.emit('panel-changed', { monitorId: RTK });
  win.__ddcDemo.emit('popup-shown');
  unlisten();
  win.__ddcDemo.emit('panel-changed', { monitorId: RTK });

  assert.deepEqual(changed, [{ monitorId: RTK }]);
  assert.deepEqual(shown, [null]);
});

test('the demo answers only after its latency', async (t) => {
  t.mock.timers.enable({ apis: ['setTimeout'] });
  const bridge = createBridge(windowAt(DEV_SERVER));
  let answered = null;

  const pending = bridge.listMonitors().then((monitors) => {
    answered = monitors;
  });
  await Promise.resolve();
  assert.equal(answered, null);
  t.mock.timers.tick(DEMO_LATENCY_MS);
  await pending;

  assert.deepEqual(
    answered.map((monitor) => monitor.id),
    [RTK],
  );
});

test('with Tauri the bridge invokes the commands with camelCase arguments', async () => {
  const invoked = [];
  const win = {
    __TAURI__: {
      core: { invoke: async (command, args) => invoked.push([command, args]) && null },
      event: { listen: async () => () => {} },
    },
  };
  const bridge = createBridge(win);

  await bridge.listMonitors();
  await bridge.selectMonitor(RTK);
  await bridge.loadPanel(RTK);
  await bridge.loadFeatures(RTK);
  await bridge.probeFeatures(RTK);
  await bridge.setFeature(RTK, INPUT, 0x11, { confirmed: true });
  await bridge.setFeature(RTK, BRIGHTNESS, 30);
  await bridge.hidePopup();

  assert.equal(bridge.mode, 'tauri');
  assert.equal(win.__ddcDemo, undefined);
  assert.deepEqual(invoked, [
    ['list_monitors', undefined],
    ['select_monitor', { monitorId: RTK }],
    ['load_panel', { monitorId: RTK }],
    ['load_features', { monitorId: RTK }],
    ['probe_features', { monitorId: RTK }],
    ['set_feature', { monitorId: RTK, code: INPUT, value: 0x11, confirmed: true }],
    ['set_feature', { monitorId: RTK, code: BRIGHTNESS, value: 30, confirmed: false }],
    ['hide_popup', undefined],
  ]);
});

test('Tauri rejections become { kind, message }', async () => {
  const failures = [
    { kind: 'timeout', message: 'monitor did not respond in time' },
    'command set_feature not allowed',
  ];
  const win = {
    __TAURI__: {
      core: {
        invoke: async () => {
          throw failures.shift();
        },
      },
      event: { listen: async () => () => {} },
    },
  };
  const bridge = createBridge(win);

  assert.deepEqual(await rejection(bridge.listMonitors()), {
    kind: 'timeout',
    message: 'monitor did not respond in time',
  });
  assert.deepEqual(await rejection(bridge.listMonitors()), {
    kind: 'unknown',
    message: 'command set_feature not allowed',
  });
  assert.deepEqual(normalizeError(new Error('boom')), { kind: 'unknown', message: 'boom' });
});

test('Tauri events hand their payload to the listener', async () => {
  const listened = [];
  const win = {
    __TAURI__: {
      core: { invoke: async () => null },
      event: {
        listen: async (event, callback) => {
          listened.push(event);
          callback({ event, payload: { monitorId: RTK } });
          return () => {};
        },
      },
    },
  };
  const bridge = createBridge(win);
  const payloads = [];

  await bridge.onPanelChanged((payload) => payloads.push(payload));
  await bridge.onPopupShown(() => {});

  assert.deepEqual(listened, ['panel-changed', 'popup-shown']);
  assert.deepEqual(payloads, [{ monitorId: RTK }]);
});
