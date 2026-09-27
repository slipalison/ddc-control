#!/usr/bin/env bash
# Smoke test of the tray icon on a Linux desktop with a StatusNotifierItem
# host (KDE Plasma, or GNOME with the AppIndicator extension).
#
# Usage: smoke-sni.sh [--fake] [--activate] [--scroll] <path-to-ddc-tray>
#
# Starts the app (with DDC_TRAY_DEBUG=1, so it says on stderr what the popup
# and the tray do) and passes only when, within 15 s, the
# org.kde.StatusNotifierWatcher lists a new item whose D-Bus connection
# belongs to the app's own process, the app is still alive 2 s later, and
# its stderr never says "panicked".
#
# --fake starts it with DDC_TRAY_FAKE=1: the app serves its simulated RTK
# monitor (brightness 75 of 100) and touches no real one
# (D-2026-09-27-tray-app-5); the app must say so on stderr.
#
# --activate then calls org.kde.StatusNotifierItem.Activate on the item —
# what the host does on a left click — and passes only when the app answers
# the call, prints "ddc-tray: popup shown" within 5 s of it, prints no
# "ddc-tray: popup hidden" in the 1.5 s that follow (the popup stays open),
# and is still alive. The popup only reads.
#
# --scroll (only with --fake: it writes) then calls
# org.kde.StatusNotifierItem.Scroll on the item, as the host does for the
# wheel: one notch up (vertical, +120) must print
# "ddc-tray: brightness 75 -> 80" within 5 s; one horizontal notch must
# print no brightness line within 1.5 s; one notch down (vertical, -120)
# must then print "ddc-tray: brightness 80 -> 75".
#
# The app is always stopped on the way out. Needs busctl (systemd) and a
# session bus; never needs root.
set -euo pipefail

readonly REGISTER_TIMEOUT_S=15
readonly ALIVE_AFTER_S=2
readonly SHOWN_TIMEOUT_S=5
readonly STAYS_SHOWN_S=1.5
readonly WRITE_TIMEOUT_S=5
readonly NO_WRITE_S=1.5
readonly STOP_TIMEOUT_S=5
readonly WHEEL_NOTCH=120
readonly WATCHER=org.kde.StatusNotifierWatcher
readonly ITEM_INTERFACE=org.kde.StatusNotifierItem
readonly DEFAULT_ITEM_PATH=/StatusNotifierItem
readonly SHOWN_LINE='ddc-tray: popup shown'
readonly HIDDEN_LINE='ddc-tray: popup hidden'
readonly BRIGHTNESS_PREFIX='ddc-tray: brightness '
readonly SIMULATED_LINE='ddc-tray: DDC_TRAY_FAKE=1, serving the simulated RTK monitor; no real monitor is touched'

app_pid=""
stderr_log=""
before=""
# How many lines the app's stderr had before the last call on its item.
mark=0

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

# The D-Bus service and object path of an item of the watcher's list.
item_service() {
    echo "${1%%/*}"
}

item_path() {
    if [[ $1 == */* ]]; then
        echo "/${1#*/}"
    else
        echo "$DEFAULT_ITEM_PATH"
    fi
}

# The app's stderr lines after the first $1.
lines_after() {
    tail -n "+$(($1 + 1))" "$stderr_log"
}

# Whether the exact line $2 is on stderr after the first $1 lines. grep
# reads to the end (no -q), so tail never dies of SIGPIPE under pipefail.
printed_after() {
    lines_after "$1" | grep -Fx -- "$2" >/dev/null
}

# Waits up to $2 s for the exact line $1 on stderr after the first $3 lines;
# $4 names the call it follows.
wait_for_line() {
    local line=$1 timeout_s=$2 lines=$3 after=$4
    local deadline=$(($(now_ms) + timeout_s * 1000))
    until printed_after "$lines" "$line"; do
        kill -0 "$app_pid" 2>/dev/null || exited_early "after $after"
        (($(now_ms) < deadline)) ||
            fail "no '$line' on stderr within ${timeout_s} s of $after"
        sleep 0.1
    done
    echo "smoke-sni: the app printed '$line' after $after"
}

# Calls Activate on the item, as a left click does, waits for the popup and
# checks it stays open.
activate() {
    local item=$1 service path
    service=$(item_service "$item")
    path=$(item_path "$item")
    mark=$(wc -l <"$stderr_log")
    busctl --user call "$service" "$path" "$ITEM_INTERFACE" Activate ii 0 0 >/dev/null ||
        fail "$service$path did not answer $ITEM_INTERFACE.Activate"
    echo "smoke-sni: called $ITEM_INTERFACE.Activate on $service$path"

    wait_for_line "$SHOWN_LINE" "$SHOWN_TIMEOUT_S" "$mark" Activate
    sleep "$STAYS_SHOWN_S"
    kill -0 "$app_pid" 2>/dev/null || exited_early "after showing the popup"
    if printed_after "$mark" "$HIDDEN_LINE"; then
        fail "the popup was hidden within ${STAYS_SHOWN_S} s of '$SHOWN_LINE' ('$HIDDEN_LINE')"
    fi
    echo "smoke-sni: the popup was still shown ${STAYS_SHOWN_S} s later"
    check_no_panic
}

# Calls Scroll on the item with delta $2 and orientation $3, as the host
# does for the wheel, and sets mark.
scroll_item() {
    local item=$1 delta=$2 orientation=$3 service path
    service=$(item_service "$item")
    path=$(item_path "$item")
    mark=$(wc -l <"$stderr_log")
    # "--": a negative delta is an argument, not an option of busctl.
    busctl --user -- call "$service" "$path" "$ITEM_INTERFACE" Scroll is "$delta" "$orientation" \
        >/dev/null || fail "$service$path did not answer $ITEM_INTERFACE.Scroll"
    echo "smoke-sni: called $ITEM_INTERFACE.Scroll $delta $orientation on $service$path"
}

# The wheel over the simulated monitor: up one notch, sideways, down one.
scroll_wheel() {
    local item=$1 written
    scroll_item "$item" "$WHEEL_NOTCH" Vertical
    wait_for_line "${BRIGHTNESS_PREFIX}75 -> 80" "$WRITE_TIMEOUT_S" "$mark" \
        "a vertical Scroll of +$WHEEL_NOTCH"

    scroll_item "$item" "$WHEEL_NOTCH" Horizontal
    sleep "$NO_WRITE_S"
    kill -0 "$app_pid" 2>/dev/null || exited_early "after a horizontal Scroll"
    written=$(lines_after "$mark" | grep -F -- "$BRIGHTNESS_PREFIX" | head -n 1 || true)
    [[ -z $written ]] || fail "a horizontal Scroll wrote the brightness: '$written'"
    echo "smoke-sni: a horizontal Scroll wrote nothing within ${NO_WRITE_S} s"

    scroll_item "$item" "-$WHEEL_NOTCH" Vertical
    wait_for_line "${BRIGHTNESS_PREFIX}80 -> 75" "$WRITE_TIMEOUT_S" "$mark" \
        "a vertical Scroll of -$WHEEL_NOTCH"
    check_no_panic
}

main() {
    local usage="usage: smoke-sni.sh [--fake] [--activate] [--scroll] <path-to-ddc-tray>"
    local activate_item=0 scroll_the_wheel=0 fake=0
    while [[ ${1:-} == --* ]]; do
        case $1 in
        --activate) activate_item=1 ;;
        --scroll) scroll_the_wheel=1 ;;
        --fake) fake=1 ;;
        *) fail "unknown option $1; $usage" ;;
        esac
        shift
    done
    local bin=${1:-}
    [[ -n $bin && $# -eq 1 ]] || fail "$usage"
    [[ -x $bin ]] || fail "not an executable file: $bin"
    ((scroll_the_wheel == 0 || fake == 1)) ||
        fail "--scroll writes the brightness, so it only runs with --fake (the simulated monitor)"
    command -v busctl >/dev/null || fail "busctl not found (systemd)"

    before=$(registered_items) ||
        fail "no $WATCHER on the session bus: run this in a desktop session with a StatusNotifierItem host (KDE Plasma, or GNOME with the AppIndicator extension)"

    stderr_log=$(mktemp -t ddc-tray-smoke.XXXXXX)
    local mode=""
    if ((fake)); then
        mode=" with DDC_TRAY_FAKE=1"
        DDC_TRAY_DEBUG=1 DDC_TRAY_FAKE=1 "$bin" >/dev/null 2>"$stderr_log" &
    else
        DDC_TRAY_DEBUG=1 "$bin" >/dev/null 2>"$stderr_log" &
    fi
    app_pid=$!
    echo "smoke-sni: started $bin as PID $app_pid$mode"

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

    if ((fake)); then
        grep -Fxq -- "$SIMULATED_LINE" "$stderr_log" ||
            fail "the app did not say it serves the simulated monitor ('$SIMULATED_LINE')"
        echo "smoke-sni: the app serves the simulated monitor"
    fi

    local done_list=""
    if ((activate_item)); then
        activate "$item"
        done_list+=", showed its popup on Activate and kept it shown"
    fi
    if ((scroll_the_wheel)); then
        scroll_wheel "$item"
        done_list+=", stepped the brightness on a vertical Scroll only"
    fi

    local pid=$app_pid
    stop_app
    check_no_panic
    echo "smoke-sni: OK — PID $pid registered its tray item, was alive ${ALIVE_AFTER_S} s later$done_list and never panicked"
}

main "$@"
