#!/usr/bin/env bash

optimize_images()
{
    file="$1"
    if pngcheck -v "$file" | grep -q -E "acTL" # Check if a file is an APNG file
    then
        oxipng --opt max --strip safe "$file"
    else
        cwebp -lossless "$file" -o "$file"
    fi
}

export -f optimize_images

cp -R ./www ./www_backup & wait $!

oxipng --opt max --strip all ./www/icon/icon.png & wait $!

find ./www -type f -name "*.png" ! -name "icon.png" | parallel optimize_images
