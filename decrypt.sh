#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Check for required commands
for cmd in jq java; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo >&2 "Error: '$cmd' is required but it is not installed. Aborting."
    exit 1
  fi
done

# Check arguments
if [ $# -ne 1 ]; then
    echo "Usage: $0 path/to/game_dir"
    exit 1
fi

game_dir="${1%/}"

if [ ! -d "$game_dir" ]; then
    echo >&2 "Error: Directory '$game_dir' does not exist."
    exit 1
fi
# Resolve to absolute path (critical for decrypter.jar path-stripping logic)
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

DECRYPTER_JAR="$SCRIPT_DIR/storage/RPG Maker MV Decrypter.jar"
if [ ! -f "$DECRYPTER_JAR" ]; then
    echo >&2 "Error: Decrypter jar not found at '$DECRYPTER_JAR'."
    exit 1
fi

# Decrypt files
echo "Decrypting game assets..."
if ! java -jar "$DECRYPTER_JAR" decrypt "$game_dir" "$game_dir"; then
    echo >&2 "Error: Decryption process failed. Aborting."
    exit 1
fi

# Safe cleanup of decrypter temp files
rm -rf ./output ./updateCache.pref

# Extensions of encrypted files
extensions=(".rpgmvo" ".rpgmvm" ".rpgmvp" ".ogg_" ".m4a_" ".png_")

echo "Cleaning up leftover encrypted files..."
for ext in "${extensions[@]}"; do
  find "$game_dir" -type f -name "*$ext" -delete
done

# Let the game know files are now decrypted
if [ -f "$data_dir/System.json" ]; then
    jq -c '.hasEncryptedAudio = false | .hasEncryptedImages = false' \
        "$data_dir/System.json" > "$data_dir/System.json.tmp" && mv "$data_dir/System.json.tmp" "$data_dir/System.json"
    echo "Updated System.json encryption flags."
fi

echo "Decryption finished successfully!"
