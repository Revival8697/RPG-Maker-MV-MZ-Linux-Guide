#!/bin/bash

set -e

# Check if jq is installed
if ! command -v jq >/dev/null 2>&1
then
    echo >&2 "jq is required but it's not installed. Aborting."; exit 1
fi

# Check if the user provided a directory path
if [ $# -ne 1 ]
then
    echo "Usage: $0 path/to/game_dir"; exit 1
fi

game_dir="${1%/}"

# Check if input path is a RPG Maker MV/MZ game
if [[ ! -f "$game_dir"/package.json ]]
then
    echo "Error: Input path is not a RPG Maker MV/MZ game."; exit 1
fi

# Check if the game is MV or MZ
if [ -d "$game_dir"/www ]
then
    echo "Game is MV."

    # Delete everything except "www" and "package.json"
    find "$game_dir" -mindepth 1 -maxdepth 1 ! -name "www" ! -name "package.json" -exec rm -rf {} \;
else
    echo "Game is MZ. Repackaging..."

    mkdir "$game_dir"/www

    # Folders and files that will not be copied to the "www" folder
    excluded_items=("www" "captures" "locales" "swiftshader" "credits.html" "d3dcompiler_47.dll" "ffmpeg.dll" "Game.exe" "icudtl.dat" "libEGL.dll" "libGLESv2.dll" "node.dll" "notification_helper.exe" "nw_100_percent.pak" "nw_200_percent.pak" "nw_elf.dll" "nw.dll" "resources.pak" "v8_context_snapshot.bin")

    # Copy folder and files not in excluded_items
    for item in "$game_dir"/*
    do
        base_item=$(basename "$item")
        if [[ ! " ${excluded_items[@]} " =~ " ${base_item} " ]]
        then
            cp -r "$item" "$game_dir"/www
        fi
    done

    # Delete everything except "www"
    find "$game_dir" -mindepth 1 -maxdepth 1 ! -name "www" -exec rm -rf {} \;

    # Copy and edit MZ "package.json"
    if [[ -e "$game_dir"/www/package.json ]]
    then
        cp "$game_dir"/www/package.json "$game_dir"/package.json
        jq '.main = "www/index.html" | .window.icon = "www/icon/icon.png"' "$game_dir"/package.json > "$game_dir"/package.json.temp && mv "$game_dir"/package.json.temp "$game_dir/package.json"
    else
        echo "Error: package.json does not exist in the 'www' directory."; exit 1
    fi
fi

echo -e "\nEnter something unique to identify this game (can simply be the game's name):"
read INPUT

# Fill the "name" parameter
if [[ -n "$INPUT" ]]
then
    jq '.name = "$INPUT"' "$game_dir"/package.json > "$game_dir"/package.json.tmp
else
    jq '.name = "default"' "$game_dir"/package.json > "$game_dir"/package.json.tmp
fi
mv "$game_dir"/package.json.tmp "$game_dir"/package.json

# Check if user has set up the Game Engine
if [ ! -f "$XDG_DATA_HOME"/nwjs/Game.sh ]
then
    echo -e "Error: Game Engine does not exist.\nRun the nwjs-manager.sh script."; exit 1
fi

# Create the symbolic link
ln -sf "$XDG_DATA_HOME"/nwjs/Game.sh "$game_dir"/Game.sh

echo "Finished!"
