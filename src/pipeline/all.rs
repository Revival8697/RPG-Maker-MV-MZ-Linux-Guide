use crate::casing::transaction::execute_fix_casing;
use crate::core::error::Result;
use crate::core::layout::detect_game_layout;
use crate::libs::{run_update_libs, UpdateMode};
use crate::optimizer::{run_optimize, OptimizeOptions};
use crate::setup::{run_setup, SetupOptions};
use std::path::Path;

pub struct AllPipelineOptions {
    pub non_interactive: bool,
    pub custom_id: Option<String>,
    pub full_libs: bool,
    pub no_backup: bool,
    pub dry_run_casing: bool,
    pub custom_key: Option<String>,
    pub keep_encrypted: bool,
    pub opt_level: String,
    pub images_only: bool,
    pub audio_only: bool,
    pub threads: Option<usize>,
    pub clean_junk: bool,
}

pub type DecryptFn<'a> = &'a dyn Fn(&Path, Option<&str>, bool, bool) -> Result<()>;

pub fn run_all_pipeline(
    game_dir: &Path,
    options: AllPipelineOptions,
    decrypt_fn: DecryptFn,
) -> Result<()> {
    println!("============================================================");
    println!(" [1/5] Setting up game environment and Linux launcher...");
    println!("============================================================");
    let setup_opts = SetupOptions {
        non_interactive: options.non_interactive,
        custom_id: options.custom_id,
        install_desktop: false,
        clean_junk: options.clean_junk,
    };
    run_setup(game_dir, setup_opts)?;

    println!("\n============================================================");
    println!(" [2/5] Decrypting game assets...");
    println!("============================================================");
    decrypt_fn(
        game_dir,
        options.custom_key.as_deref(),
        !options.no_backup,
        options.keep_encrypted,
    )?;

    println!("\n============================================================");
    println!(" [3/5] Fixing asset casing for case-sensitive filesystems...");
    println!("============================================================");
    let layout = detect_game_layout(game_dir)?;
    let asset_root = layout.asset_root(game_dir);
    let casing_report = execute_fix_casing(&asset_root, options.dry_run_casing, true)?;
    println!(
        "Matched {} engine declarations; renamed {} files, created {} symlinks.",
        casing_report.engine_references_matched,
        casing_report.files_renamed,
        casing_report.symlinks_created
    );

    println!("\n============================================================");
    println!(" [4/5] Updating rendering libraries...");
    println!("============================================================");
    let lib_mode = if options.full_libs {
        UpdateMode::Full
    } else {
        UpdateMode::PixiOnly
    };
    run_update_libs(game_dir, lib_mode, !options.no_backup)?;

    println!("\n============================================================");
    println!(" [5/5] Optimizing image and audio assets losslessly...");
    println!("============================================================");
    let opt_opts = OptimizeOptions {
        run_images: !options.audio_only,
        run_audio: !options.images_only,
        create_backup: !options.no_backup,
        opt_level: options.opt_level,
        auto_accept: true,
        threads: options.threads,
    };
    run_optimize(game_dir, &opt_opts)?;

    println!("\n============================================================");
    println!(" All operations completed successfully!");
    println!(
        " Launch your game natively via: \"{}/Game\"",
        game_dir.display()
    );
    println!("============================================================");

    Ok(())
}
