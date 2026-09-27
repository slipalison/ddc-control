import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';

import { translator } from '../../src/i18n/index.js';
import {
  I2C_DOC,
  confirmView,
  controlView,
  detectPlatform,
  errorText,
  featureView,
  featuresView,
  monitorPicker,
  panelView,
  pickMonitor,
  statusView,
  withReadBack,
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
  assert.equal(input.dangerous, true);
  assert.equal(input.currentLabel, 'DisplayPort-1');
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

  assert.deepEqual(input.options, [{ value: 0x1b, label: 'Valor 0x1B', selected: false }]);
  assert.equal(input.currentLabel, 'Valor 0x2A');
});

test('power is a button offering the modes other than the current one', () => {
  const { power } = panelView(golden.panel, pt);

  assert.equal(power.label, 'Energia');
  assert.equal(power.dangerous, true);
  assert.equal(power.currentLabel, 'Ligado');
  assert.deepEqual(power.choices, [
    { value: 0x04, label: 'Em espera (DPM)', selected: false },
    { value: 0x05, label: 'Desligado (botão de energia)', selected: false },
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
    dangerous: true,
    origin: 'probe',
    originText: 'Encontrado na sondagem',
    status: 'unsupported',
    statusText: 'Não suportado',
    widget: null,
  });
  const unknownAlias = { ...manufacturer, alias: 'mystery', name: 'Mystery', status: 'unresponsive' };
  assert.equal(featureView(unknownAlias, en).label, 'Mystery');
  assert.equal(featureView(unknownAlias, en).statusText, 'No response');
  assert.equal(featureView({ ...manufacturer, name: null }, en).label, 'Setting 0xE6');
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
      text: 'On Linux, DDC/CI needs the i2c-dev module and read/write access to /dev/i2c-*. See docs/linux-ddc-setup.md.',
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

test('the picked monitor survives while listed, and the picker appears with a choice', () => {
  const monitors = [
    { id: 'a', label: 'RTK QHD HDR' },
    { id: 'b', label: 'DELL U2723QE' },
  ];

  assert.equal(pickMonitor(monitors, 'b'), 'b');
  assert.equal(pickMonitor(monitors, 'gone'), 'a');
  assert.equal(pickMonitor([], 'a'), null);
  assert.deepEqual(monitorPicker(monitors, 'b'), {
    selectable: true,
    current: { id: 'b', label: 'DELL U2723QE' },
    options: [
      { id: 'a', label: 'RTK QHD HDR', selected: false },
      { id: 'b', label: 'DELL U2723QE', selected: true },
    ],
  });
  assert.equal(monitorPicker(monitors.slice(0, 1), 'a').selectable, false);
});
