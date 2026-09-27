#!/usr/bin/env bash
# Smoke test of the tray icon on a Linux desktop with a StatusNotifierItem
# host (KDE Plasma, or GNOME with the AppIndicator extension).
#
# Usage: smoke-sni.sh <path-to-ddc-tray>
#
# Starts the app and passes only when, within 15 s, the
# org.kde.StatusNotifierWatcher lists a new item whose D-Bus connection
# belongs to the app's own process, the app is still alive 2 s later, and
# its stderr never says "panicked". The app is always stopped on the way
# out. Needs busctl (systemd) and a session bus; never needs root.
set -euo pipefail

readonly REGISTER_TIMEOUT_S=15
readonly ALIVE_AFTER_S=2
readonly STOP_TIMEOUT_S=5
readonly WATCHER=org.kde.StatusNotifierWatcher

app_pid=""
stderr_log=""
before=""

fail() {
    echo "smoke-sni: FAIL: $*" >&2
    if [[ -n $stderr_log && -s $stderr_log ]]; then
        echo "smoke-sni: last lines of the app's stderr:" >&2
        tail -n 20 "$stderr_log" >&2
    fi
    exit 1
}

# Milliseconds on the wall clock; bash's SECONDS only counts whole seconds.
now_ms() {
    echo $(($(date +%s%N) / 1000000))
}

# SIGTERM, then SIGKILL if the app is still up STOP_TIMEOUT_S later.
stop_app() {
    [[ -n $app_pid ]] || return 0
    kill -TERM "$app_pid" 2>/dev/null || true
    local deadline=$(($(now_ms) + STOP_TIMEOUT_S * 1000))
    while kill -0 "$app_pid" 2>/dev/null && (($(now_ms) < deadline)); do
        sleep 0.1
    done
    kill -KILL "$app_pid" 2>/dev/null || true
    wait "$app_pid" 2>/dev/null || true
    app_pid=""
}

cleanup() {
    stop_app
    if [[ -n $stderr_log ]]; then
        rm -f "$stderr_log"
    fi
}
trap cleanup EXIT

# The watcher's items, one per line: ":1.N/path", or a well-known name,
# with or without a path. Fails when there is no watcher.
registered_items() {
    local reply
    reply=$(busctl --user get-property "$WATCHER" /StatusNotifierWatcher "$WATCHER" \
        RegisteredStatusNotifierItems) || return 1
    grep -o '"[^"]*"' <<<"$reply" | tr -d '"' || true
}

# PID of the process that owns the D-Bus name the item was registered by.
item_pid() {
    local service=${1%%/*}
    busctl --user call org.freedesktop.DBus /org/freedesktop/DBus \
        org.freedesktop.DBus GetConnectionUnixProcessID s "$service" 2>/dev/null |
        awk '{ print $2 }'
}

# The item the app registered, if any: a new one owned by the app's PID.
own_item() {
    local item
    while IFS= read -r item; do
        [[ -z $item ]] && continue
        grep -Fxq -- "$item" <<<"$before" && continue
        if [[ $(item_pid "$item") == "$app_pid" ]]; then
            echo "$item"
            return 0
        fi
    done < <(registered_items 2>/dev/null || true)
    return 1
}

# Fails with the app's exit status: $1 says when it exited, $2 is an
# optional hint.
exited_early() {
    local status=0
    wait "$app_pid" || status=$?
    app_pid=""
    fail "the app exited $1 with status $status${2:+ — $2}"
}

check_no_panic() {
    if grep -q 'panicked' "$stderr_log"; then
        fail "the app's stderr says 'panicked'"
    fi
}

main() {
    local bin=${1:-}
    [[ -n $bin ]] || fail "usage: smoke-sni.sh <path-to-ddc-tray>"
    [[ -x $bin ]] || fail "not an executable file: $bin"
    command -v busctl >/dev/null || fail "busctl not found (systemd)"

    before=$(registered_items) ||
        fail "no $WATCHER on the session bus: run this in a desktop session with a StatusNotifierItem host (KDE Plasma, or GNOME with the AppIndicator extension)"

    stderr_log=$(mktemp -t ddc-tray-smoke.XXXXXX)
    "$bin" >/dev/null 2>"$stderr_log" &
    app_pid=$!
    echo "smoke-sni: started $bin as PID $app_pid"

    local item="" deadline=$(($(now_ms) + REGISTER_TIMEOUT_S * 1000))
    while (($(now_ms) < deadline)); do
        kill -0 "$app_pid" 2>/dev/null ||
            exited_early "before registering a tray item" "is another instance already running?"
        if item=$(own_item); then
            break
        fi
        sleep 0.25
    done
    [[ -n $item ]] ||
        fail "no StatusNotifierItem owned by PID $app_pid within ${REGISTER_TIMEOUT_S} s"
    echo "smoke-sni: $WATCHER lists $item, owned by PID $app_pid"

    sleep "$ALIVE_AFTER_S"
    kill -0 "$app_pid" 2>/dev/null ||
        exited_early "within ${ALIVE_AFTER_S} s of registering its tray item"
    check_no_panic

    local pid=$app_pid
    stop_app
    check_no_panic
    echo "smoke-sni: OK — PID $pid registered its tray item, was alive ${ALIVE_AFTER_S} s later and never panicked"
}

main "$@"
