#!/usr/bin/env bash
set -e

# Check for required commands
for cmd in jq oxipng ffmpeg parallel; do
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
game_dir="$(cd "$game_dir" && pwd)"

# Locate assets root (MV/repackaged: www/, or standard root)
target_root=""
if [ -d "$game_dir/www" ]; then
  target_root="$game_dir/www"
elif [ -d "$game_dir/data" ] && [ -d "$game_dir/audio" ]; then
  target_root="$game_dir"
else
  echo >&2 "Error: Input path is not a valid RPG Maker game."
  exit 1
fi

# Check if files are still encrypted
if [ -f "$target_root/data/System.json" ]; then
  hasEncryptedImages=$(jq -r '.hasEncryptedImages // false' "$target_root/data/System.json")
  hasEncryptedAudio=$(jq -r '.hasEncryptedAudio // false' "$target_root/data/System.json")

  if [[ "$hasEncryptedImages" == "true" ]] || [[ "$hasEncryptedAudio" == "true" ]]; then
      echo >&2 "Files are still encrypted. Run ./decrypt.sh first."
      exit 1
  fi
fi

# Function to optimize image files losslessly without breaking PNG headers
optimize_image() {
  local file="$1"
  # Optimize PNG losslessly while keeping color profiles and safe chunks (--threads 1 prevents oversubscription with parallel)
  oxipng --opt 2 --threads 1 --strip safe "$file" 2>/dev/null || echo >&2 "oxipng skipped on $file."
}

# Function to optimize audio files losslessly while preserving RPG Maker Vorbis loop points
optimize_audio() {
  local file="$1"
  local temp_file="${file%.ogg}.opt.ogg"

  # Losslessly remux: copy audio bitstream without re-encoding, strip album art (-vn), preserve loop tags
  if ffmpeg -nostdin -y -loglevel error -i "$file" -c:a copy -vn -map_metadata 0 "$temp_file" 2>/dev/null; then
    # Only replace if newly remuxed file is smaller
    local old_size new_size
    old_size=$(stat -c%s "$file" 2>/dev/null || stat -f%z "$file")
    new_size=$(stat -c%s "$temp_file" 2>/dev/null || stat -f%z "$temp_file")
    if [ "$new_size" -lt "$old_size" ]; then
      mv "$temp_file" "$file"
    else
      rm -f "$temp_file"
    fi
  else
    rm -f "$temp_file"
  fi
}

export -f optimize_image
export -f optimize_audio

# Create backup if not already present
backup_dir="${target_root}_backup"
if [ ! -d "$backup_dir" ]; then
  echo "Creating backup at $backup_dir..."
  cp -r "$target_root" "$backup_dir"
else
  echo "Backup already exists at $backup_dir. Skipping."
fi

# Optimize icon separately
if [ -f "$target_root/icon/icon.png" ]; then
  oxipng --opt max --strip all "$target_root/icon/icon.png" 2>/dev/null || true
fi

echo "Optimizing PNG images in parallel..."
find "$target_root" -type f -name "*.png" ! -name "icon.png" -print0 | parallel -0 optimize_image

echo "Optimizing OGG audio losslessly in parallel (preserving audio stream & loop metadata)..."
find "$target_root" -type f -name "*.ogg" -print0 | parallel -0 optimize_audio

echo "Optimization finished!"
