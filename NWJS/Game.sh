#!/usr/bin/env bash

if command -v fusermount > /dev/null 2>&1
then
    echo "FUSE found."
    www_dir=$(find . -type d -name "www" -print -quit)
    if [ -n "$www_dir" ]
    then
        echo "'www' directory found."
        cp ./package.json ../NWJS/package.json & wait $!
        echo "Mounting..."
        ../NWJS/lib/cicpoffs ./www ../NWJS/www
    fi
else echo "FUSE not found. Game will launch as case sensitive."
fi

if [[ "$XDG_SESSION_TYPE" == "wayland" ]]
then
    echo "Launching as a Wayland application."
#     ../NWJS/nw --ozone-platform=wayland; # Performance degradation
    ../NWJS/nw --ozone-platform=x11
else echo "Launching as a X11 application."; ../NWJS/nw --ozone-platform=x11
fi

if command -v fusermount > /dev/null 2>&1
then
    echo "Unmounting..." &
    fusermount -u ../NWJS/www & wait $!
    rm ../NWJS/package.json
fi
echo "Finished."
