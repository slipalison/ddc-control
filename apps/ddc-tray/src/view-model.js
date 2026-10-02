// What the popup renders, built from the contract's DTOs (D-2026-09-26-
// tray-app-5). Texts come from the injected `t`; names, risks and value
// lists come from the DTOs — that is, from the core — so nothing here knows
// MCCS. A value name is shown through its `value.<slug>` key when one
// exists, else as the core wrote it — and then it is `verbatim`: the
// monitor's data, which no translation reaches (D-2026-09-27-tray-app-7).

import { errorKey, featureKey, translateOr, valueKey } from './i18n/index.js';

export const I2C_DOC = 'docs/linux-ddc-setup.md';

/** Error kinds a Linux user may fix by setting up i2c-dev access. */
const I2C_HINT_KINDS = new Set(['backend_unavailable', 'transport', 'timeout']);

/** The VCP code of the input source: the one write whose failure the popup reads differently. */
const INPUT_CODE = 0x60;

/** Error kinds after which the monitor may have left the input this computer is on. */
const SWITCHED_AWAY_KINDS = new Set(['timeout', 'transport', 'not_found']);

const CONFIRM_BODY_KEYS = new Map([
  ['input', 'confirm.body.input'],
  ['power', 'confirm.body.power'],
]);

/**
 * The OS the popup runs on, as far as the webview tells.
 * @param {{ userAgentData?: { platform?: string }, platform?: string, userAgent?: string } | undefined} nav
 * @returns {'linux' | 'windows' | 'other'}
 */
export function detectPlatform(nav) {
  const platform = nav?.userAgentData?.platform || nav?.platform || nav?.userAgent || '';
  if (/\bwin(32|64|dows)?\b/i.test(platform)) return 'windows';
  if (/linux/i.test(platform)) return 'linux';
  return 'other';
}

/**
 * The header: the monitor's name, and a picker when there is a choice. A
 * monitor whose panel did not load (`silentIds`) stays listed, marked.
 * @param {readonly { id: string, label: string }[]} monitors
 * @param {string | null} selectedId
 * @param {ReadonlySet<string>} [silentIds]
 */
export function monitorPicker(monitors, selectedId, silentIds = new Set()) {
  return {
    selectable: monitors.length > 1,
    current: monitors.find((monitor) => monitor.id === selectedId) ?? null,
    options: monitors.map(({ id, label }) => ({
      id,
      label,
      selected: id === selectedId,
      silent: silentIds.has(id),
    })),
  };
}

/**
 * The picker's options as the dropdown shows them. A silent monitor — its
 * EDID reads, DDC/CI does not answer — is named "on another input": the
 * popup cannot tell that from DDC/CI turned off, which `messageTip` adds
 * (D-2026-10-02-usb-switch-follow-6). Its name is then a translated text;
 * any other name is the monitor's data, shown verbatim.
 * @param {{ options: readonly { id: string, label: string, silent: boolean }[] }} picker
 * @param {Function} t
 */
export function pickerOptions(picker, t) {
  return picker.options.map(({ id, label, silent }) => ({
    value: id,
    label: silent ? t('header.silent', { label }) : label,
    verbatim: !silent,
  }));
}

/**
 * The tip under a status message: why no monitor or the selected one gives
 * no DDC/CI answer — on another input, or DDC/CI off in its menu — when
 * the list is empty or the selected monitor is silent; else none.
 * @param {'loading' | 'ready' | 'empty' | 'error'} state
 * @param {boolean} mute whether the selected monitor is silent
 * @param {Function} t
 */
export function messageTip(state, mute, t) {
  return state === 'empty' || mute ? t('hint.ddc') : null;
}

/** Where the last monitor whose panel loaded is kept between runs. */
export const LAST_MONITOR_KEY = 'ddc-tray.last-monitor';

/**
 * The order to try monitors in: `preferredId` first while it is listed,
 * then the others as the system lists them. Some outputs (a TV, say) show
 * an EDID but never answer DDC/CI, so the first listed may not be usable.
 * @param {readonly { id: string }[]} monitors
 * @param {string | null} preferredId
 */
export function monitorOrder(monitors, preferredId) {
  const ids = monitors.map((monitor) => monitor.id);
  if (!ids.includes(preferredId)) return ids;
  return [preferredId, ...ids.filter((id) => id !== preferredId)];
}

/**
 * Loads the panel of the first of `ids` that answers, one at a time.
 * Resolves with `{ monitorId, panel, failures }`: `monitorId` is null when
 * none answered (or `stop()` said to give up), and `failures` maps each
 * monitor tried in vain to its error, in the order tried.
 * @template P
 * @param {readonly string[]} ids
 * @param {(monitorId: string) => Promise<P>} load
 * @param {{ stop?: () => boolean }} [options]
 */
export async function firstAnswering(ids, load, { stop = () => false } = {}) {
  const failures = new Map();
  for (const monitorId of ids) {
    if (stop()) break;
    try {
      return { monitorId, panel: await load(monitorId), failures };
    } catch (error) {
      failures.set(monitorId, error);
    }
  }
  return { monitorId: null, panel: null, failures };
}

/**
 * The page's storage, or null where reading it is refused.
 * @param {{ localStorage?: Storage } | undefined} win
 */
export function storageOf(win) {
  try {
    return win?.localStorage ?? null;
  } catch {
    return null;
  }
}

/**
 * The monitor remembered in `storage`, or null when none is (or the storage
 * refuses to answer).
 * @param {Pick<Storage, 'getItem'> | null} storage
 */
export function recallMonitor(storage) {
  try {
    return storage?.getItem(LAST_MONITOR_KEY) ?? null;
  } catch {
    return null;
  }
}

/**
 * Remembers `monitorId` in `storage`; a storage that refuses (private mode,
 * quota) only loses the memory, never the popup.
 * @param {Pick<Storage, 'setItem'> | null} storage
 * @param {string} monitorId
 * @returns {boolean} whether it was stored
 */
export function rememberMonitor(storage, monitorId) {
  try {
    storage?.setItem(LAST_MONITOR_KEY, monitorId);
    return Boolean(storage);
  } catch {
    return false;
  }
}

/**
 * The quick controls of a `load_panel` answer: continuous ones as sliders,
 * the input as a segmented choice, power as a button whose `choices` are
 * the other modes (plan A-3), and any other list (the preset) as a select.
 */
export function panelView(panel, t) {
  const controls = panel.controls.map((control) => controlView(control, t));
  const choices = controls.filter((control) => control.widget === 'choice');
  const power = choices.find((control) => control.key === 'power');
  return {
    monitorId: panel.monitorId,
    sliders: controls.filter((control) => control.widget === 'slider'),
    input: choices.find((control) => control.key === 'input') ?? null,
    selects: choices.filter((control) => control.key !== 'input' && control.key !== 'power'),
    power: power ? { ...power, choices: power.options.filter((option) => !option.selected) } : null,
  };
}

/** One quick control, labelled by its `feature.<key>` text. */
export function controlView(control, t) {
  return {
    code: control.code,
    key: control.key,
    ...featureLabel(t, control.key, null, control.code),
    dangerous: control.dangerous,
    ...valueView(control.value, t),
  };
}

/** The "all settings" entries, in the order the Rust side lists them. */
export function featuresView(features, t) {
  return features.map((feature) => featureView(feature, t));
}

/**
 * One "all settings" entry: labelled by its `feature.<alias>` text, else by
 * the core's MCCS name (`labelVerbatim`); `widget` is null when reading it
 * gave no value.
 */
export function featureView(feature, t) {
  return {
    code: feature.code,
    hex: hex(feature.code),
    key: feature.alias,
    ...featureLabel(t, feature.alias, feature.name, feature.code),
    dangerous: feature.dangerous,
    origin: feature.origin,
    originText: t(`origin.${feature.origin}`),
    status: feature.status,
    statusText: feature.status === 'ok' ? null : t(`reading.${feature.status}`),
    ...(feature.value ? valueView(feature.value, t) : { widget: null }),
  };
}

/**
 * A control or feature DTO showing `readBack`, the `{ current, max }` a
 * `set_feature` answers — never the value asked for. A non-continuous
 * value is the low byte of the reading.
 */
export function withReadBack(item, { current, max }) {
  const { value } = item;
  const shown =
    value.kind === 'continuous' ? { ...value, current, max } : { ...value, current: current & 0xff };
  return { ...item, value: shown };
}

/**
 * What to tell when the monitor read back a value other than the one asked
 * (D-2026-09-30-input-switch-autostart-5): `null` when they are the same
 * after the normalization of `withReadBack`, else the text naming the value
 * kept and the value asked. For the input source it adds that the asked
 * input may have no signal, which is when a monitor goes back on its own.
 * @param {{ code: number, key?: string, alias?: string, name?: string | null, value: object }} item
 * @param {{ asked: number, readBack: { current: number, max: number } }} change
 * @param {Function} t
 */
export function readBackNotice(item, { asked, readBack }, t) {
  const kept = withReadBack(item, readBack).value;
  const wanted = withReadBack(item, { current: asked, max: readBack.max }).value;
  if (kept.current === wanted.current) return null;
  const { label } = featureLabel(t, item.key ?? item.alias, item.name ?? null, item.code);
  const params = { feature: label, kept: shownValue(kept, t), asked: shownValue(wanted, t) };
  return t(item.code === INPUT_CODE ? 'notice.inputKept' : 'notice.kept', params);
}

/**
 * The text of a failed write: an input change followed by an error that
 * says the monitor went quiet (`timeout`, `transport`, `not_found`) may have
 * worked, and the monitor may now show an input this computer cannot reach
 * (D-2026-09-30-input-switch-autostart-6). Any other failure reads as
 * `errorText`.
 * @param {{ kind: string, message?: string } | null | undefined} error
 * @param {number} code
 * @param {Function} t
 */
export function writeFailureText(error, code, t) {
  if (code === INPUT_CODE && SWITCHED_AWAY_KINDS.has(error?.kind)) return t('notice.inputUnread');
  return errorText(error, t);
}

/**
 * The status line: `loading`, `ready`, `empty` or `error`. Empty and error
 * offer `retry`; on Linux, when a missing i2c-dev setup may be the cause,
 * `hint` points at the setup guide: its text, then the guide's path.
 * @param {{ state: 'loading' | 'ready' | 'empty' | 'error', error?: { kind: string, message: string } | null }} status
 * @param {{ t: Function, platform: string }} context
 */
export function statusView({ state, error = null }, { t, platform }) {
  const hintable = state === 'empty' || (state === 'error' && I2C_HINT_KINDS.has(error?.kind));
  return {
    state,
    busy: state === 'loading',
    message: statusMessage(state, error, t),
    detail: state === 'error' ? error?.message || null : null,
    retry: state === 'empty' || state === 'error',
    hint: platform === 'linux' && hintable ? { text: t('hint.i2c'), doc: I2C_DOC } : null,
  };
}

/** The text of an error by its kind; an unknown kind reads as a generic one. */
export function errorText(error, t) {
  return translateOr(t, errorKey(error?.kind ?? 'unknown'), t('error.unknown'));
}

/**
 * The confirmation dialog of a dangerous change, saying what the change
 * does: switching the input, the power mode, or another setting.
 * @param {{ key: string | null, label: string, toLabel: string }} change
 */
export function confirmView({ key, label, toLabel }, t) {
  const body = CONFIRM_BODY_KEYS.get(key) ?? 'confirm.body.generic';
  return {
    title: t('confirm.title'),
    body: t(body, { feature: label, to: toLabel }),
    accept: t('confirm.accept'),
    cancel: t('confirm.cancel'),
  };
}

/** A byte as MCCS writes it: `0x1B`. */
export function hex(code) {
  return `0x${code.toString(16).toUpperCase().padStart(2, '0')}`;
}

function valueView(value, t) {
  return value.kind === 'continuous' ? sliderFields(value, t) : choiceFields(value, t);
}

function shownValue(value, t) {
  const view = valueView(value, t);
  return view.widget === 'slider' ? view.valueText : view.currentLabel;
}

function sliderFields({ current, max }, t) {
  const percent = max > 0 ? Math.min(100, Math.max(0, Math.round((current * 100) / max))) : 0;
  const valueText =
    max === 100 ? t('format.percent', { value: current }) : t('format.fraction', { current, max });
  return { widget: 'slider', current, max, percent, valueText };
}

function choiceFields({ current, options }, t) {
  const views = options.map((option) => ({
    value: option.value,
    ...optionLabel(option, t),
    selected: option.value === current,
  }));
  const shown = views.find((option) => option.selected) ?? unnamed(current, t);
  return {
    widget: 'choice',
    current,
    currentLabel: shown.label,
    currentVerbatim: shown.verbatim,
    options: views,
  };
}

// A name no key translates is shown as the core wrote it, as data.
function optionLabel({ value, name }, t) {
  if (!name) return unnamed(value, t);
  const translated = translateOr(t, valueKey(name), null);
  return translated === null ? { label: name, verbatim: true } : { label: translated, verbatim: false };
}

function unnamed(value, t) {
  return { label: t('format.unnamed', { hex: hex(value) }), verbatim: false };
}

function featureLabel(t, alias, name, code) {
  const translated = alias ? translateOr(t, featureKey(alias), null) : null;
  if (translated !== null) return { label: translated, labelVerbatim: false };
  if (name) return { label: name, labelVerbatim: true };
  return { label: t('format.code', { hex: hex(code) }), labelVerbatim: false };
}

function statusMessage(state, error, t) {
  if (state === 'loading') return t('state.loading');
  if (state === 'empty') return t('state.empty');
  if (state === 'error') return errorText(error, t);
  return null;
}
