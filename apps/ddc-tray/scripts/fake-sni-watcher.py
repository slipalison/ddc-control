#!/usr/bin/env python3
"""Stand-in org.kde.StatusNotifierWatcher for the private bus of smoke-sni-private.sh.

It owns the watcher's well-known name on the session bus it is started on and
implements only what the tray app and smoke-sni.sh use: the two Register*
methods, the three properties and the StatusNotifierItemRegistered signal. It
draws nothing; it only keeps the list of registered items, so the smoke can
check that the app registered its own item.

Needs python3-gobject (Gio). Exits 1 if another watcher owns the name.
"""
import sys
import warnings

import gi

gi.require_version("Gio", "2.0")
from gi.repository import Gio, GLib  # noqa: E402

# register_object is deprecated for register_object_with_closures2, which GLib
# older than 2.84 lacks; the old call works on both.
warnings.filterwarnings(
    "ignore", message="Gio.DBusConnection.register_object is deprecated", category=DeprecationWarning
)

WATCHER = "org.kde.StatusNotifierWatcher"
WATCHER_PATH = "/StatusNotifierWatcher"
PROTOCOL_VERSION = 0

INTERFACE_XML = """
<node>
  <interface name="org.kde.StatusNotifierWatcher">
    <method name="RegisterStatusNotifierItem">
      <arg type="s" direction="in" name="service"/>
    </method>
    <method name="RegisterStatusNotifierHost">
      <arg type="s" direction="in" name="service"/>
    </method>
    <property name="RegisteredStatusNotifierItems" type="as" access="read"/>
    <property name="IsStatusNotifierHostRegistered" type="b" access="read"/>
    <property name="ProtocolVersion" type="i" access="read"/>
    <signal name="StatusNotifierItemRegistered">
      <arg type="s"/>
    </signal>
  </interface>
</node>
"""

items = []


def list_entry(sender, service):
    """The entry KDE's watcher lists: bus name + path for a path, else the name."""
    return f"{sender}{service}" if service.startswith("/") else service


def on_method_call(connection, sender, path, interface, method, params, invocation):
    if method == "RegisterStatusNotifierItem":
        (service,) = params.unpack()
        entry = list_entry(sender, service)
        items.append(entry)
        connection.emit_signal(
            None, path, interface, "StatusNotifierItemRegistered", GLib.Variant("(s)", (entry,))
        )
    invocation.return_value(None)


def on_get_property(connection, sender, path, interface, name):
    return {
        "RegisteredStatusNotifierItems": lambda: GLib.Variant("as", items),
        "IsStatusNotifierHostRegistered": lambda: GLib.Variant("b", True),
        "ProtocolVersion": lambda: GLib.Variant("i", PROTOCOL_VERSION),
    }[name]()


def on_bus_acquired(connection, name):
    node = Gio.DBusNodeInfo.new_for_xml(INTERFACE_XML)
    connection.register_object(
        WATCHER_PATH, node.interfaces[0], on_method_call, on_get_property, None
    )


def on_name_lost(connection, name):
    global exit_status
    print(f"fake-sni-watcher: could not own {WATCHER} on this bus", file=sys.stderr)
    exit_status = 1
    loop.quit()


exit_status = 0
loop = GLib.MainLoop()
Gio.bus_own_name(
    Gio.BusType.SESSION,
    WATCHER,
    Gio.BusNameOwnerFlags.NONE,
    on_bus_acquired,
    None,
    on_name_lost,
)
loop.run()
sys.exit(exit_status)
