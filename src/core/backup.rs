use crate::core::error::{Result, RpgError};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub fn resolve_backup_dir(
    game_dir: &Path,
    category: &str,
    custom_backup: Option<&Path>,
) -> std::path::PathBuf {
    if let Some(custom) = custom_backup {
        custom.to_path_buf()
    } else {
        game_dir.join("backup").join(category)
    }
}

pub fn safe_backup(source: &Path, backup_dir: &Path) -> Result<()> {
    if backup_dir.exists() {
        println!(
            "Backup already exists at {}; skipping backup creation.",
            backup_dir.display()
        );
        return Ok(());
    }

    println!("Creating safety backup at {}...", backup_dir.display());
    copy_dir_all(source, backup_dir)?;
    Ok(())
}

pub fn copy_dir_all(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;
    for entry in WalkDir::new(src)
        .into_iter()
        .filter_entry(|e| {
            let p = e.path();
            if p == src {
                return true;
            }
            if p.starts_with(dst) {
                return false;
            }
            if let Ok(rel) = p.strip_prefix(src) {
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
        let path = entry.path();
        if path == src {
            continue;
        }
        let rel_path = match path.strip_prefix(src) {
            Ok(p) => p,
            Err(_) => continue,
        };
        let target = dst.join(rel_path);

        if entry.file_type().is_dir() {
            fs::create_dir_all(&target)?;
        } else if entry.file_type().is_file() {
            if let Some(parent) = target.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::copy(path, &target).map_err(|e| {
                RpgError::Io(std::io::Error::new(
                    e.kind(),
                    format!(
                        "Failed copying {} to {}: {}",
                        path.display(),
                        target.display(),
                        e
                    ),
                ))
            })?;
        } else if entry.path_is_symlink() {
            #[cfg(unix)]
            {
                if let Ok(link_target) = fs::read_link(path) {
                    let _ = fs::remove_file(&target);
                    let _ = std::os::unix::fs::symlink(link_target, &target);
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_resolve_backup_dir_visible() {
        let game_dir = Path::new("/path/to/game");
        let backup_dir = resolve_backup_dir(game_dir, "decrypt", None);
        assert_eq!(backup_dir, Path::new("/path/to/game/backup/decrypt"));
        assert!(!backup_dir.to_string_lossy().contains(".backup"));
    }

    #[test]
    fn test_resolve_backup_dir_custom() {
        let game_dir = Path::new("/path/to/game");
        let custom = Path::new("/custom/location");
        let backup_dir = resolve_backup_dir(game_dir, "decrypt", Some(custom));
        assert_eq!(backup_dir, custom);
    }

    #[test]
    fn test_safe_backup_creation_and_skip() {
        let temp = tempdir().unwrap();
        let game_dir = temp.path();
        let src_file = game_dir.join("test.txt");
        fs::write(&src_file, "content").unwrap();

        let backup_dir = resolve_backup_dir(game_dir, "decrypt", None);
        assert_eq!(backup_dir, game_dir.join("backup").join("decrypt"));

        safe_backup(game_dir, &backup_dir).unwrap();
        assert!(backup_dir.join("test.txt").is_file());
        assert_eq!(
            fs::read_to_string(backup_dir.join("test.txt")).unwrap(),
            "content"
        );

        // Ensure backup directory itself is not nested inside itself
        assert!(!backup_dir.join("backup").exists());

        // Second call should skip safely without error
        safe_backup(game_dir, &backup_dir).unwrap();
    }
}
