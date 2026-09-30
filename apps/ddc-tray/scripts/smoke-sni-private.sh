#!/usr/bin/env bash
# Runs smoke-sni.sh --fake --activate on a PRIVATE session bus, with a
# stand-in StatusNotifierWatcher (fake-sni-watcher.py) as the host.
#
# Usage: smoke-sni-private.sh <path-to-ddc-tray>
#
# smoke-sni.sh on the user's own session needs the installed ddc-tray to be
# stopped: the single-instance check makes the binary under test exit before
# it registers (D-2026-09-30-input-switch-autostart-13). On a private bus
# there is no other instance, so the smoke runs with the user's app running.
# The assertions are smoke-sni.sh's, untouched: only the StatusNotifierItem
# host changes. --fake serves the simulated monitor; no real one is touched.
#
# The script re-executes itself under dbus-run-session, so the caller's
# DBUS_SESSION_BUS_ADDRESS never leaks in. It starts the stand-in watcher,
# waits up to WATCHER_READY_TIMEOUT_S for it to answer, prints BANNER, runs
# the smoke, stops the watcher on the way out and exits with the smoke's
# status. Needs dbus-run-session, busctl (systemd), python3 and
# python3-gobject; never needs root.
set -euo pipefail

readonly WATCHER_READY_TIMEOUT_S=5
readonly WATCHER_NAME=org.kde.StatusNotifierWatcher
readonly BANNER='smoke-sni-private: private session bus, stand-in StatusNotifierWatcher'
readonly INSIDE=--inside-private-bus
readonly USAGE="usage: smoke-sni-private.sh <path-to-ddc-tray>"

here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
readonly here
bus_conf=""
watcher_pid=""

fail() {
    echo "smoke-sni-private: FAIL: $*" >&2
    exit 1
}

# Milliseconds on the wall clock; bash's SECONDS only counts whole seconds.
now_ms() {
    echo $(($(date +%s%N) / 1000000))
}

require_tools() {
    command -v dbus-run-session >/dev/null ||
        fail "dbus-run-session not found (it ships with the D-Bus daemon package)"
    command -v busctl >/dev/null || fail "busctl not found (systemd)"
    command -v python3 >/dev/null || fail "python3 not found"
    python3 -c 'import gi; gi.require_version("Gio", "2.0"); from gi.repository import Gio, GLib' \
        2>/dev/null ||
        fail "python3 cannot load the gi module with Gio 2.0 (package python3-gobject)"
}

# The stock session.conf minus <standard_session_servicedirs />: nothing on
# the private bus can be D-Bus-activated, so GTK's lookups of the portals,
# kwallet or the accessibility bus cannot start (and leave behind) services
# of the user's desktop, nor have the document portal touch /run/user/$UID/doc.
write_bus_conf() {
    cat >"$bus_conf" <<'EOF'
<!DOCTYPE busconfig PUBLIC "-//freedesktop//DTD D-Bus Bus Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/dbus/1.0/busconfig.dtd">
<busconfig>
  <type>session</type>
  <keep_umask/>
  <listen>unix:tmpdir=/tmp</listen>
  <auth>EXTERNAL</auth>
  <policy context="default">
    <allow send_destination="*" eavesdrop="true"/>
    <allow eavesdrop="true"/>
    <allow own="*"/>
  </policy>
</busconfig>
EOF
}

# Outside: checks the arguments and the tools, then runs this script again,
# inside a fresh session bus. Its exit status is the smoke's.
outside() {
    [[ $# -eq 1 ]] || fail "$USAGE"
    [[ -x $1 ]] || fail "not an executable file: $1"
    require_tools

    bus_conf=$(mktemp -t smoke-sni-private-bus.XXXXXX)
    trap 'rm -f "$bus_conf"' EXIT
    write_bus_conf

    local status=0
    SMOKE_SNI_OUTER_BUS=${DBUS_SESSION_BUS_ADDRESS:-} \
        dbus-run-session --config-file="$bus_conf" -- \
        bash "${BASH_SOURCE[0]}" "$INSIDE" "$1" || status=$?
    exit "$status"
}

stop_watcher() {
    [[ -n $watcher_pid ]] || return 0
    kill "$watcher_pid" 2>/dev/null || true
    wait "$watcher_pid" 2>/dev/null || true
    watcher_pid=""
}

# Waits until the stand-in answers on the private bus; fails if it dies or
# stays silent for WATCHER_READY_TIMEOUT_S.
wait_for_watcher() {
    local deadline=$(($(now_ms) + WATCHER_READY_TIMEOUT_S * 1000))
    until busctl --user get-property "$WATCHER_NAME" /StatusNotifierWatcher \
        "$WATCHER_NAME" ProtocolVersion >/dev/null 2>&1; do
        kill -0 "$watcher_pid" 2>/dev/null ||
            fail "the stand-in watcher exited before answering (its error is above)"
        (($(now_ms) < deadline)) ||
            fail "the stand-in watcher did not answer on the private bus within ${WATCHER_READY_TIMEOUT_S} s"
        sleep 0.1
    done
}

# Inside: the session bus is the private one. The guard stops a manual run
# of this step from reaching the user's bus: outside() always sets
# SMOKE_SNI_OUTER_BUS, and the bus here must differ from it.
inside() {
    local bin=$1
    [[ -n ${SMOKE_SNI_OUTER_BUS+set} && -n ${DBUS_SESSION_BUS_ADDRESS:-} &&
        $DBUS_SESSION_BUS_ADDRESS != "$SMOKE_SNI_OUTER_BUS" ]] ||
        fail "not on a private session bus; run smoke-sni-private.sh without $INSIDE"

    trap stop_watcher EXIT
    trap 'exit 1' INT TERM HUP
    python3 "$here/fake-sni-watcher.py" &
    watcher_pid=$!
    wait_for_watcher

    echo "$BANNER"
    local status=0
    bash "$here/smoke-sni.sh" --fake --activate "$bin" || status=$?
    exit "$status"
}

main() {
    if [[ ${1:-} == "$INSIDE" ]]; then
        [[ $# -eq 2 ]] || fail "$USAGE"
        inside "$2"
    fi
    outside "$@"
}

main "$@"
