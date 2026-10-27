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
- Run the `decrypt.sh` script:
    ```bash
    $ ./decrypt.sh path/to/game_directory
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
- https://gitlab.com/Petschko/Java-RPG-Maker-MV-Decrypter
