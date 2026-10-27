#!/usr/bin/env bash
set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "=== RPG Maker Linux Environment Check ==="

has_error=false

# Check for system-provided NW.js installation
echo -n "Checking for system-provided 'nw'... "
if command -v nw >/dev/null 2>&1; then
    NW_PATH="$(command -v nw)"
    echo "Found at $NW_PATH"
    if nw --version >/dev/null 2>&1; then
        echo "NW.js version: $(nw --version)"
    fi
else
    echo "NOT FOUND"
    echo >&2 -e "  [!] Notice: System-provided NW.js executable ('nw') was not found in your PATH."
    has_error=true
fi

# Check for required core utility: jq
echo -n "Checking for 'jq'... "
if command -v jq >/dev/null 2>&1; then
    echo "Found ($(jq --version))"
else
    echo "NOT FOUND"
    echo >&2 "  [!] Error: 'jq' is required by game-setup.sh, decrypt.sh, and optimize.sh."
    has_error=true
fi

# Check for optional tools used by optional scripts
echo -e "\nChecking optional utilities for asset tools:"
for tool in rpgmv-decrypter ffmpeg parallel oxipng; do
    echo -n "  - $tool: "
    if [ "$tool" = "rpgmv-decrypter" ]; then
        if [ -x "$SCRIPT_DIR/storage/rpgmv-decrypter" ]; then
            echo "Found (bundled in storage/)"
        elif command -v "$tool" >/dev/null 2>&1; then
            echo "Found ($(command -v "$tool"))"
        else
            echo "Not found (optional; used by decrypt.sh)"
        fi
    else
        if command -v "$tool" >/dev/null 2>&1; then
            echo "Found"
        else
            echo "Not found (optional; used by optimize.sh)"
        fi
    fi
done

if [ "$has_error" = true ]; then
    echo -e "\nEnvironment check completed with missing dependencies. Please install the missing required components above."
    exit 1
else
    echo -e "\nEnvironment is ready! You can now run games with: nw path/to/game"
fi
