#!/usr/bin/bash

set -e

# Check for required commands
for cmd in jq pngcheck oxipng cwebp ffmpeg parallel
do
  if ! command -v $cmd >/dev/null 2>&1
  then
    echo >&2 "$cmd is required but it's not installed. Aborting."; exit 1
  fi
done

# Check if the user provided a directory path
if [ $# -ne 1 ]
then
  echo "Usage: $0 path/to/game_dir"; exit 1
fi

game_dir="${1%/}"

# Check if input path is a RPG Maker MV/MZ game
if [[ ! -d "$game_dir"/www ]]
then
  echo "Error: Input path is not a RPG Maker MV/MZ game."; exit 1
fi
# Check if files are encrypted
hasEncryptedImages=$(jq -r '.hasEncryptedImages' "$game_dir"/www/data/System.json)
hasEncryptedAudio=$(jq -r '.hasEncryptedAudio' "$game_dir"/www/data/System.json)

if [[ "$hasEncryptedImages" == "true" ]] || [[ "$hasEncryptedAudio" == "true" ]]
then
    echo "Files are encrypted. Run the decrypt.sh script."; exit 1
fi

# Function to optimize image files
optimize_image() {
  local file="$1"
  # Check if a file is an APNG file
  if pngcheck -v "$file" | grep -q -E "acTL"
  then
    oxipng --opt max --strip safe "$file" || echo >&2 "oxipng failed on $file. Continuing"
  else
    cwebp -lossless "$file" -o "$file" || echo >&2 "cwebp failed on $file. Continuing."
  fi
}

# Function to optimize audio files
optimize_audio() {
    local file="$1"
    ffmpeg -i "$file" -c:a libopus "${file%.ogg}.opus"
    mv "${file%.ogg}.opus" "$file"
}

export -f optimize_image
export -f optimize_audio

# Backup original files
if [ ! -d "$game_dir"/www_backup ]
then
  cp -R "$game_dir"/www "$game_dir"/www_backup
else
  echo "Backup already exists. Skipping backup."
fi

# Optimize icon separately
if [ -f "$game_dir"/www/icon/icon.png ]
then
  oxipng --opt max --strip all "$game_dir"/www/icon/icon.png
else
  echo >&2 "Icon doesn't exist. Ignoring."
fi

# Run functions multithreaded
find "$game_dir"/www -type f -name "*.png" ! -name "icon.png" | parallel optimize_image
find "$game_dir"/www -type f -name "*.ogg" | parallel optimize_audio

echo "Finished!"
