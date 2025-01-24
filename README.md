# RPG-Maker-MV/MZ-Linux-Guide
This guide will assist you in running and managing RPG Maker MV/MZ games natively on Linux.

## Setup wine environment
- Open a terminal in this repository folder.

- Run the `environment-setup.sh` script:
    ```
    $ ./environment-setup.sh
    ```
    **Note**: You only have to run this once. You can use this script to check for updates.

## Setup game folder
- Open a terminal in this repository folder.

- Run the `game-setup.sh` script and follow the on-screen steps:
    ```
    $ ./game-setup.sh path/to/game_directory
    ```

## Optional modifications

### Update game libraries
Acording to [this thread](https://forums.rpgmakerweb.com/index.php?threads/123317), updating `pixi.js` can enhance performance (untested).


This script also update core files to the latest version if game is MV.

- Open a terminal in this repository folder.

- Run the `lib-update.sh` script:
    ```
    $ ./lib-update.sh path/to/game_directory
    ```

### Decrypt game Files
- Open a terminal in this repository folder.

- Run the `decrypt.sh` script:
    ```
    $ ./decrypt.sh path/to/game_directory
    ```

### Optimize image and audio Files
This requires decrypting the game files. This helps reduce the overall file size.
- Open a terminal in this repository folder.

- Run the `optimize.sh` script:
    ```
    $ ./optimize.sh path/to/game_directory
    ```

# Credits
Check out these open-source projects:
- https://github.com/adlerosn/cicpoffs
- https://github.com/bakustarver/rpgmakermlinux-cicpoffs
- https://github.com/doitsujin/dxvk
- https://github.com/m5kro/Painless-Porter-CLI
- https://github.com/nwjs/nw.js
- https://github.com/pixijs/pixijs
- https://gitlab.com/Petschko/Java-RPG-Maker-MV-Decrypter
- https://gitlab.winehq.org/wine/wine
