#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Check if the user provided a directory path.
if [ $# -ne 1 ]; then
    echo "Usage: $0 path/to/game_dir"
    exit 1
fi

game_dir="${1%/}"

if [ ! -d "$game_dir" ]; then
    echo >&2 "Error: Directory '$game_dir' does not exist."
    exit 1
fi
game_dir="$(cd "$game_dir" && pwd)"

# Locate pixi.js
pixi=""
if [ -f "$game_dir/www/js/libs/pixi.js" ]; then
    pixi="$game_dir/www/js/libs/pixi.js"
elif [ -f "$game_dir/js/libs/pixi.js" ]; then
    pixi="$game_dir/js/libs/pixi.js"
else
    echo >&2 "Error: Input path is not a valid RPG Maker MV/MZ game (pixi.js not found)."
    exit 1
fi

# Function to copy files from the "storage" folder
copy_files() {
    local variant="$1"
    local dest_js="$(dirname "$(dirname "$pixi")")"
    echo "Game engine detected: $variant"

    # Backup original js files if backup does not already exist
    local backup_dir="${dest_js}_backup"
    if [ ! -d "$backup_dir" ]; then
        echo "Creating backup at $backup_dir..."
        cp -r "$dest_js" "$backup_dir"
    fi

    echo "Updating engine libraries with $variant optimized files..."
    cp -r "$SCRIPT_DIR/storage/$variant"/* "$dest_js/"
}

# Check if the game is MV or MZ
if grep -q "pixi.js - v4." "$pixi"; then
    copy_files "MV"
elif grep -q "pixi.js - v5." "$pixi"; then
    copy_files "MZ"
else
    echo >&2 "Error: Could not identify pixi.js major version. Aborting."
    exit 1
fi

echo "Library update finished!"
