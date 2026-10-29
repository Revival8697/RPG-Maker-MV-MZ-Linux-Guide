//! Header representation and validation for RPG Maker MV/MZ encrypted assets.

use crate::crypto::error::{DecryptionError, Result};

/// Default byte length of the RPG Maker fake header (16 bytes).
pub const DEFAULT_HEADER_LEN: usize = 16;

/// Default signature bytes: "RPGMV\0\0\0" (8 bytes).
pub const DEFAULT_SIGNATURE: [u8; 8] = [0x52, 0x50, 0x47, 0x4D, 0x56, 0x00, 0x00, 0x00];

/// Default version bytes: "000301" (3 bytes).
pub const DEFAULT_VERSION: [u8; 3] = [0x00, 0x03, 0x01];

/// Default remaining bytes: "0000000000" (5 bytes).
pub const DEFAULT_REMAIN: [u8; 5] = [0x00, 0x00, 0x00, 0x00, 0x00];

/// Standard 16-byte PNG header: \x89PNG\r\n\x1a\n\0\0\0\rIHDR.
/// Used for key derivation and restoring PNGs without a key.
pub const PNG_HEADER: [u8; 16] = [
    0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52,
];

/// Configuration for the RPG Maker asset header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeaderConfig {
    /// Expected byte length of the fake header prepended to encrypted files.
    pub header_len: usize,
    /// Header signature bytes (typically "RPGMV\0\0\0").
    pub signature: Vec<u8>,
    /// Header version bytes (typically `[0x00, 0x03, 0x01]`).
    pub version: Vec<u8>,
    /// Remaining header padding bytes (typically 5 null bytes).
    pub remain: Vec<u8>,
    /// If true, files with mismatching fake headers will still be decrypted without throwing an error.
    pub ignore_fake_header: bool,
}

impl Default for HeaderConfig {
    fn default() -> Self {
        Self {
            header_len: DEFAULT_HEADER_LEN,
            signature: DEFAULT_SIGNATURE.to_vec(),
            version: DEFAULT_VERSION.to_vec(),
            remain: DEFAULT_REMAIN.to_vec(),
            ignore_fake_header: true,
        }
    }
}

impl HeaderConfig {
    /// Creates a new HeaderConfig with default parameters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets whether to ignore fake header validation errors.
    pub fn with_ignore_fake_header(mut self, ignore: bool) -> Self {
        self.ignore_fake_header = ignore;
        self
    }

    /// Sets custom header length in bytes.
    pub fn with_header_len(mut self, len: usize) -> Result<Self> {
        if len == 0 {
            return Err(DecryptionError::InvalidHeaderLength(0));
        }
        self.header_len = len;
        Ok(self)
    }

    /// Constructs the reference fake header byte vector by concatenating signature, version, and remain.
    /// Truncates or pads to `header_len`.
    pub fn build_fake_header(&self) -> Vec<u8> {
        let mut header = Vec::with_capacity(self.header_len);
        header.extend_from_slice(&self.signature);
        header.extend_from_slice(&self.version);
        header.extend_from_slice(&self.remain);

        if header.len() > self.header_len {
            header.truncate(self.header_len);
        } else if header.len() < self.header_len {
            header.resize(self.header_len, 0);
        }

        header
    }

    /// Validates the first `header_len` bytes against the expected fake header bytes.
    pub fn validate_fake_header(&self, data: &[u8]) -> Result<()> {
        if self.ignore_fake_header {
            return Ok(());
        }

        if data.len() < self.header_len {
            return Err(DecryptionError::FileTooShort {
                min_expected: self.header_len,
                found: data.len(),
            });
        }

        let expected = self.build_fake_header();
        let actual = &data[..self.header_len];

        if actual != expected.as_slice() {
            return Err(DecryptionError::InvalidFakeHeader {
                expected: hex::encode(&expected),
                found: hex::encode(actual),
            });
        }

        Ok(())
    }

    /// Checks if the provided byte slice starts with the expected fake header.
    pub fn matches_fake_header(&self, data: &[u8]) -> bool {
        if data.len() < self.header_len {
            return false;
        }
        let expected = self.build_fake_header();
        &data[..self.header_len] == expected.as_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_header() {
        let config = HeaderConfig::default();
        let header = config.build_fake_header();
        assert_eq!(header.len(), 16);
        assert_eq!(hex::encode(&header), "5250474d560000000003010000000000");
    }

    #[test]
    fn test_validation() {
        let config = HeaderConfig {
            ignore_fake_header: false,
            ..Default::default()
        };

        let valid_header = config.build_fake_header();
        assert!(config.validate_fake_header(&valid_header).is_ok());

        let invalid_header = vec![0u8; 16];
        assert!(config.validate_fake_header(&invalid_header).is_err());
    }
}
