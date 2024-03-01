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
    **Note**: "path/to/game_dir" is the path to the folder that contain the "Game.exe".
## Optional modifications

### Update the game libraries
Acording to [this thread](https://forums.rpgmakerweb.com/index.php?threads/123317), updating `pixi.js` can enhance performance. However, I did not test this.

- Open a terminal in this repository folder.

- Execute the `libs-update.sh` script:
    ```
    ./libs-update.sh path/to/game_dir
    ```
    **Note**: Most RPG Maker MV games should handle being updated to Pixi v4.8 without any issues. If you encounter any problems, fallback to Pixi v4.4 by renaming `www/js/libs/pixi44.js` to `www/js/libs/pixi.js`.

### Decrypting the Game Files
Some games do not encrypt their files.

1. Visit [Java-RPG-Maker-MV-Decrypter](https://gitlab.com/Petschko/Java-RPG-Maker-MV-Decrypter#how-to-use) and follow the instructions provided.

2. Delete the encrypted files. Most of the time, these are the `www/audio` and `www/img` folders.

    **Note**: `www/img/system/Loading.png` and `www/img/system/Window.png` files (if they exist) are not encrypted, so do not delete them.

3. Copy the decrypted files to your `game_dir`.

4. Open the `www/data/System.json` file, set the `hasEncryptedImages` and `hasEncryptedAudio` parameters to `false`. The edited parameters should look like this:
    ```
    ..., "hasEncryptedImages": false, "hasEncryptedAudio": false, ...
    ```

### Optimizing the Image Files
This step requires the image files to be decrypted and helps reduce the size of the image files.
- Open a terminal in this repository folder.

- Run the `optimize.sh` script:
    ```
    ./optimize.sh path/to/game_dir
    ```

    **Note**: This script requires [pngcheck](http://www.libpng.org/pub/png/apps/pngcheck.html), [oxipng](https://github.com/shssoichiro/oxipng), [libwebp](https://chromium.googlesource.com/webm/libwebp) to be installed, and optionally [parallel](https://www.gnu.org/software/parallel) for multithreading.

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
