use crate::core::error::{Result, RpgError};
use crate::core::layout::GameLayout;
use crate::embedded::LAUNCHER_GAME_SH;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

pub fn deploy_launcher_and_desktop(
    game_dir: &Path,
    layout: GameLayout,
    game_id: &str,
    display_title: &str,
    install_to_system_menu: bool,
) -> Result<(PathBuf, PathBuf)> {
    let launcher_dest = game_dir.join("Game");
    fs::write(&launcher_dest, LAUNCHER_GAME_SH).map_err(RpgError::Io)?;

    let mut perms = fs::metadata(&launcher_dest)
        .map_err(RpgError::Io)?
        .permissions();
    perms.set_mode(perms.mode() | 0o755);
    fs::set_permissions(&launcher_dest, perms).map_err(RpgError::Io)?;
    println!("Created native launcher: {}", launcher_dest.display());

    let desktop_file = game_dir.join(format!("{}.desktop", game_id));
    let icon_full = game_dir.join(layout.icon_rel());
    let icon_str = if icon_full.is_file() {
        icon_full.to_string_lossy().to_string()
    } else {
        String::new()
    };

    let desktop_content = format!(
        "[Desktop Entry]\nType=Application\nName={}\nExec=\"{}/Game\"\nPath={}\nIcon={}\nTerminal=false\nCategories=Game;\n",
        display_title,
        game_dir.display(),
        game_dir.display(),
        icon_str
    );

    fs::write(&desktop_file, &desktop_content).map_err(RpgError::Io)?;
    let mut d_perms = fs::metadata(&desktop_file)
        .map_err(RpgError::Io)?
        .permissions();
    d_perms.set_mode(d_perms.mode() | 0o755);
    fs::set_permissions(&desktop_file, d_perms).map_err(RpgError::Io)?;
    println!("Created desktop entry:   {}", desktop_file.display());

    if install_to_system_menu {
        if let Some(home) = std::env::var_os("HOME") {
            let user_apps = PathBuf::from(home).join(".local/share/applications");
            if user_apps.is_dir() {
                let target = user_apps.join(format!("{}.desktop", game_id));
                if fs::write(&target, &desktop_content).is_ok() {
                    println!("Installed desktop shortcut to: {}", target.display());
                }
            }
        }
    }

    Ok((launcher_dest, desktop_file))
}
