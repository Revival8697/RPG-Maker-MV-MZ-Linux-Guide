#!/bin/bash

set -e

# Check if the user provided a directory path.
if [ $# -ne 1 ]
then
    echo "Usage: $0 path/to/game_dir"; exit 1
fi

game_dir="${1%/}"

# Check if input path is a RPG Maker MV/MZ game
if [[ ! -f "$game_dir"/www/js/libs/pixi.js ]]
then
    echo "Error: Input path is not a RPG Maker MV/MZ game."; exit 1
else
    pixi="$game_dir"/www/js/libs/pixi.js
fi

# Function to copy files from the "storage" folder
copy_files() {
    local variant=$1
    echo "Game is $variant."
    # Backup original files
    if [[ ! -d "$game_dir"/www/js/libs_backup ]]
    then
        cp -r "$game_dir"/www/js "$game_dir"/www/js_backup
    fi
    cp -r ./storage/"$variant"/* "$game_dir"/www/js
}

# Check if the game is MV or MZ
if grep -q "pixi.js - v4." "$pixi"
then
    copy_files "MV"
elif grep -q "pixi.js - v5." "$pixi"
then
    copy_files "MZ"
else
    echo "Error: Can't detect pixi.js version. Aborting."; exit 1
fi

echo "Finished!"
