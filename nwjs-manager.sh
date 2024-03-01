#!/bin/env bash

set -e

command -v curl >/dev/null 2>&1 || { echo >&2 "curl is required but it's not installed. Aborting."; exit 1; }
command -v jq >/dev/null 2>&1 || { echo >&2 "jq is required but it's not installed. Aborting."; exit 1; }

URL="https://nwjs.io/versions"
TMP_FILE="/tmp/versions.json"
CURRENT="$XDG_DATA_HOME/porter/nwjs-version.txt"

download() {
    local version=$1
    local url="https://dl.nwjs.io/${version}/nwjs-sdk-${version}-linux-x64.tar.gz"
    local tmp_file="/tmp/nwjs.tar.gz"
    local extract_dir="/tmp/nwjs-sdk-${version}-linux-x64"
    local target_dir="$XDG_DATA_HOME/porter/nwjs"

    echo -e "\nDownloading $version:"
    if curl -fSL -o "$tmp_file" "$url" && tar -xzf "$tmp_file" -C "/tmp"
    then
        rm -rf "$target_dir"
        mkdir -p "$target_dir"
        cp -R "$extract_dir"/* "$target_dir"/
        mkdir "$target_dir"/www
        cp ./Game.sh "$target_dir"
        echo "$version" > "$CURRENT"
        echo "Finished!"
    else
        echo "Failed to download or extract $url"; exit 1
    fi
}

# Query the URL and store the response in /tmp.
echo -e "Querying available versions...\n"
if ! curl -fsSL -o "$TMP_FILE" "$URL"
then
    echo "Failed to download $URL"; exit 1
fi

# Get LATEST version from response.
LATEST=$(jq -r '.latest' "$TMP_FILE")

# Get valid VERSIONS from response.
VERSIONS=$(jq -r '.versions[].version' "$TMP_FILE" | head -10 | awk '{ORS = (NR%5 ? ", " : "\n")} {print}')
echo -e "Available versions:\n$VERSIONS\n"

echo -e "Enter a specific version (including the v) if you wish to."`
`"\nPress Enter to download the latest version:"

read INPUT

# Check if the user entered something.
if [[ -n "$INPUT" ]]
then
    # Check if the entered version is valid.
    if [[ "$VERSIONS" =~ "$INPUT" ]]
    then
        download "$INPUT"
    else
        echo "Error: $INPUT is not a valid version."
    fi
# If nwjs-version.txt doesn't exist OR if CURRENT version is not LATEST, download the latest version.
elif [[ ! -f "$CURRENT" ]] || [[ $(cat "$CURRENT") != "$LATEST" ]]
then
    download "$LATEST"
else
    echo "$LATEST is already up to date."
fi
