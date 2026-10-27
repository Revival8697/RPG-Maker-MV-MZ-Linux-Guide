#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Check for required commands
for cmd in jq; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo >&2 "Error: '$cmd' is required but it is not installed. Aborting."
    exit 1
  fi
done

# Check arguments
if [ $# -lt 1 ] || [ $# -gt 2 ]; then
    echo "Usage: $0 path/to/game_dir [hex_key]"
    exit 1
fi

game_dir="${1%/}"
custom_key="${2:-}"

if [ ! -d "$game_dir" ]; then
    echo >&2 "Error: Directory '$game_dir' does not exist."
    exit 1
fi
# Resolve to absolute path
game_dir="$(cd "$game_dir" && pwd)"

# Determine data root
data_dir=""
if [ -d "$game_dir/www/data" ]; then
    data_dir="$game_dir/www/data"
elif [ -d "$game_dir/data" ]; then
    data_dir="$game_dir/data"
else
    echo >&2 "Error: Input path is not a valid RPG Maker game (no data directory found)."
    exit 1
fi

# Locate native Rust decrypter binary (bundled in storage/ or in PATH)
DECRYPTER_BIN=""
if [ -x "$SCRIPT_DIR/storage/rpgmv-decrypter" ]; then
    DECRYPTER_BIN="$SCRIPT_DIR/storage/rpgmv-decrypter"
elif command -v rpgmv-decrypter >/dev/null 2>&1; then
    DECRYPTER_BIN="$(command -v rpgmv-decrypter)"
fi

if [ -z "$DECRYPTER_BIN" ]; then
    echo >&2 "Error: 'rpgmv-decrypter' binary not found at '$SCRIPT_DIR/storage/rpgmv-decrypter' or in PATH."
    echo >&2 "Build it using 'cargo build --release' or place the compiled binary in 'storage/'."
    exit 1
fi

# Decrypt files
echo "Decrypting game assets..."
cmd=("$DECRYPTER_BIN" "decrypt" "$game_dir" "--in-place")
if [ -n "$custom_key" ]; then
    cmd+=("-k" "$custom_key")
fi

if ! "${cmd[@]}"; then
    echo >&2 "Error: Decryption process failed. Aborting."
    exit 1
fi

# Extensions of encrypted files
extensions=(".rpgmvo" ".rpgmvm" ".rpgmvp" ".ogg_" ".m4a_" ".png_")

echo "Cleaning up leftover encrypted files..."
for ext in "${extensions[@]}"; do
  find "$game_dir" \( -type f -o -type l \) -name "*$ext" -delete
done

# Let the game know files are now decrypted
if [ -f "$data_dir/System.json" ]; then
    jq -c '.hasEncryptedAudio = false | .hasEncryptedImages = false' \
        "$data_dir/System.json" > "$data_dir/System.json.tmp" && mv "$data_dir/System.json.tmp" "$data_dir/System.json"
    echo "Updated System.json encryption flags."
fi

echo "Decryption finished successfully!"
