use crate::core::error::{Result, RpgError};
use crate::core::layout::GameLayout;
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Default)]
pub struct SystemInfo {
    pub game_title: String,
    pub has_encrypted_images: bool,
    pub has_encrypted_audio: bool,
    pub encryption_key: Option<String>,
}

pub fn read_system_info(layout: GameLayout, game_dir: &Path) -> Result<SystemInfo> {
    let sys_path = layout.data_dir(game_dir).join("System.json");
    if !sys_path.is_file() {
        return Ok(SystemInfo::default());
    }

    let content = fs::read_to_string(&sys_path).map_err(RpgError::Io)?;
    let clean_content = content.trim_start_matches('\u{feff}');
    let val: Value = serde_json::from_str(clean_content).map_err(|e| RpgError::Json {
        path: sys_path.display().to_string(),
        source: e,
    })?;

    let game_title = val
        .get("gameTitle")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .unwrap_or_default();

    let has_encrypted_images = val
        .get("hasEncryptedImages")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let has_encrypted_audio = val
        .get("hasEncryptedAudio")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let encryption_key = val
        .get("encryptionKey")
        .and_then(|v| v.as_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());

    Ok(SystemInfo {
        game_title,
        has_encrypted_images,
        has_encrypted_audio,
        encryption_key,
    })
}

pub fn update_system_encryption_flags(
    sys_path: &Path,
    has_images: bool,
    has_audio: bool,
) -> Result<()> {
    if !sys_path.is_file() {
        return Ok(());
    }

    let content = fs::read_to_string(sys_path).map_err(RpgError::Io)?;
    let clean_content = content.trim_start_matches('\u{feff}');
    let mut val: Value = serde_json::from_str(clean_content).map_err(|e| RpgError::Json {
        path: sys_path.display().to_string(),
        source: e,
    })?;

    if let Some(obj) = val.as_object_mut() {
        obj.insert("hasEncryptedImages".to_string(), Value::Bool(has_images));
        obj.insert("hasEncryptedAudio".to_string(), Value::Bool(has_audio));
        let updated = serde_json::to_string_pretty(&val).map_err(|e| RpgError::Json {
            path: sys_path.display().to_string(),
            source: e,
        })?;
        fs::write(sys_path, updated).map_err(RpgError::Io)?;
    }

    Ok(())
}
