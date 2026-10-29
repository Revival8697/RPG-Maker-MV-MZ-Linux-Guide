use crate::core::error::{Result, RpgError};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameLayout {
    Mv,
    Mz,
}

impl GameLayout {
    pub fn name(&self) -> &'static str {
        match self {
            GameLayout::Mv => "MV",
            GameLayout::Mz => "MZ",
        }
    }

    pub fn asset_root(&self, game_dir: &Path) -> PathBuf {
        match self {
            GameLayout::Mv => game_dir.join("www"),
            GameLayout::Mz => game_dir.to_path_buf(),
        }
    }

    pub fn data_dir(&self, game_dir: &Path) -> PathBuf {
        match self {
            GameLayout::Mv => game_dir.join("www").join("data"),
            GameLayout::Mz => game_dir.join("data"),
        }
    }

    pub fn js_dir(&self, game_dir: &Path) -> PathBuf {
        match self {
            GameLayout::Mv => game_dir.join("www").join("js"),
            GameLayout::Mz => game_dir.join("js"),
        }
    }

    pub fn pixi_path(&self, game_dir: &Path) -> PathBuf {
        match self {
            GameLayout::Mv => game_dir.join("www").join("js").join("libs").join("pixi.js"),
            GameLayout::Mz => game_dir.join("js").join("libs").join("pixi.js"),
        }
    }

    pub fn entry_main(&self) -> &'static str {
        match self {
            GameLayout::Mv => "www/index.html",
            GameLayout::Mz => "index.html",
        }
    }

    pub fn icon_rel(&self) -> &'static str {
        match self {
            GameLayout::Mv => "www/icon/icon.png",
            GameLayout::Mz => "icon/icon.png",
        }
    }
}

pub fn detect_game_layout(game_dir: &Path) -> Result<GameLayout> {
    if game_dir.join("www").join("index.html").is_file() {
        Ok(GameLayout::Mv)
    } else if game_dir.join("index.html").is_file() {
        Ok(GameLayout::Mz)
    } else if game_dir
        .join("www")
        .join("data")
        .join("System.json")
        .is_file()
    {
        Ok(GameLayout::Mv)
    } else if game_dir.join("data").join("System.json").is_file() {
        Ok(GameLayout::Mz)
    } else {
        Err(RpgError::InvalidGameDir(format!(
            "Input path '{}' is not a valid RPG Maker MV or MZ game (could not find index.html or System.json in root or www/)",
            game_dir.display()
        )))
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_detect_mv_layout() {
        let dir = tempdir().unwrap();
        let www = dir.path().join("www");
        fs::create_dir_all(&www).unwrap();
        fs::write(www.join("index.html"), "<html></html>").unwrap();

        let layout = detect_game_layout(dir.path()).unwrap();
        assert_eq!(layout, GameLayout::Mv);
        assert_eq!(layout.asset_root(dir.path()), www);
    }

    #[test]
    fn test_detect_mz_layout() {
        let dir = tempdir().unwrap();
        fs::write(dir.path().join("index.html"), "<html></html>").unwrap();

        let layout = detect_game_layout(dir.path()).unwrap();
        assert_eq!(layout, GameLayout::Mz);
        assert_eq!(layout.asset_root(dir.path()), dir.path());
    }
}
