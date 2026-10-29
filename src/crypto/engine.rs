//! Core streaming and in-memory decryption and encryption engine.

use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use crate::crypto::error::{DecryptionError, Result};
use crate::crypto::header::{HeaderConfig, PNG_HEADER};
use crate::crypto::key::EncryptionKey;

/// The core Decrypter and Encrypter engine.
#[derive(Debug, Clone, Default)]
pub struct Decrypter {
    pub header_config: HeaderConfig,
    pub key: Option<EncryptionKey>,
}

impl Decrypter {
    /// Creates a new Decrypter with default header configuration and an optional encryption key.
    pub fn new(key: Option<EncryptionKey>) -> Self {
        Self {
            header_config: HeaderConfig::default(),
            key,
        }
    }

    /// Sets the encryption key.
    pub fn with_key(mut self, key: EncryptionKey) -> Self {
        self.key = Some(key);
        self
    }

    /// Sets a custom header configuration.
    pub fn with_header_config(mut self, config: HeaderConfig) -> Self {
        self.header_config = config;
        self
    }

    /// Decrypts an in-memory byte slice.
    ///
    /// If `restore_png` is true, the key is not required; the XORed header is replaced with the standard PNG header.
    pub fn decrypt_bytes(&self, data: &[u8], restore_png: bool) -> Result<Vec<u8>> {
        let hlen = self.header_config.header_len;
        let min_len = hlen * 2;

        if data.len() < min_len {
            return Err(DecryptionError::FileTooShort {
                min_expected: min_len,
                found: data.len(),
            });
        }

        // Validate fake header if configured
        self.header_config.validate_fake_header(data)?;

        let mut output = Vec::with_capacity(data.len() - hlen);

        if restore_png {
            // Restore standard PNG magic header
            let restore_len = PNG_HEADER.len().min(hlen);
            output.extend_from_slice(&PNG_HEADER[..restore_len]);
            if hlen > restore_len {
                output.resize(hlen, 0);
            }
        } else {
            let key = self.key.as_ref().ok_or(DecryptionError::MissingKey)?;
            let key_bytes = key.as_bytes();

            for i in 0..hlen {
                let enc_byte = data[hlen + i];
                let key_byte = key_bytes[i % key_bytes.len()];
                output.push(enc_byte ^ key_byte);
            }
        }

        // Copy remaining untouched payload
        output.extend_from_slice(&data[min_len..]);

        Ok(output)
    }

    /// Decrypts from a reader to a writer using buffered streaming.
    ///
    /// This operates with constant O(1) memory overhead and is capable of processing multi-gigabyte files.
    pub fn decrypt_stream<R: Read, W: Write>(
        &self,
        mut reader: R,
        mut writer: W,
        restore_png: bool,
    ) -> Result<u64> {
        let hlen = self.header_config.header_len;

        // 1. Read and validate fake header
        let mut fake_header = vec![0u8; hlen];
        reader
            .read_exact(&mut fake_header)
            .map_err(|_| DecryptionError::FileTooShort {
                min_expected: hlen,
                found: 0,
            })?;

        self.header_config.validate_fake_header(&fake_header)?;

        // 2. Read masked header block
        let mut masked_block = vec![0u8; hlen];
        reader
            .read_exact(&mut masked_block)
            .map_err(|_| DecryptionError::FileTooShort {
                min_expected: hlen * 2,
                found: hlen,
            })?;

        // 3. Write unmasked bytes
        if restore_png {
            let restore_len = PNG_HEADER.len().min(hlen);
            writer.write_all(&PNG_HEADER[..restore_len])?;
            if hlen > restore_len {
                let padding = vec![0u8; hlen - restore_len];
                writer.write_all(&padding)?;
            }
        } else {
            let key = self.key.as_ref().ok_or(DecryptionError::MissingKey)?;
            let key_bytes = key.as_bytes();

            for i in 0..hlen {
                masked_block[i] ^= key_bytes[i % key_bytes.len()];
            }
            writer.write_all(&masked_block)?;
        }

        // 4. Stream remainder of the file in high-throughput buffered chunks
        let remainder_copied = std::io::copy(&mut reader, &mut writer)?;

        Ok((hlen as u64) + remainder_copied)
    }

    /// Decrypts an in-memory mutable slice in-place without heap allocations.
    /// Returns the length of valid decrypted data in `data[..len]`.
    pub fn decrypt_slice_in_place(&self, data: &mut [u8], restore_png: bool) -> Result<usize> {
        let hlen = self.header_config.header_len;
        let min_len = hlen * 2;

        if data.len() < min_len {
            return Err(DecryptionError::FileTooShort {
                min_expected: min_len,
                found: data.len(),
            });
        }

        // Validate fake header
        self.header_config.validate_fake_header(&data[..hlen])?;

        if restore_png {
            let restore_len = PNG_HEADER.len().min(hlen);
            data[hlen..hlen + restore_len].copy_from_slice(&PNG_HEADER[..restore_len]);
            if hlen > restore_len {
                data[hlen + restore_len..min_len].fill(0);
            }
        } else {
            let key = self.key.as_ref().ok_or(DecryptionError::MissingKey)?;
            let key_bytes = key.as_bytes();

            for i in 0..hlen {
                data[hlen + i] ^= key_bytes[i % key_bytes.len()];
            }
        }

        // Shift unmasked data to index 0 (discard fake header)
        data.copy_within(hlen.., 0);
        Ok(data.len() - hlen)
    }

    /// Decrypts a file on disk directly to an output destination path.
    /// Uses an atomic temporary file write and rename to guarantee crash-resilience
    /// and prevent partial writes or accidental source file truncation.
    pub fn decrypt_file(&self, src: &Path, dst: &Path, restore_png: bool) -> Result<u64> {
        let input_file = File::open(src).map_err(|e| DecryptionError::io_with_path(src, e))?;
        let reader = BufReader::with_capacity(128 * 1024, input_file);

        let parent = dst.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent).map_err(|e| DecryptionError::io_with_path(parent, e))?;

        let temp_file = tempfile::NamedTempFile::new_in(parent)
            .map_err(|e| DecryptionError::io_with_path(parent, e))?;
        let mut writer = BufWriter::with_capacity(128 * 1024, temp_file);

        let bytes_written = self.decrypt_stream(reader, &mut writer, restore_png)?;

        let temp_file = writer
            .into_inner()
            .map_err(|e| DecryptionError::io_with_path(dst, e.into_error()))?;

        temp_file
            .persist(dst)
            .map_err(|e| DecryptionError::io_with_path(dst, e.error))?;

        Ok(bytes_written)
    }

    /// Encrypts an in-memory byte slice into RPG Maker format.
    pub fn encrypt_bytes(&self, data: &[u8]) -> Result<Vec<u8>> {
        let hlen = self.header_config.header_len;
        if data.len() < hlen {
            return Err(DecryptionError::FileTooShort {
                min_expected: hlen,
                found: data.len(),
            });
        }

        let key = self.key.as_ref().ok_or(DecryptionError::MissingKey)?;
        let key_bytes = key.as_bytes();

        let fake_header = self.header_config.build_fake_header();
        let mut output = Vec::with_capacity(fake_header.len() + data.len());

        // Prepend fake header
        output.extend_from_slice(&fake_header);

        // Mask first header_len bytes
        for i in 0..hlen {
            output.push(data[i] ^ key_bytes[i % key_bytes.len()]);
        }

        // Append remainder untouched
        output.extend_from_slice(&data[hlen..]);

        Ok(output)
    }

    /// Encrypts from a reader to a writer using buffered streaming.
    pub fn encrypt_stream<R: Read, W: Write>(&self, mut reader: R, mut writer: W) -> Result<u64> {
        let hlen = self.header_config.header_len;
        let key = self.key.as_ref().ok_or(DecryptionError::MissingKey)?;
        let key_bytes = key.as_bytes();

        // 1. Read first block
        let mut first_block = vec![0u8; hlen];
        reader
            .read_exact(&mut first_block)
            .map_err(|_| DecryptionError::FileTooShort {
                min_expected: hlen,
                found: 0,
            })?;

        // 2. Write fake header
        let fake_header = self.header_config.build_fake_header();
        writer.write_all(&fake_header)?;

        // 3. Mask and write first block
        for i in 0..hlen {
            first_block[i] ^= key_bytes[i % key_bytes.len()];
        }
        writer.write_all(&first_block)?;

        // 4. Stream rest
        let remainder_copied = std::io::copy(&mut reader, &mut writer)?;

        Ok((fake_header.len() + hlen) as u64 + remainder_copied)
    }

    /// Encrypts a file on disk directly to a destination path.
    /// Uses an atomic temporary file write and rename to guarantee crash-resilience.
    pub fn encrypt_file(&self, src: &Path, dst: &Path) -> Result<u64> {
        let input_file = File::open(src).map_err(|e| DecryptionError::io_with_path(src, e))?;
        let reader = BufReader::with_capacity(128 * 1024, input_file);

        let parent = dst.parent().unwrap_or_else(|| Path::new("."));
        fs::create_dir_all(parent).map_err(|e| DecryptionError::io_with_path(parent, e))?;

        let temp_file = tempfile::NamedTempFile::new_in(parent)
            .map_err(|e| DecryptionError::io_with_path(parent, e))?;
        let mut writer = BufWriter::with_capacity(128 * 1024, temp_file);

        let bytes_written = self.encrypt_stream(reader, &mut writer)?;

        let temp_file = writer
            .into_inner()
            .map_err(|e| DecryptionError::io_with_path(dst, e.into_error()))?;

        temp_file
            .persist(dst)
            .map_err(|e| DecryptionError::io_with_path(dst, e.error))?;

        Ok(bytes_written)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roundtrip_encryption_decryption() {
        let original_data =
            b"Hello, this is a test string simulating an image or audio asset payload!";
        let key = EncryptionKey::from_hex("0123456789abcdef0123456789abcdef").unwrap();
        let decrypter = Decrypter::new(Some(key));

        let encrypted = decrypter.encrypt_bytes(original_data).unwrap();
        assert_eq!(encrypted.len(), original_data.len() + 16);

        let decrypted = decrypter.decrypt_bytes(&encrypted, false).unwrap();
        assert_eq!(decrypted, original_data);
    }

    #[test]
    fn test_restore_png_without_key() {
        let original_png_start = PNG_HEADER.to_vec();
        let mut original_data = original_png_start.clone();
        original_data.extend_from_slice(b" additional image data here");

        let key = EncryptionKey::from_hex("abcdef0123456789abcdef0123456789").unwrap();
        let encrypter = Decrypter::new(Some(key));
        let encrypted = encrypter.encrypt_bytes(&original_data).unwrap();

        // Decrypt with restore_png without providing key
        let decrypter_no_key = Decrypter::new(None);
        let restored = decrypter_no_key.decrypt_bytes(&encrypted, true).unwrap();

        assert_eq!(&restored[..16], &PNG_HEADER);
        assert_eq!(restored, original_data);
    }

    #[test]
    fn test_decrypt_slice_in_place() {
        let original_data =
            b"In-place zero copy decryption test payload for performance evaluation!";
        let key = EncryptionKey::from_hex("0123456789abcdef0123456789abcdef").unwrap();
        let decrypter = Decrypter::new(Some(key));

        let encrypted = decrypter.encrypt_bytes(original_data).unwrap();
        let mut in_place_buf = encrypted.clone();

        let len = decrypter
            .decrypt_slice_in_place(&mut in_place_buf, false)
            .unwrap();
        assert_eq!(len, original_data.len());
        assert_eq!(&in_place_buf[..len], original_data.as_slice());
    }
}
