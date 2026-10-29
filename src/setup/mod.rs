pub mod cleaner;
pub mod launcher;
pub mod package;

use crate::core::error::Result;
use crate::core::layout::detect_game_layout;
use crate::core::manifest::read_system_info;
use std::path::Path;

pub struct SetupOptions {
    pub non_interactive: bool,
    pub custom_id: Option<String>,
    pub install_desktop: bool,
    pub clean_junk: bool,
}

pub fn run_setup(game_dir: &Path, options: SetupOptions) -> Result<()> {
    let layout = detect_game_layout(game_dir)?;
    println!("Detected RPG Maker {} layout.", layout.name());

    cleaner::purge_windows_binaries(game_dir)?;

    if options.clean_junk {
        let junk = cleaner::clean_developer_junk(game_dir, false)?;
        if !junk.is_empty() {
            println!("Cleaned {} non-runtime developer junk files.", junk.len());
        }
    }

    let sys_info = read_system_info(layout, game_dir)?;
    let display_title = if !sys_info.game_title.is_empty() {
        sys_info.game_title
    } else {
        game_dir
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("rpg-game")
            .to_string()
    };

    let slug = generate_slug(&display_title);
    let default_id = if slug.is_empty() {
        format!("rpg-game-{}", std::process::id())
    } else {
        slug
    };

    let game_id = options.custom_id.unwrap_or(default_id);

    package::configure_package_json(game_dir, layout, &game_id, &display_title)?;
    launcher::deploy_launcher_and_desktop(
        game_dir,
        layout,
        &game_id,
        &display_title,
        options.install_desktop,
    )?;

    println!("\nGame setup complete! ({} layout)", layout.name());
    println!(
        "You can now launch your game via: \"{}/Game\"",
        game_dir.display()
    );

    Ok(())
}

fn generate_slug(title: &str) -> String {
    let mut slug = title
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect::<String>();
    while slug.contains("--") {
        slug = slug.replace("--", "-");
    }
    slug.trim_matches('-').to_string()
}
