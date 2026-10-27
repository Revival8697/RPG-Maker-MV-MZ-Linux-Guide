#!/usr/bin/env bash
set -e

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
for tool in java ffmpeg parallel oxipng; do
    echo -n "  - $tool: "
    if command -v "$tool" >/dev/null 2>&1; then
        echo "Found"
    else
        echo "Not found (optional; used by $(case $tool in java) echo "decrypt.sh" ;; *) echo "optimize.sh" ;; esac))"
    fi
done

if [ "$has_error" = true ]; then
    echo -e "\nEnvironment check completed with missing dependencies. Please install the missing required components above."
    exit 1
else
    echo -e "\nEnvironment is ready! You can now run games with: nw path/to/game"
fi
