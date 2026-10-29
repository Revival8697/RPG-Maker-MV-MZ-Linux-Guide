//! Project discovery utilities for locating System.json, project files, and encrypted assets.

use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::crypto::extensions::is_encrypted_image;

/// Known relative paths to the System.json file in RPG Maker projects.
const SYSTEM_JSON_PATHS: &[&str] = &[
    "data/System.json",
    "www/data/System.json",
    "data/system.json",
    "www/data/system.json",
];

/// Known project marker files.
const PROJECT_MARKER_FILES: &[&str] = &[
    "Game.rpgproject",
    "game.rmmzproject",
    "package.json",
    "Game.exe",
];

/// Attempts to locate `System.json` within the given project directory.
pub fn find_system_json(project_dir: &Path) -> Option<PathBuf> {
    // 1. Try known standard locations first
    for rel in SYSTEM_JSON_PATHS {
        let candidate = project_dir.join(rel);
        if candidate.is_file() {
            return Some(candidate);
        }
    }

    // 2. Fall back to recursive search up to depth 4
    for entry in WalkDir::new(project_dir)
        .max_depth(4)
        .into_iter()
        .filter_entry(|e| {
            if let Ok(rel) = e.path().strip_prefix(project_dir) {
                if rel
                    .components()
                    .any(|c| c.as_os_str() == "backup" || c.as_os_str() == ".backup")
                {
                    return false;
                }
            }
            true
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(name) = entry.file_name().to_str() {
                if name.eq_ignore_ascii_case("System.json") {
                    return Some(entry.into_path());
                }
            }
        }
    }

    None
}

/// Locates the first encrypted PNG image in the project directory.
/// Useful for deriving the encryption key when `System.json` is missing or obfuscated.
pub fn find_first_encrypted_image(project_dir: &Path) -> Option<PathBuf> {
    for entry in WalkDir::new(project_dir)
        .into_iter()
        .filter_entry(|e| {
            if let Ok(rel) = e.path().strip_prefix(project_dir) {
                if rel
                    .components()
                    .any(|c| c.as_os_str() == "backup" || c.as_os_str() == ".backup")
                {
                    return false;
                }
            }
            true
        })
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
                if is_encrypted_image(ext) {
                    return Some(entry.into_path());
                }
            }
        }
    }

    None
}

/// Checks whether a given directory appears to be a valid RPG Maker project directory.
pub fn verify_rpg_dir(dir: &Path) -> bool {
    if !dir.is_dir() {
        return false;
    }

    // Check for any marker files
    for marker in PROJECT_MARKER_FILES {
        if dir.join(marker).exists() {
            return true;
        }
    }

    // Check for standard folders
    if dir.join("data").is_dir() || dir.join("www").is_dir() {
        return true;
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_find_system_json() {
        let dir = tempdir().unwrap();
        let data_dir = dir.path().join("www").join("data");
        std::fs::create_dir_all(&data_dir).unwrap();
        let sys_file = data_dir.join("System.json");
        std::fs::write(&sys_file, b"{}").unwrap();

        let found = find_system_json(dir.path()).unwrap();
        assert_eq!(found, sys_file);
    }
}
