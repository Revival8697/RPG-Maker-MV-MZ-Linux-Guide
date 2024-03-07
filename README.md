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
    **Note**: Most RPG Maker MV games should handle being updated to Pixi v4.8 without any issues. If you encounter any problems, fallback to Pixi v4.4 by renaming `www/js/libs/pixi44.js` to `www/js/libs/pixi.js`.

### Decrypting the Game Files
- Open a terminal in this repository folder.

- Execute the `decrypt.sh` script:
    ```
    ./decrypt.sh path/to/game_dir
    ```

### Optimizing the Image Files
This step requires the image files to be decrypted and helps reduce the size of the image files.
- Open a terminal in this repository folder.

- Run the `optimize.sh` script:
    ```
    ./optimize.sh path/to/game_dir
    ```

# Credits
Check out these open-source projects:
- https://github.com/adlerosn/cicpoffs
- https://github.com/nwjs/nw.js
- https://github.com/m5kro/Painless-Porter-CLI
- https://github.com/pixijs/pixijs
- https://github.com/darsain/fpsmeter
- https://github.com/pixijs/tilemap/tree/v4.x
- https://github.com/pieroxy/lz-string
- https://github.com/effekseer/EffekseerForWebGL
- https://github.com/localForage/localForage
- https://gitlab.com/Petschko/Java-RPG-Maker-MV-Decrypter
