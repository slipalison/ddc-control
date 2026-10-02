// The popup (D-2026-09-26-tray-app-5): what the bridge answers, rendered
// through the view-model, and every change sent back through the bridge.
// Texts come from i18n keys (D-2026-09-26-tray-app-6) and nothing here knows
// MCCS: names, risks and value lists come from the DTOs. A dangerous change
// waits for the in-app dialog before the bridge sees it. Under the strict
// CSP (D-2026-09-26-tray-app-7) no markup is parsed from strings and no
// inline style is set — a slider's fill is the custom property `--fill`.
// Lists are in-page dropdowns, never a native select element: its menu is
// a window of its own, and the popup hid on losing the focus to it
// (D-2026-09-27-tray-app-1). What the monitor or the core wrote — a name,
// a value name no locale translates, a code, a backend's message — is data
// and carries `translate="no"` (D-2026-09-27-tray-app-7). On the demo, a
// text the page shows that is neither a translation nor data is reported
// as a console error (D-2026-09-27-tray-app-11).

import { createBridge } from './bridge.js';
import { createWriteQueue } from './debounce.js';
import { createDropdown } from './dropdown.js';
import { createIcon } from './icons.js';
import { textGuard } from './i18n/guard.js';
import { LOCALES, detectLocale, pseudoRequested, translator } from './i18n/index.js';
import {
  confirmView,
  controlView,
  detectPlatform,
  errorText,
  featureView,
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
} from './view-model.js';

const CONTROL_ICONS = Object.freeze({
  brightness: 'sun',
  contrast: 'contrast',
  volume: 'volume',
  preset: 'droplet',
  power: 'power',
});

const bridge = createBridge(window);
const locale = detectLocale(navigator);
// The texts are checked on the demo only: `?pseudo=1` marks the
// translated ones (D-2026-09-27-tray-app-7), and the guard reports any
// other that is not data (D-2026-09-27-tray-app-11). Inside the app
// neither is ever on.
const demo = bridge.mode === 'demo';
const pseudo = demo && pseudoRequested(window.location.search);
const guard = textGuard({ enabled: demo });
const t = guard.track(translator(locale, LOCALES, { pseudo }));
const platform = detectPlatform(navigator);
const storage = storageOf(window);

const byId = (id) => document.getElementById(id);

const ui = {
  app: byId('app'),
  monitorName: byId('monitor-name'),
  monitorMeta: byId('monitor-meta'),
  title: byId('title'),
  power: byId('power'),
  refresh: byId('refresh'),
  content: byId('content'),
  skeleton: byId('skeleton'),
  panel: byId('panel'),
  quick: byId('quick'),
  sliders: byId('sliders'),
  quickRows: byId('quick-rows'),
  inputCard: byId('input-card'),
  inputChips: byId('input-chips'),
  more: byId('more'),
  moreStatus: byId('more-status'),
  moreSpinner: byId('more-spinner'),
  moreStatusText: byId('more-status-text'),
  moreRetry: byId('more-retry'),
  featureList: byId('feature-list'),
  probe: byId('probe'),
  probeLabel: byId('probe-label'),
  probeResults: byId('probe-results'),
  probeList: byId('probe-list'),
  probeNote: byId('probe-note'),
  message: byId('message'),
  messageArt: byId('message-art'),
  messageTitle: byId('message-title'),
  messageTip: byId('message-tip'),
  messageHint: byId('message-hint'),
  messageDetail: byId('message-detail'),
  retry: byId('retry'),
  toast: byId('toast'),
  toastText: byId('toast-text'),
  toastRetry: byId('toast-retry'),
  announcer: byId('announcer'),
  confirm: byId('confirm'),
  confirmTitle: byId('confirm-title'),
  confirmBody: byId('confirm-body'),
  confirmNote: byId('confirm-note'),
  confirmChoices: byId('confirm-choices'),
  confirmChoiceList: byId('confirm-choice-list'),
  confirmCancel: byId('confirm-cancel'),
  confirmAccept: byId('confirm-accept'),
};

const model = {
  state: 'loading',
  settled: false,
  generation: 0,
  monitors: [],
  /** The monitor picked or being loaded — the header's. */
  selectedId: null,
  /** The monitor whose panel the widgets show; writes go to it. */
  shownId: null,
  /** Monitors whose panel did not load the last time they were tried. */
  silent: new Set(),
  /** What the error state's "Try again" does: the last load that failed. */
  retry: null,
  shape: null,
  /** Controls and features of the shown monitor by code: `{ kind, dto, widget, editing }`. */
  entries: new Map(),
  power: null,
  features: 'idle',
  probing: false,
  /** Whether the toast tells a failure no later load or write mends. */
  toastLasts: false,
};

// One lane per monitor and code: a slider burst becomes one write of its
// last value, and a code never has two writes in flight.
const writes = createWriteQueue({
  write: (lane, { monitorId, code, value, confirmed }) =>
    bridge.setFeature(monitorId, code, value, { confirmed }),
  onResult: showReadBack,
  onError: writeFailed,
});

// Every list stays inside the popup's window.
const appBounds = () => ui.app.getBoundingClientRect();

const monitorDropdown = createDropdown({
  id: 'monitor-picker',
  label: t('header.monitor'),
  className: 'title-picker',
  onChange: (monitorId) => void openMonitor(monitorId),
  bounds: appBounds,
});

start();

function start() {
  guard.watch(document, window.MutationObserver);
  monitorDropdown.node.hidden = true;
  ui.title.append(monitorDropdown.node);
  translatePage(document);
  mountIcons(document);
  wire();
  void listen();
  void refresh();
}

// ---------------------------------------------------------------- page setup

function translatePage(root) {
  document.documentElement.lang = locale;
  for (const node of root.querySelectorAll('[data-i18n]')) node.textContent = t(node.dataset.i18n);
  for (const node of root.querySelectorAll('[data-i18n-attr]')) {
    for (const pair of node.dataset.i18nAttr.split(';')) {
      const [attribute, key] = pair.split(':').map((part) => part.trim());
      node.setAttribute(attribute, t(key));
    }
  }
}

function mountIcons(root) {
  for (const node of root.querySelectorAll('[data-icon]')) {
    node.setAttribute('aria-hidden', 'true');
    node.replaceChildren(createIcon(node.dataset.icon));
  }
}

function wire() {
  ui.refresh.addEventListener('click', () => void refresh());
  ui.power.addEventListener('click', () => void changePower(model.power));
  ui.retry.addEventListener('click', () => void (model.retry ?? refresh)());
  ui.toastRetry.addEventListener('click', () => void refresh());
  ui.inputChips.addEventListener('keydown', moveChipFocus);
  ui.more.addEventListener('toggle', () => {
    if (ui.more.open && model.features === 'idle') void loadFeatures();
  });
  ui.moreRetry.addEventListener('click', () => void loadFeatures());
  ui.probe.addEventListener('click', () => void probe());
  ui.confirmCancel.addEventListener('click', () => ui.confirm.close('cancel'));
  ui.confirmAccept.addEventListener('click', () => ui.confirm.close('accept'));
  ui.confirm.addEventListener('click', (event) => {
    if (event.target === ui.confirm) ui.confirm.close('cancel');
  });
  document.addEventListener('keydown', hideOnEscape);
  if (bridge.mode === 'tauri') document.addEventListener('contextmenu', (event) => event.preventDefault());
}

async function listen() {
  try {
    await bridge.onPopupShown(() => void refresh());
    await bridge.onPanelChanged(panelChanged);
  } catch (error) {
    showToast(errorText(error, t));
    // The popup starts hidden, and its first load would hide this before
    // anyone saw it; without the tray's events, what it shows may go stale.
    model.toastLasts = true;
  }
}

// Esc hides the popup, unless the dialog is open: then Esc cancels it.
function hideOnEscape(event) {
  if (event.key !== 'Escape' || ui.confirm.open || event.defaultPrevented) return;
  bridge.hidePopup().catch((error) => showToast(errorText(error, t)));
}

// A tray shortcut changed the shown monitor: read its panel again. Any
// other state is read afresh when the popup shows next.
function panelChanged(payload) {
  if (model.state !== 'ready') return;
  if (payload?.monitorId && payload.monitorId !== model.shownId) return;
  void openMonitor(model.shownId);
}

// ------------------------------------------------------------------ loading

// Lists the monitors and shows the first that answers: the last one that
// worked, then the others in the system's order. Only when none answers is
// it an error — a TV listed first with a mute DDC/CI must not be one.
async function refresh() {
  const generation = begin();
  const stale = () => generation !== model.generation;
  try {
    const monitors = await listMonitors();
    if (stale()) return;
    if (monitors.length === 0) {
      showState('empty');
      return;
    }
    const order = monitorOrder(monitors, recallMonitor(storage) ?? model.selectedId);
    const found = await firstAnswering(order, (id) => bridge.loadPanel(id), { stop: stale });
    if (stale()) return;
    for (const monitorId of found.failures.keys()) model.silent.add(monitorId);
    if (found.monitorId === null) {
      model.selectedId = order[0];
      fail(found.failures.get(order[0]), refresh);
      return;
    }
    await showLoaded(found.monitorId, found.panel, stale);
  } catch (error) {
    if (!stale()) fail(error, refresh);
  } finally {
    end(generation);
  }
}

async function listMonitors() {
  try {
    model.monitors = await bridge.listMonitors();
  } catch (error) {
    model.monitors = [];
    throw error;
  }
  return model.monitors;
}

// The monitor the user picked (or the one a tray shortcut changed): its
// failure is shown as such, with "Try again" retrying that same monitor.
async function openMonitor(monitorId) {
  const generation = begin();
  const stale = () => generation !== model.generation;
  model.selectedId = monitorId;
  try {
    const panel = await bridge.loadPanel(monitorId);
    if (!stale()) await showLoaded(monitorId, panel, stale);
  } catch (error) {
    if (stale()) return;
    model.silent.add(monitorId);
    fail(error, () => openMonitor(monitorId));
  } finally {
    end(generation);
  }
}

// A monitor answered: it becomes the tray shortcuts' target and the one
// the next run tries first.
async function showLoaded(monitorId, panel, stale) {
  await bridge.selectMonitor(monitorId);
  if (stale()) return;
  model.selectedId = monitorId;
  model.shownId = monitorId;
  model.silent.delete(monitorId);
  model.retry = null;
  rememberMonitor(storage, monitorId);
  const shape = shapeOf(panel);
  if (shape === model.shape) patchPanel(panel);
  else buildPanel(panel, shape);
  hideToast();
  showState('ready');
}

function fail(error, retry) {
  model.retry = retry;
  showState('error', error);
}

function begin() {
  model.generation += 1;
  setBusy(true);
  if (!model.settled) showState('loading');
  return model.generation;
}

function end(generation) {
  if (generation === model.generation) setBusy(false);
}

function setBusy(busy) {
  ui.refresh.classList.toggle('is-busy', busy);
  ui.retry.classList.toggle('is-busy', busy);
  ui.retry.setAttribute('aria-disabled', String(busy));
}

// A panel keeps its widgets while the monitor and the shape of its controls
// stay the same, so a revalidation never moves focus or a dragged thumb.
function shapeOf(panel) {
  return JSON.stringify([
    panel.monitorId,
    panel.controls.map(({ code, value }) => [
      code,
      value.kind,
      value.options?.map((choice) => choice.value) ?? value.max,
    ]),
  ]);
}

// ------------------------------------------------------------------- states

function showState(state, error = null) {
  const status = statusView({ state, error }, { t, platform });
  model.state = state;
  model.settled ||= state !== 'loading';
  ui.app.dataset.state = state;
  ui.skeleton.hidden = state !== 'loading';
  ui.panel.hidden = state !== 'ready';
  ui.power.hidden = state !== 'ready' || !model.power;
  ui.message.hidden = !status.retry;
  ui.content.setAttribute('aria-busy', String(status.busy));
  if (status.retry) paintMessage(status);
  if (state === 'error') announce(status.message);
  paintHeader();
}

// After an error the picker stays, so another monitor is one pick away.
function paintHeader() {
  const picker = monitorPicker(model.monitors, model.selectedId, model.silent);
  const shown = model.state === 'ready' || model.state === 'error';
  const current = shown ? picker.current : null;
  const choosing = shown && picker.selectable;
  ui.monitorName.textContent = current?.label ?? t('app.title');
  markVerbatim(ui.monitorName, Boolean(current));
  // With a choice, the name is the picker's button.
  ui.title.classList.toggle('is-picker', choosing);
  ui.monitorName.hidden = choosing;
  monitorDropdown.node.hidden = !choosing;
  if (choosing) paintPicker(picker);
  else monitorDropdown.close();
  ui.monitorMeta.textContent = metaText(current);
}

function metaText(current) {
  if (model.state === 'loading') return t('state.loading');
  if (current && model.state === 'ready') return t('header.meta', { manufacturer: current.manufacturer });
  if (current) return t('header.noAnswer');
  return t('header.tagline');
}

function paintPicker(picker) {
  const options = picker.options.map(({ id, label, silent }) => ({
    value: id,
    label: silent ? t('header.silent', { label }) : label,
    verbatim: !silent,
  }));
  monitorDropdown.update({ options, value: model.selectedId });
}

function paintMessage(status) {
  const empty = status.state === 'empty';
  const mute = !empty && model.silent.has(model.selectedId);
  ui.messageArt.dataset.tone = empty ? 'neutral' : 'danger';
  ui.messageArt.replaceChildren(createIcon(empty ? 'empty' : 'alert'));
  ui.messageTitle.textContent = status.message;
  setText(ui.messageTip, empty || mute ? t('hint.ddc') : null);
  setText(ui.messageDetail, status.detail);
  ui.messageHint.hidden = !status.hint;
  // The space is text, not a margin: a copied hint keeps it.
  if (status.hint) ui.messageHint.replaceChildren(status.hint.text, ' ', verbatim('code', '', status.hint.doc));
}

function showToast(text) {
  model.toastLasts = false;
  ui.toastText.textContent = text;
  ui.toast.hidden = false;
  announce(text);
}

// A load or a write that works mends what the toast tells, unless it lasts.
function hideToast() {
  if (!model.toastLasts) ui.toast.hidden = true;
}

function announce(text) {
  ui.announcer.textContent = text;
}

// -------------------------------------------------------------- quick panel

function buildPanel(panel, shape) {
  const view = panelView(panel, t);
  const dtos = new Map(panel.controls.map((control) => [control.code, control]));
  model.entries = new Map();
  model.shape = shape;
  const mount = (item, make) =>
    mountEntry({ kind: 'control', monitorId: panel.monitorId, dto: dtos.get(item.code) }, item, make);

  ui.sliders.replaceChildren(...view.sliders.map((item) => mount(item, quickSlider)));
  ui.quickRows.replaceChildren(...view.selects.map((item) => mount(item, selectRow)));
  ui.sliders.hidden = view.sliders.length === 0;
  ui.quickRows.hidden = view.selects.length === 0;
  ui.quick.hidden = ui.sliders.hidden && ui.quickRows.hidden;
  ui.inputCard.hidden = !view.input;
  if (view.input) mount(view.input, chipGroup);
  else ui.inputChips.replaceChildren();
  model.power = null;
  if (view.power) mount(view.power, powerButton);
  resetFeatures();
}

function patchPanel(panel) {
  for (const control of panel.controls) patchEntry(control);
  if (model.features === 'ready' && ui.more.open) void loadFeatures();
  else if (model.features !== 'loading') model.features = 'idle';
}

function mountEntry(seed, view, make) {
  const entry = { ...seed, widget: null, editing: false };
  entry.widget = make(entry, view);
  entry.widget.update(view);
  model.entries.set(view.code, entry);
  return entry.widget.node;
}

function patchEntry(dto) {
  const entry = model.entries.get(dto.code);
  if (!entry) return;
  entry.dto = dto;
  if (writes.busy(laneOf(entry)) || entry.editing) return;
  entry.widget.update(viewOf(entry));
}

function viewOf(entry) {
  return entry.kind === 'control' ? controlView(entry.dto, t) : featureView(entry.dto, t);
}

/** The entry's view as if the monitor held `value`. */
function viewWith(entry, value) {
  const dto = withReadBack(entry.dto, { current: value, max: entry.dto.value.max });
  return viewOf({ ...entry, dto });
}

function shownValue(view) {
  return view.widget === 'slider' ? view.valueText : view.currentLabel;
}

function quickSlider(entry, view) {
  const icon = CONTROL_ICONS[view.key] ?? 'sliders';
  return sliderWidget(entry, view, { id: `control-${view.code}`, icon });
}

function sliderWidget(entry, view, { id, icon = null, marks = [] }) {
  const node = element('div', icon ? 'slider has-icon' : 'slider');
  const label = markVerbatim(element('label', 'slider-label', view.label), view.labelVerbatim);
  label.htmlFor = id;
  const pill = element('span', 'pill');
  pill.setAttribute('aria-hidden', 'true');
  const input = element('input', 'range');
  Object.assign(input, { type: 'range', id, min: '0', step: '1' });
  const head = element('div', 'slider-head');
  head.append(label, ...marks, pill);
  if (icon) node.append(tile(icon));
  node.append(head, input);
  bindSlider(entry, input);
  return { node, update: (next) => paintSlider(input, pill, next), flag: () => flash(pill) };
}

function paintSlider(input, pill, view) {
  input.max = String(view.max);
  input.value = String(view.current);
  input.setAttribute('aria-valuetext', view.valueText);
  input.style.setProperty('--fill', `${view.percent}%`);
  pill.textContent = view.valueText;
}

// A safe slider writes while it moves (debounced) and flushes on release;
// a dangerous one only previews until released, then asks first.
function bindSlider(entry, input) {
  const release = () => {
    entry.editing = false;
  };
  input.addEventListener('pointerdown', () => {
    entry.editing = true;
  });
  input.addEventListener('pointerup', release);
  input.addEventListener('pointercancel', release);
  input.addEventListener('input', () => {
    const value = Number(input.value);
    entry.widget.update(viewWith(entry, value));
    if (!entry.dto.dangerous) queueWrite(entry, value);
  });
  input.addEventListener('change', () => {
    release();
    if (entry.dto.dangerous) void requestChange(entry, Number(input.value));
    else writes.flush(laneOf(entry));
  });
}

function chipGroup(entry, view) {
  const chips = view.options.map((choice) => chip(entry, choice));
  ui.inputChips.replaceChildren(...chips);
  const checked = () => chips.find((node) => node.getAttribute('aria-checked') === 'true');
  return {
    node: ui.inputChips,
    update: (next) => paintChips(chips, next),
    pending: (value) => {
      for (const node of chips) node.classList.toggle('is-pending', Number(node.dataset.value) === value);
    },
    flag: () => flash(checked() ?? ui.inputChips),
  };
}

function chip(entry, choice) {
  const node = element('button', 'chip');
  node.type = 'button';
  node.setAttribute('role', 'radio');
  node.dataset.value = String(choice.value);
  const mark = element('span', 'chip-mark');
  mark.append(createIcon('check'), element('span', 'spinner'));
  node.append(mark, markVerbatim(element('span', 'chip-label', choice.label), choice.verbatim));
  node.addEventListener('click', () => void requestChange(entry, choice.value));
  return node;
}

function paintChips(chips, view) {
  let focusable = chips[0];
  for (const node of chips) {
    const checked = Number(node.dataset.value) === view.current;
    node.setAttribute('aria-checked', String(checked));
    node.classList.remove('is-pending');
    node.tabIndex = -1;
    if (checked) focusable = node;
  }
  if (focusable) focusable.tabIndex = 0;
}

// Arrows move the focus between inputs; Space or Enter picks one. The pick
// does not follow the focus, because every pick is a confirmed change.
function moveChipFocus(event) {
  const chips = [...ui.inputChips.querySelectorAll('[role="radio"]')];
  const from = chips.indexOf(document.activeElement);
  const to = chipTarget(event.key, from, chips.length);
  if (from < 0 || to === null) return;
  event.preventDefault();
  chips[from].tabIndex = -1;
  chips[to].tabIndex = 0;
  chips[to].focus();
}

function chipTarget(key, from, count) {
  if (key === 'ArrowRight' || key === 'ArrowDown') return (from + 1) % count;
  if (key === 'ArrowLeft' || key === 'ArrowUp') return (from - 1 + count) % count;
  if (key === 'Home') return 0;
  if (key === 'End') return count - 1;
  return null;
}

function selectRow(entry, view) {
  const id = `control-${view.code}`;
  const node = element('div', 'row');
  const text = element('div', 'row-text');
  const label = markVerbatim(element('span', 'row-label', view.label), view.labelVerbatim);
  label.id = `${id}-label`;
  text.append(label);
  const widget = selectWidget(entry, id, label.id);
  node.append(tile(CONTROL_ICONS[view.key] ?? 'sliders'), text, widget.node);
  return { ...widget, node };
}

function selectWidget(entry, id, labelId) {
  const dropdown = createDropdown({
    id,
    labelledBy: labelId,
    className: 'select',
    onChange: (value) => void requestChange(entry, value),
    bounds: appBounds,
  });
  return {
    node: dropdown.node,
    update: (next) => paintSelect(dropdown, next),
    flag: () => flash(dropdown.button),
  };
}

// A current value outside the list shows as a choice that cannot be
// picked, so the dropdown never silently displays another value.
function paintSelect(dropdown, view) {
  const options = view.options.map(({ value, label, verbatim }) => ({ value, label, verbatim }));
  if (!view.options.some((choice) => choice.selected)) {
    options.unshift({ value: view.current, label: view.currentLabel, verbatim: view.currentVerbatim, disabled: true });
  }
  dropdown.update({ options, value: view.current });
}

// Power lives in the header, as a button that always opens the dialog.
function powerButton(entry) {
  model.power = entry;
  const update = (next) => {
    ui.power.classList.remove('is-busy');
    ui.power.setAttribute('aria-disabled', String(!next.options.some((choice) => !choice.selected)));
  };
  const pending = () => ui.power.classList.add('is-busy');
  return { node: ui.power, update, pending, flag: () => flash(ui.power) };
}

// ------------------------------------------------------------------- writes

function laneOf(entry) {
  return `${entry.monitorId}:${entry.dto.code}`;
}

function queueWrite(entry, value, { confirmed = false, now = false } = {}) {
  const lane = laneOf(entry);
  writes.push(lane, { monitorId: entry.monitorId, code: entry.dto.code, value, confirmed });
  if (now) writes.flush(lane);
}

/** A change the user asked for; a dangerous one waits for the dialog. */
async function requestChange(entry, value) {
  if (value === viewOf(entry).current) {
    entry.widget.update(viewOf(entry));
    return;
  }
  if (entry.dto.dangerous && !(await confirmChange(entry, value))) {
    entry.widget.update(viewOf(entry));
    return;
  }
  entry.widget.pending?.(value);
  queueWrite(entry, value, { confirmed: entry.dto.dangerous, now: true });
}

async function changePower(entry) {
  if (!entry) return;
  const view = viewOf(entry);
  const choices = view.options.filter((choice) => !choice.selected);
  if (choices.length === 0) return;
  const describe = (choice) => confirmView({ key: view.key, label: view.label, toLabel: choice.label }, t);
  const value = await askDialog({ ...describe(choices[0]), tone: 'danger', choices, describe });
  if (value === null) return;
  entry.widget.pending(value);
  queueWrite(entry, value, { confirmed: true, now: true });
}

// The answer is the value read back: it replaces what was asked for, and a
// difference is flagged, announced and told in the toast, which names the
// value the monitor kept (D-2026-09-30-input-switch-autostart-5).
function showReadBack(lane, readBack, sent) {
  const entry = entryOf(sent);
  if (!entry) return;
  const notice = readBackNotice(entry.dto, { asked: sent.value, readBack }, t);
  entry.dto = withReadBack(entry.dto, readBack);
  if (writes.busy(lane) || entry.editing) return;
  const view = viewOf(entry);
  entry.widget.update(view);
  if (notice === null) {
    hideToast();
    return;
  }
  entry.widget.flag();
  showToast(notice);
  announce(t('announce.readBack', { feature: view.label, value: shownValue(view) }));
}

// An input change whose follow-up failed may still have switched the
// monitor away (D-2026-09-30-input-switch-autostart-6).
function writeFailed(lane, error, sent) {
  const entry = entryOf(sent);
  if (entry && !writes.busy(lane)) entry.widget.update(viewOf(entry));
  showToast(writeFailureText(error, sent.code, t));
}

function entryOf({ monitorId, code }) {
  const entry = model.entries.get(code);
  return entry?.monitorId === monitorId ? entry : null;
}

// ------------------------------------------------------------------- dialog

function confirmChange(entry, value) {
  const view = viewOf(entry);
  const toLabel = shownValue(viewWith(entry, value));
  const texts = confirmView({ key: view.key, label: view.label, toLabel }, t);
  const input = view.key === 'input';
  const note = input ? t('confirm.recover.input') : null;
  return askDialog({ ...texts, note, tone: input ? 'accent' : 'danger' });
}

/**
 * Opens the confirmation dialog; resolves with `true` (or the picked
 * choice's value) on accept and `null` on cancel, Esc or a backdrop click.
 */
function askDialog(request) {
  paintDialog(request);
  ui.confirm.returnValue = '';
  ui.confirm.showModal();
  ui.confirmCancel.focus();
  return new Promise((resolve) => {
    ui.confirm.addEventListener('close', () => resolve(dialogAnswer(request)), { once: true });
  });
}

function dialogAnswer(request) {
  if (ui.confirm.returnValue !== 'accept') return null;
  if (!request.choices) return true;
  const picked = ui.confirmChoiceList.querySelector('input:checked');
  return picked ? Number(picked.value) : null;
}

function paintDialog({ title, body, accept, cancel, note = null, tone, choices = null, describe = null }) {
  ui.confirm.dataset.tone = tone;
  ui.confirmTitle.textContent = title;
  ui.confirmBody.textContent = body;
  setText(ui.confirmNote, note);
  ui.confirmAccept.textContent = accept;
  ui.confirmCancel.textContent = cancel;
  ui.confirmAccept.classList.toggle('button-danger', tone === 'danger');
  ui.confirmAccept.classList.toggle('button-accent', tone !== 'danger');
  ui.confirmChoices.hidden = !choices;
  const radios = (choices ?? []).map((choice, index) => radio(choice, index, describe));
  ui.confirmChoiceList.replaceChildren(...radios);
}

function radio(choice, index, describe) {
  const label = element('label', 'choice');
  const input = element('input');
  Object.assign(input, { type: 'radio', name: 'confirm-choice', checked: index === 0 });
  input.value = String(choice.value);
  input.addEventListener('change', () => {
    ui.confirmBody.textContent = describe(choice).body;
  });
  label.append(input, markVerbatim(element('span', 'choice-label', choice.label), choice.verbatim));
  return label;
}

// ------------------------------------------------------------ all settings

function resetFeatures() {
  model.features = 'idle';
  ui.featureList.replaceChildren();
  delete ui.featureList.dataset.signature;
  ui.probeList.replaceChildren();
  ui.probeResults.hidden = true;
  setFeatureStatus(null);
  if (ui.more.open) void loadFeatures();
}

async function loadFeatures() {
  const monitorId = model.shownId;
  model.features = 'loading';
  if (ui.featureList.children.length === 0) setFeatureStatus('loading');
  try {
    const features = await bridge.loadFeatures(monitorId);
    if (monitorId !== model.shownId) return;
    syncFeatures(features, monitorId);
    model.features = 'ready';
    setFeatureStatus(features.length === 0 ? 'empty' : null);
  } catch (error) {
    if (monitorId !== model.shownId) return;
    model.features = 'error';
    setFeatureStatus('error', error);
  }
}

// Same features as shown: patch their values in place; else rebuild.
function syncFeatures(features, monitorId) {
  const signature = JSON.stringify(features.map(({ code, status, value }) => [code, status, value?.kind]));
  if (ui.featureList.dataset.signature === signature) {
    for (const feature of features) patchEntry(feature);
    return;
  }
  ui.featureList.dataset.signature = signature;
  ui.featureList.replaceChildren(...features.map((feature) => featureItem(feature, monitorId)));
}

function setFeatureStatus(kind, error = null) {
  ui.moreStatus.hidden = kind === null;
  ui.moreStatus.dataset.kind = kind ?? '';
  ui.moreSpinner.hidden = kind !== 'loading';
  ui.moreRetry.hidden = kind !== 'error';
  ui.moreStatusText.textContent = featureStatusText(kind, error);
}

function featureStatusText(kind, error) {
  if (kind === 'loading') return t('more.loading');
  if (kind === 'empty') return t('more.empty');
  if (kind === 'error') return errorText(error, t);
  return '';
}

function featureItem(dto, monitorId) {
  const item = element('li', 'feature');
  const seed = { kind: 'feature', monitorId, dto };
  const view = viewOf(seed);
  item.append(view.widget ? mountEntry(seed, view, featureWidget) : silentFeature(view));
  return item;
}

function featureWidget(entry, view) {
  const id = `feature-${view.code}`;
  const marks = featureMarks(view);
  if (view.widget === 'slider') return sliderWidget(entry, view, { id, marks });
  const node = element('div', 'slider');
  const head = element('div', 'slider-head');
  const label = markVerbatim(element('span', 'slider-label', view.label), view.labelVerbatim);
  label.id = `${id}-label`;
  head.append(label, ...marks);
  const widget = selectWidget(entry, id, label.id);
  node.append(head, widget.node);
  return { ...widget, node };
}

function silentFeature(view) {
  const node = element('div', 'feature-row is-silent');
  node.append(markVerbatim(element('span', 'slider-label', view.label), view.labelVerbatim), ...featureMarks(view));
  node.append(element('span', 'feature-status', view.statusText));
  return node;
}

function featureMarks(view) {
  const marks = [verbatim('span', 'code', view.hex)];
  if (view.dangerous) marks.push(element('span', 'tag', t('tag.dangerous')));
  return marks;
}

async function probe() {
  if (model.probing) return;
  const monitorId = model.shownId;
  setProbing(true);
  try {
    const found = await bridge.probeFeatures(monitorId);
    if (monitorId === model.shownId) showProbe(found, monitorId);
  } catch (error) {
    if (monitorId === model.shownId) showToast(errorText(error, t));
  } finally {
    setProbing(false);
  }
}

// Probing reads many codes a monitor never declared: the ones that answer
// become controls, the silent rest is only counted.
function showProbe(found, monitorId) {
  const answering = found.filter((feature) => feature.status === 'ok');
  const silent = found.length - answering.length;
  ui.probeList.replaceChildren(...answering.map((feature) => featureItem(feature, monitorId)));
  ui.probeList.hidden = answering.length === 0;
  const note = answering.length === 0 ? t('more.probeEmpty') : t('more.probeSilent', { count: silent });
  setText(ui.probeNote, answering.length === 0 || silent > 0 ? note : null);
  ui.probeResults.hidden = false;
}

function setProbing(busy) {
  model.probing = busy;
  ui.probe.classList.toggle('is-busy', busy);
  ui.probe.setAttribute('aria-disabled', String(busy));
  ui.probeLabel.textContent = busy ? t('more.probing') : t('more.probe');
  ui.probeResults.setAttribute('aria-busy', String(busy));
}

// ---------------------------------------------------------------- DOM bits

function element(tag, className = '', text = null) {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== null) node.textContent = text;
  return node;
}

/** A text as the monitor or the core wrote it: data, never translated. */
function verbatim(tag, className, text) {
  return markVerbatim(element(tag, className, text), true);
}

/** Marks `node` as showing data (`translate="no"`) or a translation. */
function markVerbatim(node, isVerbatim) {
  node.translate = !isVerbatim;
  return node;
}

function tile(icon) {
  const node = element('span', 'tile');
  node.setAttribute('aria-hidden', 'true');
  node.append(createIcon(icon));
  return node;
}

function setText(node, text) {
  node.textContent = text ?? '';
  node.hidden = !text;
}

// Restarts the short highlight that marks a value the monitor corrected.
function flash(node) {
  node.classList.remove('is-corrected');
  void node.getBoundingClientRect();
  node.classList.add('is-corrected');
}
