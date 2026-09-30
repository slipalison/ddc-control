#!/usr/bin/env python3
"""Reads and clicks the menu of a StatusNotifierItem, for smoke-autostart-private.sh.

The item's `Menu` property names the object that serves the menu over
com.canonical.dbusmenu (ksni puts it on the item's own connection). This is
what a desktop's tray host does when the user opens the menu and clicks an entry.

Usage:
  sni-dbusmenu.py item <pid>
      Prints "<service> <item-path>" of the item the session bus's
      StatusNotifierWatcher lists that is owned by process <pid>; exits 1 when
      there is none yet.
  sni-dbusmenu.py state <service> <item-path> <label>
      Prints "<toggle-type> <toggle-state>" of the menu entry labelled
      <label>: "checkmark 0" unmarked, "checkmark 1" marked, "none -1" for an
      entry that is not checkable. Exits 1 when no entry has that label.
  sni-dbusmenu.py click <service> <item-path> <label>
      Sends the entry labelled <label> the "clicked" event a host sends on a
      click. Exits 1 when no entry has that label.

Needs python3-gobject (Gio) and a session bus. Nothing here touches a monitor.
"""
import sys
import time
import warnings

import gi

gi.require_version("Gio", "2.0")
from gi.repository import Gio, GLib  # noqa: E402

CALL_TIMEOUT_MS = 5000
WATCHER = "org.kde.StatusNotifierWatcher"
WATCHER_PATH = "/StatusNotifierWatcher"
ITEM_INTERFACE = "org.kde.StatusNotifierItem"
DEFAULT_ITEM_PATH = "/StatusNotifierItem"
DBUSMENU = "com.canonical.dbusmenu"
ROOT_ID = 0
WHOLE_TREE = -1


def call(bus, service, path, interface, method, params, reply_type):
    """One blocking method call; `params` is a GLib.Variant tuple or None."""
    reply = bus.call_sync(
        service,
        path,
        interface,
        method,
        params,
        GLib.VariantType(reply_type),
        Gio.DBusCallFlags.NONE,
        CALL_TIMEOUT_MS,
        None,
    )
    return reply.unpack()


def get_property(bus, service, path, interface, name):
    params = GLib.Variant("(ss)", (interface, name))
    (value,) = call(bus, service, path, "org.freedesktop.DBus.Properties", "Get", params, "(v)")
    return value


def item_service_and_path(entry):
    """Splits a watcher entry: ":1.N/path", or a well-known name with no path."""
    service, slash, rest = entry.partition("/")
    return service, (slash + rest) if slash else DEFAULT_ITEM_PATH


def owner_pid(bus, service):
    params = GLib.Variant("(s)", (service,))
    (pid,) = call(
        bus, "org.freedesktop.DBus", "/org/freedesktop/DBus", "org.freedesktop.DBus",
        "GetConnectionUnixProcessID", params, "(u)",
    )
    return pid


def find_item(bus, pid):
    entries = get_property(bus, WATCHER, WATCHER_PATH, WATCHER, "RegisteredStatusNotifierItems")
    for entry in entries:
        service, path = item_service_and_path(entry)
        if owner_pid(bus, service) == pid:
            return service, path
    return None


def menu_layout(bus, service, item_path):
    """The menu tree as (id, properties, children), children being such tuples."""
    menu_path = get_property(bus, service, item_path, ITEM_INTERFACE, "Menu")
    # A host announces the menu before it reads it.
    call(bus, service, menu_path, DBUSMENU, "AboutToShow", GLib.Variant("(i)", (ROOT_ID,)), "(b)")
    params = GLib.Variant("(iias)", (ROOT_ID, WHOLE_TREE, []))
    _revision, layout = call(bus, service, menu_path, DBUSMENU, "GetLayout", params, "(u(ia{sv}av))")
    return menu_path, layout


def find_entry(layout, label):
    """The (id, properties) of the first entry labelled `label`, or None."""
    entry_id, properties, children = layout
    if properties.get("label") == label:
        return entry_id, properties
    for child in children:
        found = find_entry(child, label)
        if found:
            return found
    return None


def labelled_entry(bus, service, item_path, label):
    menu_path, layout = menu_layout(bus, service, item_path)
    found = find_entry(layout, label)
    if not found:
        print(f"sni-dbusmenu: no menu entry labelled {label!r}", file=sys.stderr)
        sys.exit(1)
    return menu_path, found


def command_item(bus, pid):
    found = find_item(bus, int(pid))
    if not found:
        sys.exit(1)
    print(*found)


def command_state(bus, service, item_path, label):
    _menu_path, (_entry_id, properties) = labelled_entry(bus, service, item_path, label)
    print(properties.get("toggle-type") or "none", properties.get("toggle-state", -1))


def command_click(bus, service, item_path, label):
    menu_path, (entry_id, _properties) = labelled_entry(bus, service, item_path, label)
    timestamp = int(time.time()) & 0xFFFFFFFF
    params = GLib.Variant(
        "(isvu)", (entry_id, "clicked", GLib.Variant("i", 0), timestamp)
    )
    call(bus, service, menu_path, DBUSMENU, "Event", params, "()")


COMMANDS = {"item": (command_item, 1), "state": (command_state, 3), "click": (command_click, 3)}


def main(argv):
    if not argv or argv[0] not in COMMANDS or len(argv) - 1 != COMMANDS[argv[0]][1]:
        print(__doc__, file=sys.stderr)
        return 2
    command, _arity = COMMANDS[argv[0]]
    try:
        bus = Gio.bus_get_sync(Gio.BusType.SESSION, None)
        command(bus, *argv[1:])
    except GLib.Error as error:
        print(f"sni-dbusmenu: {error.message}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    warnings.filterwarnings("ignore", category=DeprecationWarning)
    sys.exit(main(sys.argv[1:]))
