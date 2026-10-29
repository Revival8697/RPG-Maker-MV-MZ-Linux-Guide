pub mod audio;
pub mod image;

use crate::core::backup::{resolve_backup_dir, safe_backup};
use crate::core::error::{Result, RpgError};
use crate::core::layout::detect_game_layout;
use crate::core::manifest::read_system_info;
use rayon::prelude::*;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use walkdir::WalkDir;

pub struct OptimizeOptions {
    pub run_images: bool,
    pub run_audio: bool,
    pub create_backup: bool,
    pub opt_level: String,
    pub auto_accept: bool,
    pub threads: Option<usize>,
}

impl Default for OptimizeOptions {
    fn default() -> Self {
        Self {
            run_images: true,
            run_audio: true,
            create_backup: true,
            opt_level: "2".to_string(),
            auto_accept: false,
            threads: None,
        }
    }
}

pub fn run_optimize(game_dir: &Path, options: &OptimizeOptions) -> Result<()> {
    let layout = detect_game_layout(game_dir)?;
    let target_root = layout.asset_root(game_dir);

    // Check if assets are encrypted
    let sys_info = read_system_info(layout, game_dir)?;
    if sys_info.has_encrypted_images || sys_info.has_encrypted_audio {
        println!("  [!] Warning: Game assets are marked as encrypted in System.json.");
        println!("      Consider running 'rpgmaker-helper decrypt' prior to optimization.");
    }

    if options.create_backup {
        let backup_dir = resolve_backup_dir(game_dir, "optimize", None);
        safe_backup(&target_root, &backup_dir)?;
    }

    // Build scoped Rayon threadpool if requested
    let pool = if let Some(t) = options.threads {
        if t > 0 {
            Some(
                rayon::ThreadPoolBuilder::new()
                    .num_threads(t)
                    .build()
                    .map_err(|e| RpgError::General(format!("Failed to build thread pool: {e}")))?,
            )
        } else {
            None
        }
    } else {
        None
    };

    let run_in_pool = |action: &(dyn Fn() + Sync)| {
        if let Some(ref p) = pool {
            p.install(action);
        } else {
            action();
        }
    };

    // 1. PNG Optimization
    if options.run_images {
        let preset_num: u8 = match options.opt_level.as_str() {
            "max" => 6,
            s => s.parse().unwrap_or(2).clamp(1, 6),
        };

        println!(
            "Optimizing PNG images using native oxipng (level {})...",
            preset_num
        );

        let icon_path = target_root.join("icon").join("icon.png");
        let mut icon_saved = 0u64;
        if icon_path.is_file() {
            if let Ok(res) = image::optimize_png_file(&icon_path, 6) {
                if res.optimized {
                    icon_saved = res.original_size.saturating_sub(res.new_size);
                }
            }
        }

        let mut png_files: Vec<PathBuf> = Vec::new();
        for entry in WalkDir::new(&target_root)
            .into_iter()
            .filter_entry(|e| {
                if let Ok(rel) = e.path().strip_prefix(&target_root) {
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
                let p = entry.into_path();
                // Avoid redundant re-optimization of icon.png
                if p == icon_path {
                    continue;
                }
                if let Some(ext) = p.extension().and_then(|s| s.to_str()) {
                    if ext.eq_ignore_ascii_case("png") {
                        png_files.push(p);
                    }
                }
            }
        }

        let total_pngs = png_files.len();
        let optimized_count = AtomicUsize::new(if icon_saved > 0 { 1 } else { 0 });
        let bytes_saved = AtomicU64::new(icon_saved);
        let completed = AtomicUsize::new(0);

        run_in_pool(&|| {
            png_files.par_iter().for_each(|path| {
                if let Ok(res) = image::optimize_png_file(path, preset_num) {
                    if res.optimized {
                        optimized_count.fetch_add(1, Ordering::Relaxed);
                        let diff = res.original_size.saturating_sub(res.new_size);
                        bytes_saved.fetch_add(diff, Ordering::Relaxed);
                    }
                }
                let done = completed.fetch_add(1, Ordering::Relaxed) + 1;
                if done % 20 == 0 || done == total_pngs {
                    let pct = (done as f64 / total_pngs.max(1) as f64) * 100.0;
                    let saved_mb = bytes_saved.load(Ordering::Relaxed) as f64 / (1024.0 * 1024.0);
                    print!(
                        "\r  [{}/{}] {:.1}% (Saved: {:.2} MB)    ",
                        done, total_pngs, pct, saved_mb
                    );
                    let _ = std::io::stdout().flush();
                }
            });
        });
        println!();

        let final_saved_mb = bytes_saved.load(Ordering::Relaxed) as f64 / (1024.0 * 1024.0);
        println!(
            "Processed {} PNG images ({} reduced, {:.2} MB saved).",
            total_pngs + if icon_path.is_file() { 1 } else { 0 },
            optimized_count.load(Ordering::Relaxed),
            final_saved_mb
        );
    }

    // 2. Audio Optimization (Pure Rust OGG remuxer)
    if options.run_audio {
        println!("Optimizing OGG audio losslessly using native in-process remuxer...");

        let mut ogg_files: Vec<PathBuf> = Vec::new();
        for entry in WalkDir::new(&target_root)
            .into_iter()
            .filter_entry(|e| {
                if let Ok(rel) = e.path().strip_prefix(&target_root) {
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
                    if ext.eq_ignore_ascii_case("ogg") {
                        ogg_files.push(entry.into_path());
                    }
                }
            }
        }

        let total_oggs = ogg_files.len();
        let stripped_count = AtomicUsize::new(0);
        let audio_saved = AtomicU64::new(0);
        let completed = AtomicUsize::new(0);

        run_in_pool(&|| {
            ogg_files.par_iter().for_each(|file| {
                let pre_len = fs::metadata(file).map(|m| m.len()).unwrap_or(0);
                if let Ok(true) = audio::optimize_ogg_file(file) {
                    stripped_count.fetch_add(1, Ordering::Relaxed);
                    let post_len = fs::metadata(file).map(|m| m.len()).unwrap_or(pre_len);
                    if post_len < pre_len {
                        audio_saved.fetch_add(pre_len - post_len, Ordering::Relaxed);
                    }
                }
                let done = completed.fetch_add(1, Ordering::Relaxed) + 1;
                if done % 5 == 0 || done == total_oggs {
                    let pct = (done as f64 / total_oggs.max(1) as f64) * 100.0;
                    let saved_mb = audio_saved.load(Ordering::Relaxed) as f64 / (1024.0 * 1024.0);
                    print!(
                        "\r  [{}/{}] {:.1}% (Saved: {:.2} MB)    ",
                        done, total_oggs, pct, saved_mb
                    );
                    let _ = std::io::stdout().flush();
                }
            });
        });
        println!();

        let final_audio_mb = audio_saved.load(Ordering::Relaxed) as f64 / (1024.0 * 1024.0);
        println!(
            "Processed {} OGG audio files ({} stripped of bloat/repacked, {:.2} MB saved).",
            total_oggs,
            stripped_count.load(Ordering::Relaxed),
            final_audio_mb
        );
    }

    println!("Optimization finished successfully!");
    Ok(())
}
