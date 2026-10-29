use crate::casing::transaction::execute_fix_casing;
use crate::core::error::Result;
use crate::core::layout::detect_game_layout;
use crate::core::manifest::read_system_info;
use crate::crypto::extensions::is_encrypted_extension;
use crate::setup::cleaner::clean_developer_junk;
use serde_json::Value;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

pub fn run_scan(game_dir: &Path) -> Result<()> {
    let layout = detect_game_layout(game_dir)?;
    let asset_root = layout.asset_root(game_dir);

    println!("============================================================");
    println!(" RPG Maker MV/MZ Game Audit & Diagnostic Scanner");
    println!("============================================================");
    println!(" Target Directory: {}", game_dir.display());
    println!(
        " Engine Layout:    {} (Asset root: {})",
        layout.name(),
        asset_root.display()
    );

    let sys_info = read_system_info(layout, game_dir)?;
    let display_title = if !sys_info.game_title.is_empty() {
        sys_info.game_title.clone()
    } else {
        game_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string()
    };
    println!(" Game Title:       {}", display_title);

    // Package.json check
    let pkg_file = game_dir.join("package.json");
    if pkg_file.is_file() {
        if let Ok(content) = fs::read_to_string(&pkg_file) {
            if let Ok(val) = serde_json::from_str::<Value>(&content) {
                let pkg_name = val
                    .get("name")
                    .and_then(|v| v.as_str())
                    .unwrap_or("not set");
                let pkg_main = val
                    .get("main")
                    .and_then(|v| v.as_str())
                    .unwrap_or("not set");
                println!(" Package Name:     {}", pkg_name);
                println!(" Package Main:     {}", pkg_main);
            }
        }
    } else {
        println!(
            " Package Name:     MISSING package.json (run 'rpgmaker-helper setup' to configure)"
        );
    }

    let launcher_file = game_dir.join("Game");
    if launcher_file.is_file() {
        println!(" Linux Launcher:   Present ({})", launcher_file.display());
    } else {
        println!(" Linux Launcher:   Not configured (run 'rpgmaker-helper setup' to deploy)");
    }

    // Encryption audit
    let mut encrypted_count = 0;
    let mut png_count = 0;
    let mut ogg_count = 0;

    for entry in WalkDir::new(&asset_root)
        .into_iter()
        .filter_entry(|e| {
            if let Ok(rel) = e.path().strip_prefix(&asset_root) {
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
                if is_encrypted_extension(ext) {
                    encrypted_count += 1;
                }
                if ext.eq_ignore_ascii_case("png") {
                    png_count += 1;
                } else if ext.eq_ignore_ascii_case("ogg") {
                    ogg_count += 1;
                }
            }
        }
    }

    print!(" Encryption:       ");
    if encrypted_count > 0 || sys_info.has_encrypted_images || sys_info.has_encrypted_audio {
        println!(
            "ENCRYPTED ({} encrypted files detected on disk)",
            encrypted_count
        );
        if sys_info.encryption_key.is_some() {
            println!(" Key in System:    Present");
        }
    } else {
        println!("Decrypted (0 encrypted files found)");
    }

    println!(
        " Asset Inventory:  {} PNG images, {} OGG audio files",
        png_count, ogg_count
    );

    // Non-runtime developer junk audit
    let junk_files = clean_developer_junk(game_dir, true)?;
    if !junk_files.is_empty() {
        println!(
            " Developer Junk:   {} files detected (.clip, .bak, .tmx, .trans, or CSP dumps)",
            junk_files.len()
        );
        for j in junk_files.iter().take(5) {
            let rel = j.strip_prefix(game_dir).unwrap_or(j);
            println!("                   - {}", rel.display());
        }
        if junk_files.len() > 5 {
            println!("                   ... and {} more", junk_files.len() - 5);
        }
    } else {
        println!(" Developer Junk:   Clean (0 junk files found)");
    }

    // Font audit: detect unreferenced fonts
    let fonts_dir = asset_root.join("fonts");
    if fonts_dir.is_dir() {
        let mut all_font_paths = Vec::new();
        for entry in WalkDir::new(&fonts_dir).into_iter().filter_map(|e| e.ok()) {
            if entry.file_type().is_file() {
                if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if matches!(ext_lower.as_str(), "ttf" | "otf" | "woff" | "woff2" | "ttc") {
                        all_font_paths.push(entry.into_path());
                    }
                }
            }
        }

        // Collect all reference texts across JSON, CSS, and JS files
        let mut corpus = String::new();
        for scan_root in [
            &asset_root.join("data"),
            &asset_root.join("dataEx"),
            &asset_root.join("css"),
            &fonts_dir,
        ] {
            if scan_root.is_dir() {
                for entry in WalkDir::new(scan_root).into_iter().filter_map(|e| e.ok()) {
                    if entry.file_type().is_file() {
                        if let Ok(c) = fs::read_to_string(entry.path()) {
                            corpus.push_str(&c);
                            corpus.push(' ');
                        }
                    }
                }
            }
        }
        let plugins_file = asset_root.join("js").join("plugins.js");
        if plugins_file.is_file() {
            if let Ok(c) = fs::read_to_string(&plugins_file) {
                corpus.push_str(&c);
            }
        }

        let mut unreferenced_fonts = Vec::new();
        for fp in &all_font_paths {
            let fname = fp.file_name().and_then(|s| s.to_str()).unwrap_or("");
            let stem = fp.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            if !corpus.contains(fname) && !corpus.contains(stem) {
                let size_mb =
                    fs::metadata(fp).map(|m| m.len()).unwrap_or(0) as f64 / (1024.0 * 1024.0);
                unreferenced_fonts.push((fname.to_string(), size_mb));
            }
        }

        if !unreferenced_fonts.is_empty() {
            println!(
                " Fonts Audit:      {} font files found ({} unreferenced, occupying space)",
                all_font_paths.len(),
                unreferenced_fonts.len()
            );
            for (f, sz) in &unreferenced_fonts {
                println!(
                    "  [!] Warning: Potentially unreferenced font: fonts/{} ({:.2} MB)",
                    f, sz
                );
            }
        }
    }

    // Asset Casing Inspection (read-only dry run)
    println!("\n--- Asset Casing Inspection ---");
    let casing_report = execute_fix_casing(&asset_root, true, true)?;
    if casing_report.renamed_list.is_empty() && casing_report.symlink_list.is_empty() {
        println!("  All declared asset references match disk casing (no changes needed).");
    } else {
        for (old_name, new_name) in &casing_report.renamed_list {
            println!("  - {} -> {}", old_name, new_name);
        }
        for (alias_name, target) in &casing_report.symlink_list {
            println!("  - {} -> {} (symlink alias)", alias_name, target);
        }
    }

    println!("============================================================");
    println!(" Scan complete. Recommendations:");
    if encrypted_count > 0 || sys_info.has_encrypted_images || sys_info.has_encrypted_audio {
        println!(
            "  - Run 'rpgmaker-helper decrypt \"{}\"' to decrypt assets.",
            game_dir.display()
        );
    }
    if !launcher_file.is_file() {
        println!(
            "  - Run 'rpgmaker-helper setup \"{}\"' to configure Linux NW.js launcher.",
            game_dir.display()
        );
    }
    if !casing_report.renamed_list.is_empty() || !casing_report.symlink_list.is_empty() {
        println!(
            "  - Run 'rpgmaker-helper fix-casing \"{}\"' to resolve case mismatches.",
            game_dir.display()
        );
    }
    println!(
        "  - Run 'rpgmaker-helper optimize \"{}\"' to compress PNG and OGG assets.",
        game_dir.display()
    );
    println!("============================================================");

    Ok(())
}
