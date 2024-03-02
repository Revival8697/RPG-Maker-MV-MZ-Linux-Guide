#!/usr/bin/bash

set -e

# Check if cicpoffs is installed
if ! command -v cicpoffs >/dev/null 2>&1
then
    echo >&2 "cicpoffs is required but it's not installed."
    echo >&2 "Install it here: https://github.com/adlerosn/cicpoffs/releases/latest."
    exit 1
fi

# Check if the "www" folder exists
if [ ! -d "./www" ]
then
    echo "Error: 'www' folder not found."; exit 1
fi

# Copy package.json
if ! cp ./package.json "$XDG_DATA_HOME"/nwjs/package.json
then
    echo "Error: Failed to copy package.json."; exit 1
fi

# Mount the game files non case sensitive
if ! cicpoffs ./www "$XDG_DATA_HOME"/nwjs/www
then
    echo "Error: Failed to mount."; exit 1
fi

# Launch the main executable
if ! "$XDG_DATA_HOME"/nwjs/nw --ozone-platform=x11
then
    echo "Error: Failed to launch."; exit 1
fi

# Unmount the game files
if ! fusermount -u "$XDG_DATA_HOME"/nwjs/www
then
    echo "Error: Failed to unmount."; exit 1
fi

# Remove package.json in the game engine folder
if ! rm "$XDG_DATA_HOME"/nwjs/package.json
then
    echo "Error: Failed to remove package.json."; exit 1
fi

echo "Finished!"
