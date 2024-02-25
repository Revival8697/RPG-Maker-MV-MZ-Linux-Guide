# RPG-Maker-MV/MZ-Linux-Guide
This guide will advise you on how to manage and play RPG Maker MV/MZ natively on Linux.

## Prepare the game files
1. Create a new folder, this will be where you store your game. I will name it `game_dir` in this guide.

2. Copy the `www` folder and the `package.json` file to the `game_dir`.

    Note: If you don't have a `www` folder (usually with RPG Maker MZ), you have to create it yourself:

- Create the `www` folder.

- Move the game directories and files into the `www` directory.
    - Game folders examples: `audio, css, data, dataEx, effects, fonts, icon, img, js, save`. 
    
    - Game files examples: `index.html, package.json`.

    This is game dependent, not all games have the same folders and files.

- Copy `game_dir/www/package.json` to `game_dir/package.json`.

- Open `game_dir/package.json`.
    - Edit the `name` parameter to an unique id (could be be the game name or `title`).

    - Edit the `main` parameter so it point to `www/index.html` (**NOT** `index.html`). 
    
    Example for `package.json`:
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

2. Rename the extracted folder to `NWJS`.

3. Copy the `NWJS` folder in this repository and paste it over your `NWJS` folder.

4. Create an empty `www` folder inside `NWJS`.

Note: When a newer version of the game engine is released, you can repeat the steps to update it.

## Linking the game to the engine.
- Copy the `NWJS/Game.sh` file into your `game_dir`.

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
**Make a backup before this point.**

### Update the game libraries:
You can check if a game is MV or MZ by opening `www/js/libs/pixi.js`, MV games use Pixi v4 while MZ games use Pixi v5.

Acording to [this thread](https://forums.rpgmakerweb.com/index.php?threads/123317), updating `pixi.js` improves performance, though I have not tested this.

- MV games: All libraries that can be safely updated are in this repository `js/MV` folder.
    - Simply copy the files in `js/MV` over to your game's `www/js/libs` folder.

    - Rename `pixi48.js` to `pixi.js`.

    Most games should handle just fine being updated to Pixi v4.8, but should you encounter any issues, fallback to Pixi v4.4.

- MZ games: All libraries that can be safely updated are in this repository `js/MZ` folder.
    - Simply copy the files in `js/MZ` over to your game's `www/js/libs` folder.

### Games files decryption
1. Go to [Java-RPG-Maker-MV-Decrypter](https://gitlab.com/Petschko/Java-RPG-Maker-MV-Decrypter) and follow the instructions.

2. Delete the encrypted files in your game's `www` folder, most of the time, the `audio` and the `img` folders.

    Note: For unknown reasons, `www/img/system/Loading.png` and `www/img/system/Window.png` (if exist) are usually not encrypted so don't delete them!

3. Copy the decrypted files to the game folder.

4. Edit the `www/data/System.json` file so the game see decrypted files:
    - Change the `hasEncryptedImages` and `hasEncryptedAudio` parameters to `false`.

    - It should looks like this: `"hasEncryptedImages":false,"hasEncryptedAudio":false,"encryptionKey":"random_number"`

### Convert game images to webp
This requires the image files to be decrypted. This reduces the images size.

1. Open a terminal emulator in the `www/img` folder.

2. Run `find . -name "*.png" | parallel -eta cwebp -lossless {} -o {}`.

    Note:
    - This requires [parallel](https://www.gnu.org/software/parallel) and [libwebp](https://chromium.googlesource.com/webm/libwebp).

    - The `www/icon/icon.png` file must be a PNG. You can optimize this file by running `oxipng --opt max --strip all icon.png`, this requires [oxipng](https://github.com/shssoichiro/oxipng).
    This command can also be used with the `--recursive` option to optimize to game images if the game can't read webp images.

# Credits
- [nwjs](https://github.com/nwjs/nw.js) for the NW.JS engine.
- [m5kro](https://github.com/m5kro/Painless-Porter) for the `Game.sh` file.
- [pixijs](https://github.com/pixijs/pixijs) for the Pixi engine.
- [effekseer](https://github.com/effekseer/EffekseerForWebGL) for the Effekseer library.
- [localForage](https://github.com/localForage/localForage) for the localForage library.
