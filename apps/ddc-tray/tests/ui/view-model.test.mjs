import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import { translator } from '../../src/i18n/index.js';
import {
  I2C_DOC,
  LAST_MONITOR_KEY,
  confirmView,
  controlView,
  detectPlatform,
  errorText,
  featureView,
  featuresView,
  firstAnswering,
  monitorOrder,
  monitorPicker,
  panelView,
  readBackNotice,
  recallMonitor,
  rememberMonitor,
  statusView,
  storageOf,
  withReadBack,
  writeFailureText,
} from '../../src/view-model.js';

const golden = JSON.parse(
  readFileSync(new URL('../fixtures/contract-rtk.json', import.meta.url), 'utf8'),
);
const en = translator('en');
const pt = translator('pt-BR');

const brightness = (current, max) => ({
  code: 0x10,
  key: 'brightness',
  dangerous: false,
  value: { kind: 'continuous', current, max },
});
const labels = (options) => options.map((option) => option.label);

test('brightness, contrast and volume are sliders with percent and value text', () => {
  const { sliders } = panelView(golden.panel, en);

  assert.deepEqual(
    sliders.map(({ code, label, current, max, percent, valueText, widget }) => [
      code,
      label,
      current,
      max,
      percent,
      valueText,
      widget,
    ]),
    [
      [0x10, 'Brightness', 75, 100, 75, '75%', 'slider'],
      [0x12, 'Contrast', 50, 100, 50, '50%', 'slider'],
      [0x62, 'Volume', 30, 100, 30, '30%', 'slider'],
    ],
  );
});

test('a slider whose maximum is not 100 reads as a fraction with a rounded percent', () => {
  assert.deepEqual(controlView(brightness(40, 80), pt), {
    code: 0x10,
    key: 'brightness',
    label: 'Brilho',
    labelVerbatim: false,
    dangerous: false,
    widget: 'slider',
    current: 40,
    max: 80,
    percent: 50,
    valueText: '40 de 80',
  });
  assert.equal(controlView(brightness(1, 3), en).percent, 33);
  assert.equal(controlView(brightness(0, 0), en).percent, 0);
});

test('the input is a segmented choice with the current source selected', () => {
  const { input } = panelView(golden.panel, pt);

  assert.equal(input.label, 'Entrada');
  assert.equal(input.labelVerbatim, false);
  assert.equal(input.dangerous, true);
  assert.equal(input.currentLabel, 'DisplayPort-1');
  assert.equal(input.currentVerbatim, true);
  assert.deepEqual(labels(input.options), [
    'VGA-1',
    'DVI-1',
    'DVI-2',
    'DisplayPort-1',
    'DisplayPort-2',
    'HDMI-1',
    'HDMI-2',
  ]);
  assert.deepEqual(
    input.options.filter((option) => option.selected).map((option) => option.value),
    [0x0f],
  );
});

test('value names translate through their key and keep the core name otherwise', () => {
  const [preset] = panelView(golden.panel, pt).selects;

  assert.equal(preset.label, 'Predefinição de cor');
  assert.equal(preset.currentLabel, 'sRGB');
  assert.deepEqual(labels(preset.options), [
    'sRGB',
    'Nativo',
    '5000 K',
    '6500 K',
    '7500 K',
    '9300 K',
    'Usuário 1',
  ]);
  assert.deepEqual(labels(panelView(golden.panel, en).selects[0].options).slice(0, 2), ['sRGB', 'Native']);
});

// A name shown as the core wrote it is the monitor's data, never a
// translation: the page marks it `translate="no"` (D-2026-09-27-tray-app-7).
test('a value name no key translates is verbatim, a translated one is not', () => {
  const {
    selects: [preset],
    input,
    power,
  } = panelView(golden.panel, pt);

  assert.deepEqual(
    preset.options.map(({ label, verbatim }) => [label, verbatim]),
    [
      ['sRGB', true],
      ['Nativo', false],
      ['5000 K', true],
      ['6500 K', true],
      ['7500 K', true],
      ['9300 K', true],
      ['Usuário 1', false],
    ],
  );
  assert.equal(preset.currentVerbatim, true);
  assert.ok(input.options.every((option) => option.verbatim), 'input names are the core\'s');
  assert.equal(power.currentVerbatim, false);
  assert.ok(power.options.every((option) => !option.verbatim), 'power modes are translated');
});

test('a value the core does not name shows its byte', () => {
  const input = controlView(
    {
      code: 0x60,
      key: 'input',
      dangerous: true,
      value: { kind: 'nonContinuous', current: 0x2a, options: [{ value: 0x1b, name: null }] },
    },
    pt,
  );

  assert.deepEqual(input.options, [{ value: 0x1b, label: 'Valor 0x1B', verbatim: false, selected: false }]);
  assert.equal(input.currentLabel, 'Valor 0x2A');
  assert.equal(input.currentVerbatim, false);
});

test('power is a button offering the modes other than the current one', () => {
  const { power } = panelView(golden.panel, pt);

  assert.equal(power.label, 'Energia');
  assert.equal(power.dangerous, true);
  assert.equal(power.currentLabel, 'Ligado');
  assert.deepEqual(power.choices, [
    { value: 0x04, label: 'Em espera (DPM)', verbatim: false, selected: false },
    { value: 0x05, label: 'Desligado (botão de energia)', verbatim: false, selected: false },
  ]);
});

test('a panel missing controls leaves their slots empty', () => {
  const view = panelView({ monitorId: 'm1', controls: [brightness(10, 100)] }, en);

  assert.deepEqual(
    { ...view, sliders: view.sliders.map((slider) => slider.code) },
    { monitorId: 'm1', sliders: [0x10], input: null, selects: [], power: null },
  );
});

test('all settings are labelled by their feature key, with widgets by kind', () => {
  const views = featuresView(golden.features, pt);

  assert.deepEqual(
    views.map(({ hex, label, widget, dangerous }) => [hex, label, widget, dangerous]),
    [
      ['0x0C', 'Temperatura de cor', 'slider', false],
      ['0x16', 'Ganho de vermelho', 'slider', false],
      ['0x18', 'Ganho de verde', 'slider', false],
      ['0x1A', 'Ganho de azul', 'slider', false],
      ['0x87', 'Nitidez', 'slider', false],
      ['0xCA', 'Menu na tela', 'choice', true],
      ['0xCC', 'Idioma do menu', 'choice', false],
    ],
  );
  assert.equal(views[4].valueText, '5 de 10');
  assert.equal(views[5].currentLabel, 'Ativado');
  assert.equal(views[6].currentLabel, 'Inglês');
  assert.deepEqual(views.map((view) => view.statusText), Array(7).fill(null));
  assert.deepEqual(views.map((view) => view.labelVerbatim), Array(7).fill(false));
});

test('a feature without a label key keeps the core name, and a failed reading has no widget', () => {
  const manufacturer = {
    code: 0xe6,
    alias: null,
    name: 'Manufacturer specific (0xE6)',
    dangerous: true,
    origin: 'probe',
    status: 'unsupported',
    value: null,
  };

  assert.deepEqual(featureView(manufacturer, pt), {
    code: 0xe6,
    hex: '0xE6',
    key: null,
    label: 'Manufacturer specific (0xE6)',
    labelVerbatim: true,
    dangerous: true,
    origin: 'probe',
    originText: 'Encontrado na sondagem',
    status: 'unsupported',
    statusText: 'Não suportado',
    widget: null,
  });
  const unknownAlias = { ...manufacturer, alias: 'mystery', name: 'Mystery', status: 'unresponsive' };
  assert.equal(featureView(unknownAlias, en).label, 'Mystery');
  assert.equal(featureView(unknownAlias, en).labelVerbatim, true);
  assert.equal(featureView(unknownAlias, en).statusText, 'No response');
  assert.equal(featureView({ ...manufacturer, name: null }, en).label, 'Setting 0xE6');
  assert.equal(featureView({ ...manufacturer, name: null }, en).labelVerbatim, false);
});

test('the value read back replaces the shown one, a list value by its low byte', () => {
  const input = golden.panel.controls.find((control) => control.key === 'input');

  assert.deepEqual(withReadBack(brightness(75, 100), { current: 42, max: 100 }).value, {
    kind: 'continuous',
    current: 42,
    max: 100,
  });
  const switched = withReadBack(input, { current: 0x0111, max: 0x03 });
  assert.equal(switched.value.current, 0x11);
  assert.deepEqual(switched.value.options, input.value.options);
  assert.equal(controlView(switched, en).currentLabel, 'HDMI-1');
});

// D-2026-09-30-input-switch-autostart-5: the monitor may keep a value other
// than the asked one, and the popup says so instead of only flagging it.
const inputControl = golden.panel.controls.find((control) => control.key === 'input');
const INPUT_CODE = 0x60;
const DISPLAYPORT_1 = 0x0f;
const DISPLAYPORT_2 = 0x10;

// The notices for DisplayPort-1 kept instead of DisplayPort-2, and for 75%
// kept instead of 80%, written out in full rather than drawn from the
// templates under test: a template that swaps the value kept and the value
// asked fails on them (D-2026-10-01-input-switch-autostart-3).
const INPUT_KEPT_EN =
  'The monitor is still on DisplayPort-1, not DisplayPort-2. DisplayPort-2 may have no signal; the monitor goes back to an input that has one.';
const INPUT_KEPT_PT =
  'O monitor continua em DisplayPort-1, não em DisplayPort-2. DisplayPort-2 pode estar sem sinal; o monitor volta para uma entrada que tenha.';
const KEPT_EN = 'Brightness: the monitor kept 75% instead of 80%.';
const KEPT_PT = 'Brilho: o monitor manteve 75% em vez de 80%.';

test('a read-back that differs from the request gives a notice naming both values', () => {
  const slider = readBackNotice(brightness(75, 100), { asked: 80, readBack: { current: 75, max: 100 } }, en);
  const input = readBackNotice(
    inputControl,
    { asked: DISPLAYPORT_2, readBack: { current: DISPLAYPORT_1, max: 0x12 } },
    pt,
  );

  assert.equal(slider, 'Brightness: the monitor kept 75% instead of 80%.');
  assert.match(input, /DisplayPort-1/);
  assert.match(input, /DisplayPort-2/);
  assert.equal(input, INPUT_KEPT_PT);
  assert.equal(
    readBackNotice(inputControl, { asked: DISPLAYPORT_2, readBack: { current: DISPLAYPORT_1, max: 0x12 } }, en),
    INPUT_KEPT_EN,
  );
  assert.equal(slider, KEPT_EN);
  assert.equal(readBackNotice(brightness(75, 100), { asked: 80, readBack: { current: 75, max: 100 } }, pt), KEPT_PT);
});

test('the input notice says the asked input may have no signal', () => {
  const change = { asked: DISPLAYPORT_2, readBack: { current: DISPLAYPORT_1, max: 0x12 } };
  const generic = readBackNotice(brightness(75, 100), { asked: 80, readBack: { current: 75, max: 100 } }, en);

  assert.match(readBackNotice(inputControl, change, en), /DisplayPort-2 may have no signal/);
  assert.match(readBackNotice(inputControl, change, pt), /DisplayPort-2 pode estar sem sinal/);
  assert.doesNotMatch(generic, /signal/);
  assert.doesNotMatch(readBackNotice(brightness(75, 100), { asked: 80, readBack: { current: 75, max: 100 } }, pt), /sinal/);
  assert.equal(readBackNotice(inputControl, change, en), INPUT_KEPT_EN);
  assert.equal(readBackNotice(inputControl, change, pt), INPUT_KEPT_PT);
  assert.equal(generic, KEPT_EN);
  assert.equal(readBackNotice(brightness(75, 100), { asked: 80, readBack: { current: 75, max: 100 } }, pt), KEPT_PT);
});

test('a read-back equal to the request gives no notice', () => {
  const same = readBackNotice(brightness(75, 100), { asked: 75, readBack: { current: 75, max: 100 } }, en);
  const exact = readBackNotice(inputControl, { asked: DISPLAYPORT_2, readBack: { current: DISPLAYPORT_2, max: 0x12 } }, en);
  const highByte = readBackNotice(inputControl, { asked: 0x11, readBack: { current: 0x0111, max: 0x12 } }, en);

  assert.deepEqual([same, exact, highByte], [null, null, null]);
});

test('a failed read after an input write says the monitor may have switched away', () => {
  for (const kind of ['timeout', 'transport', 'not_found']) {
    const error = { kind, message: 'no answer' };

    assert.match(writeFailureText(error, INPUT_CODE, en), /may have switched/, kind);
    assert.match(writeFailureText(error, INPUT_CODE, pt), /pode ter comutado/, kind);
    assert.equal(
      writeFailureText(error, INPUT_CODE, en),
      'The monitor did not answer after the input change. It may have switched to an input this computer cannot reach.',
      kind,
    );
    assert.equal(
      writeFailureText(error, INPUT_CODE, pt),
      'O monitor não respondeu depois da troca de entrada. Ele pode ter comutado para uma entrada que este computador não alcança.',
      kind,
    );
  }
  const refused = { kind: 'invalid_value', message: 'value 2 is not an allowed value for feature 0x60' };
  assert.equal(writeFailureText(refused, INPUT_CODE, en), errorText(refused, en));
  for (const kind of ['unsupported', 'needs_confirmation', 'backend_unavailable', 'weird']) {
    assert.equal(writeFailureText({ kind, message: '' }, INPUT_CODE, pt), errorText({ kind, message: '' }, pt), kind);
  }
  const timeout = { kind: 'timeout', message: '' };
  assert.equal(writeFailureText(timeout, 0x10, en), errorText(timeout, en));
  assert.equal(writeFailureText(null, INPUT_CODE, en), errorText(null, en));
});

test('loading and ready states offer no retry', () => {
  assert.deepEqual(statusView({ state: 'loading' }, { t: pt, platform: 'linux' }), {
    state: 'loading',
    busy: true,
    message: 'Procurando monitores…',
    detail: null,
    retry: false,
    hint: null,
  });
  assert.deepEqual(statusView({ state: 'ready' }, { t: en, platform: 'linux' }), {
    state: 'ready',
    busy: false,
    message: null,
    detail: null,
    retry: false,
    hint: null,
  });
});

test('empty offers retry and, only on Linux, the i2c setup hint', () => {
  const linux = statusView({ state: 'empty' }, { t: en, platform: 'linux' });
  const windows = statusView({ state: 'empty' }, { t: en, platform: 'windows' });

  assert.equal(I2C_DOC, 'docs/linux-ddc-setup.md');
  assert.deepEqual(linux, {
    state: 'empty',
    busy: false,
    message: 'No monitor with DDC/CI was found.',
    detail: null,
    retry: true,
    hint: {
      text: 'On Linux, DDC/CI needs the i2c-dev module and read/write access to /dev/i2c-*. Setup guide:',
      doc: 'docs/linux-ddc-setup.md',
    },
  });
  assert.deepEqual(windows, { ...linux, hint: null });
});

test('errors show their text and detail, with the hint only when i2c may be the cause', () => {
  const unavailable = { kind: 'backend_unavailable', message: 'no /dev/i2c-* device' };
  const refused = { kind: 'invalid_value', message: 'value 2 is not an allowed value for feature 0x60' };

  const linux = statusView({ state: 'error', error: unavailable }, { t: pt, platform: 'linux' });
  const other = statusView({ state: 'error', error: refused }, { t: pt, platform: 'linux' });

  assert.equal(linux.message, 'O controle de monitores não está disponível neste computador.');
  assert.equal(linux.detail, 'no /dev/i2c-* device');
  assert.equal(linux.retry, true);
  assert.equal(linux.hint.doc, I2C_DOC);
  assert.equal(other.message, 'O monitor não aceita este valor.');
  assert.equal(other.hint, null);
  assert.equal(statusView({ state: 'error', error: unavailable }, { t: pt, platform: 'windows' }).hint, null);
});

test('an error of an unknown kind reads as a generic one', () => {
  assert.equal(errorText({ kind: 'timeout', message: '' }, pt), 'O monitor não respondeu a tempo.');
  assert.equal(errorText({ kind: 'weird', message: '' }, pt), 'Algo deu errado.');
  assert.equal(errorText(null, en), 'Something went wrong.');
});

test('the confirmation says what the dangerous change does', () => {
  assert.deepEqual(confirmView({ key: 'input', label: 'Entrada', toLabel: 'HDMI-1' }, pt), {
    title: 'Confirmar alteração',
    body: 'Mudar a entrada para HDMI-1? A tela passará a mostrar essa fonte e pode ficar escura se ela não tiver sinal.',
    accept: 'Aplicar',
    cancel: 'Cancelar',
  });
  assert.match(confirmView({ key: 'power', label: 'Power', toLabel: 'Off (power button)' }, en).body, /^Set power to Off \(power button\)\?/);
  assert.equal(
    confirmView({ key: 'osd-lock', label: 'On-screen menu', toLabel: 'Disabled' }, en).body,
    'Set On-screen menu to Disabled? This setting can change how the monitor behaves.',
  );
});

test('the platform comes from what the webview reports', () => {
  const cases = [
    [{ platform: 'Linux x86_64' }, 'linux'],
    [{ platform: 'Win32' }, 'windows'],
    [{ userAgentData: { platform: 'Windows' }, platform: '' }, 'windows'],
    [{ userAgent: 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/605.1.15' }, 'linux'],
    [{ platform: 'MacIntel', userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) Darwin' }, 'other'],
    [{ platform: '', userAgent: 'Darwin/23.0' }, 'other'],
    [undefined, 'other'],
  ];
  for (const [nav, expected] of cases) {
    assert.equal(detectPlatform(nav), expected, JSON.stringify(nav));
  }
});

test('the picker appears only with a choice and marks the selected monitor', () => {
  const monitors = [
    { id: 'a', label: 'RTK QHD HDR' },
    { id: 'b', label: 'DELL U2723QE' },
  ];

  assert.deepEqual(monitorPicker(monitors, 'b'), {
    selectable: true,
    current: { id: 'b', label: 'DELL U2723QE' },
    options: [
      { id: 'a', label: 'RTK QHD HDR', selected: false, silent: false },
      { id: 'b', label: 'DELL U2723QE', selected: true, silent: false },
    ],
  });
  assert.equal(monitorPicker(monitors.slice(0, 1), 'a').selectable, false);
});

const TV = 'GSM-LG-TV-SSCR2-01010101';
const RTK = 'RTK-RTK-QHD-HDR-01010101';
const DELL = 'DEL-DELL-U2723QE-7X9K2L3';
const listed = [TV, RTK, DELL].map((id) => ({ id, label: id }));

test('a monitor that did not answer stays in the picker, marked silent', () => {
  const picker = monitorPicker(listed, RTK, new Set([TV]));

  assert.deepEqual(
    picker.options.map(({ id, selected, silent }) => [id, selected, silent]),
    [
      [TV, false, true],
      [RTK, true, false],
      [DELL, false, false],
    ],
  );
});

test('monitors are tried from the remembered one, then in the order listed', () => {
  assert.deepEqual(monitorOrder(listed, DELL), [DELL, TV, RTK]);
  assert.deepEqual(monitorOrder(listed, TV), [TV, RTK, DELL]);
  assert.deepEqual(monitorOrder(listed, 'unplugged'), [TV, RTK, DELL]);
  assert.deepEqual(monitorOrder(listed, null), [TV, RTK, DELL]);
  assert.deepEqual(monitorOrder([], RTK), []);
});

// A load that answers for `answering` ids and times out for the others,
// recording the order it was asked in.
function loader(answering) {
  const asked = [];
  const load = async (id) => {
    asked.push(id);
    if (answering.includes(id)) return { monitorId: id };
    throw { kind: 'timeout', message: `${id} did not respond in time` };
  };
  return { asked, load };
}

test('the first monitor that answers is shown, after the silent ones before it', async () => {
  const { asked, load } = loader([RTK, DELL]);

  const found = await firstAnswering([TV, RTK, DELL], load);

  assert.equal(found.monitorId, RTK);
  assert.deepEqual(found.panel, { monitorId: RTK });
  assert.deepEqual(asked, [TV, RTK]);
  assert.deepEqual([...found.failures.keys()], [TV]);
  assert.equal(found.failures.get(TV).kind, 'timeout');
});

test('the remembered monitor answers first and nothing else is read', async () => {
  const { asked, load } = loader([TV, RTK, DELL]);

  const found = await firstAnswering(monitorOrder(listed, DELL), load);

  assert.equal(found.monitorId, DELL);
  assert.deepEqual(asked, [DELL]);
  assert.equal(found.failures.size, 0);
});

test('only when no monitor answers is there nothing to show', async () => {
  const { asked, load } = loader([]);

  const found = await firstAnswering([TV, RTK], load);

  assert.deepEqual({ monitorId: found.monitorId, panel: found.panel }, { monitorId: null, panel: null });
  assert.deepEqual(asked, [TV, RTK]);
  assert.deepEqual([...found.failures.keys()], [TV, RTK]);
});

test('a stale search stops before trying the next monitor', async () => {
  const { asked, load } = loader([DELL]);
  let stale = false;

  const found = await firstAnswering([TV, RTK, DELL], async (id) => {
    stale = true;
    return load(id);
  }, { stop: () => stale });

  assert.equal(found.monitorId, null);
  assert.deepEqual(asked, [TV]);
});

function memoryStorage() {
  const items = new Map();
  return {
    items,
    getItem: (key) => (items.has(key) ? items.get(key) : null),
    setItem: (key, value) => items.set(key, String(value)),
  };
}

const refusing = {
  getItem: () => {
    throw new Error('SecurityError');
  },
  setItem: () => {
    throw new Error('QuotaExceededError');
  },
};

test('the last monitor that answered is remembered between runs', () => {
  const storage = memoryStorage();

  assert.equal(recallMonitor(storage), null);
  assert.equal(rememberMonitor(storage, RTK), true);
  assert.equal(recallMonitor(storage), RTK);
  assert.deepEqual([...storage.items], [[LAST_MONITOR_KEY, RTK]]);
});

test('a storage that refuses only loses the memory', () => {
  const locked = {};
  Object.defineProperty(locked, 'localStorage', {
    get() {
      throw new Error('SecurityError');
    },
  });

  assert.equal(recallMonitor(refusing), null);
  assert.equal(rememberMonitor(refusing, RTK), false);
  assert.equal(recallMonitor(null), null);
  assert.equal(rememberMonitor(null, RTK), false);
  assert.equal(storageOf(locked), null);
  assert.equal(storageOf(undefined), null);
  const storage = memoryStorage();
  assert.equal(storageOf({ localStorage: storage }), storage);
});
