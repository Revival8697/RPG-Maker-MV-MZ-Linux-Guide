#!/bin/bash

set -e

command -v jq >/dev/null 2>&1 || { echo >&2 "jq is required but it's not installed. Aborting."; exit 1; }
command -v java >/dev/null 2>&1 || { echo >&2 "java is required but it's not installed. Aborting."; exit 1; }

# Check if the user provided a directory path.
if [ $# -ne 1 ]
then
    echo "Usage: $0 path/to/game_dir"; exit 1
fi

game_dir="${1%/}"

# Check if the input path is a directory
if [[ ! -d "$game_dir" ]]
then
    echo "Error: Input path is not a directory."; exit 1
fi

# Check if input path is a RPG Maker MV/MZ game
if [[ ! -d "$game_dir"/www ]]
then
    echo "Error: Input path is not a RPG Maker MV/MZ game."; exit 1
fi

java -jar ./storage/"RPG Maker MV Decrypter.jar" decrypt "$game_dir" "$game_dir"

if [ $? -ne 0 ]
then
    echo "Failed to decrypt. Aborting."; exit 1
fi

rm -r ./output

# Extensions of files to delete
extensions=(".rpgmvo" ".rpgmvm" ".rpgmvp" ".ogg_" ".m4a_" ".png_")

for file in "${extensions[@]}"
do
  find "$game_dir" -type f -name "*$file" -delete
done

# Let the game know files are decrypted
jq '.hasEncryptedAudio = false | .hasEncryptedImages = false' "$game_dir"/www/data/System.json > "$game_dir"/www/data/System.json.temp
mv "$game_dir"/www/data/System.json.temp "$game_dir"/www/data/System.json

echo "Finished."
