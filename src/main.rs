use clap::Parser;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::exit;
use walkdir::WalkDir;

use rpgmaker_core::casing::transaction::execute_fix_casing;
use rpgmaker_core::cli::{Cli, Commands};
use rpgmaker_core::core::backup::{resolve_backup_dir, safe_backup};
use rpgmaker_core::core::error::Result;
use rpgmaker_core::core::layout::detect_game_layout;
use rpgmaker_core::core::manifest::{read_system_info, update_system_encryption_flags};
use rpgmaker_core::crypto::{
    decrypted_path, find_first_encrypted_image, is_encrypted_extension, Decrypter, EncryptionKey,
    FileProcessor, ProcessorOptions,
};
use rpgmaker_core::libs::{run_update_libs, UpdateMode};
use rpgmaker_core::optimizer::{run_optimize, OptimizeOptions};
use rpgmaker_core::pipeline::{run_all_pipeline, AllPipelineOptions};
use rpgmaker_core::scan::run_scan;
use rpgmaker_core::setup::cleaner::clean_developer_junk;
use rpgmaker_core::setup::{run_setup, SetupOptions};

fn main() {
    let cli = Cli::parse();

    let result: Result<()> = (|| match cli.command {
        Commands::Scan { game_dir } => {
            let abs_dir = resolve_dir(&game_dir);
            run_scan(&abs_dir)
        }

        Commands::Setup {
            game_dir,
            yes,
            id,
            install_desktop,
            clean_junk,
        } => {
            let abs_dir = resolve_dir(&game_dir);
            let opts = SetupOptions {
                non_interactive: yes,
                custom_id: id,
                install_desktop,
                clean_junk,
            };
            run_setup(&abs_dir, opts)
        }

        Commands::FixCasing {
            game_dir,
            dry_run,
            no_symlinks,
        } => {
            let abs_dir = resolve_dir(&game_dir);
            let layout = detect_game_layout(&abs_dir)?;
            let asset_root = layout.asset_root(&abs_dir);

            println!("============================================================");
            println!(" RPG Maker MV/MZ Asset Case-Sensitivity Fixer");
            println!("============================================================");
            println!("Target directory: {}", abs_dir.display());
            println!("Asset root:       {}", asset_root.display());
            if dry_run {
                println!("Mode:             DRY-RUN (No files will be modified)");
            } else {
                println!("Mode:             EXECUTION (Two-step safe renaming enabled)");
            }
            println!("============================================================");

            let report = execute_fix_casing(&asset_root, dry_run, !no_symlinks)?;
            println!(
                "\nModified Files List ({} files renamed):",
                report.files_renamed
            );
            for (old_name, new_name) in &report.renamed_list {
                println!("  - {} ==> {}", old_name, new_name);
            }
            if !no_symlinks && !report.symlink_list.is_empty() {
                println!(
                    "\nCreated Symlinks List ({} aliases):",
                    report.symlinks_created
                );
                for (alias_name, target) in &report.symlink_list {
                    println!("  - {} --> {}", alias_name, target);
                }
            }
            println!("============================================================");
            if report.verification_passed {
                println!("  [✓] All asset references match disk casing perfectly.");
            } else {
                eprintln!(
                    "  [✗] Verification failed with {} errors.",
                    report.verification_errors
                );
            }
            println!("Done.");
            Ok(())
        }

        Commands::Decrypt {
            target,
            key,
            flag_key,
            backup,
            keep_encrypted,
            output,
            in_place,
            restore,
            threads,
        } => {
            let abs_target = resolve_dir(&target);
            let chosen_key = flag_key.or(key);
            execute_decrypt(
                &abs_target,
                chosen_key.as_deref(),
                backup,
                keep_encrypted,
                output,
                in_place,
                restore,
                threads,
            )
        }

        Commands::UpdateLibs {
            game_dir,
            pixi_only: _,
            full,
            no_backup,
        } => {
            let abs_dir = resolve_dir(&game_dir);
            let mode = if full {
                UpdateMode::Full
            } else {
                UpdateMode::PixiOnly
            };
            run_update_libs(&abs_dir, mode, !no_backup)
        }

        Commands::Optimize {
            game_dir,
            images_only,
            audio_only,
            no_backup,
            level,
            yes,
            threads,
        } => {
            let abs_dir = resolve_dir(&game_dir);
            let opts = OptimizeOptions {
                run_images: !audio_only,
                run_audio: !images_only,
                create_backup: !no_backup,
                opt_level: level,
                auto_accept: yes,
                threads,
            };
            run_optimize(&abs_dir, &opts)
        }

        Commands::CleanJunk { game_dir, dry_run } => {
            let abs_dir = resolve_dir(&game_dir);
            let junk = clean_developer_junk(&abs_dir, dry_run)?;
            if dry_run {
                println!("Dry-run: {} developer junk files found:", junk.len());
                for f in &junk {
                    let rel = f.strip_prefix(&abs_dir).unwrap_or(f);
                    println!("  - {}", rel.display());
                }
            } else {
                println!("Purged {} developer junk files.", junk.len());
            }
            Ok(())
        }

        Commands::All {
            game_dir,
            yes,
            id,
            full,
            no_backup,
            dry_run,
            key,
            keep_encrypted,
            level,
            threads,
            clean_junk,
            images_only,
            audio_only,
        } => {
            let abs_dir = resolve_dir(&game_dir);
            let pipeline_opts = AllPipelineOptions {
                non_interactive: yes,
                custom_id: id,
                full_libs: full,
                no_backup,
                dry_run_casing: dry_run,
                custom_key: key,
                keep_encrypted,
                opt_level: level,
                images_only,
                audio_only,
                threads,
                clean_junk,
            };
            let decrypt_cb = |dir: &Path, k: Option<&str>, bk: bool, keep: bool| {
                execute_decrypt(dir, k, bk, keep, None, true, false, None)
            };
            run_all_pipeline(&abs_dir, pipeline_opts, &decrypt_cb)
        }
    })();

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        exit(1);
    }
}

fn resolve_dir(input: &Path) -> PathBuf {
    fs::canonicalize(input).unwrap_or_else(|_| input.to_path_buf())
}

fn detect_key(target: &Path) -> Result<EncryptionKey> {
    if let Ok(layout) = detect_game_layout(target) {
        if let Ok(sys_info) = read_system_info(layout, target) {
            if let Some(k_str) = sys_info.encryption_key {
                if let Ok(key) = EncryptionKey::from_hex(&k_str) {
                    return Ok(key);
                }
            }
        }
    }

    // Try deriving from encrypted PNG
    let png_candidate = if target.is_file() {
        Some(target.to_path_buf())
    } else {
        find_first_encrypted_image(target)
    };

    if let Some(path) = png_candidate {
        EncryptionKey::derive_from_png_file(&path, 16).map_err(|e| {
            rpgmaker_core::core::error::RpgError::Crypto(format!(
                "Failed deriving key from PNG {}: {}",
                path.display(),
                e
            ))
        })
    } else {
        Err(rpgmaker_core::core::error::RpgError::Crypto(format!(
            "Could not locate System.json or encrypted PNG image in '{}'",
            target.display()
        )))
    }
}

#[allow(clippy::too_many_arguments)]
fn execute_decrypt(
    target: &Path,
    key_str: Option<&str>,
    backup: bool,
    keep_encrypted: bool,
    output: Option<PathBuf>,
    in_place: bool,
    restore: bool,
    threads: Option<usize>,
) -> Result<()> {
    if backup && target.is_dir() {
        let backup_dir = resolve_backup_dir(target, "decrypt", None);
        safe_backup(target, &backup_dir)?;
    }

    let encryption_key = if restore {
        None
    } else if let Some(k) = key_str {
        Some(EncryptionKey::from_hex(k).map_err(|e| {
            rpgmaker_core::core::error::RpgError::Crypto(format!("Invalid hex key: {}", e))
        })?)
    } else {
        println!("Auto-detecting encryption key...");
        match detect_key(target) {
            Ok(k) => {
                println!("Encryption key detected successfully.");
                Some(k)
            }
            Err(e) => {
                return Err(rpgmaker_core::core::error::RpgError::Crypto(format!(
                    "Could not auto-detect encryption key: {}\nPlease specify the key manually via -k/--key.",
                    e
                )));
            }
        }
    };

    let engine = Decrypter::new(encryption_key);

    let mut proc_opts = ProcessorOptions {
        output_dir: output,
        in_place: in_place || target.is_dir(),
        overwrite: true,
        restore_images: restore,
        worker_threads: threads,
        progress_callback: None,
    };

    if proc_opts.output_dir.is_none() && !proc_opts.in_place {
        proc_opts.in_place = true;
    }

    println!("Decrypting game assets...");
    let processor = FileProcessor::new(&engine, proc_opts);
    let summary = processor.decrypt_all(target).map_err(|e| {
        rpgmaker_core::core::error::RpgError::Crypto(format!("Decryption process failed: {}", e))
    })?;

    println!(
        "Processed {} files ({} bytes) in {:.2?}",
        summary.files_processed, summary.bytes_written, summary.elapsed
    );

    if !summary.failed_files.is_empty() {
        eprintln!(
            "  [!] Warning: {} files failed to decrypt:",
            summary.failed_files.len()
        );
        for f in summary.failed_files.iter().take(10) {
            eprintln!("      - {}: {}", f.path.display(), f.error);
        }
    }

    // 1. Retarget existing symlinks pointing to encrypted files before purging originals
    if target.is_dir() {
        for entry in WalkDir::new(target)
            .into_iter()
            .filter_entry(|e| {
                if let Ok(rel) = e.path().strip_prefix(target) {
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
            if entry.path_is_symlink() {
                let link_path = entry.path();
                if let Ok(link_target) = fs::read_link(link_path) {
                    let target_is_enc = link_target
                        .extension()
                        .and_then(|s| s.to_str())
                        .map(is_encrypted_extension)
                        .unwrap_or(false);

                    if target_is_enc {
                        let new_target = decrypted_path(&link_target);
                        let new_link_path = if let Some(link_ext) =
                            link_path.extension().and_then(|s| s.to_str())
                        {
                            if is_encrypted_extension(link_ext) {
                                decrypted_path(link_path)
                            } else {
                                link_path.to_path_buf()
                            }
                        } else {
                            link_path.to_path_buf()
                        };

                        let _ = fs::remove_file(link_path);
                        #[cfg(unix)]
                        {
                            if let Err(e) = std::os::unix::fs::symlink(&new_target, &new_link_path)
                            {
                                eprintln!(
                                    "  [!] Warning: Failed retargeting symlink {}: {}",
                                    new_link_path.display(),
                                    e
                                );
                            }
                        }
                    }
                }
            }
        }
    }

    // 2. Clean up leftover encrypted files if not keeping (only regular files, NOT symlinks)
    if !keep_encrypted && target.is_dir() {
        println!("Cleaning up leftover encrypted files...");
        for entry in WalkDir::new(target)
            .into_iter()
            .filter_entry(|e| {
                if let Ok(rel) = e.path().strip_prefix(target) {
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
            if entry.file_type().is_file() && !entry.path_is_symlink() {
                if let Some(ext) = entry.path().extension().and_then(|s| s.to_str()) {
                    if is_encrypted_extension(ext) {
                        let _ = fs::remove_file(entry.path());
                    }
                }
            }
        }
    }

    // Update System.json encryption flags
    if let Ok(layout) = detect_game_layout(target) {
        let sys_path = layout.data_dir(target).join("System.json");
        if sys_path.is_file() {
            let _ = update_system_encryption_flags(&sys_path, false, false);
            println!("Updated System.json encryption flags.");
        }
    }

    println!("Decryption finished successfully!");
    Ok(())
}
