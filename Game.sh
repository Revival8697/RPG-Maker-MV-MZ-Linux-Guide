#!/usr/bin/bash

set -e

command -v cicpoffs >/dev/null 2>&1 || { echo -e >&2 "cicpoffs is required but it's not installed."`
`"\nInstall it here: https://github.com/adlerosn/cicpoffs/releases/latest."; exit 1; }

if [ -d ./www ]
then
    echo "'www' directory found."
    if cp ./package.json "$XDG_DATA_HOME"/nwjs/package.json
    then
        echo "Copied package.json successfully."
    else
        echo "Error: Failed to copy package.json."; exit 1
    fi

    echo "Mounting..."
    if cicpoffs ./www "$XDG_DATA_HOME"/nwjs/www
    then
        echo "Mounted successfully."
    else
        echo "Error: Failed to mount."; exit 1
    fi

else
    echo "Error: 'www' directory not found."; exit 1
fi

if [[ "$XDG_SESSION_TYPE" == "wayland" ]]
then
    echo "Launching as a native Wayland application."
    if ! "$XDG_DATA_HOME"/nwjs/nw --ozone-platform=x11  # Performance degradation if launch as Wayland
    then
        echo "Error: Failed to launch."; exit 1
    fi

elif ! "$XDG_DATA_HOME"/nwjs/nw --ozone-platform=x11
then
    echo "Error: Failed to launch."; exit 1
fi

echo "Unmounting..."
if fusermount -u "$XDG_DATA_HOME"/nwjs/www
then
    echo "Unmounted successfully."
else
    echo "Error: Failed to unmount."; exit 1
fi

if rm "$XDG_DATA_HOME"/nwjs/package.json
then
    echo "Removed package.json successfully."
else
    echo "Error: Failed to remove package.json."; exit 1
fi

echo "Finished!"
