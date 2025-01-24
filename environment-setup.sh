#!/bin/bash

# Check for required commands
for cmd in wget unzip wine; do
  if ! command -v "$cmd" >/dev/null 2>&1; then
    echo >&2 "$cmd is required but it's not installed. Aborting."
    exit 1
  fi
done

# Set the target directory
TARGET_DIR="$XDG_DATA_HOME/nwjs-v0.48.4-win-x64"

# Check if the target directory exists
if [ -d "$TARGET_DIR" ]; then
  echo "nwjs-v0.48.4-win-x64 is already in its correct location. Exiting."
  exit 1
fi

echo "Downloading NW.js archive."
wget -q -O /tmp/nwjs-v0.48.4-win-x64.zip https://dl.nwjs.io/v0.48.4/nwjs-v0.48.4-win-x64.zip

echo "Extracting NW.js archive."
unzip -q /tmp/nwjs-v0.48.4-win-x64.zip -d "$XDG_DATA_HOME"

echo "Removing temporary file."
rm /tmp/nwjs-v0.48.4-win-x64.zip

echo "Copying launch script."
cp ./storage/Game.sh "$TARGET_DIR/Game.sh"

echo "Creating wine prefix."
wine hostname
sleep 1

echo "Installing DXVK."
./storage/dxvk-2.5.3/setup_dxvk.sh uninstall
./storage/dxvk-2.5.3/setup_dxvk.sh install

echo "Finished."
