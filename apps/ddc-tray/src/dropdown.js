// The popup's dropdown (D-2026-09-27-tray-app-1): the WAI-ARIA select-only
// combobox, drawn inside the page. A native select element opens its list
// as a window of its own (WebKitGTK and WebView2 alike): the popup lost the
// focus to it and hid on blur, so picking a value closed the panel.
//
// `dropdownReduce` is the whole behaviour, as a pure function of a state
// and an event; `placeList` says where the list fits in the window; and
// `createDropdown` only binds them to the DOM. Texts come from the caller,
// and the only style the script sets is a `--dropdown-*` custom property.
// A `verbatim` label is the monitor's own (its name, a value name no locale
// translates): it is shown under `translate="no"` (D-2026-09-27-tray-app-7).

import { createIcon } from './icons.js';

/** How far PageUp and PageDown move, in options. */
export const PAGE_SIZE = 10;

/** Room kept between the list and the edge of the window, in px. */
export const LIST_MARGIN = 8;

/** Room between the button and the list, in px. */
export const LIST_GAP = 4;

/**
 * @typedef {{ value: unknown, label: string, disabled?: boolean, verbatim?: boolean }} DropdownOption
 * @typedef {{
 *   open: boolean,
 *   options: readonly DropdownOption[],
 *   selected: number,
 *   active: number,
 * }} DropdownState
 * `selected` is the index of the value shown (-1 when it is not listed);
 * `active`, the option the keyboard or the pointer is on (-1 when none
 * can be picked).
 * @typedef {{
 *   state: DropdownState,
 *   change: { value: unknown } | null,
 *   focus: boolean,
 *   handled: boolean,
 * }} DropdownOutcome
 * `change` is the value the user picked, only when it differs from the one
 * shown; `focus`, whether the focus goes back to the button; `handled`,
 * whether the key was the dropdown's (its default action is prevented).
 */

/**
 * A closed dropdown showing `value` among `options`.
 * @param {readonly DropdownOption[]} [options]
 * @param {unknown} [value]
 * @returns {DropdownState}
 */
export function dropdownState(options = [], value = undefined) {
  const selected = options.findIndex((option) => Object.is(option.value, value));
  return { open: false, options, selected, active: startIndex(options, selected) };
}

/**
 * What `event` does to the dropdown in `state`:
 * - `{ type: 'toggle' }`: the button was clicked;
 * - `{ type: 'key', key, altKey }`: a key was pressed on the button;
 * - `{ type: 'pick', index }`: an option was clicked;
 * - `{ type: 'hover', index }`: the pointer moved onto an option;
 * - `{ type: 'dismiss' }`: a click outside, a scroll that takes the button
 *   out of sight, a resize or the focus leaving it — closes the list and
 *   changes nothing.
 * @param {DropdownState} state
 * @param {{ type: string, key?: string, altKey?: boolean, index?: number }} event
 * @returns {DropdownOutcome}
 */
export function dropdownReduce(state, event) {
  switch (event.type) {
    case 'toggle':
      return state.open ? closed(state, { focus: true }) : opened(state, startIndex(state.options, state.selected));
    case 'key':
      return state.open ? keyWhenOpen(state, event) : keyWhenClosed(state, event);
    case 'pick':
      return state.open && isEnabled(state.options, event.index) ? chosen(state, event.index) : same(state);
    case 'hover':
      return state.open && isEnabled(state.options, event.index) ? moved(state, event.index) : same(state);
    case 'dismiss':
      return state.open ? closed(state, { handled: false }) : same(state);
    default:
      return same(state);
  }
}

/**
 * Where the list goes: below the button when it fits there, else on the
 * side with more room, always inside `bounds`; a list taller than that
 * room scrolls. At least as wide as the button, never wider than the
 * window; lined up with the button's left edge, or with its right edge
 * when it would pass the window's.
 * @param {{ top: number, bottom: number, left: number, right: number, width: number }} anchor the button's box
 * @param {{ top: number, bottom: number, left: number, right: number }} bounds the window's box
 * @param {{ width: number, height: number }} list the list's natural size
 * @param {{ margin?: number, gap?: number }} [spacing]
 * @returns {{ side: 'below' | 'above', top: number, left: number, width: number, maxHeight: number }}
 */
export function placeList(anchor, bounds, list, { margin = LIST_MARGIN, gap = LIST_GAP } = {}) {
  const below = Math.max(0, bounds.bottom - margin - (anchor.bottom + gap));
  const above = Math.max(0, anchor.top - gap - (bounds.top + margin));
  const side = list.height <= below || below >= above ? 'below' : 'above';
  const maxHeight = Math.min(list.height, side === 'below' ? below : above);
  const top = side === 'below' ? anchor.bottom + gap : anchor.top - gap - maxHeight;
  const room = Math.max(0, bounds.right - bounds.left - 2 * margin);
  const width = Math.min(Math.max(list.width, anchor.width), room);
  const preferred = anchor.left + width <= bounds.right - margin ? anchor.left : anchor.right - width;
  const left = Math.min(Math.max(preferred, bounds.left + margin), bounds.right - margin - width);
  return { side, top, left, width, maxHeight };
}

// ------------------------------------------------------------- transitions

function keyWhenClosed(state, { key }) {
  if (key === 'Enter' || key === ' ' || key === 'ArrowDown' || key === 'ArrowUp') {
    return opened(state, startIndex(state.options, state.selected));
  }
  if (key === 'Home') return opened(state, nextEnabled(state.options, -1, 1));
  if (key === 'End') return opened(state, nextEnabled(state.options, state.options.length, -1));
  return same(state, { handled: false });
}

function keyWhenOpen(state, { key, altKey = false }) {
  const { options, active } = state;
  switch (key) {
    case 'ArrowDown':
      return moved(state, walk(options, active, 1, 1));
    case 'ArrowUp':
      return altKey ? chooseActive(state) : moved(state, walk(options, active, -1, 1));
    case 'PageDown':
      return moved(state, walk(options, active, 1, PAGE_SIZE));
    case 'PageUp':
      return moved(state, walk(options, active, -1, PAGE_SIZE));
    case 'Home':
      return moved(state, nextEnabled(options, -1, 1));
    case 'End':
      return moved(state, nextEnabled(options, options.length, -1));
    case 'Enter':
    case ' ':
      return chooseActive(state);
    case 'Escape':
      return closed(state, { focus: true });
    case 'Tab':
      return closed(state, { handled: false });
    default:
      return same(state, { handled: false });
  }
}

function opened(state, active) {
  if (state.options.length === 0) return same(state);
  return outcome({ ...state, open: true, active });
}

function closed(state, { focus = false, handled = true } = {}) {
  return outcome({ ...state, open: false }, { focus, handled });
}

function moved(state, active) {
  return outcome({ ...state, active: active < 0 ? state.active : active });
}

function chooseActive(state) {
  if (!isEnabled(state.options, state.active)) return closed(state, { focus: true });
  return chosen(state, state.active);
}

function chosen(state, index) {
  const change = index === state.selected ? null : { value: state.options[index].value };
  return outcome({ ...state, open: false, selected: index, active: index }, { change, focus: true });
}

function same(state, { handled = true } = {}) {
  return outcome(state, { handled });
}

function outcome(state, { change = null, focus = false, handled = true } = {}) {
  return { state, change, focus, handled };
}

// ----------------------------------------------------------------- options

function isEnabled(options, index) {
  return Number.isInteger(index) && index >= 0 && index < options.length && !options[index].disabled;
}

/** The option a list opens on: the one shown, else the first enabled one. */
function startIndex(options, selected) {
  return isEnabled(options, selected) ? selected : nextEnabled(options, -1, 1);
}

/** The first enabled option after `from` in `direction` (±1), or -1. */
function nextEnabled(options, from, direction) {
  for (let index = from + direction; index >= 0 && index < options.length; index += direction) {
    if (isEnabled(options, index)) return index;
  }
  return -1;
}

/** `count` enabled options from `from` in `direction`, stopping at the last one there is. */
function walk(options, from, direction, count) {
  let reached = from;
  for (let step = 0; step < count; step += 1) {
    const next = nextEnabled(options, reached, direction);
    if (next < 0) break;
    reached = next;
  }
  return reached;
}

// --------------------------------------------------------------------- DOM

/**
 * A dropdown bound to the DOM: `node` goes in the page, `update` shows new
 * options and a new value, and `onChange(value)` hears a value the user
 * picked — the dropdown shows it until the next `update`.
 * @param {{
 *   id: string,
 *   label?: string | null,
 *   labelledBy?: string | null,
 *   className?: string,
 *   onChange: (value: unknown) => void,
 *   bounds?: (() => { top: number, bottom: number, left: number, right: number }) | null,
 *   doc?: Document,
 * }} options `label` or `labelledBy` names it; `bounds` is the box the
 * list must stay in (the window by default).
 */
export function createDropdown({
  id,
  label = null,
  labelledBy = null,
  className = '',
  onChange,
  bounds = null,
  doc = globalThis.document,
}) {
  const win = doc.defaultView;
  const node = element(doc, 'div', ['dropdown', className].filter(Boolean).join(' '));
  const button = comboButton(doc, id, { label, labelledBy });
  const shown = element(doc, 'span', 'dropdown-value');
  const chevron = element(doc, 'span', 'dropdown-chevron');
  chevron.setAttribute('aria-hidden', 'true');
  chevron.append(createIcon('chevron', doc));
  button.append(shown, chevron);
  const list = listbox(doc, `${id}-list`, { label, labelledBy });
  node.append(button, list);

  let state = dropdownState();
  let signature = '';
  let items = [];

  const frame = () => bounds?.() ?? { top: 0, left: 0, right: win.innerWidth, bottom: win.innerHeight };
  const setVar = (name, value) => list.style.setProperty(`--dropdown-${name}`, value);
  const dismissOutside = (event) => {
    if (!node.contains(event.target)) dispatch({ type: 'dismiss' });
  };
  // A scroll moves the list with its button, and closes it once the
  // button is out of sight. (A scroll event arrives a frame late, so one
  // from before the list opened must not close it.)
  const followScroll = (event) => {
    if (event.target === list) return;
    const view = event.target instanceof win.Element ? event.target.getBoundingClientRect() : frame();
    const box = button.getBoundingClientRect();
    if (box.bottom <= view.top || box.top >= view.bottom) dispatch({ type: 'dismiss' });
    else place();
  };
  const dismiss = () => dispatch({ type: 'dismiss' });

  function dispatch(event, domEvent = null) {
    const result = dropdownReduce(state, event);
    if (result.handled) domEvent?.preventDefault();
    const wasOpen = state.open;
    state = result.state;
    render(wasOpen);
    if (result.focus) button.focus();
    if (result.change) onChange(result.change.value);
  }

  function render(wasOpen) {
    const { open, options, selected, active } = state;
    const current = options[selected];
    shown.textContent = current?.label ?? '';
    shown.translate = !current?.verbatim;
    button.dataset.value = current ? String(current.value) : '';
    button.setAttribute('aria-expanded', String(open));
    node.classList.toggle('is-open', open);
    paintOptions(options, selected, active);
    if (open && active >= 0) button.setAttribute('aria-activedescendant', optionId(active));
    else button.removeAttribute('aria-activedescendant');
    list.hidden = !open;
    if (open !== wasOpen) listen(open);
    if (open) {
      place();
      reveal(items[active]);
    }
  }

  function paintOptions(options, selected, active) {
    const next = JSON.stringify(
      options.map((option) => [String(option.value), option.label, Boolean(option.disabled), Boolean(option.verbatim)]),
    );
    if (next !== signature) {
      signature = next;
      items = options.map((option, index) => optionItem(doc, optionId(index), option));
      list.replaceChildren(...items);
    }
    items.forEach((item, index) => {
      item.setAttribute('aria-selected', String(index === selected));
      item.classList.toggle('is-active', index === active);
    });
  }

  function listen(open) {
    const method = open ? 'addEventListener' : 'removeEventListener';
    doc[method]('pointerdown', dismissOutside, true);
    doc[method]('scroll', followScroll, true);
    win[method]('resize', dismiss);
    win[method]('blur', dismiss);
  }

  // Measured at its natural size, then placed where it fits.
  function place() {
    setVar('top', '0px');
    setVar('left', '0px');
    setVar('width', 'max-content');
    setVar('max-height', 'none');
    const size = list.getBoundingClientRect();
    const natural = { width: Math.ceil(size.width), height: Math.ceil(size.height) };
    const spot = placeList(button.getBoundingClientRect(), frame(), natural);
    node.dataset.side = spot.side;
    setVar('top', `${spot.top}px`);
    setVar('left', `${spot.left}px`);
    setVar('width', `${spot.width}px`);
    setVar('max-height', `${spot.maxHeight}px`);
  }

  // Scrolls the list, and only the list, to show `item`.
  function reveal(item) {
    if (!item) return;
    const top = item.offsetTop;
    const bottom = top + item.offsetHeight;
    if (top < list.scrollTop) list.scrollTop = top;
    else if (bottom > list.scrollTop + list.clientHeight) list.scrollTop = bottom - list.clientHeight;
  }

  function optionId(index) {
    return `${id}-option-${index}`;
  }

  button.addEventListener('click', () => dispatch({ type: 'toggle' }));
  button.addEventListener('keydown', (event) =>
    dispatch({ type: 'key', key: event.key, altKey: event.altKey }, event),
  );
  button.addEventListener('focusout', (event) => {
    if (!node.contains(event.relatedTarget)) dispatch({ type: 'dismiss' });
  });
  // A press on the list must not take the focus from the button.
  list.addEventListener('pointerdown', (event) => event.preventDefault());
  list.addEventListener('click', (event) => {
    const index = indexOf(event.target, items);
    if (index >= 0) dispatch({ type: 'pick', index });
  });
  list.addEventListener('pointermove', (event) => {
    const index = indexOf(event.target, items);
    if (index >= 0 && index !== state.active) dispatch({ type: 'hover', index });
  });

  return {
    node,
    button,
    list,
    /** Shows `options` and `value`; an open list stays open on its option. */
    update({ options, value }) {
      const next = dropdownState(options, value);
      const keep = state.open && isEnabled(options, state.active);
      const wasOpen = state.open;
      state = { ...next, open: state.open && options.length > 0, active: keep ? state.active : next.active };
      render(wasOpen);
    },
    /** Closes the list without a change. */
    close: () => dispatch({ type: 'dismiss' }),
    isOpen: () => state.open,
  };
}

function comboButton(doc, id, { label, labelledBy }) {
  const button = element(doc, 'div', 'dropdown-button');
  button.id = id;
  button.tabIndex = 0;
  button.setAttribute('role', 'combobox');
  button.setAttribute('aria-haspopup', 'listbox');
  button.setAttribute('aria-expanded', 'false');
  button.setAttribute('aria-controls', `${id}-list`);
  name(button, { label, labelledBy });
  return button;
}

function listbox(doc, id, { label, labelledBy }) {
  const list = element(doc, 'ul', 'dropdown-list');
  list.id = id;
  list.tabIndex = -1;
  list.hidden = true;
  list.setAttribute('role', 'listbox');
  name(list, { label, labelledBy });
  return list;
}

function optionItem(doc, id, option) {
  const item = element(doc, 'li', 'dropdown-option');
  item.id = id;
  item.dataset.value = String(option.value);
  item.setAttribute('role', 'option');
  if (option.disabled) item.setAttribute('aria-disabled', 'true');
  const check = element(doc, 'span', 'dropdown-check');
  check.setAttribute('aria-hidden', 'true');
  check.append(createIcon('check', doc));
  const label = element(doc, 'span', 'dropdown-label', option.label);
  label.translate = !option.verbatim;
  item.append(check, label);
  return item;
}

function name(node, { label, labelledBy }) {
  if (labelledBy) node.setAttribute('aria-labelledby', labelledBy);
  else if (label) node.setAttribute('aria-label', label);
}

function indexOf(target, items) {
  const item = target?.closest?.('[role="option"]');
  return item ? items.indexOf(item) : -1;
}

function element(doc, tag, className = '', text = null) {
  const node = doc.createElement(tag);
  if (className) node.className = className;
  if (text !== null) node.textContent = text;
  return node;
}
