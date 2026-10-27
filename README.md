# RPG-Maker-MV/MZ-Linux-Guide
This guide will assist you in running and managing RPG Maker MV/MZ games natively on Linux.

## Prerequisites
Make sure NW.js is installed on your system and accessible as `nw` in your `$PATH`.

## Verify system environment
- Open a terminal in this repository folder.

- Run the `environment-setup.sh` script to verify your system `nw` executable and other utilities:
    ```bash
    $ ./environment-setup.sh
    ```

## Setup game folder
- Run the `game-setup.sh` script:
    ```bash
    $ ./game-setup.sh path/to/game_directory
    ```
- Once set up, run the game using either method:
    ```bash
    # 1. Directly using the system nw CLI:
    $ nw path/to/game_directory

    # 2. Using the launcher script inside the game folder:
    $ path/to/game_directory/Game.sh
    ```

## Optional modifications

### Update game libraries
According to [this thread](https://forums.rpgmakerweb.com/index.php?threads/123317), updating `pixi.js` can enhance performance.

This script also updates core engine files if the game is MV.

- Run the `lib-update.sh` script:
    ```bash
    $ ./lib-update.sh path/to/game_directory
    ```

### Decrypt game files
Decryption is handled natively using the high-performance Rust-based `rpgmv-decrypter` utility (precompiled in `./storage/rpgmv-decrypter` or discovered via `$PATH`). This completely removes the legacy Java/JRE dependency and provides multi-threaded parallel decryption with automatic key discovery.

- Run the `decrypt.sh` script:
    ```bash
    $ ./decrypt.sh path/to/game_directory [optional_hex_key]
    ```

If an explicit encryption key is omitted, the tool automatically detects the key from `System.json` or derives it directly from encrypted PNG image headers.

*(Optional)* To compile or update `rpgmv-decrypter` from source using the Rust toolchain:
```bash
git clone https://github.com/Revival8697/Rust-RPG-Maker-MV-Decrypter.git
cd Rust-RPG-Maker-MV-Decrypter
cargo build --release
cp target/release/rpgmv-decrypter /path/to/RPG-Maker-MV-MZ-Linux-Guide/storage/
```

### Optimize image and audio files
This requires decrypting the game files first. It losslessly optimizes PNG images and remuxes OGG audio to strip redundant metadata/embedded artwork.
- Run the `optimize.sh` script:
    ```bash
    $ ./optimize.sh path/to/game_directory
    ```

# Credits
Check out these open-source projects:
- https://github.com/nwjs/nw.js
- https://github.com/pixijs/pixijs
- https://github.com/Revival8697/Rust-RPG-Maker-MV-Decrypter
