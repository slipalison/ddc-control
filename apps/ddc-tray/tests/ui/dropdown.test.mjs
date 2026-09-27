import { test } from 'node:test';
import assert from 'node:assert/strict';

import {
  LIST_GAP,
  LIST_MARGIN,
  PAGE_SIZE,
  dropdownReduce,
  dropdownState,
  placeList,
} from '../../src/dropdown.js';

// The in-page dropdown of D-2026-09-27-tray-app-1, as the pure function
// the DOM binding runs: what each key, click and dismissal does. Its DOM
// is checked in a browser by tests/e2e/dropdown.spec.mjs.

const PRESETS = Object.freeze([
  { value: 0x01, label: 'sRGB' },
  { value: 0x02, label: 'Native' },
  { value: 0x04, label: '5000 K' },
  { value: 0x05, label: '6500 K' },
  { value: 0x0b, label: 'User 1' },
]);

/** The dropdown showing `value`, after `events` in order. */
function after(value, ...events) {
  return events.reduce((outcome, event) => dropdownReduce(outcome.state, event), {
    state: dropdownState(PRESETS, value),
  });
}

const key = (name, altKey = false) => ({ type: 'key', key: name, altKey });
const OPEN = key('Enter');

test('a new dropdown is closed, on the option of its value', () => {
  assert.deepEqual(dropdownState(PRESETS, 0x04), { open: false, options: PRESETS, selected: 2, active: 2 });
});

test('a value that is not listed selects nothing and starts on the first option', () => {
  assert.deepEqual(dropdownState(PRESETS, 0x7f), { open: false, options: PRESETS, selected: -1, active: 0 });
  assert.deepEqual(dropdownState(), { open: false, options: [], selected: -1, active: -1 });
});

test('Enter, Space, ArrowDown, ArrowUp and Alt+ArrowDown open the list on the current option', () => {
  for (const event of [key('Enter'), key(' '), key('ArrowDown'), key('ArrowUp'), key('ArrowDown', true)]) {
    const { state, change, focus, handled } = after(0x04, event);
    assert.deepEqual(
      { open: state.open, active: state.active, selected: state.selected, change, focus, handled },
      { open: true, active: 2, selected: 2, change: null, focus: false, handled: true },
      JSON.stringify(event),
    );
  }
});

test('Home and End open the list on the first and the last option', () => {
  assert.deepEqual([after(0x04, key('Home')).state.open, after(0x04, key('Home')).state.active], [true, 0]);
  assert.deepEqual([after(0x04, key('End')).state.open, after(0x04, key('End')).state.active], [true, 4]);
});

test('on a closed list, other keys are not the dropdown’s: Esc and Tab pass through', () => {
  for (const name of ['Escape', 'Tab', 'a', 'ArrowLeft']) {
    const { state, handled, change } = after(0x04, key(name));
    assert.deepEqual([state.open, handled, change], [false, false, null], name);
  }
});

test('ArrowDown and ArrowUp move one option and stop at the ends', () => {
  assert.equal(after(0x04, OPEN, key('ArrowDown')).state.active, 3);
  assert.equal(after(0x04, OPEN, key('ArrowUp')).state.active, 1);
  assert.equal(after(0x0b, OPEN, key('ArrowDown'), key('ArrowDown')).state.active, 4);
  assert.equal(after(0x01, OPEN, key('ArrowUp'), key('ArrowUp')).state.active, 0);
});

test('Home, End, PageUp and PageDown jump, staying inside the list', () => {
  assert.equal(PAGE_SIZE, 10);
  assert.equal(after(0x04, OPEN, key('Home')).state.active, 0);
  assert.equal(after(0x04, OPEN, key('End')).state.active, 4);
  assert.equal(after(0x04, OPEN, key('PageDown')).state.active, 4);
  assert.equal(after(0x04, OPEN, key('PageUp')).state.active, 0);
});

test('moving never changes the value nor closes the list', () => {
  const { state, change } = after(0x04, OPEN, key('ArrowDown'), key('End'), key('Home'));

  assert.deepEqual([state.open, state.selected, change], [true, 2, null]);
});

test('Enter picks the active option, closes the list and gives the focus back', () => {
  const outcome = after(0x04, OPEN, key('ArrowDown'), key('Enter'));

  assert.deepEqual(outcome, {
    state: { open: false, options: PRESETS, selected: 3, active: 3 },
    change: { value: 0x05 },
    focus: true,
    handled: true,
  });
});

test('Space and Alt+ArrowUp pick the active option too', () => {
  assert.deepEqual(after(0x04, OPEN, key('ArrowUp'), key(' ')).change, { value: 0x02 });
  assert.deepEqual(after(0x04, OPEN, key('End'), key('ArrowUp', true)).change, { value: 0x0b });
});

test('picking the value already shown closes the list without a change', () => {
  const { state, change, focus } = after(0x04, OPEN, key('Enter'));

  assert.deepEqual([state.open, state.selected, change, focus], [false, 2, null, true]);
});

test('Esc closes the list without a change, keeps the key from the popup and gives the focus back', () => {
  const outcome = after(0x04, OPEN, key('ArrowDown'), key('ArrowDown'), key('Escape'));

  assert.deepEqual(outcome, {
    state: { open: false, options: PRESETS, selected: 2, active: 4 },
    change: null,
    focus: true,
    handled: true,
  });
});

test('Tab closes the list without a change and lets the focus move on', () => {
  const { state, change, focus, handled } = after(0x04, OPEN, key('ArrowDown'), key('Tab'));

  assert.deepEqual([state.open, state.selected, change, focus, handled], [false, 2, null, false, false]);
});

test('a click on the button opens the list, and a second one closes it unchanged', () => {
  const opened = after(0x04, { type: 'toggle' });
  const closed = dropdownReduce(opened.state, { type: 'toggle' });

  assert.deepEqual([opened.state.open, opened.state.active], [true, 2]);
  assert.deepEqual([closed.state.open, closed.change, closed.focus], [false, null, true]);
});

test('a click on an option picks it', () => {
  const { state, change, focus } = after(0x04, { type: 'toggle' }, { type: 'pick', index: 0 });

  assert.deepEqual([state.open, state.selected, change, focus], [false, 0, { value: 0x01 }, true]);
});

test('a click outside, a scroll or the focus leaving close the list and change nothing', () => {
  const { state, change, focus, handled } = after(0x04, OPEN, key('End'), { type: 'dismiss' });

  assert.deepEqual([state.open, state.selected, change, focus, handled], [false, 2, null, false, false]);
});

test('a dismissal of a closed list and a pick without an open list do nothing', () => {
  assert.deepEqual(after(0x04, { type: 'dismiss' }).state, dropdownState(PRESETS, 0x04));
  assert.deepEqual(after(0x04, { type: 'pick', index: 0 }).change, null);
  assert.equal(after(0x04, { type: 'pick', index: 0 }).state.open, false);
});

test('hovering an option makes it the active one', () => {
  assert.equal(after(0x04, OPEN, { type: 'hover', index: 4 }).state.active, 4);
  assert.equal(after(0x04, { type: 'hover', index: 4 }).state.active, 2, 'a closed list ignores the pointer');
});

test('a disabled option is shown but skipped, never picked', () => {
  const options = [{ value: 0x7f, label: 'Value 0x7F', disabled: true }, ...PRESETS];
  const start = { state: dropdownState(options, 0x7f) };
  const run = (...events) => events.reduce((outcome, event) => dropdownReduce(outcome.state, event), start);

  assert.equal(run(OPEN).state.active, 1, 'opens on the first option that can be picked');
  assert.equal(run(OPEN, key('Home')).state.active, 1);
  assert.equal(run(OPEN, key('ArrowUp')).state.active, 1);
  assert.equal(run(OPEN, { type: 'hover', index: 0 }).state.active, 1);
  assert.deepEqual(run(OPEN, { type: 'pick', index: 0 }).change, null);
  assert.equal(run(OPEN, { type: 'pick', index: 0 }).state.open, true);
  assert.deepEqual(run(OPEN, key('Enter')).change, { value: 0x01 });
});

test('an empty dropdown never opens', () => {
  const outcome = dropdownReduce(dropdownState([], null), key('Enter'));

  assert.deepEqual([outcome.state.open, outcome.change], [false, null]);
});

test('the reducer leaves the state it was given untouched', () => {
  const state = dropdownState(PRESETS, 0x04);
  const before = structuredClone(state);

  dropdownReduce(dropdownReduce(state, OPEN).state, key('End'));

  assert.deepEqual(state, before);
});

// ------------------------------------------------------------ placement

const WINDOW = Object.freeze({ top: 0, left: 0, right: 360, bottom: 560 });

function anchor(top, left, width = 100, height = 32) {
  return { top, bottom: top + height, left, right: left + width, width };
}

test('the list is placed 4 px below or above the button, 8 px from the window edge', () => {
  assert.equal(LIST_GAP, 4);
  assert.equal(LIST_MARGIN, 8);
});

test('a list that fits below the button opens below it, at its natural height', () => {
  const spot = placeList(anchor(230, 240, 96), WINDOW, { width: 120, height: 220 });

  assert.deepEqual(spot, { side: 'below', top: 266, left: 216, width: 120, maxHeight: 220 });
});

test('a list with no room below opens above the button', () => {
  const spot = placeList(anchor(460, 24, 312), WINDOW, { width: 200, height: 220 });

  assert.deepEqual(spot, { side: 'above', top: 236, left: 24, width: 312, maxHeight: 220 });
});

test('a list taller than the room on either side scrolls inside the side with more room', () => {
  const below = placeList(anchor(100, 24), WINDOW, { width: 100, height: 900 });
  const above = placeList(anchor(400, 24), WINDOW, { width: 100, height: 900 });

  assert.deepEqual(below, { side: 'below', top: 136, left: 24, width: 100, maxHeight: 416 });
  assert.deepEqual(above, { side: 'above', top: 8, left: 24, width: 100, maxHeight: 388 });
  for (const spot of [below, above]) {
    assert.ok(spot.top >= WINDOW.top + LIST_MARGIN, 'inside the top of the window');
    assert.ok(spot.top + spot.maxHeight <= WINDOW.bottom - LIST_MARGIN, 'inside the bottom of the window');
  }
});

test('a list wider than the window is cut to it and kept inside', () => {
  const spot = placeList(anchor(40, 54, 130), WINDOW, { width: 900, height: 100 });

  assert.deepEqual([spot.left, spot.width], [8, 344]);
});

test('a list never narrower than its button, and lined up with its right edge near the window’s', () => {
  const narrow = placeList(anchor(40, 54, 130), WINDOW, { width: 60, height: 100 });
  const wide = placeList(anchor(230, 240, 96), WINDOW, { width: 200, height: 100 });

  assert.equal(narrow.width, 130);
  assert.deepEqual([wide.left, wide.left + wide.width], [136, 336]);
});

test('bounds that are not at the origin (the demo page) are honoured', () => {
  const frame = { top: 100, left: 500, right: 860, bottom: 660 };
  const spot = placeList(anchor(600, 520, 100), frame, { width: 100, height: 200 });

  assert.deepEqual(spot, { side: 'above', top: 396, left: 520, width: 100, maxHeight: 200 });
});
