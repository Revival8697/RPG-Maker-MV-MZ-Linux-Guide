# RPG-Maker-MV/MZ-Linux-Guide
This guide will advise you on how to manage and play RPG Maker MV/MZ natively on Linux.

Requires [cicpoffs](https://github.com/adlerosn/cicpoffs) to be installed.

## Prepare the game files
1. Create a new folder, this will be where you store your game. I will name it `game_dir` in this guide.

2. Copy the `www` folder and the `package.json` file to the `game_dir`.

    Note: If you don't have a `www` folder (RPG Maker MZ games), you have to create it yourself:

- Create the `www` folder.

- Move the game folders and files into the `www` folder.
    - Game folders examples: `audio, css, data, effects, fonts, icon, img, js, save`.

        Most of the time, these are all folders inside your game EXCEPT `locales` and `swiftshader`. 
    
    - Game files examples: `index.html, package.json`.

        All games should have these 2 files.

    This is game dependent, not all games have the same folders and files.

3. Edit `game_dir/package.json`.
    - Edit the `name` parameter to something unique. This can simply be the game's name or `title`.

    - For RPG Maker MZ: Edit the `main` parameter to `www/index.html` and the `icon` parameter to `www/icon/icon.png`. 
    
    Example:
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

## Prepare the game engine
- Run `scripts/nwjs-manager.sh`
    ```
    ./scripts/nwjs-manager.sh
    ```

## Link the game files and engine together
- Open a terminal in your `game_dir` and run the following command:
    ```
    ln -s $XDG_DATA_HOME/porter/nwjs/Game.sh ./Game.sh
    ```

- The file structure should look like this:
    ```
    tree -L 1 ./game_dir
    game_dir
    ├── Game.sh <-- Symbolic link
    ├── package.json
    ├── game_pic.webp <-- Image file
    └── www <-- Contain game files

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
    └── www <-- Empty, act as mount point
    ```

- You should an image file inside your `game_dir` so you could easily identify your game.

- This can be done for as many RPG Maker MV/MZ games you have.

## Optional
**Make a backup before proceeding past this point.**

### Update the game libraries:
Acording to [this thread](https://forums.rpgmakerweb.com/index.php?threads/123317), updating `pixi.js` improves performance, though I have not tested this.

- Open in a terminal in this repo.

- Run `./scripts/libs-update.sh`
    ```
    ./scripts/libs-update.sh path/to/game_dir
    ```
    Note: Most RPG Maker MV games should handle just fine being updated to Pixi v4.8, but should you encounter any issues, fallback to Pixi v4.4.
    - Rename `game_dir/www/js/libs/pixi44.js` to `game_dir/www/js/libs/pixi.js`.

### Decrypt the game files
Some games don't encrypt their files.

1. Go to [Java-RPG-Maker-MV-Decrypter](https://gitlab.com/Petschko/Java-RPG-Maker-MV-Decrypter) and follow the instructions.

2. Delete the encrypted files in your game's `www` folder, most of the time, the `audio` and the `img` folders.

    Note: `www/img/system/Loading.png` and `www/img/system/Window.png` (if exist) are not encrypted so don't delete them!

3. Copy the decrypted files to your `game_dir`.

4. Open the `www/data/System.json` file, edit the `hasEncryptedImages` and `hasEncryptedAudio` parameters to `false`.

    The edited parameters should look like this:
    ```
    ..., "hasEncryptedImages": false, "hasEncryptedAudio": false, ...
    ```

### Optimize the image files
Requires the image files to be decrypted. Reduces the image files size.

- Open in a terminal in this repo folder.

- Run `./scripts/optimize.sh`:
    ```
    ./scripts/optimize.sh path/to/game_dir
    ```

Note: This script requires [pngcheck](http://www.libpng.org/pub/png/apps/pngcheck.html), [oxipng](https://github.com/shssoichiro/oxipng), [libwebp](https://chromium.googlesource.com/webm/libwebp) and optionally [parallel](https://www.gnu.org/software/parallel) for multithreading to be installed.

# Credits
- https://github.com/adlerosn/cicpoffs
- https://github.com/nwjs/nw.js
- https://github.com/m5kro/Painless-Porter-CLI
- https://github.com/pixijs/pixijs
- https://github.com/darsain/fpsmeter
- https://github.com/pixijs/tilemap/tree/v4.x
- https://github.com/pieroxy/lz-string
- https://github.com/effekseer/EffekseerForWebGL
- https://github.com/localForage/localForage
