#!/usr/bin/env bash
set -euo pipefail

# Resolve game directory relative to this script
GAME_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

show_error() {
    local msg="$1"
    echo >&2 "Error: $msg"
    if [ ! -t 0 ] && [ -n "${DISPLAY:-${WAYLAND_DISPLAY:-}}" ]; then
        if command -v notify-send >/dev/null 2>&1; then
            notify-send "RPG Maker Game Launcher" "$msg" --icon=dialog-error 2>/dev/null || true
        elif command -v zenity >/dev/null 2>&1; then
            zenity --error --text="$msg" 2>/dev/null || true
        elif command -v kdialog >/dev/null 2>&1; then
            kdialog --error "$msg" 2>/dev/null || true
        fi
    fi
}

# Verify system-provided 'nw' executable is available
if ! command -v nw >/dev/null 2>&1; then
    show_error "System NW.js executable ('nw') was not found in PATH.\nPlease install NW.js or add it to PATH."
    exit 1
fi

# Optional extra flags from environment (e.g. Wayland or GPU workarounds)
EXTRA_FLAGS=()
if [ -n "${RPG_MAKER_NW_ARGS:-}" ]; then
    read -r -a EXTRA_FLAGS <<< "$RPG_MAKER_NW_ARGS"
fi

# Launch the game natively using the system NW.js installation
exec nw ${EXTRA_FLAGS[@]+"${EXTRA_FLAGS[@]}"} "$GAME_DIR" "$@"
