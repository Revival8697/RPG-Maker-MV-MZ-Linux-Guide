#!/bin/bash

# Check for required commands
for cmd in wget unzip wine
do
  if ! command -v $cmd >/dev/null 2>&1
  then
    echo >&2 "$cmd is required but it's not installed. Aborting."; exit 1
  fi
done

# Set the target directory

# Check if the target directory exists
if [ -d "$XDG_DATA_HOME/nwjs-v0.48.4-win-x64" ]; then
  echo "nwjs-v0.48.4-win-x64 is already is its corret location. Exiting."
  exit 1
fi

echo "Downloading NW.js archive."
wget -q -O /tmp/nwjs-v0.48.4-win-x64.zip https://dl.nwjs.io/v0.48.4/nwjs-v0.48.4-win-x64.zip

echo "Extarcting NW.js archive."
unzip -q /tmp/nwjs-v0.48.4-win-x64.zip -d "$XDG_DATA_HOME"

echo "Removing temporary file."
rm /tmp/nwjs-v0.48.4-win-x64.zip

echo "Copying launch script."
cp ./storage/Game.sh "$XDG_DATA_HOME/nwjs-v0.48.4-win-x64/Game.sh"

echo "Installing DXVK."
./storage/dxvk-2.4/setup_dxvk.sh install

echo "Installing VKD3D."
./storage/vkd3d-proton-2.13/setup_vkd3d_proton.sh install

echo "Finished."
