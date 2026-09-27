// The popup's only way to the monitors (D-2026-09-26-tray-app-4): the Tauri
// commands when `window.__TAURI__` exists, else an in-memory demo of the same
// contract, picked by `?demo=rtk|two-monitors|empty|error`, for the browser,
// the Playwright suite and `node --test`. The demo enforces what the core
// does to the UI: a dangerous write needs `confirmed`, a value must be one
// the feature accepts, and the answer is the value read back.
//
// The demo runs only on a local dev server (D-2026-09-27-tray-app-6).
// Inside the app (`tauri://localhost`, `http://tauri.localhost`) a missing
// `__TAURI__` means the app is broken, and simulated values would pass for
// the monitor's: there every command fails as `backend_unavailable`.

import { scenarioMonitors, scenarioName } from './demo-data.js';

export const DEMO_LATENCY_MS = 60;

/** A probe reads many silent codes, so the demo makes it visibly slower. */
const PROBE_LATENCY_FACTOR = 8;

const realTimers = Object.freeze({
  setTimeout: (callback, ms) => globalThis.setTimeout(callback, ms),
});

const DEV_PROTOCOLS = new Set(['http:', 'https:']);
const DEV_HOSTS = new Set(['localhost', '127.0.0.1']);

/** Why every command fails when the page is neither in Tauri nor on a dev server. */
export const UNAVAILABLE_MESSAGE = 'the Tauri API is missing from this window, so no monitor can be reached';

/**
 * The bridge for `win`: Tauri's when it is there; without `__TAURI__`, the
 * demo's on a local dev server, else one that refuses every command. The
 * demo also exposes `win.__ddcDemo` ({ scenario, writes, selected, hides,
 * emit }): `hides` counts the times the popup asked to be hidden.
 * @param {object} win the page's `window`
 * @param {{ latencyMs?: number, timers?: { setTimeout: Function } }} [options]
 */
export function createBridge(win, { latencyMs = DEMO_LATENCY_MS, timers = realTimers } = {}) {
  const tauri = win?.__TAURI__;
  if (typeof tauri?.core?.invoke === 'function') return tauriBridge(tauri);
  if (tauri == null && isLocalDevServer(win?.location)) return demoBridge(win, latencyMs, timers);
  return unavailableBridge();
}

/**
 * Whether `location` is a page served by a dev server on this machine:
 * `http:`/`https:` on `localhost` or `127.0.0.1`, any port.
 * @param {{ protocol?: string, hostname?: string } | undefined} location
 */
export function isLocalDevServer(location) {
  return DEV_PROTOCOLS.has(location?.protocol) && DEV_HOSTS.has(location?.hostname);
}

/**
 * A rejection as the contract's `{ kind, message }`; anything else — a
 * command the capabilities refuse, say — is kind `unknown`.
 * @param {unknown} error
 */
export function normalizeError(error) {
  if (typeof error?.kind === 'string') {
    return { kind: error.kind, message: String(error.message ?? '') };
  }
  return { kind: 'unknown', message: error instanceof Error ? error.message : String(error) };
}

function api(mode, invoke, listen) {
  return Object.freeze({
    mode,
    listMonitors: () => invoke('list_monitors'),
    selectMonitor: (monitorId) => invoke('select_monitor', { monitorId }),
    loadPanel: (monitorId) => invoke('load_panel', { monitorId }),
    loadFeatures: (monitorId) => invoke('load_features', { monitorId }),
    probeFeatures: (monitorId) => invoke('probe_features', { monitorId }),
    setFeature: (monitorId, code, value, { confirmed = false } = {}) =>
      invoke('set_feature', { monitorId, code, value, confirmed }),
    hidePopup: () => invoke('hide_popup'),
    onPopupShown: (handler) => listen('popup-shown', handler),
    onPanelChanged: (handler) => listen('panel-changed', handler),
  });
}

function tauriBridge(tauri) {
  const invoke = async (command, args) => {
    try {
      return await tauri.core.invoke(command, args);
    } catch (error) {
      throw normalizeError(error);
    }
  };
  const listen = (event, handler) => tauri.event.listen(event, (message) => handler(message.payload));
  return api('tauri', invoke, listen);
}

// No event ever comes, so listening succeeds and does nothing: the first
// command already shows the error, and a second message would repeat it.
function unavailableBridge() {
  const invoke = async () => {
    throw uiError('backend_unavailable', UNAVAILABLE_MESSAGE);
  };
  const listen = async () => () => {};
  return api('unavailable', invoke, listen);
}

function demoBridge(win, latencyMs, timers) {
  const scenario = scenarioName(win?.location?.search);
  const monitors = scenarioMonitors(scenario);
  const listeners = new Map();
  const demo = { scenario, writes: [], selected: null, hides: 0, emit };
  if (win) win.__ddcDemo = demo;

  const handlers = {
    list_monitors: () => monitors.map((monitor) => monitor.info),
    select_monitor: ({ monitorId }) => {
      demo.selected = monitorOf(monitors, monitorId).info.id;
      return null;
    },
    load_panel: ({ monitorId }) => panelDto(answering(monitors, monitorId)),
    load_features: ({ monitorId }) => answering(monitors, monitorId).features.map(featureDto),
    probe_features: ({ monitorId }) => answering(monitors, monitorId).probe.map(featureDto),
    set_feature: (args) => write(monitors, demo.writes, args),
    hide_popup: () => {
      demo.hides += 1;
      return null;
    },
  };

  const wait = (ms) => (ms > 0 ? new Promise((done) => timers.setTimeout(done, ms)) : Promise.resolve());

  const invoke = async (command, args = {}) => {
    await wait(command === 'probe_features' ? latencyMs * PROBE_LATENCY_FACTOR : latencyMs);
    if (scenario === 'error') {
      throw uiError('backend_unavailable', 'no DDC/CI backend could be started (demo)');
    }
    if (!Object.hasOwn(handlers, command)) throw uiError('unknown', `unknown command ${command}`);
    return copy(handlers[command](args));
  };

  const listen = async (event, handler) => {
    const handlersOf = listeners.get(event) ?? new Set();
    listeners.set(event, handlersOf);
    handlersOf.add(handler);
    return () => {
      handlersOf.delete(handler);
    };
  };

  function emit(event, payload = null) {
    for (const handler of listeners.get(event) ?? []) handler(copy(payload));
  }

  return api('demo', invoke, listen);
}

function write(monitors, writes, { monitorId, code, value, confirmed }) {
  const entry = writableEntry(answering(monitors, monitorId), code);
  if (entry.dangerous && confirmed !== true) {
    throw uiError('needs_confirmation', `writing feature ${hex(code)} is dangerous and was not confirmed`);
  }
  validate(entry, value);
  entry.reading.current = value;
  writes.push({ monitorId, code, value, confirmed: confirmed === true });
  return { current: entry.reading.current, max: entry.reading.max };
}

function monitorOf(monitors, monitorId) {
  const monitor = monitors.find((candidate) => candidate.info.id === monitorId);
  if (!monitor) throw uiError('not_found', `monitor ${monitorId} not found`);
  return monitor;
}

// A mute monitor is listed, but every DDC/CI transaction with it fails with
// its error — for `load_panel`, exactly what the Rust side answers.
function answering(monitors, monitorId) {
  const monitor = monitorOf(monitors, monitorId);
  if (monitor.mute) throw uiError(monitor.mute.kind, monitor.mute.message);
  return monitor;
}

function writableEntry(monitor, code) {
  const entry = [...monitor.controls, ...monitor.features, ...monitor.probe].find(
    (candidate) => candidate.code === code,
  );
  if (!entry || entry.status === 'unsupported') {
    throw uiError('unsupported', `feature ${hex(code)} is not supported`);
  }
  if (entry.status === 'unresponsive') throw uiError('timeout', 'monitor did not respond in time');
  return entry;
}

function validate(entry, value) {
  if (entry.options && !entry.options.some((option) => option.value === value)) {
    throw uiError('invalid_value', `value ${value} is not an allowed value for feature ${hex(entry.code)}`);
  }
  if (!entry.options && value > entry.reading.max) {
    throw uiError(
      'invalid_value',
      `value ${value} for feature ${hex(entry.code)} exceeds its maximum ${entry.reading.max}`,
    );
  }
}

function panelDto(monitor) {
  return {
    monitorId: monitor.info.id,
    controls: monitor.controls.map((entry) => ({
      code: entry.code,
      key: entry.alias,
      dangerous: entry.dangerous,
      value: valueDto(entry),
    })),
  };
}

function featureDto(entry) {
  return {
    code: entry.code,
    alias: entry.alias,
    name: entry.name,
    dangerous: entry.dangerous,
    origin: entry.origin,
    status: entry.status,
    value: entry.status === 'ok' ? valueDto(entry) : null,
  };
}

// A non-continuous value lives in the low byte (SL) of the reading.
function valueDto(entry) {
  if (entry.options) {
    return { kind: 'nonContinuous', current: entry.reading.current & 0xff, options: entry.options };
  }
  return { kind: 'continuous', current: entry.reading.current, max: entry.reading.max };
}

function uiError(kind, message) {
  return { kind, message };
}

function hex(code) {
  return `0x${code.toString(16).toUpperCase().padStart(2, '0')}`;
}

function copy(value) {
  return value === undefined ? null : JSON.parse(JSON.stringify(value));
}
