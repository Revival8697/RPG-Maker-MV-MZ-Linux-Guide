use crate::core::error::{Result, RpgError};
use crate::core::layout::GameLayout;
use serde_json::Value;
use std::fs;
use std::path::Path;

pub fn configure_package_json(
    game_dir: &Path,
    layout: GameLayout,
    game_id: &str,
    window_title: &str,
) -> Result<()> {
    let root_pkg = game_dir.join("package.json");

    // For MV: if package.json is missing at root, look in www/
    if layout == GameLayout::Mv && !root_pkg.is_file() {
        let www_pkg = game_dir.join("www").join("package.json");
        if www_pkg.is_file() {
            fs::copy(&www_pkg, &root_pkg).map_err(RpgError::Io)?;
        }
    }

    if !root_pkg.is_file() {
        return Err(RpgError::Setup(format!(
            "package.json not found in '{}'",
            game_dir.display()
        )));
    }

    let pkg_content = fs::read_to_string(&root_pkg).map_err(RpgError::Io)?;
    let mut pkg_json: Value = serde_json::from_str(&pkg_content).map_err(|e| RpgError::Json {
        path: root_pkg.display().to_string(),
        source: e,
    })?;

    if let Some(obj) = pkg_json.as_object_mut() {
        obj.insert("name".to_string(), Value::String(game_id.to_string()));
        obj.insert(
            "main".to_string(),
            Value::String(layout.entry_main().to_string()),
        );

        let current_flags = obj.get("js-flags").and_then(|v| v.as_str()).unwrap_or("");
        let new_flags = if current_flags.is_empty() {
            "--expose-gc".to_string()
        } else if !current_flags.contains("--expose-gc") {
            format!("{} --expose-gc", current_flags)
        } else {
            current_flags.to_string()
        };
        obj.insert("js-flags".to_string(), Value::String(new_flags));

        let window_obj = obj
            .entry("window".to_string())
            .or_insert_with(|| Value::Object(serde_json::Map::new()));
        if let Some(w) = window_obj.as_object_mut() {
            let current_title = w.get("title").and_then(|v| v.as_str()).unwrap_or("");
            if current_title.is_empty() {
                w.insert("title".to_string(), Value::String(window_title.to_string()));
            }

            let current_icon = w.get("icon").and_then(|v| v.as_str()).unwrap_or("");
            if current_icon.is_empty() {
                w.insert(
                    "icon".to_string(),
                    Value::String(layout.icon_rel().to_string()),
                );
            }
        }
    }

    let updated_pkg = serde_json::to_string_pretty(&pkg_json).map_err(|e| RpgError::Json {
        path: root_pkg.display().to_string(),
        source: e,
    })?;
    fs::write(&root_pkg, updated_pkg).map_err(RpgError::Io)?;

    Ok(())
}
