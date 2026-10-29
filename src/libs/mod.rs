use crate::core::backup::{resolve_backup_dir, safe_backup};
use crate::core::error::{Result, RpgError};
use crate::core::layout::{detect_game_layout, GameLayout};
use crate::embedded::{MV_CORE_FILES, MV_PIXI_JS, MZ_PIXI_JS};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UpdateMode {
    PixiOnly,
    Full,
}

pub fn run_update_libs(game_dir: &Path, mode: UpdateMode, create_backup: bool) -> Result<()> {
    let layout = detect_game_layout(game_dir)?;
    let pixi_path = layout.pixi_path(game_dir);

    if !pixi_path.is_file() {
        return Err(RpgError::InvalidGameDir(format!(
            "Input path is not a valid RPG Maker game (pixi.js not found at '{}').",
            pixi_path.display()
        )));
    }

    let js_dir = layout.js_dir(game_dir);
    if create_backup {
        let backup_dir = resolve_backup_dir(game_dir, "libs", None);
        safe_backup(&js_dir, &backup_dir)?;
    }

    let pixi_content = fs::read_to_string(&pixi_path).map_err(RpgError::Io)?;

    let is_v4 = if pixi_content.contains("pixi.js - v4.") {
        true
    } else if pixi_content.contains("pixi.js - v5.") {
        false
    } else {
        println!(
            "  [!] Notice: pixi.js does not contain a standard version banner; falling back to layout detection ({}).",
            layout.name()
        );
        matches!(layout, GameLayout::Mv)
    };

    if is_v4 {
        println!("Detected RPG Maker MV (Pixi.js v4).");
        if mode == UpdateMode::Full {
            println!(
                "Full update: Deploying Pixi.js v4.8.9 and core engine scripts to {}/...",
                js_dir.display()
            );
            let libs_dir = js_dir.join("libs");
            fs::create_dir_all(&libs_dir)?;
            fs::write(libs_dir.join("pixi.js"), MV_PIXI_JS)?;

            for (file_name, data) in MV_CORE_FILES {
                fs::write(js_dir.join(file_name), data)?;
            }
        } else {
            println!("Safe update: Updating Pixi.js to v4.8.9 (--pixi-only mode)...");
            let dst_pixi = js_dir.join("libs").join("pixi.js");
            fs::create_dir_all(dst_pixi.parent().unwrap())?;
            fs::write(&dst_pixi, MV_PIXI_JS)?;
        }
    } else {
        println!("Detected RPG Maker MZ (Pixi.js v5).");
        println!("Updating Pixi.js to v5.3.12...");
        let dst_pixi = js_dir.join("libs").join("pixi.js");
        fs::create_dir_all(dst_pixi.parent().unwrap())?;
        fs::write(&dst_pixi, MZ_PIXI_JS)?;
    }

    println!("Library update finished successfully!");
    Ok(())
}
