#!/usr/bin/env bash
set -e

# Resolve game directory relative to this script
GAME_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Verify system-provided 'nw' executable is available
if ! command -v nw >/dev/null 2>&1; then
    echo >&2 "Error: System-provided 'nw' executable not found in PATH."
    exit 1
fi

# Launch the game natively using the system NW.js installation
exec nw "$GAME_DIR" "$@"
