#!/usr/bin/env bash
# End-to-end smoke of the USB switch follow on the RELEASE binary, on a
# PRIVATE session bus, with a sandboxed HOME and a fake USB tree
# (D-2026-10-02-usb-switch-follow-9, -11).
#
# Usage: smoke-follow-private.sh <path-to-ddc-tray>
#
# The unit tests prove the follow's loop and its menu items one by one; this
# proves that `run()` starts the loop and that the menu sets the follow up,
# end to end. It reads and clicks the menu of the tray item over
# com.canonical.dbusmenu (sni-dbusmenu.py, as a desktop's tray host does),
# edits a fake sysfs tree of USB devices (a keyboard with a serial, a mouse
# without one, and the switch: a hub), reads the settings file with python3,
# and passes only when, in this order:
#   - "Follow USB switch" is an UNMARKED checkmark;
#   - after a click on "Learn USB switch" the app says it started learning,
#     the keyboard and the mouse are removed from the tree, and the settings
#     file holds exactly their two ids — not the hub's —, the simulated
#     monitor, no target input, and enabled false;
#   - the devices come back, and a click on "DisplayPort 2" records input 16
#     (0x10), still with enabled false;
#   - a click on "Follow USB switch" marks it and records enabled true;
#   - removing the devices makes the app print the switch line
#     ("ddc-tray: follow: switching <id> to input 0x10") exactly once;
#   - giving them back switches nothing;
#   - with the follow turned off again, removing them switches nothing;
# and the app's stderr never says "panicked" nor that the follow could not
# do something, and the listings of the REAL ~/.config/autostart and
# ~/.config/ddc-control are the same before and after.
#
# Each removal and each return waits for the follow's own diagnostic edge
# ("learned devices absent" / "present"): the loop's proof of life. A step
# that must switch nothing passes only after SILENCE_S more seconds — longer
# than the follow's debounce (3 reads of 500 ms) plus a read — with the
# switch line still printed once.
#
# The app runs with HOME and XDG_CONFIG_HOME in a temporary directory,
# APPIMAGE and the language variables unset (the menu is in English),
# DDC_TRAY_FAKE=1 — the simulated monitor, so no real one is touched; the
# script requires the app's own line saying so before any click —,
# DDC_TRAY_DEBUG=1 for the diagnostic lines, and DDC_TRAY_USB_ROOT on the
# fake tree, which the app reads only in simulation. The private bus, the
# stand-in watcher and the re-execution under dbus-run-session are
# private-bus.sh's. The app and the watcher are always stopped on the way
# out, and the temporary directory removed. The script fails on any violated
# condition, including a missing python3-gobject and a binary that exits
# early.
set -euo pipefail

readonly PRIVATE_BUS_TAG=smoke-follow-private
readonly BANNER='smoke-follow-private: private session bus, stand-in StatusNotifierWatcher, sandboxed HOME, fake USB root'
readonly USAGE="usage: smoke-follow-private.sh <path-to-ddc-tray>"
readonly FOLLOW='Follow USB switch'
readonly LEARN='Learn USB switch'
readonly TARGET='DisplayPort 2'
readonly REGISTER_TIMEOUT_S=15
# Learning ends after the debounce (3 reads of 500 ms) plus a read; a click
# runs off the menu's thread. Ample either way.
readonly STEP_TIMEOUT_S=10
readonly STOP_TIMEOUT_S=5
# Longer than the debounce plus one read: a switch on its way has come.
readonly SILENCE_S=3
readonly SIMULATED_LINE='ddc-tray: DDC_TRAY_FAKE=1, serving the simulated RTK monitor; no real monitor is touched'
readonly MONITOR_ID='RTK-RTK-QHD-HDR-01010101'
readonly SWITCH_LINE="ddc-tray: follow: switching $MONITOR_ID to input 0x10"
readonly LEARNING_LINE='ddc-tray: follow: learning started'
readonly PRESENT_LINE='ddc-tray: follow: learned devices present'
readonly ABSENT_LINE='ddc-tray: follow: learned devices absent'
readonly KEYBOARD_ID='046d:c31c:SMOKEKB1'
readonly MOUSE_ID='046d:c077'
# What the app prints when the follow could not do something: every one of
# its reports names the USB follow or a switch.
readonly FOLLOW_FAILURE='^ddc-tray: could not .*(USB|switch)'

here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
readonly here
# shellcheck source=private-bus.sh
source "$here/private-bus.sh"

app_pid=""
stderr_log=""
sandbox=""
service=""
item_path=""

on_fail() {
    if [[ -n $stderr_log && -s $stderr_log ]]; then
        echo "$PRIVATE_BUS_TAG: last lines of the app's stderr:" >&2
        tail -n 20 "$stderr_log" >&2
    fi
    if [[ -n $sandbox && -f $(config_file) ]]; then
        echo "$PRIVATE_BUS_TAG: the settings file:" >&2
        cat "$(config_file)" >&2
    fi
}

say() {
    echo "smoke-follow: $*"
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
    stop_watcher
    [[ -z $stderr_log ]] || rm -f "$stderr_log"
    [[ -z $sandbox ]] || rm -rf "$sandbox"
}

# Fails with the app's exit status: $1 says when it exited.
exited_early() {
    local status=0
    wait "$app_pid" || status=$?
    app_pid=""
    fail "the app exited $1 with status $status"
}

# The entries of a REAL directory under ~/.config, one per line, sorted. The
# script's own HOME is the real one: only the app's is sandboxed.
real_listing() {
    ls -A "$HOME/.config/$1" 2>/dev/null | sort || true
}

config_file() {
    echo "$sandbox/.config/ddc-control/usb-follow.json"
}

# --- the fake USB tree -------------------------------------------------

# Writes a device directory as the kernel lays it out: $1 the entry, then
# attribute=value pairs.
write_device() {
    local dir="$sandbox/usb/$1"
    shift
    mkdir -p "$dir"
    local pair
    for pair in "$@"; do
        printf '%s\n' "${pair#*=}" >"$dir/${pair%%=*}"
    done
}

# The switch: a hub, which stays whatever the button does.
plug_switch() {
    write_device 1-1 idVendor=05e3 idProduct=0610 bDeviceClass=09 serial=HUB0001
    write_device 1-1:1.0 bInterfaceClass=09
}

plug_devices() {
    write_device 1-1.1 idVendor=046d idProduct=c31c bDeviceClass=00 serial=SMOKEKB1
    write_device 1-1.1:1.0 bInterfaceClass=03
    write_device 1-1.2 idVendor=046d idProduct=c077 bDeviceClass=00
    write_device 1-1.2:1.0 bInterfaceClass=03
}

unplug_devices() {
    rm -rf "$sandbox/usb/1-1.1" "$sandbox/usb/1-1.1:1.0" \
        "$sandbox/usb/1-1.2" "$sandbox/usb/1-1.2:1.0"
}

# --- what the app shows --------------------------------------------------

# How many times the app printed exactly the line $1.
count() {
    grep -cxF -- "$1" "$stderr_log" || true
}

# Whether the app printed the line $1 more than $2 times.
more_than() {
    (($(count "$1") > $2))
}

# The menu entry labelled $1 as the menu shows it now, "<type> <state>".
menu_state() {
    python3 "$here/sni-dbusmenu.py" state "$service" "$item_path" "$1"
}

menu_is() {
    [[ $(menu_state "$1") == "checkmark $2" ]]
}

click_item() {
    python3 "$here/sni-dbusmenu.py" click "$service" "$item_path" "$1" ||
        fail "could not click '$1' in the menu of $service$item_path"
}

# Whether the settings file holds exactly the two learned ids, the simulated
# monitor, enabled $1 (true or false) and the target input $2 (a decimal
# code, or none).
config_is() {
    python3 - "$(config_file)" "$1" "$2" "$KEYBOARD_ID" "$MOUSE_ID" "$MONITOR_ID" <<'EOF'
import json
import sys

path, enabled, target, keyboard, mouse, monitor = sys.argv[1:]
try:
    with open(path, encoding="utf-8") as file:
        config = json.load(file)
except (OSError, ValueError):
    sys.exit(1)
expected_target = None if target == "none" else int(target)
holds = (
    config.get("version") == 1
    and sorted(config.get("devices", [])) == sorted([keyboard, mouse])
    and config.get("monitor_id") == monitor
    and config.get("enabled") is (enabled == "true")
    and config.get("target_input") == expected_target
)
sys.exit(0 if holds else 1)
EOF
}

# Runs "$@" until it succeeds, for up to $1 s; fails with "$2" if the app
# exits or the time runs out.
wait_until() {
    local timeout_s=$1 what=$2
    shift 2
    local deadline=$(($(now_ms) + timeout_s * 1000))
    until "$@" >/dev/null 2>&1; do
        kill -0 "$app_pid" 2>/dev/null || exited_early "while waiting for $what"
        (($(now_ms) < deadline)) || fail "$what did not happen within ${timeout_s} s"
        sleep 0.1
    done
}

# Waits SILENCE_S, then fails unless the app is alive and printed the
# switch line exactly $1 times.
switches_after_silence() {
    sleep "$SILENCE_S"
    kill -0 "$app_pid" 2>/dev/null || exited_early "during the silence window"
    local switches
    switches=$(count "$SWITCH_LINE")
    ((switches == $1)) ||
        fail "the app printed '$SWITCH_LINE' $switches times, not $1, after $2"
}

# --- the app ---------------------------------------------------------------

# A temporary HOME with its .config, and the fake tree with the switch, the
# keyboard and the mouse plugged in.
make_sandbox() {
    sandbox=$(mktemp -d -t smoke-follow-home.XXXXXX)
    [[ $sandbox != "$HOME" && -d $sandbox ]] || fail "could not make a temporary HOME"
    mkdir -p "$sandbox/.config" "$sandbox/usb"
    plug_switch
    plug_devices
}

# Starts the app in the sandbox: nothing of the user's configuration, the
# language English, the simulated monitor, the fake USB tree.
start_app() {
    local bin=$1
    stderr_log=$(mktemp -t ddc-tray-smoke-follow.XXXXXX)
    env -u APPIMAGE -u LANGUAGE -u LC_MESSAGES \
        HOME="$sandbox" XDG_CONFIG_HOME="$sandbox/.config" \
        LC_ALL=en_US.UTF-8 LANG=en_US.UTF-8 \
        XDG_CACHE_HOME="$sandbox/.cache" XDG_DATA_HOME="$sandbox/.local/share" \
        DDC_TRAY_DEBUG=1 DDC_TRAY_FAKE=1 DDC_TRAY_USB_ROOT="$sandbox/usb" \
        "$bin" >/dev/null 2>"$stderr_log" &
    app_pid=$!
    say "started $bin as PID $app_pid with DDC_TRAY_FAKE=1, HOME=$sandbox and DDC_TRAY_USB_ROOT=$sandbox/usb"
}

# Waits up to REGISTER_TIMEOUT_S for the item the app registered, and sets
# service and item_path.
find_item() {
    local found="" deadline=$(($(now_ms) + REGISTER_TIMEOUT_S * 1000))
    until found=$(python3 "$here/sni-dbusmenu.py" item "$app_pid" 2>/dev/null); do
        kill -0 "$app_pid" 2>/dev/null ||
            exited_early "before registering a tray item"
        (($(now_ms) < deadline)) ||
            fail "no StatusNotifierItem owned by PID $app_pid within ${REGISTER_TIMEOUT_S} s"
        sleep 0.25
    done
    read -r service item_path <<<"$found"
    say "$WATCHER_NAME lists $service$item_path, owned by PID $app_pid"
}

# Fails unless the app said it serves the simulated monitor. Run before the
# first click, so a run without DDC_TRAY_FAKE=1 stops before it does anything.
require_simulated_monitor() {
    grep -Fxq -- "$SIMULATED_LINE" "$stderr_log" ||
        fail "the app did not say it serves the simulated monitor ('$SIMULATED_LINE')"
    say "the app serves the simulated monitor"
}

check_stderr() {
    if grep -Fq -- 'panicked' "$stderr_log"; then
        fail "the app's stderr says 'panicked'"
    fi
    local failure
    if failure=$(grep -E -m1 -- "$FOLLOW_FAILURE" "$stderr_log"); then
        fail "the app's stderr says '$failure'"
    fi
}

# --- the steps ---------------------------------------------------------------

check_the_start() {
    menu_is "$FOLLOW" 0 ||
        fail "'$FOLLOW' is not an unmarked checkmark at the start: '$(menu_state "$FOLLOW" || echo absent)'"
    [[ ! -e $(config_file) ]] || fail "the sandboxed settings file already exists"
    say "'$FOLLOW' is an unmarked checkmark"
}

learn_the_switch() {
    local started
    started=$(count "$LEARNING_LINE")
    click_item "$LEARN"
    wait_until "$STEP_TIMEOUT_S" "'$LEARNING_LINE' after the click on '$LEARN'" \
        more_than "$LEARNING_LINE" "$started"
    unplug_devices
    wait_until "$STEP_TIMEOUT_S" "a settings file with the keyboard and the mouse learned" \
        config_is false none
    say "learning saw exactly the keyboard and the mouse leave, not the hub"
}

give_the_devices_back() {
    local present
    present=$(count "$PRESENT_LINE")
    plug_devices
    wait_until "$STEP_TIMEOUT_S" "'$PRESENT_LINE' after the devices came back" \
        more_than "$PRESENT_LINE" "$present"
}

remove_the_devices() {
    local absent
    absent=$(count "$ABSENT_LINE")
    unplug_devices
    wait_until "$STEP_TIMEOUT_S" "'$ABSENT_LINE' after the devices left" \
        more_than "$ABSENT_LINE" "$absent"
}

pick_the_target() {
    click_item "$TARGET"
    wait_until "$STEP_TIMEOUT_S" "input 16 in the settings file after the click on '$TARGET'" \
        config_is false 16
    wait_until "$STEP_TIMEOUT_S" "'$TARGET' marked" menu_is "$TARGET" 1
    say "the config holds the 2 learned devices, the simulated monitor and input 0x10, with enabled false"
}

turn_the_follow_on() {
    click_item "$FOLLOW"
    wait_until "$STEP_TIMEOUT_S" "'$FOLLOW' marked after the click" menu_is "$FOLLOW" 1
    wait_until "$STEP_TIMEOUT_S" "enabled true in the settings file" config_is true 16
    say "after the click '$FOLLOW' is marked and the config says enabled"
}

check_leaving_switches_once() {
    (($(count "$SWITCH_LINE") == 0)) || fail "the app switched before the devices left"
    remove_the_devices
    wait_until "$STEP_TIMEOUT_S" "'$SWITCH_LINE'" more_than "$SWITCH_LINE" 0
    switches_after_silence 1 "the devices left"
    say "leaving switched the simulated monitor to input 0x10 exactly once"
}

check_arriving_switches_nothing() {
    give_the_devices_back
    switches_after_silence 1 "the devices came back"
    say "arriving switched nothing"
}

check_leaving_with_the_follow_off() {
    click_item "$FOLLOW"
    wait_until "$STEP_TIMEOUT_S" "'$FOLLOW' unmarked after the click" menu_is "$FOLLOW" 0
    wait_until "$STEP_TIMEOUT_S" "enabled false in the settings file" config_is false 16
    remove_the_devices
    switches_after_silence 1 "the devices left with the follow off"
    say "with follow off, leaving switched nothing"
}

# Outside: checks the arguments, then runs this script again inside a fresh
# session bus.
outside() {
    [[ $# -eq 1 ]] || fail "$USAGE"
    [[ -x $1 ]] || fail "not an executable file: $1"
    run_on_private_bus "${BASH_SOURCE[0]}" "$1"
}

inside() {
    local bin=$1
    require_private_bus
    local autostart_before config_before
    autostart_before=$(real_listing autostart)
    config_before=$(real_listing ddc-control)

    trap cleanup EXIT
    trap 'exit 1' INT TERM HUP
    start_watcher

    echo "$BANNER"
    make_sandbox
    start_app "$bin"
    find_item
    require_simulated_monitor
    check_the_start
    learn_the_switch
    give_the_devices_back
    pick_the_target
    turn_the_follow_on
    check_leaving_switches_once
    check_arriving_switches_nothing
    check_leaving_with_the_follow_off

    kill -0 "$app_pid" 2>/dev/null || exited_early "before the end of the smoke"
    stop_app
    check_stderr
    [[ $(real_listing autostart) == "$autostart_before" ]] ||
        fail "the listing of the real $HOME/.config/autostart changed"
    [[ $(real_listing ddc-control) == "$config_before" ]] ||
        fail "the listing of the real $HOME/.config/ddc-control changed"
    echo "smoke-follow: OK — the tray followed the fake USB switch end to end"
}

main() {
    if [[ ${1:-} == "$INSIDE" ]]; then
        [[ $# -eq 2 ]] || fail "$USAGE"
        inside "$2"
        return
    fi
    outside "$@"
}

main "$@"
