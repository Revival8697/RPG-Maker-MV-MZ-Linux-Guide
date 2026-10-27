#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Check required commands
if ! command -v jq >/dev/null 2>&1; then
    echo >&2 "Error: 'jq' is required but it is not installed. Aborting."
    exit 1
fi

if ! command -v nw >/dev/null 2>&1; then
    echo >&2 "Notice: System 'nw' command was not found in your PATH."
    echo >&2 "Run ./environment-setup.sh to verify system dependencies."
fi

# Check argument
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

# Detect package.json location and ensure root copy exists
if [ -f "$game_dir/package.json" ]; then
    :
elif [ -f "$game_dir/www/package.json" ]; then
    cp "$game_dir/www/package.json" "$game_dir/package.json"
else
    echo >&2 "Error: Input path is not a valid RPG Maker MV/MZ game (no package.json found)."
    exit 1
fi

# Helper function to remove Windows-specific runtime binaries safely
cleanup_windows_binaries() {
    local target_dir="$1"
    rm -rf \
        "$target_dir"/*.exe \
        "$target_dir"/*.dll \
        "$target_dir"/*.pak \
        "$target_dir"/*.bin \
        "$target_dir"/*.dat \
        "$target_dir"/captures \
        "$target_dir"/locales \
        "$target_dir"/swiftshader \
        "$target_dir"/credits.html 2>/dev/null || true
}

# Check if the game is MV or MZ
if [ -d "$game_dir/www" ]; then
    echo "Detected RPG Maker MV layout."
    cleanup_windows_binaries "$game_dir"
else
    echo "Detected RPG Maker MZ layout. Restructuring into www/ layout..."
    mkdir -p "$game_dir/www"

    # Move game data safely into www/, ignoring windows runtime files
    for item in "$game_dir"/*; do
        [ -e "$item" ] || continue
        base_item="$(basename "$item")"
        case "$base_item" in
            www|package.json|Game.sh)
                ;;
            *.exe|*.dll|*.pak|*.bin|*.dat|captures|locales|swiftshader|credits.html)
                rm -rf "$item"
                ;;
            *)
                mv "$item" "$game_dir/www/"
                ;;
        esac
    done

    # Ensure package.json exists at root
    if [ ! -f "$game_dir/package.json" ] && [ -f "$game_dir/www/package.json" ]; then
        cp "$game_dir/www/package.json" "$game_dir/package.json"
    fi
fi

# Detect game title from System.json if available
SYS_TITLE=""
if [ -f "$game_dir/www/data/System.json" ]; then
    SYS_TITLE="$(jq -r '.gameTitle // empty' "$game_dir/www/data/System.json" 2>/dev/null || true)"
elif [ -f "$game_dir/data/System.json" ]; then
    SYS_TITLE="$(jq -r '.gameTitle // empty' "$game_dir/data/System.json" 2>/dev/null || true)"
fi

DEFAULT_TITLE="${SYS_TITLE:-$(basename "$game_dir")}"
DEFAULT_ID="$(echo "$DEFAULT_TITLE" | tr '[:upper:]' '[:lower:]' | tr -cs 'a-z0-9_-' '-' | sed 's/^-//;s/-$//')"
[ -z "$DEFAULT_ID" ] && DEFAULT_ID="game-$(date +%s)"

# Update package.json configuration for native NW.js
echo -e "\nEnter a unique name/identifier for this game (press Enter for '$DEFAULT_ID'):"
read -r INPUT || true

# Sanitize input or fall back to default
GAME_ID="$DEFAULT_ID"
if [ -n "$INPUT" ]; then
    # Convert spaces to hyphens and lowercase for NW.js name compatibility
    GAME_ID="$(echo "$INPUT" | tr '[:upper:]' '[:lower:]' | tr -cs 'a-z0-9_-' '-' | sed 's/^-//;s/-$//')"
fi
[ -z "$GAME_ID" ] && GAME_ID="$DEFAULT_ID"

# Configure package.json properly with main, js-flags, and window parameters
jq \
  --arg name "$GAME_ID" \
  --arg title "$DEFAULT_TITLE" \
  '.name = $name |
   .main = "www/index.html" |
   ."js-flags" = "--expose-gc" |
   .window.title = (if (.window.title != null and .window.title != "") then .window.title elif ($title != "") then $title else $name end) |
   .window.icon = "www/icon/icon.png"' \
  "$game_dir/package.json" > "$game_dir/package.json.tmp" && mv "$game_dir/package.json.tmp" "$game_dir/package.json"

# Deploy native Game.sh launcher script
LAUNCHER_SOURCE="$SCRIPT_DIR/storage/Game.sh"
if [ -f "$LAUNCHER_SOURCE" ]; then
    cp "$LAUNCHER_SOURCE" "$game_dir/Game.sh"
    chmod +x "$game_dir/Game.sh"
    echo "Created native launcher: $game_dir/Game.sh"
fi

echo -e "\nGame setup complete!"
echo "You can now run your game using:"
echo "  1) Directly via NW.js CLI:  $ nw \"$game_dir\""
echo "  2) Via the launch script:   $ \"$game_dir/Game.sh\""
