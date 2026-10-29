use crate::core::error::Result;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Copy)]
pub struct OptimizationResult {
    pub optimized: bool,
    pub original_size: u64,
    pub new_size: u64,
}

pub fn optimize_png_file(path: &Path, preset: u8) -> Result<OptimizationResult> {
    let original_size = fs::metadata(path).map(|m| m.len()).unwrap_or(0);

    let mut opts = oxipng::Options::from_preset(preset.clamp(1, 6));
    opts.strip = oxipng::StripChunks::Safe;
    // Single-threaded execution per image: oxipng is compiled with default-features = false
    // so it executes single-threaded without nested Rayon oversubscription (N x M threads).

    let res = oxipng::optimize(
        &oxipng::InFile::Path(path.to_path_buf()),
        &oxipng::OutFile::Path {
            path: Some(path.to_path_buf()),
            preserve_attrs: true,
        },
        &opts,
    );

    let new_size = fs::metadata(path).map(|m| m.len()).unwrap_or(original_size);
    let optimized = res.is_ok() && new_size < original_size;

    Ok(OptimizationResult {
        optimized,
        original_size,
        new_size,
    })
}
