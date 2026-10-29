//! File extension mappings and asset classification for RPG Maker MV and MZ.

use std::path::{Path, PathBuf};

/// The asset engine format variant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EngineFormat {
    /// RPG Maker MV (.rpgmvp, .rpgmvm, .rpgmvo)
    Mv,
    /// RPG Maker MZ (.png_, .m4a_, .ogg_)
    Mz,
}

/// The media type of an encrypted RPG Maker asset.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetKind {
    ImagePng,
    AudioM4a,
    AudioOgg,
}

impl AssetKind {
    /// Returns the standard decrypted file extension.
    pub fn decrypted_extension(&self) -> &'static str {
        match self {
            AssetKind::ImagePng => "png",
            AssetKind::AudioM4a => "m4a",
            AssetKind::AudioOgg => "ogg",
        }
    }

    /// Returns the encrypted extension for MV or MZ.
    pub fn encrypted_extension(&self, format: EngineFormat) -> &'static str {
        match (self, format) {
            (AssetKind::ImagePng, EngineFormat::Mv) => "rpgmvp",
            (AssetKind::AudioM4a, EngineFormat::Mv) => "rpgmvm",
            (AssetKind::AudioOgg, EngineFormat::Mv) => "rpgmvo",
            (AssetKind::ImagePng, EngineFormat::Mz) => "png_",
            (AssetKind::AudioM4a, EngineFormat::Mz) => "m4a_",
            (AssetKind::AudioOgg, EngineFormat::Mz) => "ogg_",
        }
    }
}

/// Identifies whether an extension corresponds to an encrypted RPG Maker asset.
pub fn is_encrypted_extension(ext: &str) -> bool {
    let lower = ext.to_ascii_lowercase();
    matches!(
        lower.as_str(),
        "rpgmvp" | "rpgmvm" | "rpgmvo" | "png_" | "m4a_" | "ogg_"
    )
}

/// Checks whether an extension belongs to an encrypted image.
pub fn is_encrypted_image(ext: &str) -> bool {
    let lower = ext.to_ascii_lowercase();
    matches!(lower.as_str(), "rpgmvp" | "png_")
}

/// Checks whether an extension belongs to an encrypted audio file.
pub fn is_encrypted_audio(ext: &str) -> bool {
    let lower = ext.to_ascii_lowercase();
    matches!(lower.as_str(), "rpgmvm" | "rpgmvo" | "m4a_" | "ogg_")
}

/// Resolves the plaintext file extension from an encrypted extension.
pub fn target_decrypted_extension(ext: &str) -> Option<&'static str> {
    let lower = ext.to_ascii_lowercase();
    match lower.as_str() {
        "rpgmvp" | "png_" => Some("png"),
        "rpgmvm" | "m4a_" => Some("m4a"),
        "rpgmvo" | "ogg_" => Some("ogg"),
        _ => None,
    }
}

/// Resolves the encrypted extension for a given plaintext extension.
pub fn target_encrypted_extension(ext: &str, is_mv: bool) -> Option<&'static str> {
    let lower = ext.to_ascii_lowercase();
    match (lower.as_str(), is_mv) {
        ("png", true) => Some("rpgmvp"),
        ("m4a", true) => Some("rpgmvm"),
        ("ogg", true) => Some("rpgmvo"),
        ("png", false) => Some("png_"),
        ("m4a", false) => Some("m4a_"),
        ("ogg", false) => Some("ogg_"),
        _ => None,
    }
}

/// Returns a new PathBuf with the decrypted extension substituted.
pub fn decrypted_path(path: &Path) -> PathBuf {
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        if let Some(decrypted_ext) = target_decrypted_extension(ext) {
            let mut new_path = path.to_path_buf();
            new_path.set_extension(decrypted_ext);
            return new_path;
        }
    }
    path.to_path_buf()
}

/// Returns a new PathBuf with the encrypted extension substituted.
pub fn encrypted_path(path: &Path, is_mv: bool) -> PathBuf {
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        if let Some(enc_ext) = target_encrypted_extension(ext, is_mv) {
            let mut new_path = path.to_path_buf();
            new_path.set_extension(enc_ext);
            return new_path;
        }
    }
    path.to_path_buf()
}

/// Checks whether a plaintext file is excluded from encryption by default
/// (such as loading screen, window skins, or icon files).
pub fn is_encryption_excluded(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_ascii_lowercase();
    let normalized = path_str.replace('\\', "/");

    normalized.ends_with("img/system/loading.png")
        || normalized.ends_with("img/system/window.png")
        || normalized.ends_with("icon/icon.png")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extension_mapping() {
        assert_eq!(target_decrypted_extension("rpgmvp"), Some("png"));
        assert_eq!(target_decrypted_extension("rpgmvm"), Some("m4a"));
        assert_eq!(target_decrypted_extension("rpgmvo"), Some("ogg"));
        assert_eq!(target_decrypted_extension("png_"), Some("png"));
        assert_eq!(target_decrypted_extension("m4a_"), Some("m4a"));
        assert_eq!(target_decrypted_extension("ogg_"), Some("ogg"));
        assert_eq!(target_decrypted_extension("txt"), None);

        assert_eq!(target_encrypted_extension("png", true), Some("rpgmvp"));
        assert_eq!(target_encrypted_extension("png", false), Some("png_"));
    }

    #[test]
    fn test_decrypted_path() {
        let original = Path::new("test/image.rpgmvp");
        assert_eq!(decrypted_path(original), PathBuf::from("test/image.png"));
    }
}
