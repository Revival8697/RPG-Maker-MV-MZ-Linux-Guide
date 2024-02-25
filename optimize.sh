#!/usr/bin/env bash

set -e

command -v pngcheck >/dev/null 2>&1 || { echo >&2 "pngcheck is required but it's not installed. Aborting."; exit 1; }
command -v oxipng >/dev/null 2>&1 || { echo >&2 "oxipng is required but it's not installed. Aborting."; exit 1; }
command -v cwebp >/dev/null 2>&1 || { echo >&2 "cwebp is required but it's not installed. Aborting."; exit 1; }
command -v parallel >/dev/null 2>&1 || { echo >&2 "parallel is required but it's not installed. Aborting."; exit 1; }

optimize_images()
{
    file="$1"
    if pngcheck -v "$file" | grep -q -E "acTL" # Check if a file is an APNG file
    then
        oxipng --opt max --strip safe "$file" || echo >&2 "oxipng failed on $file. Continuing"
    else
        cwebp -lossless "$file" -o "$file" || echo >&2 "cwebp failed on $file. Continuing."
    fi
}

export -f optimize_images

[ -d ./www ] || { echo >&2 "./www doesn't exist. Aborting."; exit 1; }

if [ ! -d ./www_backup ]
then
    cp -R ./www ./www_backup
else
    echo "./www_backup already exists. Skipping backup."
fi

if [ -f ./www/icon/icon.png ]
then
    oxipng --opt max --strip all ./www/icon/icon.png
else
    echo >&2 "./www/icon/icon.png doesn't exist. Ignoring."
fi

find ./www -type f -name "*.png" ! -name "icon.png" | parallel optimize_images

echo "Finished."
