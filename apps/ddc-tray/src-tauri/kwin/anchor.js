// ddc-tray's popup placement on KDE Plasma under Wayland
// (D-2026-09-27-tray-app-3). Wayland lets no app place its own window, so
// the app loads this KWin script over D-Bus at start and unloads it when it
// quits. Each time the popup is mapped — it is unmapped when hidden — the
// script moves it next to the pointer, that is, next to the tray icon or
// the menu item it was opened from, inside the work area of the pointer's
// screen, and gives it what tauri.conf.json asks and Wayland ignores: no
// taskbar or switcher entry, above other windows. Only the window of the
// app's own process, class and title is touched.

const POPUP_PID = __PID__;
const POPUP_CLASS = "ddc-tray";
const POPUP_CAPTION = "DDC Control";
const MARGIN = 8;

function isPopup(window) {
  return window.pid === POPUP_PID && window.resourceClass === POPUP_CLASS && window.caption === POPUP_CAPTION;
}

function clamp(value, low, high) {
  return Math.min(Math.max(value, low), Math.max(low, high));
}

function place(window) {
  const pointer = workspace.cursorPos;
  const output = workspace.screenAt(pointer) || window.output;
  const area = workspace.clientArea(KWin.MaximizeArea, output, workspace.currentDesktop);
  const frame = window.frameGeometry;
  const x = clamp(pointer.x - frame.width / 2, area.x + MARGIN, area.x + area.width - frame.width - MARGIN);
  const y = clamp(pointer.y - frame.height / 2, area.y + MARGIN, area.y + area.height - frame.height - MARGIN);
  window.skipTaskbar = true;
  window.skipSwitcher = true;
  window.skipPager = true;
  window.keepAbove = true;
  window.frameGeometry = { x: Math.round(x), y: Math.round(y), width: frame.width, height: frame.height };
}

workspace.windowAdded.connect((window) => {
  if (isPopup(window)) place(window);
});
