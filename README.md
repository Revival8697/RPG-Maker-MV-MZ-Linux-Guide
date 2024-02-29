# RPG-Maker-MV/MZ-Linux-Guide
This guide will assist you in running and managing RPG Maker MV/MZ games natively on Linux.

Prerequisite: [cicpoffs](https://github.com/adlerosn/cicpoffs) must be installed.

## Preparing the Game Files
1. Create a new folder where you will store your game. For the purpose of this guide, let’s call it `game_dir`.

2. Copy the `www` folder and the `package.json` file into `game_dir`.
    **Note**: If your game does not have a `www` folder, you will need to create it:
- Create a new folder and name it `www`.

- Move the game folders and files into the `www` folder. The specific folders and files vary depending on the game, but here are the common ones:
    - Folders: `audio`, `css`, `data`, `effects`, `fonts`, `icon`, `img`, `js`, `save`. Typically, you can copy all folders in yout game excluding `locales` and `swiftshader`.

    - Files: `index.html`, `package.json`. All games should have these 2 files.

3. Edit `game_dir/package.json`.
    - Change the `name` parameter to something unique. This could be the game’s name or `title`.

    - If you have to create the `www` folder: Change the `main` parameter to `www/index.html` and the `icon` parameter to `www/icon/icon.png`.

    Here’s an example of how your `package.json` might look:
    ```
    {
        "name": "Fun game",
        "main": "www/index.html",
        "js-flags": "--expose-gc",
        "window": {
            "title": "Fun game v0.69",
            "toolbar": false,
            "width": 816,
            "height": 624,
            "icon": "www/icon/icon.png"
        }
    }
    ```

## Setting up the Game Engine
- Run the `nwjs-manager.sh` script and follow the on-screen steps:
    ```
    ./nwjs-manager.sh
    ```

## Link the Game Files and Engine
- Open a terminal in your `game_dir` folder and run the following command:
    ```
    ln -s $XDG_DATA_HOME/porter/nwjs/Game.sh ./Game.sh
    ```

- Your file structure should look like this:
    ```
    tree -L 1 ./game_dir
    game_dir
    ├── Game.sh <-- Symbolic link
    ├── package.json
    ├── game_pic.webp <-- Image file
    └── www <-- Contains game files

    tree -L 1 $XDG_DATA_HOME/porter/nwjs
    nwjs
    ├── chrome_crashpad_handler
    ├── credits.html
    ├── Game.sh <-- File being linked
    ├── icudtl.dat
    ├── lib
    ├── locales
    ├── nw
    ├── nw_100_percent.pak
    ├── nw_200_percent.pak
    ├── resources.pak
    ├── swiftshader
    ├── v8_context_snapshot.bin
    └── www <-- Empty, acts as mount point
    ```

- You should have an image file inside your `game_dir`. This will help you easily identify your game.

- You can repeat this process for as many RPG Maker MV/MZ games as you have.

## Optional modifications

### Update the game libraries:
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

Note: This script requires [pngcheck](http://www.libpng.org/pub/png/apps/pngcheck.html), [oxipng](https://github.com/shssoichiro/oxipng), [libwebp](https://chromium.googlesource.com/webm/libwebp), and optionally [parallel](https://www.gnu.org/software/parallel) for multithreading to be installed.

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
