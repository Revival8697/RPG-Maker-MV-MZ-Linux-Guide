use crate::core::error::Result;
use std::fs;
use std::path::Path;

pub fn purge_windows_binaries(game_dir: &Path) -> Result<usize> {
    println!(
        "Cleaning up Windows-specific NW.js runtime binaries in {}...",
        game_dir.display()
    );
    let mut purged_count = 0;

    let runtime_files = [
        "credits.html",
        "debug.log",
        "icudtl.dat",
        "natives_blob.bin",
        "snapshot_blob.bin",
        "v8_context_snapshot.bin",
        "Game.exe",
        "notification_helper.exe",
        "d3dcompiler_47.dll",
        "ffmpeg.dll",
        "libEGL.dll",
        "libGLESv2.dll",
        "node.dll",
        "nw.dll",
        "nw_elf.dll",
        "nw_100_percent.pak",
        "nw_200_percent.pak",
        "resources.pak",
    ];

    for name in &runtime_files {
        let p = game_dir.join(name);
        if p.is_file() && fs::remove_file(&p).is_ok() {
            purged_count += 1;
        }
    }

    let runtime_dirs = ["locales", "swiftshader", "captures"];
    for name in &runtime_dirs {
        let p = game_dir.join(name);
        if p.is_dir() && fs::remove_dir_all(&p).is_ok() {
            purged_count += 1;
        }
    }

    // Scan root directory specifically for any leftover *.exe, *.dll, *.pak
    if let Ok(entries) = fs::read_dir(game_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() {
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if matches!(ext_lower.as_str(), "exe" | "dll" | "pak")
                        && fs::remove_file(&path).is_ok()
                    {
                        purged_count += 1;
                    }
                }
            }
        }
    }

    Ok(purged_count)
}

/// Identifies and purges non-runtime editor artifacts (.clip, .psd, .bak, .tmx, .trans, CSFCHUNK files)
pub fn clean_developer_junk(game_dir: &Path, dry_run: bool) -> Result<Vec<std::path::PathBuf>> {
    let mut junk_paths = Vec::new();

    for entry in walkdir::WalkDir::new(game_dir)
        .into_iter()
        .filter_entry(|e| {
            if let Ok(rel) = e.path().strip_prefix(game_dir) {
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
        if !entry.file_type().is_file() || entry.path_is_symlink() {
            continue;
        }

        let p = entry.path();
        let mut is_junk = false;

        if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
            let ext_lower = ext.to_lowercase();
            if matches!(ext_lower.as_str(), "clip" | "psd" | "bak" | "tmx" | "trans") {
                is_junk = true;
            }
        } else {
            // Extensionless file: check for CSFCHUNK header (Clip Studio Paint chunk dump)
            if let Ok(file) = fs::File::open(p) {
                use std::io::Read;
                let mut head = [0u8; 8];
                let mut reader = std::io::BufReader::new(file);
                if reader.read_exact(&mut head).is_ok() && &head == b"CSFCHUNK" {
                    is_junk = true;
                }
            }
        }

        if is_junk {
            junk_paths.push(p.to_path_buf());
            if !dry_run {
                let _ = fs::remove_file(p);
            }
        }
    }

    Ok(junk_paths)
}
