use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(
    name = "rpgmaker-helper",
    version = "2.0.0",
    about = "RPG Maker MV/MZ Linux Helper - High-performance native CLI",
    long_about = "Prepares, fixes, decrypts, and optimizes RPG Maker MV and MZ games to run natively on Linux using NW.js."
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Audit game layout, packages, casing mismatches, and encryption (read-only)
    #[command(alias = "audit", alias = "inspect")]
    Scan {
        /// Path to game directory
        game_dir: PathBuf,
    },

    /// Configure game for native Linux NW.js, purge Windows runtime, deploy launchers
    #[command(alias = "init")]
    Setup {
        /// Path to game directory
        game_dir: PathBuf,

        /// Non-interactive mode (automatically accept detected title and ID)
        #[arg(short = 'y', long = "yes")]
        yes: bool,

        /// Custom identifier/name for package.json
        #[arg(long = "id")]
        id: Option<String>,

        /// Install desktop shortcut into ~/.local/share/applications/
        #[arg(long = "install-desktop")]
        install_desktop: bool,

        /// Clean non-runtime developer junk files (.clip, .bak, .tmx, .trans, CSP dumps)
        #[arg(long = "clean-junk")]
        clean_junk: bool,
    },

    /// Fix asset casing mismatches for case-sensitive Linux filesystems
    #[command(alias = "casing", alias = "fix-case")]
    FixCasing {
        /// Path to game directory
        game_dir: PathBuf,

        /// Preview changes without altering files on disk
        #[arg(short = 'n', long = "dry-run")]
        dry_run: bool,

        /// Do not generate case-alias symlinks
        #[arg(long = "no-symlinks")]
        no_symlinks: bool,
    },

    /// Decrypt MV (.rpgmvp, .rpgmvo) and MZ (.png_, .ogg_) assets
    #[command(alias = "dec")]
    Decrypt {
        /// Path to game directory or encrypted file
        target: PathBuf,

        /// 32-character hexadecimal encryption key
        key: Option<String>,

        /// 32-character hex key via flag
        #[arg(short = 'k', long = "key")]
        flag_key: Option<String>,

        /// Create a safety backup before decrypting
        #[arg(short = 'b', long = "backup")]
        backup: bool,

        /// Preserve original encrypted files alongside decrypted assets
        #[arg(long = "keep-encrypted")]
        keep_encrypted: bool,

        /// Output directory (defaults to in-place)
        #[arg(short = 'o', long = "output")]
        output: Option<PathBuf>,

        /// In-place decryption in the game directory
        #[arg(short = 'i', long = "in-place")]
        in_place: bool,

        /// Restore PNG images using standard PNG magic header (no key needed)
        #[arg(long = "restore")]
        restore: bool,

        /// Limit worker threads for parallel processing
        #[arg(short = 'j', long = "threads")]
        threads: Option<usize>,
    },

    /// Update game libraries (Pixi.js v4.8.9 for MV, v5.3.12 for MZ)
    #[command(alias = "libs", alias = "update-lib", alias = "lib-update")]
    UpdateLibs {
        /// Path to game directory
        game_dir: PathBuf,

        /// Update only pixi.js (recommended; preserves custom plugins/scripts)
        #[arg(long = "pixi-only", default_value_t = true)]
        pixi_only: bool,

        /// Update pixi.js AND MV core engine scripts (rpg_*.js for MV)
        #[arg(long = "full")]
        full: bool,

        /// Skip creating a backup before updating
        #[arg(long = "no-backup")]
        no_backup: bool,
    },

    /// Losslessly compress PNGs (oxipng) and remux OGG audio
    #[command(alias = "opt")]
    Optimize {
        /// Path to game directory
        game_dir: PathBuf,

        /// Optimize only PNG image assets
        #[arg(long = "images-only")]
        images_only: bool,

        /// Optimize only OGG audio assets
        #[arg(long = "audio-only")]
        audio_only: bool,

        /// Skip creating a backup before optimization
        #[arg(long = "no-backup")]
        no_backup: bool,

        /// oxipng optimization preset level (1-6 or max, default: 2)
        #[arg(long = "level", default_value = "2")]
        level: String,

        /// Auto-accept interactive prompts
        #[arg(short = 'y', long = "yes")]
        yes: bool,

        /// Limit worker threads for parallel processing
        #[arg(short = 'j', long = "threads")]
        threads: Option<usize>,
    },

    /// Strip non-runtime developer junk files (.clip, .psd, .bak, .tmx, .trans, CSP dumps)
    #[command(alias = "junk", alias = "clean")]
    CleanJunk {
        /// Path to game directory
        game_dir: PathBuf,

        /// Dry-run mode: list junk files without deleting
        #[arg(long = "dry-run")]
        dry_run: bool,
    },

    /// Run full pipeline: setup -> decrypt -> fix-casing -> update-libs -> optimize
    All {
        /// Path to game directory
        game_dir: PathBuf,

        /// Non-interactive mode
        #[arg(short = 'y', long = "yes")]
        yes: bool,

        /// Custom package name for package.json
        #[arg(long = "id")]
        id: Option<String>,

        /// Update pixi.js AND MV core engine scripts
        #[arg(long = "full")]
        full: bool,

        /// Skip creating safety backups
        #[arg(long = "no-backup")]
        no_backup: bool,

        /// Preview casing fixes without altering files
        #[arg(short = 'n', long = "dry-run")]
        dry_run: bool,

        /// Specify 32-character decryption key
        #[arg(short = 'k', long = "key")]
        key: Option<String>,

        /// Preserve original encrypted files alongside decrypted assets
        #[arg(long = "keep-encrypted")]
        keep_encrypted: bool,

        /// oxipng optimization level (1-6 or max, default: 2)
        #[arg(long = "level", default_value = "2")]
        level: String,

        /// Limit worker threads for parallel processing
        #[arg(short = 'j', long = "threads")]
        threads: Option<usize>,

        /// Clean developer junk files (.clip, .bak, .tmx, .trans, CSP dumps)
        #[arg(long = "clean-junk")]
        clean_junk: bool,

        /// Optimize only PNG images
        #[arg(long = "images-only")]
        images_only: bool,

        /// Optimize only OGG audio
        #[arg(long = "audio-only")]
        audio_only: bool,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn verify_cli() {
        Cli::command().debug_assert();
    }
}
