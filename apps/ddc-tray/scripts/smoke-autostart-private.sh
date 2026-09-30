#!/usr/bin/env bash
# End-to-end smoke of "Start with system" on the RELEASE binary, on a PRIVATE
# session bus, with a sandboxed HOME (D-2026-09-30-input-switch-autostart-15).
#
# Usage: smoke-autostart-private.sh <path-to-ddc-tray>
#
# smoke-sni.sh only proves that the tray item registers. This reads the menu
# of the item over com.canonical.dbusmenu (sni-dbusmenu.py, as a desktop's
# tray host does), finds "Start with system" as an UNMARKED checkmark, clicks
# it (Event "clicked") and passes only when, within ENTRY_TIMEOUT_S of each
# step:
#   - after the click, <HOME>/.config/autostart holds exactly one .desktop
#     entry, whose Exec= is the absolute path of the binary (readlink -f), and
#     the item is MARKED;
#   - after a second click, the directory is empty again and the item is
#     UNMARKED;
# and the app's stderr never says "panicked", "could not read the
# start-with-system entry" (what the app prints when `run()` did not register
# the autostart plugin) or "could not change", and the listing of the REAL
# ~/.config/autostart is the same before and after.
#
# The app runs with HOME in a temporary directory (with its .config, which
# the autostart crate does not create), XDG_CONFIG_HOME, APPIMAGE and the
# language variables unset (the menu is in English), and DDC_TRAY_FAKE=1: the
# simulated monitor, so no real one is touched. The private bus, the
# stand-in watcher and the re-execution under dbus-run-session are
# private-bus.sh's, shared with smoke-sni-private.sh. The app and the watcher
# are always stopped on the way out, and the temporary HOME removed. The
# script fails on any violated condition, including a missing python3-gobject
# and a binary that exits early.
set -euo pipefail

readonly PRIVATE_BUS_TAG=smoke-autostart-private
readonly BANNER='smoke-autostart-private: private session bus, stand-in StatusNotifierWatcher, sandboxed HOME'
readonly USAGE="usage: smoke-autostart-private.sh <path-to-ddc-tray>"
readonly LABEL='Start with system'
readonly REGISTER_TIMEOUT_S=15
readonly ENTRY_TIMEOUT_S=5
readonly STOP_TIMEOUT_S=5
readonly FORBIDDEN_STDERR=(
    'panicked'
    'could not read the start-with-system entry'
    'could not change'
)

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
}

say() {
    echo "smoke-autostart: $*"
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

# The entries of the REAL ~/.config/autostart, one per line, sorted. The
# script's own HOME is the real one: only the app's is sandboxed.
real_autostart_listing() {
    ls -A "$HOME/.config/autostart" 2>/dev/null | sort || true
}

entries() {
    ls -A "$sandbox/.config/autostart" 2>/dev/null || true
}

# The item as the menu shows it now, "<toggle-type> <toggle-state>".
menu_state() {
    python3 "$here/sni-dbusmenu.py" state "$service" "$item_path" "$LABEL"
}

click_item() {
    python3 "$here/sni-dbusmenu.py" click "$service" "$item_path" "$LABEL" ||
        fail "could not click '$LABEL' in the menu of $service$item_path"
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

menu_is() {
    [[ $(menu_state) == "checkmark $1" ]]
}

exactly_one_desktop_entry() {
    local listing
    listing=$(entries)
    [[ $listing == *.desktop && $listing != *$'\n'* ]]
}

no_entry() {
    [[ -z $(entries) ]]
}

# Starts the app in the sandbox: HOME in a temporary directory, nothing of the
# user's configuration, the language English, the simulated monitor.
start_app() {
    local bin=$1
    sandbox=$(mktemp -d -t smoke-autostart-home.XXXXXX)
    [[ $sandbox != "$HOME" && -d $sandbox ]] || fail "could not make a temporary HOME"
    mkdir "$sandbox/.config"

    stderr_log=$(mktemp -t ddc-tray-smoke.XXXXXX)
    env -u XDG_CONFIG_HOME -u APPIMAGE -u LANGUAGE -u LC_MESSAGES \
        HOME="$sandbox" LC_ALL=en_US.UTF-8 LANG=en_US.UTF-8 \
        XDG_CACHE_HOME="$sandbox/.cache" XDG_DATA_HOME="$sandbox/.local/share" \
        DDC_TRAY_DEBUG=1 DDC_TRAY_FAKE=1 \
        "$bin" >/dev/null 2>"$stderr_log" &
    app_pid=$!
    say "started $bin as PID $app_pid with DDC_TRAY_FAKE=1 and HOME=$sandbox"
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

# The entry the first click wrote: its only Exec= line is the release binary
# (the autostart crate ends it with a space, for the arguments it has none of).
check_desktop_entry() {
    local bin=$1 file expected execs
    file="$sandbox/.config/autostart/$(entries)"
    expected=$(readlink -f "$bin")
    execs=$(sed -n 's/^Exec=//p' "$file" | sed -E 's/[[:space:]]+$//')
    [[ $execs == "$expected" ]] ||
        fail "$file has the Exec= lines '${execs:-none}', not exactly '$expected'"
    say "$file has Exec=$expected"
}

check_stderr() {
    local forbidden
    for forbidden in "${FORBIDDEN_STDERR[@]}"; do
        if grep -Fq -- "$forbidden" "$stderr_log"; then
            fail "the app's stderr says '$forbidden'"
        fi
    done
}

check_the_toggle() {
    local bin=$1
    menu_is 0 || fail "'$LABEL' is not an unmarked checkmark at the start: '$(menu_state || echo absent)'"
    no_entry || fail "the sandboxed HOME already had an autostart entry: $(entries)"
    say "'$LABEL' is an unmarked checkmark and the autostart directory is empty"

    click_item
    wait_until "$ENTRY_TIMEOUT_S" "exactly one .desktop entry after the first click" exactly_one_desktop_entry
    check_desktop_entry "$bin"
    wait_until "$ENTRY_TIMEOUT_S" "the item marked after the first click" menu_is 1
    say "the first click wrote the entry and the item is marked"

    click_item
    wait_until "$ENTRY_TIMEOUT_S" "the entry removed after the second click" no_entry
    wait_until "$ENTRY_TIMEOUT_S" "the item unmarked after the second click" menu_is 0
    say "the second click removed the entry and the item is unmarked"
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
    local before
    before=$(real_autostart_listing)

    trap cleanup EXIT
    trap 'exit 1' INT TERM HUP
    start_watcher

    echo "$BANNER"
    start_app "$bin"
    find_item
    check_the_toggle "$bin"

    kill -0 "$app_pid" 2>/dev/null || exited_early "before the end of the smoke"
    stop_app
    check_stderr
    [[ $(real_autostart_listing) == "$before" ]] ||
        fail "the listing of the real $HOME/.config/autostart changed"
    echo "smoke-autostart: OK — the Start with system item wrote the desktop entry on click and removed it on the next one"
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
