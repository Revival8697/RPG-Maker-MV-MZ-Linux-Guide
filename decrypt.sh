#!/bin/bash

set -e

# Check for required commands
for cmd in jq java
do
  if ! command -v $cmd >/dev/null
  then
    echo >&2 "$cmd is required but it's not installed. Aborting."; exit 1
  fi
done

# Check if the user provided a directory path.
if [ $# -ne 1 ]; then
    echo "Usage: $0 path/to/game_dir"
    exit 1
fi

game_dir="${1%/}"

# Check if input path is a RPG Maker MV/MZ game
if [[ ! -d "$game_dir"/www ]]
then
    echo "Error: Input path is not a RPG Maker MV/MZ game."; exit 1
fi

# Decrypt files
if ! java -jar ./storage/"RPG Maker MV Decrypter.jar" decrypt "$game_dir" "$game_dir"
then
    echo "Failed to decrypt. Aborting."; exit 1
fi

# Cleanup
rm -r ./output ./updateCache.pref

# Extensions of encrypted files
extensions=(".rpgmvo" ".rpgmvm" ".rpgmvp" ".ogg_" ".m4a_" ".png_")

# Delete encrypted files
for file in "${extensions[@]}"
do
  find "$game_dir" -type f -name "*$file" -delete
done

# Let the game know files are now decrypted
jq -c '.hasEncryptedAudio = false | .hasEncryptedImages = false' "$game_dir"/www/data/System.json > "$game_dir"/www/data/System.json.temp && mv "$game_dir"/www/data/System.json.temp "$game_dir"/www/data/System.json

echo "Finished."
