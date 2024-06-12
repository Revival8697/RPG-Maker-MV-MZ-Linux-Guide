# RPG-Maker-MV/MZ-Linux-Guide
This guide will assist you in running and managing RPG Maker MV/MZ games natively on Linux.

## Setting up the Game Engine
- Open a terminal in this repository folder.

- Run the `nwjs-manager.sh` script and follow the on-screen steps:
    ```
    ./nwjs-manager.sh
    ```
    **Note**: You only have to run this once. You can use this script to check for updates.

## Preparing the Game Folder
- Open a terminal in this repository folder.

- Run the `game-setup.sh` script and follow the on-screen steps:
    ```
    ./game-setup.sh path/to/game_dir
    ```

## Optional modifications

### Update the game libraries
Acording to [this thread](https://forums.rpgmakerweb.com/index.php?threads/123317), updating `pixi.js` can enhance performance. However, I did not test this.

- Open a terminal in this repository folder.

- Execute the `js-update.sh` script:
    ```
    ./js-update.sh path/to/game_dir
    ```

### Decrypting the Game Files
- Open a terminal in this repository folder.

- Execute the `decrypt.sh` script:
    ```
    ./decrypt.sh path/to/game_dir
    ```

### Optimizing the Image and Audio Files
This requires decrypting the game files. This helps reduce the overall file size.
- Open a terminal in this repository folder.

- Run the `optimize.sh` script:
    ```
    ./optimize.sh path/to/game_dir
    ```

# Credits
Check out these open-source projects:
- https://github.com/bakustarver/rpgmakermlinux-cicpoffs
- https://github.com/m5kro/Painless-Porter-CLI
- https://github.com/adlerosn/cicpoffs
- https://github.com/nwjs/nw.js
- https://github.com/pixijs/pixijs
- https://gitlab.com/Petschko/Java-RPG-Maker-MV-Decrypter
