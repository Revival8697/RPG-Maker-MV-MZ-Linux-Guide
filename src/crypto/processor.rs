//! Parallel file traversal, batch decryption, and progress tracking using Rayon.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::crypto::engine::Decrypter;
use crate::crypto::error::{DecryptionError, Result};
use crate::crypto::extensions::{
    decrypted_path, encrypted_path, is_encrypted_extension, is_encrypted_image,
    is_encryption_excluded,
};

/// Thread-safe callback invoked during batch processing with `(completed_files, total_files)`.
pub type ProgressCallback = Arc<dyn Fn(usize, usize) + Send + Sync>;

/// Information about a single file processing failure.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FailedFile {
    pub path: PathBuf,
    pub error: String,
}

/// Configuration options for the file batch processor.
#[derive(Clone)]
pub struct ProcessorOptions {
    /// Destination directory or file path for output.
    pub output_dir: Option<PathBuf>,
    /// Whether to write output files directly in the source directory next to original files.
    pub in_place: bool,
    /// Whether to overwrite existing files at the destination.
    pub overwrite: bool,
    /// If true, only process encrypted images and restore them using the standard PNG header (no key needed).
    pub restore_images: bool,
    /// Maximum number of worker threads for parallel processing. Defaults to logical CPU count.
    pub worker_threads: Option<usize>,
    /// Optional progress callback hook.
    pub progress_callback: Option<ProgressCallback>,
}

impl Default for ProcessorOptions {
    fn default() -> Self {
        Self {
            output_dir: None,
            in_place: false,
            overwrite: true,
            restore_images: false,
            worker_threads: None,
            progress_callback: None,
        }
    }
}

impl std::fmt::Debug for ProcessorOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProcessorOptions")
            .field("output_dir", &self.output_dir)
            .field("in_place", &self.in_place)
            .field("overwrite", &self.overwrite)
            .field("restore_images", &self.restore_images)
            .field("worker_threads", &self.worker_threads)
            .field("progress_callback", &self.progress_callback.is_some())
            .finish()
    }
}

/// Statistics collected during a batch processing run.
#[derive(Debug, Default)]
pub struct BatchStats {
    pub files_processed: AtomicUsize,
    pub files_skipped: AtomicUsize,
    pub files_failed: AtomicUsize,
    pub bytes_written: AtomicU64,
    pub failures: Mutex<Vec<FailedFile>>,
}

/// Final summary report of a batch processing operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessSummary {
    pub files_processed: usize,
    pub files_skipped: usize,
    pub files_failed: usize,
    pub bytes_written: u64,
    #[serde(skip_deserializing, serialize_with = "serialize_duration")]
    pub elapsed: Duration,
    pub elapsed_ms: u128,
    pub throughput_mbps: f64,
    pub failed_files: Vec<FailedFile>,
}

fn serialize_duration<S>(d: &Duration, s: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    s.serialize_str(&format!("{:.2?}", d))
}

/// Batch file processor for decrypting, restoring, and encrypting assets.
pub struct FileProcessor<'a> {
    engine: &'a Decrypter,
    options: ProcessorOptions,
}

impl<'a> FileProcessor<'a> {
    /// Creates a new FileProcessor instance.
    pub fn new(engine: &'a Decrypter, options: ProcessorOptions) -> Self {
        Self { engine, options }
    }

    /// Builds a scoped Rayon thread pool if a specific worker thread count is configured.
    fn build_scoped_pool(&self) -> Result<Option<rayon::ThreadPool>> {
        if let Some(threads) = self.options.worker_threads {
            if threads > 0 {
                let pool = rayon::ThreadPoolBuilder::new()
                    .num_threads(threads)
                    .build()
                    .map_err(|e| {
                        DecryptionError::Other(format!("Failed to build thread pool: {e}"))
                    })?;
                return Ok(Some(pool));
            }
        }
        Ok(None)
    }

    /// Executes the batch decryption workflow over an input path (single file or directory).
    pub fn decrypt_all(&self, input_path: &Path) -> Result<ProcessSummary> {
        if !input_path.exists() {
            return Err(DecryptionError::PathNotFound(input_path.to_path_buf()));
        }

        let pool = self.build_scoped_pool()?;
        let start_time = Instant::now();
        let stats = BatchStats::default();

        if input_path.is_file() {
            // Process a single file directly
            self.process_single_decrypt_file(
                input_path,
                input_path.parent().unwrap_or(Path::new("")),
                &stats,
                true,
            );
            if let Some(ref cb) = self.options.progress_callback {
                cb(1, 1);
            }
        } else {
            // Gather all candidate encrypted files
            let candidates = self.discover_encrypted_files(input_path);
            let total = candidates.len();

            let run_parallel = |action: &(dyn Fn() + Sync)| {
                if let Some(ref p) = pool {
                    p.install(action);
                } else {
                    action();
                }
            };

            run_parallel(&|| {
                candidates.par_iter().for_each(|src_file| {
                    self.process_single_decrypt_file(src_file, input_path, &stats, false);
                    if let Some(ref cb) = self.options.progress_callback {
                        let done = stats.files_processed.load(Ordering::Relaxed)
                            + stats.files_skipped.load(Ordering::Relaxed)
                            + stats.files_failed.load(Ordering::Relaxed);
                        cb(done, total);
                    }
                });
            });
        }

        let elapsed = start_time.elapsed();
        let elapsed_ms = elapsed.as_millis();
        let bytes = stats.bytes_written.load(Ordering::Relaxed);
        let secs = elapsed.as_secs_f64();
        let mb = (bytes as f64) / (1024.0 * 1024.0);
        let throughput = if secs > 0.0 { mb / secs } else { 0.0 };
        let failed_files = stats.failures.into_inner().unwrap_or_default();

        Ok(ProcessSummary {
            files_processed: stats.files_processed.load(Ordering::Relaxed),
            files_skipped: stats.files_skipped.load(Ordering::Relaxed),
            files_failed: stats.files_failed.load(Ordering::Relaxed),
            bytes_written: bytes,
            elapsed,
            elapsed_ms,
            throughput_mbps: throughput,
            failed_files,
        })
    }

    /// Discovers all encrypted candidate files in a directory tree.
    fn discover_encrypted_files(&self, root_dir: &Path) -> Vec<PathBuf> {
        WalkDir::new(root_dir)
            .into_iter()
            .filter_entry(|e| {
                if let Ok(rel) = e.path().strip_prefix(root_dir) {
                    if rel
                        .components()
                        .any(|c| c.as_os_str() == "backup" || c.as_os_str() == ".backup")
                    {
                        return false;
                    }
                }
                true
            })
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| {
                let path = e.into_path();
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    if self.options.restore_images {
                        if is_encrypted_image(ext) {
                            return Some(path);
                        }
                    } else if is_encrypted_extension(ext) {
                        return Some(path);
                    }
                }
                None
            })
            .collect()
    }

    /// Processes and decrypts an individual file.
    fn process_single_decrypt_file(
        &self,
        src: &Path,
        root_dir: &Path,
        stats: &BatchStats,
        is_single_file_target: bool,
    ) {
        let dest = self.resolve_output_path(src, root_dir, false, true, is_single_file_target);

        // Check overwrite protection
        if dest.exists() && !self.options.overwrite {
            stats.files_skipped.fetch_add(1, Ordering::Relaxed);
            return;
        }

        let is_image = src
            .extension()
            .and_then(|s| s.to_str())
            .map(is_encrypted_image)
            .unwrap_or(false);

        let restore_png = self.options.restore_images && is_image;

        match self.engine.decrypt_file(src, &dest, restore_png) {
            Ok(bytes) => {
                stats.files_processed.fetch_add(1, Ordering::Relaxed);
                stats.bytes_written.fetch_add(bytes, Ordering::Relaxed);
            }
            Err(err) => {
                if let Ok(mut failures) = stats.failures.lock() {
                    failures.push(FailedFile {
                        path: src.to_path_buf(),
                        error: err.to_string(),
                    });
                }
                stats.files_failed.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Executes batch encryption workflow over plaintext assets in `input_path`.
    pub fn encrypt_all(&self, input_path: &Path, is_mv: bool) -> Result<ProcessSummary> {
        if !input_path.exists() {
            return Err(DecryptionError::PathNotFound(input_path.to_path_buf()));
        }

        let pool = self.build_scoped_pool()?;
        let start_time = Instant::now();
        let stats = BatchStats::default();

        if input_path.is_file() {
            self.process_single_encrypt_file(
                input_path,
                input_path.parent().unwrap_or(Path::new("")),
                is_mv,
                &stats,
                true,
            );
            if let Some(ref cb) = self.options.progress_callback {
                cb(1, 1);
            }
        } else {
            let candidates = self.discover_plaintext_files(input_path);
            let total = candidates.len();

            let run_parallel = |action: &(dyn Fn() + Sync)| {
                if let Some(ref p) = pool {
                    p.install(action);
                } else {
                    action();
                }
            };

            run_parallel(&|| {
                candidates.par_iter().for_each(|src_file| {
                    self.process_single_encrypt_file(src_file, input_path, is_mv, &stats, false);
                    if let Some(ref cb) = self.options.progress_callback {
                        let done = stats.files_processed.load(Ordering::Relaxed)
                            + stats.files_skipped.load(Ordering::Relaxed)
                            + stats.files_failed.load(Ordering::Relaxed);
                        cb(done, total);
                    }
                });
            });
        }

        let elapsed = start_time.elapsed();
        let elapsed_ms = elapsed.as_millis();
        let bytes = stats.bytes_written.load(Ordering::Relaxed);
        let secs = elapsed.as_secs_f64();
        let mb = (bytes as f64) / (1024.0 * 1024.0);
        let throughput = if secs > 0.0 { mb / secs } else { 0.0 };
        let failed_files = stats.failures.into_inner().unwrap_or_default();

        Ok(ProcessSummary {
            files_processed: stats.files_processed.load(Ordering::Relaxed),
            files_skipped: stats.files_skipped.load(Ordering::Relaxed),
            files_failed: stats.files_failed.load(Ordering::Relaxed),
            bytes_written: bytes,
            elapsed,
            elapsed_ms,
            throughput_mbps: throughput,
            failed_files,
        })
    }

    /// Discovers candidate plaintext image and audio files for encryption.
    fn discover_plaintext_files(&self, root_dir: &Path) -> Vec<PathBuf> {
        WalkDir::new(root_dir)
            .into_iter()
            .filter_entry(|e| {
                if let Ok(rel) = e.path().strip_prefix(root_dir) {
                    if rel
                        .components()
                        .any(|c| c.as_os_str() == "backup" || c.as_os_str() == ".backup")
                    {
                        return false;
                    }
                }
                true
            })
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .filter_map(|e| {
                let path = e.into_path();
                if is_encryption_excluded(&path) {
                    return None;
                }
                if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
                    let lower = ext.to_ascii_lowercase();
                    if matches!(lower.as_str(), "png" | "m4a" | "ogg") {
                        return Some(path);
                    }
                }
                None
            })
            .collect()
    }

    /// Processes and encrypts an individual file.
    fn process_single_encrypt_file(
        &self,
        src: &Path,
        root_dir: &Path,
        is_mv: bool,
        stats: &BatchStats,
        is_single_file_target: bool,
    ) {
        let dest = self.resolve_output_path(src, root_dir, true, is_mv, is_single_file_target);

        if dest.exists() && !self.options.overwrite {
            stats.files_skipped.fetch_add(1, Ordering::Relaxed);
            return;
        }

        match self.engine.encrypt_file(src, &dest) {
            Ok(bytes) => {
                stats.files_processed.fetch_add(1, Ordering::Relaxed);
                stats.bytes_written.fetch_add(bytes, Ordering::Relaxed);
            }
            Err(err) => {
                if let Ok(mut failures) = stats.failures.lock() {
                    failures.push(FailedFile {
                        path: src.to_path_buf(),
                        error: err.to_string(),
                    });
                }
                stats.files_failed.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Determines the destination path preserving relative subdirectories,
    /// or directing to a specific output file when target is a single file.
    fn resolve_output_path(
        &self,
        src: &Path,
        root_dir: &Path,
        is_encrypt: bool,
        is_mv: bool,
        is_single_file_target: bool,
    ) -> PathBuf {
        let target_filename = if is_encrypt {
            encrypted_path(src, is_mv)
        } else {
            decrypted_path(src)
        };

        if self.options.in_place || self.options.output_dir.is_none() {
            // Save in place next to original file
            return target_filename;
        }

        let base_out = self.options.output_dir.as_ref().unwrap();

        if is_single_file_target {
            let file_name = target_filename.file_name().unwrap_or_default();
            if base_out.is_dir() {
                base_out.join(file_name)
            } else if base_out.extension().is_some() {
                // User provided an explicit output file path: e.g. -o restored.png
                base_out.to_path_buf()
            } else {
                base_out.join(file_name)
            }
        } else if let Ok(rel) = src.strip_prefix(root_dir) {
            let rel_target = if is_encrypt {
                encrypted_path(rel, is_mv)
            } else {
                decrypted_path(rel)
            };
            base_out.join(rel_target)
        } else {
            base_out.join(target_filename.file_name().unwrap_or_default())
        }
    }
}
