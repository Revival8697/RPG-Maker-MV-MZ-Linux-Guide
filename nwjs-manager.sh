#!/bin/bash

set -e

# Check for required commands
for cmd in curl jq
do
  if ! command -v $cmd >/dev/null 2>&1
  then
    echo >&2 "$cmd is required but it's not installed. Aborting."; exit 1
  fi
done

URL="https://nwjs.io/versions"
TMP_FILE="/tmp/versions.json"
CURRENT="$XDG_DATA_HOME/nwjs/version.txt"

# Function to download, extract NWJS. Copy "Game.sh" to "target_dir"
download() {
    local version=$1
    local url="https://dl.nwjs.io/${version}/nwjs-sdk-${version}-linux-x64.tar.gz"
    local tmp_file="/tmp/nwjs.tar.gz"
    local extract_dir="/tmp/nwjs-sdk-${version}-linux-x64"
    local target_dir="$XDG_DATA_HOME"/nwjs

    echo -e "\nDownloading $version:"
    if curl -fSL -o "$tmp_file" "$url" && tar -xzf "$tmp_file" -C "/tmp"
    then
        rm -rf "$target_dir"
        mkdir -p "$target_dir"
        cp -R "$extract_dir"/* "$target_dir"/
        mkdir "$target_dir"/www
        cp ./Game.sh "$target_dir"
        echo "$version" > "$CURRENT"
    else
        echo "Error: Failed to download or extract."; exit 1
    fi
}

# Query the URL
echo -e "Querying available versions...\n"
if ! curl -fsSL -o "$TMP_FILE" "$URL"
then
    echo "Error: Failed to query."; exit 1
fi

LATEST=$(jq -r '.latest' "$TMP_FILE")
VERSIONS=$(jq -r '.versions[].version' "$TMP_FILE" | awk '{ORS = (NR%5 ? ", " : "\n")} {print}')
LAST20=$(echo "$VERSIONS" | head -4)

echo -e "Listing 20 most recent versions:\n$LAST20\n"
echo -e "Enter the version (including the v) you wish to download.\nLeft blank and press Enter to download the latest version."

read INPUT

# Check if the user enter a valid version
if [[ -n "$INPUT" ]]
then
    if [[ "$VERSIONS" =~ "$INPUT" ]]
    then
        download "$INPUT"
    else
        echo "Error: $INPUT is not a valid version."
    fi
# If nwjs-version.txt doesn't exist of if the current version is not the latest, download the latest version
elif [[ ! -f "$CURRENT" ]] || [[ $(cat "$CURRENT") != "$LATEST" ]]
then
    download "$LATEST"
else
    echo "$LATEST is already up to date."
fi

echo "Finished!"
