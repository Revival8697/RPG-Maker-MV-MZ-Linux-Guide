#!/bin/bash

# Check if nwjs_dir and game_dir exist
if [ ! -d ~/.local/share/nwjs-v0.48.4-win-x64/ ] || [ ! -d ./www ]; then
    echo "Error: nwjs_dir or game_dir not found."
    exit 1
fi

# Set the source and destination directories
src_dir=~/.local/share/nwjs-v0.48.4-win-x64/
dst_dir=./

# Remove existing symlinks in game_dir
find "$dst_dir" -type l -not -name "Game.sh" -exec rm {} \;

# Loop through all files and folders in the source directory
for item in "$src_dir"/*; do
  # Skip the Game.sh file
  if [ "$(basename "$item")" != "Game.sh" ]; then
    # Create a symbolic link in the destination directory
    ln -s "$item" "$dst_dir/$(basename "$item")"
  fi
done

wine ./nw.exe

# Remove existing symlinks in game_dir
find "$dst_dir" -type l -not -name "Game.sh" -exec rm {} \;
