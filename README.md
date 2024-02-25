# RPG-Maker-MV/MZ-Linux-Guide
This guide will advise you on how to manage and play RPG Maker MV/MZ natively on Linux.

Requires [attr](https://savannah.nongnu.org/projects/attr) and [fuse2](https://github.com/libfuse/libfuse) to be installed.

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
    - Edit the `name` parameter to an unique id. Tthis can simply the game's name or `title`.

    - For RPG Maker MZ: Edit the `main` parameter so it point to `www/index.html` (**NOT** `index.html`). 
    
    Example:
    ```
    {
        "name": "0123 Super fun game",
        "main": "www/index.html",
        "chromium-args": "--force-color-profile=srgb",
        "window": {
            "title": "",
            "width": 1024,
            "height": 768,
            "position": "center",
            "icon": "icon/icon.png"
        }
    }
    ```

## Prepare the game engine
1. Download the Stable Linux 64-Bit NORMAL build from [this page](https://nwjs.io/downloads) and extract it to the same directory containing the `game_dir`.

    Note: If you want to debug the game, you can download the SDK build instead. Use F12 to launch DevTools.

2. Rename the extracted folder to `NWJS`.

3. Copy the `NWJS` folder in this repository and paste it over your `NWJS` folder.

Note: When a newer version of the game engine is released, you can repeat the steps to update it.

## Link the game files and engine together
- Copy the `NWJS/Game.sh` file in this repository into your `game_dir`.

- The file structure should look like this:
    ```
    tree -L 2
    .
    ├── game_dir
    │   ├── Game.sh
    │   ├── package.json
    │   ├── game_pic.webp
    │   └── www
    └── NWJS
        ├── chrome_crashpad_handler
        ├── credits.html
        ├── Game.sh
        ├── icudtl.dat
        ├── lib
        ├── locales
        ├── nw
        ├── nw_100_percent.pak
        ├── nw_200_percent.pak
        ├── resources.pak
        ├── swiftshader
        ├── v8_context_snapshot.bin
        └── www
    ```

- You can have a `game_pic` image file inside your `game_dir` so you could easily identify your game.

- This can be done for as many RPG Maker MV/MZ games you have.

## Optional
**Make a backup before proceeding past this point.**

### Update the game libraries:
You can check if a game is MV or MZ by opening `www/js/libs/pixi.js`, MV games use Pixi v4 while MZ games use Pixi v5.

Acording to [this thread](https://forums.rpgmakerweb.com/index.php?threads/123317), updating `pixi.js` improves performance, though I have not tested this.

- MV games: All libraries that can be safely updated are in this repository `js/MV` folder.
    - Simply copy the files in `js/MV` over to your game's `www/js/libs` folder.

    Most games should handle just fine being updated to Pixi v4.8, but should you encounter any issues, fallback to Pixi v4.4.
    - Rename `pixi44.js` to `pixi.js`.

- MZ games: All libraries that can be safely updated are in this repository `js/MZ` folder.
    - Simply copy the files in `js/MZ` over to your game's `www/js/libs` folder.

### Decrypt the game files
1. Go to [Java-RPG-Maker-MV-Decrypter](https://gitlab.com/Petschko/Java-RPG-Maker-MV-Decrypter) and follow the instructions.

2. Delete the encrypted files in your game's `www` folder, most of the time, the `audio` and the `img` folders.

    Note: For unknown reasons, `www/img/system/Loading.png` and `www/img/system/Window.png` (if exist) are usually not encrypted so don't delete them!

3. Copy the decrypted files to the game folder.

4. Open the `www/data/System.json` file, edit the `hasEncryptedImages` and `hasEncryptedAudio` parameters to `false`.

    The edited parameters should look like this:
    ```
    ..., "hasEncryptedImages": false, "hasEncryptedAudio": false, ...
    ```

### Optimize the image files
Requires the image files to be decrypted. Reduces the image files size.

1. Copy the `optimize.sh` file in this repository to your game folder.

2. Open a terminal in your game folder and run the script:

    ```
    ./optimize.sh
    ```

Note: This script requires [pngcheck](http://www.libpng.org/pub/png/apps/pngcheck.html), [oxipng](https://github.com/shssoichiro/oxipng) and [libwebp](https://chromium.googlesource.com/webm/libwebp) to be installed.

# Credits
- [adlerosn](https://github.com/adlerosn/cicpoffs) for the Windows case-insensitive workaround.
- [nwjs](https://github.com/nwjs/nw.js) for the NW.JS engine.
- [m5kro](https://github.com/m5kro/Painless-Porter) for the `Game.sh` file.
- [pixijs](https://github.com/pixijs/pixijs) for the Pixi library.
- [effekseer](https://github.com/effekseer/EffekseerForWebGL) for the Effekseer library.
- [localForage](https://github.com/localForage/localForage) for the localForage library.
