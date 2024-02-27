#!/usr/bin/env bash

set -e

command -v cicpoffs >/dev/null 2>&1 || { echo >&2 "cicpoffs is required but it's not installed. Aborting."; exit 1; }

if command -v fusermount > /dev/null 2>&1
then
    echo "FUSE found."
    if [ -d ./www ]
    then
        echo "'www' directory found."
        if cp ./package.json ../nwjs/package.json
        then
            echo "Copied package.json successfully."
        else
            echo "Failed to copy package.json. Exiting."; exit 1
        fi
        echo "Mounting..."
        if cicpoffs ./www ../nwjs/www
        then
            echo "Mounted successfully."
        else
            echo "Failed to mount. Exiting."; exit 1
        fi
    else
        echo "'www' directory not found. Exiting."; exit 1
    fi
else
    echo "FUSE not found. Game will launch as case sensitive."
fi

if [[ "$XDG_SESSION_TYPE" == "wayland" ]]
then
    echo "Launching as a Wayland application."
    # ../nwjs/nw --ozone-platform=wayland; # Performance degradation
    if ! ../nwjs/nw --ozone-platform=x11
    then
        echo "Failed to launch. Exiting."; exit 1
    fi
else
    echo "Launching as a X11 application."
    if ! ../nwjs/nw --ozone-platform=x11
    then
        echo "Failed to launch. Exiting."; exit 1
    fi
fi

if command -v fusermount > /dev/null 2>&1
then
    echo "Unmounting..." &
    if fusermount -u ../nwjs/www
    then
        echo "Unmounted successfully."
    else
        echo "Failed to unmount. Exiting."; exit 1
    fi
    if rm ../nwjs/package.json
    then
        echo "Removed package.json successfully."
    else
        echo "Failed to remove package.json. Exiting."; exit 1
    fi
fi
echo "Finished."
