import { test } from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import vm from 'node:vm';

// The KWin script that places the popup on KDE Plasma under Wayland
// (D-2026-09-27-tray-app-3), run as KWin runs it — a plain script with a
// `workspace` and a `KWin` in scope — against fakes of both. The Rust side
// fills in the process id; the live check on Plasma is in the SUMMARY.

const SOURCE = readFileSync(new URL('../../src-tauri/kwin/anchor.js', import.meta.url), 'utf8');
const PID = 4242;
const MAXIMIZE_AREA = 2;

/** The dev machine's top screen: 3072×1728 with a 34 px panel on top. */
const TOP_PANEL_AREA = Object.freeze({ x: 0, y: 34, width: 3072, height: 1694 });

/** The script loaded against a fake KWin; `map(window)` maps a window. */
function kwin({ pointer, area = TOP_PANEL_AREA }) {
  const listeners = [];
  const asked = [];
  const output = { name: 'HDMI-A-2' };
  const workspace = {
    cursorPos: pointer,
    currentDesktop: { id: 'desktop-1' },
    windowAdded: { connect: (listener) => listeners.push(listener) },
    screenAt: (point) => {
      asked.push(['screenAt', point]);
      return output;
    },
    clientArea: (option, screen, desktop) => {
      asked.push(['clientArea', option, screen.name, desktop.id]);
      return area;
    },
  };
  vm.runInNewContext(SOURCE.replace('__PID__', String(PID)), { workspace, KWin: { MaximizeArea: MAXIMIZE_AREA } });
  return {
    asked,
    listeners,
    map: (window) => listeners.forEach((listener) => listener(window)),
  };
}

/** The geometry the script gave `window`, as a plain object of this realm. */
function geometry(window) {
  return { ...window.frameGeometry };
}

function popup(overrides = {}) {
  return {
    pid: PID,
    resourceClass: 'ddc-tray',
    caption: 'DDC Control',
    output: { name: 'HDMI-A-2' },
    frameGeometry: { x: 1356, y: 601, width: 360, height: 560 },
    skipTaskbar: false,
    skipSwitcher: false,
    skipPager: false,
    keepAbove: false,
    ...overrides,
  };
}

test('the script waits for windows to be mapped and asks nothing before', () => {
  const { listeners, asked } = kwin({ pointer: { x: 2338, y: 17 } });

  assert.equal(listeners.length, 1);
  assert.deepEqual(asked, []);
});

test('opened from an icon in a top panel, the popup hangs below the panel, centred on the icon', () => {
  const { map, asked } = kwin({ pointer: { x: 2338, y: 17 } });
  const window = popup();

  map(window);

  assert.deepEqual(geometry(window), { x: 2158, y: 42, width: 360, height: 560 });
  assert.deepEqual(asked, [
    ['screenAt', { x: 2338, y: 17 }],
    ['clientArea', MAXIMIZE_AREA, 'HDMI-A-2', 'desktop-1'],
  ]);
});

test('opened from an icon in a bottom panel, the popup sits on the panel', () => {
  const { map } = kwin({ pointer: { x: 1500, y: 1060 }, area: { x: 0, y: 0, width: 1920, height: 1040 } });
  const window = popup();

  map(window);

  assert.deepEqual(geometry(window), { x: 1320, y: 472, width: 360, height: 560 });
});

test('near a corner the popup stays 8 px inside the work area', () => {
  const right = kwin({ pointer: { x: 3070, y: 10 } });
  const left = kwin({ pointer: { x: 3, y: 1700 } });
  const topRight = popup();
  const bottomLeft = popup();

  right.map(topRight);
  left.map(bottomLeft);

  assert.deepEqual(geometry(topRight), { x: 2704, y: 42, width: 360, height: 560 });
  assert.deepEqual(geometry(bottomLeft), { x: 8, y: 1160, width: 360, height: 560 });
});

test('away from any edge the popup is centred on the pointer', () => {
  const { map } = kwin({ pointer: { x: 2338, y: 471 } });
  const window = popup();

  map(window);

  assert.deepEqual(geometry(window), { x: 2158, y: 191, width: 360, height: 560 });
});

test('a screen that is not at the origin is honoured, and fractional positions are rounded', () => {
  const { map } = kwin({ pointer: { x: 1000.4, y: 2900 }, area: { x: 580, y: 1728, width: 2048, height: 1280 } });
  const window = popup();

  map(window);

  assert.deepEqual(geometry(window), { x: 820, y: 2440, width: 360, height: 560 });
});

test('the popup gets what tauri.conf.json asks and Wayland ignores: no taskbar entry, kept above', () => {
  const { map } = kwin({ pointer: { x: 2338, y: 17 } });
  const window = popup();

  map(window);

  assert.deepEqual(
    [window.skipTaskbar, window.skipSwitcher, window.skipPager, window.keepAbove],
    [true, true, true, true],
  );
});

test('only the popup of the app’s own process, class and title is touched', () => {
  const { map } = kwin({ pointer: { x: 2338, y: 17 } });
  const others = [
    popup({ pid: PID + 1 }),
    popup({ resourceClass: 'org.kde.konsole' }),
    popup({ caption: 'DDC Control — other' }),
  ];

  for (const window of others) map(window);

  for (const window of others) {
    assert.deepEqual(geometry(window), { x: 1356, y: 601, width: 360, height: 560 });
    assert.equal(window.skipTaskbar, false);
  }
});

test('the process id is the only blank the app fills in', () => {
  assert.equal(SOURCE.match(/__PID__/g)?.length, 1);
  assert.match(SOURCE, /^const POPUP_PID = __PID__;$/m);
});
