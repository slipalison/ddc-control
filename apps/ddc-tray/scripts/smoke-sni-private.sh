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
# The private bus, the stand-in watcher and the re-execution under
# dbus-run-session are private-bus.sh's, shared with
# smoke-autostart-private.sh. This script prints BANNER once the stand-in
# answers, runs the smoke, stops the watcher on the way out and exits with
# the smoke's status.
set -euo pipefail

readonly PRIVATE_BUS_TAG=smoke-sni-private
readonly BANNER='smoke-sni-private: private session bus, stand-in StatusNotifierWatcher'
readonly USAGE="usage: smoke-sni-private.sh <path-to-ddc-tray>"

here=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
readonly here
# shellcheck source=private-bus.sh
source "$here/private-bus.sh"

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

    trap stop_watcher EXIT
    trap 'exit 1' INT TERM HUP
    start_watcher

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
