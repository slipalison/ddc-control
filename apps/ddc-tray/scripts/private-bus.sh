# Shared by smoke-sni-private.sh and smoke-autostart-private.sh: a PRIVATE
# session bus with a stand-in StatusNotifierWatcher (fake-sni-watcher.py) on
# it, so a smoke runs whatever the user's own session has running
# (D-2026-09-30-input-switch-autostart-13, -15). Sourced, never run.
#
# Before sourcing, the caller sets PRIVATE_BUS_TAG (its name: the prefix of
# every message) and `here` (the directory of the scripts). It may define
# on_fail, which fail() runs before it exits.
#
# A wrapper runs in two steps. Outside, it checks its arguments, then
# run_on_private_bus re-executes it under dbus-run-session with INSIDE as the
# first argument; the caller's DBUS_SESSION_BUS_ADDRESS never leaks in.
# Inside, require_private_bus refuses a bus that is not the private one, and
# start_watcher brings the stand-in up and waits for it to answer. Needs
# dbus-run-session, busctl (systemd), python3 and python3-gobject; never
# needs root.

readonly WATCHER_READY_TIMEOUT_S=5
readonly WATCHER_NAME=org.kde.StatusNotifierWatcher
readonly INSIDE=--inside-private-bus

bus_conf=""
watcher_pid=""

fail() {
    echo "$PRIVATE_BUS_TAG: FAIL: $*" >&2
    if declare -F on_fail >/dev/null; then
        on_fail
    fi
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

# Outside: checks the tools, then runs the script $1 again, with the
# arguments $2..., inside a fresh session bus. Its exit status is the
# script's.
run_on_private_bus() {
    local script=$1
    shift
    require_tools

    bus_conf=$(mktemp -t "$PRIVATE_BUS_TAG-bus.XXXXXX")
    trap 'rm -f "$bus_conf"' EXIT
    write_bus_conf

    local status=0
    PRIVATE_BUS_OUTER_ADDRESS=${DBUS_SESSION_BUS_ADDRESS:-} \
        dbus-run-session --config-file="$bus_conf" -- \
        bash "$script" "$INSIDE" "$@" || status=$?
    exit "$status"
}

# Inside: the session bus is the private one. The guard stops a manual run
# of the inside step from reaching the user's bus: run_on_private_bus always
# sets PRIVATE_BUS_OUTER_ADDRESS, and the bus here must differ from it.
require_private_bus() {
    [[ -n ${PRIVATE_BUS_OUTER_ADDRESS+set} && -n ${DBUS_SESSION_BUS_ADDRESS:-} &&
        $DBUS_SESSION_BUS_ADDRESS != "$PRIVATE_BUS_OUTER_ADDRESS" ]] ||
        fail "not on a private session bus; run $PRIVATE_BUS_TAG.sh without $INSIDE"
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

# Starts the stand-in watcher and waits for it to answer. The caller's EXIT
# trap runs stop_watcher.
start_watcher() {
    python3 "$here/fake-sni-watcher.py" &
    watcher_pid=$!
    wait_for_watcher
}
