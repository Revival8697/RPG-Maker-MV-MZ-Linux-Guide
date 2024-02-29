#!/usr/bin/env bash

set -e

command -v cicpoffs >/dev/null 2>&1 || { echo -e >&2 "cicpoffs is required but it's not installed."`
`"\nInstall it here: https://github.com/adlerosn/cicpoffs/releases/latest."; exit 1; }

if [ -d ./www ]
then
    echo "'www' directory found."
    if cp ./package.json $XDG_DATA_HOME/porter/nwjs/package.json
    then
        echo "Copied package.json successfully."
    else
        echo "Failed to copy package.json. Exiting."; exit 1
    fi

    echo "Mounting..."
    if cicpoffs ./www $XDG_DATA_HOME/porter/nwjs/www
    then
        echo "Mounted successfully."
    else
        echo "Failed to mount. Exiting."; exit 1
    fi

else
    echo "'www' directory not found. Exiting."; exit 1
fi

if [[ "$XDG_SESSION_TYPE" == "wayland" ]]
then
    echo "Launching as a Wayland application."
    # $XDG_DATA_HOME/porter/nwjs/nw --ozone-platform=wayland; # Performance degradation
    if ! $XDG_DATA_HOME/porter/nwjs/nw --ozone-platform=x11
    then
        echo "Failed to launch. Exiting."; exit 1
    fi

else
    echo "Launching as a X11 application."
    if ! $XDG_DATA_HOME/porter/nwjs/nw --ozone-platform=x11
    then
        echo "Failed to launch. Exiting."; exit 1
    fi
fi

echo "Unmounting..."
if fusermount -u $XDG_DATA_HOME/porter/nwjs/www
then
    echo "Unmounted successfully."
else
    echo "Failed to unmount. Exiting."; exit 1
fi

if rm $XDG_DATA_HOME/porter/nwjs/package.json
then
    echo "Removed package.json successfully."
else
    echo "Failed to remove package.json. Exiting."; exit 1
fi

echo "Finished."
