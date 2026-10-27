#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PYTHON_HELPER="$SCRIPT_DIR/storage/fix-asset-casing.py"

# Function: Display usage
usage() {
    cat <<EOF
Usage: $0 path/to/game_dir [OPTIONS]

Resolves RPG Maker MV/MZ asset loading errors caused by case-sensitive filesystem
mismatches on Linux (e.g., ext4) by renaming mismatched files to match engine references.

Options:
    --dry-run, -n    Preview changes without renaming any files
    --help, -h       Display this help message
EOF
    exit "${1:-1}"
}

DRY_RUN=false
TARGET_DIR=""

# Parse arguments
while [ $# -gt 0 ]; do
    case "$1" in
        --dry-run|-n)
            DRY_RUN=true
            shift
            ;;
        --help|-h)
            usage 0
            ;;
        *)
            if [ -z "$TARGET_DIR" ]; then
                TARGET_DIR="$1"
            else
                echo >&2 "Error: Unexpected argument '$1'"
                usage
            fi
            shift
            ;;
    esac
done

if [ -z "$TARGET_DIR" ]; then
    echo >&2 "Error: Missing game directory argument."
    usage
fi

# 1. Environment & Game Root Detection
echo "============================================================"
echo " RPG Maker MV/MZ Asset Case-Sensitivity Fixer"
echo "============================================================"

if ! command -v python3 >/dev/null 2>&1; then
    echo >&2 "Error: 'python3' is required for JSON AST parsing but was not found in PATH."
    exit 1
fi

if [ ! -f "$PYTHON_HELPER" ]; then
    echo >&2 "Error: Python helper script not found at '$PYTHON_HELPER'."
    exit 1
fi

if [ ! -d "$TARGET_DIR" ]; then
    echo >&2 "Error: Directory '$TARGET_DIR' does not exist."
    exit 1
fi

TARGET_DIR="$(cd "$TARGET_DIR" && pwd)"
echo "Target directory: $TARGET_DIR"

# Verify read/write permissions
if [ ! -r "$TARGET_DIR" ]; then
    echo >&2 "Error: Target directory '$TARGET_DIR' is not readable."
    exit 1
fi

if [ "$DRY_RUN" = false ] && [ ! -w "$TARGET_DIR" ]; then
    echo >&2 "Error: Target directory '$TARGET_DIR' is not writable. Write permissions are required to rename files."
    exit 1
fi

# Account for nested layouts (www/data, www/img) and standard layouts (data, img)
ASSET_ROOT=""
if [ -d "$TARGET_DIR/www/data" ] && [ -d "$TARGET_DIR/www/img" ]; then
    ASSET_ROOT="$TARGET_DIR/www"
    echo "Detected layout:  Nested (RPG Maker MV www/ directory)"
elif [ -d "$TARGET_DIR/data" ] && [ -d "$TARGET_DIR/img" ]; then
    ASSET_ROOT="$TARGET_DIR"
    echo "Detected layout:  Standard (Root data/ and img/ directories)"
else
    echo >&2 "Error: Could not locate RPG Maker MV/MZ asset directories (data/ and img/) in '$TARGET_DIR'."
    exit 1
fi

echo "Asset root:       $ASSET_ROOT"

# Confirm permissions on data/ and img/
for sub in data img; do
    dir_path="$ASSET_ROOT/$sub"
    if [ ! -r "$dir_path" ]; then
        echo >&2 "Error: Directory '$dir_path' is not readable."
        exit 1
    fi
    if [ "$DRY_RUN" = false ] && [ ! -w "$dir_path" ]; then
        echo >&2 "Error: Directory '$dir_path' is not writable."
        exit 1
    fi
done

echo "Permissions:      Read/Write confirmed"
if [ "$DRY_RUN" = true ]; then
    echo "Mode:             DRY-RUN (No files will be modified)"
else
    echo "Mode:             EXECUTION (Two-step safe renaming enabled)"
fi
echo "============================================================"

# Execute standalone Python helper for reference extraction, matching, and safe renaming
python3 "$PYTHON_HELPER" "$ASSET_ROOT" "$DRY_RUN"

echo "============================================================"
echo "Done."
