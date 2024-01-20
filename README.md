# RPG-Maker-MV/MZ-Update-Guide
A guide for users to update their RPG Maker MV/MZ engine and plugins.

- RPG Maker MV/MZ use the [NW.js](https://nwjs.io), but the one included is always out of date. The version that RPG Maker MV use (as of writing) is NW.js [v0.29.0](https://nwjs.io/blog/v0.29.0) released in 07 March 2018.

## That's old! How do I update it?
### Check if your game is a RPG Maker MV/MZ Game
See if your game folder contain a file named `package.json`. Rename it to `package.json.backup` and launch the Game.exe, a window listing NW.js version should show up.

Remember to rename `package.json.backup` to `package.json` after testing.

### Download the latest version of NW.js
Go to [the official download page](https://nwjs.io/downloads) and download the latest `Stable` `64-bit` `Normal` release for your OS. [A mirror][https://registry.npmmirror.com/binary.html?path=nwjs] is also avaiable.

Extract the downloaded file with a file archiver like [7-zip](https://7-zip.org).

Go inside the extracted folder and rename `nw.exe` to `Game.exe`. For Linux users, rename `nw` to `Game` and make it executable: `chmod +x Game`.

Note:
- The latest Windows version that [Wine-GE v8.25](https://github.com/GloriousEggroll/wine-ge-custom/releases/tag/GE-Proton8-25) can run without issues is [v0.68.1](https://dl.nwjs.io/v0.68.1).
- The latest version that Windows 7 support is [v0.72.0](https://dl.nwjs.io/v0.72.0) - based on Chromium 109, [source](https://archive.is/dCOEx).

### Copy the games resources

Copy the `www` folder and the `package.json` file from your game folder to the extracted `NW.js` folder.


Open `package.json` (NOT the one inside the `www` folder) using a IDE like [Notepad++](https://notepad-plus-plus.org)and modify `"name":""` to `"name":"something_unique"`. Remember to save the file!

And you are done!

Note:
- If you don't have a `www` folder, you can make one and copy game resources to it, see below for examples of `www` folder. This is game dependent so you would have to figure it out yourself or overwrite the Game folder with the extracted NW.js folder.

- The `package.json` file inside the `www` folder seems to serve no purpose?

![Example 1](/img/example_www_0.png "www folder of VHMV")

![Example 1](/img/example_www_1.png "www folder of RJ317690")

## Update the game plugins

**Make a backup of them first.**

The plugins can be found in `/www/js/libs`.

The most notable one that can be updated is `pixi.js`.

## Pixi.js

Open `/www/js/libs/pixi.js` in your game folder. Its version is written on the second line.

Out of 16 pixi v4.x games I have:
- 14 can be updated to [v4.8.9](https://pixijs.download/v4.8.9/pixi.min.js) (16 Jan 2020).
- 2 can be updated to [v4.4.5](https://pixijs.download/v4.4.5/pixi.min.js) (04 Nov 2020).
- 1 have a modifed variant of v4.0.3 (Oct 12 2016) that I'm unable to update.

Out of 2 pixi v5.x games I have:
- 2 can be updated to [v5.3.12](https://pixijs.download/v5.3.12/pixi.min.js) (23 Mar 2022).

Note:
- Updating from pixi v4 to v5 require modifying game codes.
- `.min.js` is functionally the same as `.js` for end user with reduced file size. Just rename the file to not break compatibility.

## Others
- fpsmeter.js: All of my games already have at [its latest version](https://github.com/darsain/fpsmeter/blob/master/dist/fpsmeter.min.js).
- iphone-inline-video.browser.js: Serve no purpose? [Link to the latest version](https://github.com/fregante/iphone-inline-video/blob/master/dist/iphone-inline-video.min.js).
- lz-string.js: [Link to the latest version](https://github.com/pieroxy/lz-string/blob/cf06e9a0e61daa8b120a474bbc80666f959ff7d4/libs/lz-string.min.js).
- pixi-picture.js: Don't touch it. Just don't.
- pixi-tilemap.js: All pixi v4 games aside from the v4.0.3 one can be update to [the latest version](https://github.com/pixijs/tilemap/blob/v4.x/dist/pixi-tilemap.js).
- effekseer.min.js and effekseer.wasm: [The latest version can be extracted from this ZIP](https://github.com/effekseer/EffekseerForWebGL/releases/latest).
- pako.min.js: [Link to the latest version](https://github.com/nodeca/pako/blob/master/dist/pako.min.js).
- pixi-sound.js: [Link to the latest version](https://github.com/pixijs/sound/releases/tag/v3.0.5).
- three.min.js: [Link to the latest version](https://github.com/mrdoob/three.js/blob/2ab27ea33ef2c991558e392d4f476ac08975be0d/build/three.min.js) (My Firefox browser can't load this page).
- xlsx.core.min.js: [Link to the latest version](https://github.com/SheetJS/sheetjs/blob/github/dist/xlsx.core.min.js).
