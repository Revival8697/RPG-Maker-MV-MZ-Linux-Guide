# RPG Maker MV/MZ Linux Helper

A high-performance helper tool and guide to run, configure, fix, and optimize RPG Maker MV and MZ games natively on Linux using NW.js.

Built with a native multi-threaded CLI engine, it consolidates asset decryption, case-sensitivity repairs for Linux filesystems, runtime binary stripping, and lossless in-process image/audio optimization into a single self-contained, zero-dependency static executable.

---

## Prerequisites

- **NW.js**: Installed on your system and accessible as `nw` in `$PATH` (download from [https://nwjs.io/](https://nwjs.io/) or install via your distribution's package manager).
- **Rust Toolchain**: To compile the native binary from source (`cargo` & `rustc` 1.70+).

---

## Building from Source

Build the release binary using Cargo:

```bash
cargo build --release
```

The compiled executable is generated at `./target/release/rpgmaker-helper`.

---

## Quick Start

Manage and optimize any RPG Maker MV or MZ game using the unified CLI:

```bash
# 1. Audit and scan a game folder (non-destructive diagnostic scan)
$ rpgmaker-helper scan path/to/game_directory

# 2. Run the complete automated workflow (setup -> decrypt -> fix-casing -> update-libs -> optimize)
$ rpgmaker-helper all path/to/game_directory
```

---

## CLI Subcommands Overview

| Command | Description |
| :--- | :--- |
| `scan <game_dir>` | Read-only diagnostic audit of game layout, `package.json`, encryption status, and asset casing mismatches. |
| `setup <game_dir>` | Purges Windows runtime binaries, configures `package.json`, and deploys native Linux launchers (`Game` and desktop shortcut). |
| `fix-casing <game_dir>` | Resolves case-sensitive filesystem errors by renaming mismatched files and generating case-alias symlinks. |
| `decrypt <game_dir>` | Decrypts MV (`.rpgmvp`, `.rpgmvo`) and MZ (`.png_`, `.ogg_`) assets using native in-process multithreading. |
| `update-libs <game_dir>` | Updates `pixi.js` to modern optimized versions (v4.8.9 for MV, v5.3.12 for MZ) from embedded binary assets. |
| `optimize <game_dir>` | Losslessly optimizes PNG images (`oxipng`) and remuxes OGG audio. |
| `all <game_dir>` | Runs the full end-to-end preparation and optimization pipeline. |

---

## Command Reference & Examples

### 1. Audit & Scan Game Directory
Performs a read-only audit of layout structure, `package.json` parameters, engine version, encryption flags, and casing mismatches:
```bash
$ rpgmaker-helper scan path/to/game_directory
```

### 2. Setup Game Folder
Prepares the game for native NW.js execution, strips unnecessary Windows runtime files (`.exe`, `.dll`, `.pak`), configures `package.json` (`--expose-gc`, window title, icon), and generates both a native shell launcher (`Game`) and an XDG desktop entry (`<game-id>.desktop`):
```bash
$ rpgmaker-helper setup path/to/game_directory

# Or run non-interactively with a custom package identifier:
$ rpgmaker-helper setup -y --id custom-game-id path/to/game_directory
```

Once set up, launch the game via:
```bash
# 1. Directly via NW.js:
$ nw path/to/game_directory

# 2. Using the launcher script inside the game folder:
$ path/to/game_directory/Game

# 3. Via the generated desktop shortcut:
$ path/to/game_directory/<game-id>.desktop
```

### 3. Fix Asset Casing (Linux Case-Sensitivity Errors)
Linux filesystems (ext4, btrfs, xfs) are case-sensitive. RPG Maker games developed on Windows frequently contain casing mismatches in `data/*.json`, event routes, movies, or `plugins.js` compared to actual filenames on disk, leading to `Failed to load: ...` errors.

This command parses all engine declarations, handles directory-level casing differences, and safely renames mismatched files using an atomic two-step pattern:
```bash
$ rpgmaker-helper fix-casing path/to/game_directory

# Preview changes without modifying files:
$ rpgmaker-helper fix-casing --dry-run path/to/game_directory
```

### 4. Decrypt Game Assets
Decryption runs natively with multi-threaded parallel processing:
```bash
$ rpgmaker-helper decrypt path/to/game_directory [optional_hex_key]

# Options:
#   -k, --key <hex_key>   Specify 32-character hexadecimal key (auto-detected if omitted)
#   -b, --backup          Create a backup before decrypting
#   --keep-encrypted      Preserve encrypted files alongside decrypted files
#   --restore             Restore PNG images without a key using standard PNG magic headers
$ rpgmaker-helper decrypt --backup path/to/game_directory
```

If an explicit encryption key is omitted, the tool automatically detects the key from `System.json` or derives it directly from encrypted PNG headers via known-plaintext analysis.

### 5. Update Game Libraries
Updates `pixi.js` to modern optimized versions (v4.8.9 for MV, v5.3.12 for MZ). By default, updates in safe `--pixi-only` mode to preserve custom game plugins and scripts:
```bash
# Safe mode (updates Pixi.js only; recommended):
$ rpgmaker-helper update-libs path/to/game_directory

# Full mode for MV (updates Pixi.js and core rpg_*.js scripts):
$ rpgmaker-helper update-libs --full path/to/game_directory
```

### 6. Optimize Image and Audio Assets
Losslessly optimizes PNG images using native in-process `oxipng` and remuxes OGG audio to strip embedded artwork:
```bash
$ rpgmaker-helper optimize path/to/game_directory

# Optional flags:
#   -y, --yes          Auto-accept prompts
#   --images-only      Optimize only PNGs
#   --audio-only       Optimize only OGGs
#   --level <1-6|max>  Oxipng optimization level (default: 2)
#   --no-backup        Skip backup creation
```

---

## Credits

- [NW.js](https://github.com/nwjs/nw.js)
- [PixiJS](https://github.com/pixijs/pixijs)
- [oxipng](https://github.com/oxipng/oxipng)

## Acknowledgments

- **AI Provenance**: This codebase is entirely AI-generated by Google Gemini.