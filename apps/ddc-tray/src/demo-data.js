// The demo's monitors (D-2026-09-26-tray-app-8), written out as the Rust
// side would read them. `rtk` mirrors the simulated monitor of
// `src-tauri/src/fixture.rs` (plan A-2) — the contract test pins it to the
// golden — and its probe mirrors the probe test of `panel/tests.rs`; the
// mute TV's error is pinned the same way, to the mute golden. Every name is
// the core's catalog name, verbatim; every message of a failing call, the
// core's (or Tauri's) wording.

export const SCENARIOS = Object.freeze(['rtk', 'two-monitors', 'empty', 'error']);

export const DEFAULT_SCENARIO = 'rtk';

/**
 * The scenario `?demo=` asks for in `search`; unknown or absent is `rtk`.
 * @param {string | undefined} search
 */
export function scenarioName(search) {
  const asked = new URLSearchParams(search ?? '').get('demo');
  return SCENARIOS.includes(asked) ? asked : DEFAULT_SCENARIO;
}

/**
 * What the demo's backend answers a call that fails with, as the core
 * (`crates/ddc-core/src/domain/error.rs`) and Tauri word it: English, as
 * any backend message, which the popup shows only as data. A `code` is
 * written as MCCS writes it (`0x60`).
 */
export const DEMO_MESSAGES = Object.freeze({
  timeout: () => 'monitor did not respond in time',
  noBackend: () => 'no DDC/CI backend could be started (demo)',
  unknownCommand: (command) => `unknown command ${command}`,
  notConfirmed: (code) => `writing feature ${code} is dangerous and was not confirmed`,
  notFound: (monitorId) => `monitor ${monitorId} not found`,
  unsupported: (code) => `feature ${code} is not supported`,
  notAllowed: (value, code) => `value ${value} is not an allowed value for feature ${code}`,
  aboveMax: (value, code, max) => `value ${value} for feature ${code} exceeds its maximum ${max}`,
  // How Tauri refuses a command no capability grants in a release build
  // (`webview/mod.rs` of tauri 2.12): a string, which the bridge reads as
  // kind `unknown`.
  refusedByTauri: (command) => `Command ${command} not allowed by ACL`,
});

/**
 * The commands `?fail=` can make fail, by the word that names each:
 * `fail=write`, `fail=features`, `fail=probe` (they time out), `fail=events`
 * (listening to the tray's events, Tauri's `plugin:event|listen`) and
 * `fail=hide` (they are refused), or a comma-separated list.
 */
export const FAILURES = Object.freeze({
  write: 'set_feature',
  features: 'load_features',
  probe: 'probe_features',
  events: 'plugin:event|listen',
  hide: 'hide_popup',
});

/**
 * The commands `?fail=` in `search` asks the demo to fail; an unknown word
 * is ignored, as an unknown scenario is.
 * @param {string | undefined} search
 * @returns {Set<string>}
 */
export function failingCommands(search) {
  const words = new URLSearchParams(search ?? '')
    .getAll('fail')
    .flatMap((list) => list.split(','))
    .map((word) => word.trim());
  return new Set(words.filter((word) => Object.hasOwn(FAILURES, word)).map((word) => FAILURES[word]));
}

/**
 * Fresh monitors of scenario `name`, in enumeration order. Each entry holds
 * the raw reading (`current`, `max`) a write changes, and `options` when
 * the feature is non-continuous; a monitor with a `mute` error fails every
 * DDC/CI call with it.
 * @param {string} name
 */
export function scenarioMonitors(name) {
  switch (name) {
    case 'two-monitors':
      return [muteTv(), rtk(), dell()];
    case 'empty':
    case 'error':
      return [];
    default:
      return [rtk()];
  }
}

function continuous(current, max) {
  return { reading: { current, max }, options: null };
}

function choice(current, max, options) {
  return {
    reading: { current, max },
    options: options.map(([value, name]) => ({ value, name })),
  };
}

function control(code, alias, dangerous, value) {
  return { code, alias, dangerous, status: 'ok', ...value };
}

function setting(code, alias, name, dangerous, value, status = 'ok', origin = 'caps') {
  return { code, alias, name, dangerous, origin, status, ...value };
}

function probed(code, alias, name, dangerous, status, value = { reading: null, options: null }) {
  return setting(code, alias, name, dangerous, value, status, 'probe');
}

const RTK_INPUTS = [
  [0x01, 'VGA-1'],
  [0x03, 'DVI-1'],
  [0x04, 'DVI-2'],
  [0x0f, 'DisplayPort-1'],
  [0x10, 'DisplayPort-2'],
  [0x11, 'HDMI-1'],
  [0x12, 'HDMI-2'],
];

const RTK_PRESETS = [
  [0x01, 'sRGB'],
  [0x02, 'Display Native'],
  [0x04, '5000 K'],
  [0x05, '6500 K'],
  [0x06, '7500 K'],
  [0x08, '9300 K'],
  [0x0b, 'User 1'],
];

const POWER_MODES = [
  [0x01, 'On'],
  [0x04, 'Off (DPM)'],
  [0x05, 'Off (write-only)'],
];

const OSD_CONTROL = [
  [0x01, 'OSD disabled'],
  [0x02, 'OSD enabled'],
];

const OSD_LANGUAGES = [
  [0x01, 'Chinese (traditional)'],
  [0x02, 'English'],
  [0x03, 'French'],
  [0x04, 'German'],
  [0x06, 'Japanese'],
  [0x0a, 'Spanish'],
  [0x0d, 'Chinese (simplified)'],
];

const AUTO_SETUP = [
  [0x00, 'Off'],
  [0x01, 'Run'],
  [0x02, 'Continuous'],
];

function rtk() {
  return {
    info: {
      id: 'RTK-RTK-QHD-HDR-01010101',
      label: 'RTK QHD HDR',
      manufacturer: 'RTK',
      model: 'RTK QHD HDR',
    },
    controls: [
      control(0x10, 'brightness', false, continuous(75, 100)),
      control(0x12, 'contrast', false, continuous(50, 100)),
      control(0x62, 'volume', false, continuous(30, 100)),
      control(0x60, 'input', true, choice(0x0f, 0x03, RTK_INPUTS)),
      control(0x14, 'preset', false, choice(0x01, 0x0b, RTK_PRESETS)),
      control(0xd6, 'power', true, choice(0x01, 0x05, POWER_MODES)),
    ],
    features: [
      setting(0x0c, 'color-temp', 'Color Temperature Request', false, continuous(70, 100)),
      setting(0x16, 'red-gain', 'Video Gain (Red)', false, continuous(50, 100)),
      setting(0x18, 'green-gain', 'Video Gain (Green)', false, continuous(48, 100)),
      setting(0x1a, 'blue-gain', 'Video Gain (Blue)', false, continuous(46, 100)),
      setting(0x87, 'sharpness', 'Sharpness', false, continuous(5, 10)),
      setting(0xca, 'osd-lock', 'OSD/Button Control', true, choice(0x02, 0x02, OSD_CONTROL)),
      setting(0xcc, 'osd-language', 'OSD Language', false, choice(0x02, 0x0d, OSD_LANGUAGES)),
    ],
    probe: [
      probed(0x1e, 'auto-setup', 'Auto Setup', true, 'ok', choice(0x00, 0x02, AUTO_SETUP)),
      probed(0x20, 'h-position', 'Horizontal Position', true, 'unsupported'),
      probed(0x30, 'v-position', 'Vertical Position', true, 'unsupported'),
      probed(0x6c, 'red-black-level', 'Video Black Level (Red)', false, 'ok', continuous(50, 100)),
      probed(0x6e, 'green-black-level', 'Video Black Level (Green)', false, 'unsupported'),
      probed(0x70, 'blue-black-level', 'Video Black Level (Blue)', false, 'unsupported'),
      probed(0x7e, 'trapezoid', 'Trapezoid', true, 'unresponsive'),
      probed(0xe6, null, 'Manufacturer specific (0xE6)', true, 'unsupported'),
      probed(0xf1, null, 'Manufacturer specific (0xF1)', true, 'unsupported'),
    ],
  };
}

// A TV listed first, as on the development machine: its EDID reads, but
// it never answers DDC/CI, so its panel fails with the error the real
// backend gives for it, and the popup has to move on to the next monitor.
function muteTv() {
  return {
    info: {
      id: 'GSM-LG-TV-SSCR2-01010101',
      label: 'LG TV SSCR2',
      manufacturer: 'GSM',
      model: 'LG TV SSCR2',
    },
    mute: {
      kind: 'transport',
      message:
        'transport error: DDC/CI I2C error: Input/output error (os error 5) (gave up after attempt 3 of 3)',
    },
    controls: [],
    features: [],
    probe: [],
  };
}

// A second monitor with no volume (its reading fails, so the panel leaves it
// out) and an input the catalog does not name (0x1B, shown by number).
function dell() {
  return {
    info: {
      id: 'DEL-DELL-U2723QE-7X9K2L3',
      label: 'DELL U2723QE',
      manufacturer: 'DEL',
      model: 'DELL U2723QE',
    },
    controls: [
      control(0x10, 'brightness', false, continuous(40, 100)),
      control(0x12, 'contrast', false, continuous(75, 100)),
      control(
        0x60,
        'input',
        true,
        choice(0x11, 0x1b, [
          [0x0f, 'DisplayPort-1'],
          [0x11, 'HDMI-1'],
          [0x1b, null],
        ]),
      ),
      control(
        0x14,
        'preset',
        false,
        choice(0x05, 0x0b, [
          [0x01, 'sRGB'],
          [0x05, '6500 K'],
          [0x0b, 'User 1'],
        ]),
      ),
      control(0xd6, 'power', true, choice(0x01, 0x05, POWER_MODES)),
    ],
    features: [setting(0x87, 'sharpness', 'Sharpness', false, continuous(50, 100))],
    probe: [],
  };
}
