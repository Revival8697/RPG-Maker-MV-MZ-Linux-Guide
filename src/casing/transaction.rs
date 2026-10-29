use crate::casing::parser::extract_all_references;
use crate::casing::resolver::select_canonical_stem;
use crate::core::error::Result;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub const VALID_EXTS: &[&str] = &[
    "png", "rpgmvp", "png_", "ogg", "m4a", "rpgmvo", "rpgmvm", "ogg_", "m4a_", "webm", "mp4",
    "ogv", "efkefc", "efkmat", "efk", "efkmodel",
];

#[derive(Debug, Clone, Default)]
pub struct CasingReport {
    pub total_files_scanned: usize,
    pub engine_references_matched: usize,
    pub files_renamed: usize,
    pub symlinks_created: usize,
    pub renamed_list: Vec<(String, String)>,
    pub symlink_list: Vec<(String, String)>,
    pub verification_passed: bool,
    pub verification_errors: usize,
}

pub fn execute_fix_casing(
    asset_root: &Path,
    dry_run: bool,
    create_symlinks: bool,
) -> Result<CasingReport> {
    let folder_refs = extract_all_references(asset_root);

    let mut renamed_files: Vec<(String, String)> = Vec::new();
    let mut created_symlinks: Vec<(String, String)> = Vec::new();
    let mut total_files_scanned = 0;
    let mut total_matches = 0;
    let pid = std::process::id();
    let mut seq = 0usize;

    for (folder, counter) in &folder_refs {
        let disk_folder = match resolve_ci_dir(asset_root, folder) {
            Some(p) => p,
            None => continue,
        };

        // Group references by lowercase stem
        let mut grouped_refs: HashMap<String, HashMap<String, usize>> = HashMap::new();
        for (ref_name, count) in counter {
            let base = Path::new(ref_name)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            let stem = Path::new(base)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            if !stem.is_empty() {
                let entry = grouped_refs.entry(stem.to_lowercase()).or_default();
                *entry.entry(stem.to_string()).or_insert(0) += count;
            }
        }

        // Walk directory
        for entry in walkdir::WalkDir::new(&disk_folder)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let old_path = entry.path();
            if entry.path_is_symlink() || !entry.file_type().is_file() {
                continue;
            }

            total_files_scanned += 1;
            let file_name = match old_path.file_name().and_then(|s| s.to_str()) {
                Some(s) => s,
                None => continue,
            };

            let disk_stem = match old_path.file_stem().and_then(|s| s.to_str()) {
                Some(s) => s,
                None => continue,
            };
            let disk_ext = match old_path.extension().and_then(|s| s.to_str()) {
                Some(s) => s,
                None => continue,
            };

            let lower_ext = disk_ext.to_lowercase();
            if !VALID_EXTS.contains(&lower_ext.as_str()) {
                continue;
            }

            let lower_key = disk_stem.to_lowercase();
            if let Some(casing_counts) = grouped_refs.get(&lower_key) {
                total_matches += 1;
                let canonical_stem = select_canonical_stem(casing_counts);
                let target_filename = format!("{}.{}", canonical_stem, disk_ext);
                let parent_dir = old_path.parent().unwrap_or(Path::new(""));
                let new_path = parent_dir.join(&target_filename);

                let mut was_renamed = false;
                if file_name != target_filename {
                    // Check if target already exists and is distinct
                    if new_path.exists() && !new_path.is_symlink() {
                        // Check if files have identical contents (or identical decrypted payloads)
                        let is_duplicate = if let (Ok(b1), Ok(b2)) =
                            (fs::read(old_path), fs::read(&new_path))
                        {
                            b1 == b2
                                || (b1.len() == b2.len() && b1.len() > 16 && b1[16..] == b2[16..])
                        } else {
                            false
                        };

                        if is_duplicate {
                            println!("  [*] Deduplicating case-collision: replacing {} with symlink to {}", file_name, target_filename);
                            let rel_old = old_path
                                .strip_prefix(asset_root)
                                .unwrap_or(old_path)
                                .to_string_lossy()
                                .to_string();
                            if !dry_run {
                                let _ = fs::remove_file(old_path);
                                #[cfg(unix)]
                                {
                                    let _ = std::os::unix::fs::symlink(&target_filename, old_path);
                                }
                            }
                            created_symlinks.push((rel_old, target_filename.clone()));
                            continue;
                        }

                        if let (Ok(m1), Ok(m2)) = (fs::metadata(old_path), fs::metadata(&new_path))
                        {
                            #[cfg(unix)]
                            {
                                use std::os::unix::fs::MetadataExt;
                                if m1.ino() != m2.ino() || m1.dev() != m2.dev() {
                                    println!("  [!] Notice: Target {} already exists with distinct contents; preserving both.", target_filename);
                                    continue;
                                }
                            }
                        }
                    }

                    seq += 1;
                    let tmp_filename = format!("{}.tmp_case_{}_{}", file_name, pid, seq);
                    let tmp_path = parent_dir.join(&tmp_filename);

                    let rel_old = old_path
                        .strip_prefix(asset_root)
                        .unwrap_or(old_path)
                        .to_string_lossy()
                        .to_string();
                    let rel_new = new_path
                        .strip_prefix(asset_root)
                        .unwrap_or(&new_path)
                        .to_string_lossy()
                        .to_string();

                    if !dry_run {
                        if let Err(e) = fs::rename(old_path, &tmp_path) {
                            eprintln!("  [!] Error renaming {} to tmp: {}", rel_old, e);
                        } else if let Err(e) = fs::rename(&tmp_path, &new_path) {
                            eprintln!("  [!] Error renaming tmp to {}: {}", rel_new, e);
                            let _ = fs::rename(&tmp_path, old_path);
                        } else {
                            renamed_files.push((rel_old, rel_new));
                            was_renamed = true;
                        }
                    } else {
                        renamed_files.push((rel_old, rel_new));
                        was_renamed = true;
                    }
                }

                // If alternate casing was also declared, or asset was renamed, create case-alias symlink
                if create_symlinks {
                    let mut aliases_to_create: Vec<String> = Vec::new();
                    if was_renamed {
                        // Backwards-compatible alias for the old name so dynamic JS calls resolve cleanly
                        aliases_to_create.push(file_name.to_string());
                    }
                    for declared_casing in casing_counts.keys() {
                        let alias_filename = format!("{}.{}", declared_casing, disk_ext);
                        if alias_filename != target_filename
                            && !aliases_to_create.contains(&alias_filename)
                        {
                            aliases_to_create.push(alias_filename);
                        }
                    }

                    for alias_filename in aliases_to_create {
                        let alias_path = parent_dir.join(&alias_filename);
                        let rel_alias = alias_path
                            .strip_prefix(asset_root)
                            .unwrap_or(&alias_path)
                            .to_string_lossy()
                            .to_string();

                        let mut already_valid = false;
                        if alias_path.is_symlink() {
                            if let Ok(link_target) = fs::read_link(&alias_path) {
                                if link_target == Path::new(&target_filename) {
                                    already_valid = true;
                                }
                            }
                        }

                        if !already_valid {
                            // In dry-run, alias_path may be equal to old_path (which hasn't moved on disk yet).
                            // Only skip if alias_path exists, is not a symlink, and is not old_path.
                            if alias_path.exists()
                                && !alias_path.is_symlink()
                                && (!dry_run || alias_path != old_path)
                            {
                                continue;
                            }

                            if !dry_run {
                                if alias_path.is_symlink() {
                                    let _ = fs::remove_file(&alias_path);
                                }
                                #[cfg(unix)]
                                {
                                    if let Err(e) =
                                        std::os::unix::fs::symlink(&target_filename, &alias_path)
                                    {
                                        eprintln!(
                                            "  [!] Error creating symlink {}: {}",
                                            rel_alias, e
                                        );
                                    } else {
                                        created_symlinks.push((rel_alias, target_filename.clone()));
                                    }
                                }
                            } else {
                                created_symlinks.push((rel_alias, target_filename.clone()));
                            }
                        }
                    }
                }
            }
        }
    }

    // Post-verification
    let mut verification_errors = 0;
    for (folder, counter) in &folder_refs {
        let disk_folder = match resolve_ci_dir(asset_root, folder) {
            Some(p) => p,
            None => continue,
        };

        let mut disk_files: HashSet<String> = HashSet::new();
        for entry in walkdir::WalkDir::new(&disk_folder)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() || entry.path_is_symlink() {
                if let Some(name) = entry.file_name().to_str() {
                    disk_files.insert(name.to_string());
                }
            }
        }

        let disk_files_lower: HashMap<String, String> = disk_files
            .iter()
            .map(|f| (f.to_lowercase(), f.clone()))
            .collect();

        for ref_name in counter.keys() {
            let base = Path::new(ref_name)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            let stem = Path::new(base)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            if stem.is_empty() {
                continue;
            }

            let found_exact = VALID_EXTS.iter().any(|ext| {
                let candidate = format!("{}.{}", stem, ext);
                disk_files.contains(&candidate)
            });

            if !found_exact && !dry_run {
                for ext in VALID_EXTS {
                    let candidate_lower = format!("{}.{}", stem, ext).to_lowercase();
                    if let Some(actual) = disk_files_lower.get(&candidate_lower) {
                        eprintln!("  [!] Verification Failure: '{}' in '{}' still mismatches disk casing '{}'!", ref_name, folder, actual);
                        verification_errors += 1;
                        break;
                    }
                }
            }
        }
    }

    Ok(CasingReport {
        total_files_scanned,
        engine_references_matched: total_matches,
        files_renamed: renamed_files.len(),
        symlinks_created: created_symlinks.len(),
        renamed_list: renamed_files,
        symlink_list: created_symlinks,
        verification_passed: verification_errors == 0,
        verification_errors,
    })
}

pub fn resolve_ci_dir(base_root: &Path, rel_dir: &str) -> Option<PathBuf> {
    let mut current = base_root.to_path_buf();
    let parts: Vec<&str> = rel_dir
        .split(['/', '\\'])
        .filter(|p| !p.is_empty() && *p != ".")
        .collect();

    for part in parts {
        if !current.is_dir() {
            return None;
        }

        let direct = current.join(part);
        if direct.is_dir() {
            current = direct;
            continue;
        }

        let target_lower = part.to_lowercase();
        let mut matched = false;
        if let Ok(entries) = fs::read_dir(&current) {
            for entry in entries.flatten() {
                if let Ok(ty) = entry.file_type() {
                    if ty.is_dir() {
                        if let Some(name) = entry.file_name().to_str() {
                            if name.to_lowercase() == target_lower {
                                current = entry.path();
                                matched = true;
                                break;
                            }
                        }
                    }
                }
            }
        }

        if !matched {
            return None;
        }
    }

    Some(current)
}
