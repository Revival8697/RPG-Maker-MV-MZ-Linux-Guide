//! Encryption key handling, extraction from System.json, and derivation from encrypted PNGs.

use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::crypto::error::{DecryptionError, Result};
use crate::crypto::header::PNG_HEADER;

/// Representation of an RPG Maker MV/MZ decryption key.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncryptionKey {
    bytes: Vec<u8>,
}

impl EncryptionKey {
    /// Parses an encryption key from a hexadecimal string (e.g. 32 hex characters for a 16-byte key).
    pub fn from_hex(hex_str: &str) -> Result<Self> {
        let cleaned: String = hex_str
            .trim()
            .chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        let bytes = hex::decode(&cleaned)?;
        if bytes.is_empty() {
            return Err(DecryptionError::MissingKey);
        }
        Ok(Self { bytes })
    }

    /// Creates an encryption key from raw bytes.
    pub fn from_bytes(bytes: impl Into<Vec<u8>>) -> Self {
        Self {
            bytes: bytes.into(),
        }
    }

    /// Returns the key bytes as a slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// Returns the length of the key in bytes.
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    /// Checks if the key is empty.
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    /// Converts the key to a lowercase hexadecimal string.
    pub fn to_hex(&self) -> String {
        hex::encode(&self.bytes)
    }

    /// Derives an encryption key from the encrypted header of a PNG image.
    ///
    /// RPG Maker encrypts PNG files by prepending a fake header (length `header_len`, default 16 bytes)
    /// and XORing the next `header_len` bytes with the key. Since the first 16 bytes of any standard PNG
    /// are constant (`89504E470D0A1A0A0000000D49484452`), we can recover the key via:
    /// `key_byte = encrypted_byte ^ png_header_byte`.
    pub fn derive_from_encrypted_png_bytes(data: &[u8], header_len: usize) -> Result<Self> {
        let key_len = PNG_HEADER.len().min(header_len);
        let min_required = header_len + key_len;

        if data.len() < min_required {
            return Err(DecryptionError::FileTooShort {
                min_expected: min_required,
                found: data.len(),
            });
        }

        let mut key_bytes = Vec::with_capacity(key_len);
        for i in 0..key_len {
            let encrypted_byte = data[header_len + i];
            let png_byte = PNG_HEADER[i];
            key_bytes.push(encrypted_byte ^ png_byte);
        }

        Ok(Self::from_bytes(key_bytes))
    }

    /// Derives an encryption key from an encrypted PNG file path.
    pub fn derive_from_png_file(path: &Path, header_len: usize) -> Result<Self> {
        let mut file = fs::File::open(path).map_err(|e| DecryptionError::io_with_path(path, e))?;
        use std::io::Read;

        let needed = header_len + PNG_HEADER.len().min(header_len);
        let mut buffer = vec![0u8; needed];
        file.read_exact(&mut buffer)
            .map_err(|e| DecryptionError::io_with_path(path, e))?;

        Self::derive_from_encrypted_png_bytes(&buffer, header_len)
    }
}

/// Metadata extracted from `System.json`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemJsonInfo {
    #[serde(rename = "encryptionKey", default)]
    pub encryption_key: Option<String>,

    #[serde(rename = "hasEncryptedImages", default)]
    pub has_encrypted_images: Option<bool>,

    #[serde(rename = "hasEncryptedAudio", default)]
    pub has_encrypted_audio: Option<bool>,
}

impl SystemJsonInfo {
    /// Parses System.json content and returns the parsed info.
    pub fn parse(json_content: &str) -> Result<Self> {
        let info: Self = serde_json::from_str(json_content)?;
        Ok(info)
    }

    /// Checks if either images or audio are marked as encrypted.
    pub fn is_encrypted(&self) -> bool {
        self.has_encrypted_images.unwrap_or(false) || self.has_encrypted_audio.unwrap_or(false)
    }
}

/// Extracts the encryption key and system information from `System.json` file content.
pub fn extract_key_from_system_json(
    json_content: &str,
    custom_field_name: Option<&str>,
) -> Result<(EncryptionKey, SystemJsonInfo)> {
    let field = custom_field_name.unwrap_or("encryptionKey");
    let json_val: serde_json::Value = serde_json::from_str(json_content)?;

    let key_str = json_val
        .get(field)
        .and_then(|v| v.as_str())
        .ok_or_else(|| DecryptionError::KeyNotFoundInJson {
            searched_field: field.to_string(),
        })?;

    let key = EncryptionKey::from_hex(key_str)?;
    let info = SystemJsonInfo::parse(json_content).unwrap_or_default();

    Ok((key, info))
}

/// Loads and extracts the encryption key from a `System.json` file at the specified path.
pub fn load_key_from_system_file(
    path: &Path,
    custom_field_name: Option<&str>,
) -> Result<(EncryptionKey, SystemJsonInfo)> {
    let content = fs::read_to_string(path).map_err(|e| DecryptionError::io_with_path(path, e))?;
    extract_key_from_system_json(&content, custom_field_name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_conversion() {
        let hex_val = "d41d8cd98f00b204e9800998ecf8427e";
        let key = EncryptionKey::from_hex(hex_val).unwrap();
        assert_eq!(key.len(), 16);
        assert_eq!(key.to_hex(), hex_val);
    }

    #[test]
    fn test_extract_from_json() {
        let json = r#"{
            "gameTitle": "Test RPG",
            "hasEncryptedImages": true,
            "hasEncryptedAudio": true,
            "encryptionKey": "deadbeefcafebabe0123456789abcdef"
        }"#;

        let (key, info) = extract_key_from_system_json(json, None).unwrap();
        assert_eq!(key.to_hex(), "deadbeefcafebabe0123456789abcdef");
        assert!(info.is_encrypted());
        assert_eq!(info.has_encrypted_images, Some(true));
    }

    #[test]
    fn test_derive_from_png() {
        let original_key_hex = "d41d8cd98f00b204e9800998ecf8427e";
        let key = EncryptionKey::from_hex(original_key_hex).unwrap();

        let mut fake_file = vec![0u8; 32];
        // 0..16 is fake header (dummy)
        // 16..32 is XORed PNG header
        for i in 0..16 {
            fake_file[16 + i] = PNG_HEADER[i] ^ key.as_bytes()[i];
        }

        let derived = EncryptionKey::derive_from_encrypted_png_bytes(&fake_file, 16).unwrap();
        assert_eq!(derived.to_hex(), original_key_hex);
    }
}
