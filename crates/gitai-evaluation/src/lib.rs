//! Evaluation library: duplicate detection and config.rs checks

use rayon::prelude::*;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Debug, Clone, serde::Serialize)]
pub struct DuplicateFileGroup {
    pub content_hash: String,
    pub files: Vec<PathBuf>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ConfigCheckResult {
    pub config_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct FileCount {
    pub path: PathBuf,
    pub count: usize,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ErrorPatternReport {
    pub box_dyn_error_occurrences: usize,
    pub gitai_error_occurrences: usize,
    pub domain_error_occurrences: usize,
    pub git_error_occurrences: usize,
    pub files_box_dyn: Vec<FileCount>,
    pub files_gitai_error: Vec<FileCount>,
    pub files_domain_error: Vec<FileCount>,
    pub files_git_error: Vec<FileCount>,
    // Enhanced consistency fields
    pub adoption_rate: f64,
    pub inconsistent_files: Vec<PathBuf>,
    pub migration_candidates: Vec<PathBuf>,
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct NearDuplicatePair {
    pub a: PathBuf,
    pub b: PathBuf,
    pub similarity: f64, // 0.0 - 1.0
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct EvaluationSummary {
    pub duplicate_groups: Vec<DuplicateFileGroup>,
    pub near_duplicates: Vec<NearDuplicatePair>,
    pub config_check: ConfigCheckResult,
    pub error_patterns: ErrorPatternReport,
}

fn is_rust_source(path: &Path) -> bool {
    path.extension().map(|e| e == "rs").unwrap_or(false)
}

fn file_sha256(path: &Path) -> anyhow::Result<String> {
    let data = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let hash = hasher.finalize();
    Ok(hex::encode(hash))
}

pub fn scan_duplicates(root: &Path) -> anyhow::Result<Vec<DuplicateFileGroup>> {
    let paths: Vec<PathBuf> = WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();

    let hashes: Vec<(String, PathBuf)> = paths
        .par_iter()
        .filter(|p| is_rust_source(p))
        .filter_map(|p| match file_sha256(p) {
            Ok(h) => Some((h, p.clone())),
            Err(_) => None,
        })
        .collect();

    let mut groups: BTreeMap<String, Vec<PathBuf>> = BTreeMap::new();
    for (h, p) in hashes {
        groups.entry(h).or_default().push(p);
    }

    let result = groups
        .into_iter()
        .filter_map(|(h, files)| {
            if files.len() > 1 {
                Some(DuplicateFileGroup {
                    content_hash: h,
                    files,
                })
            } else {
                None
            }
        })
        .collect();

    Ok(result)
}

pub fn check_config_rs(root: &Path) -> anyhow::Result<ConfigCheckResult> {
    let configs: Vec<PathBuf> = WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .filter(|p| p.file_name().map(|n| n == "config.rs").unwrap_or(false))
        .collect();

    Ok(ConfigCheckResult {
        config_paths: configs,
    })
}

pub fn analyze_error_patterns(root: &Path) -> anyhow::Result<ErrorPatternReport> {
    use regex::Regex;

    let re_box_dyn =
        Regex::new(r"Box\s*<\s*dyn\s+(?:std::error::)?Error(?:\s*\+\s*Send\s*\+\s*Sync)?\s*>")
            .unwrap();
    let re_gitai_error = Regex::new(r"\bGitAIError\b").unwrap();
    let re_domain_error = Regex::new(r"\bDomainError\b").unwrap();
    let re_git_error = Regex::new(r"\bGitError\b").unwrap();

    let files: Vec<PathBuf> = WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .filter(|p| is_rust_source(p))
        .collect();

    let per_file: Vec<(PathBuf, usize, usize, usize, usize)> = files
        .par_iter()
        .filter_map(|p| {
            let content = fs::read_to_string(p).ok()?;
            let c_box = re_box_dyn.find_iter(&content).count();
            let c_gitai = re_gitai_error.find_iter(&content).count();
            let c_domain = re_domain_error.find_iter(&content).count();
            let c_git = re_git_error.find_iter(&content).count();
            Some((p.clone(), c_box, c_gitai, c_domain, c_git))
        })
        .collect();

    let mut files_box_dyn: Vec<FileCount> = Vec::new();
    let mut files_gitai_error: Vec<FileCount> = Vec::new();
    let mut files_domain_error: Vec<FileCount> = Vec::new();
    let mut files_git_error: Vec<FileCount> = Vec::new();

    let mut total_box = 0usize;
    let mut total_gitai = 0usize;
    let mut total_domain = 0usize;
    let mut total_git = 0usize;

    use std::collections::HashSet;
    let mut any_error_files: HashSet<PathBuf> = HashSet::new();
    let mut gitai_error_files: HashSet<PathBuf> = HashSet::new();
    let mut migration_candidates: HashSet<PathBuf> = HashSet::new();
    let mut inconsistent_files: HashSet<PathBuf> = HashSet::new();

    for (p, cb, cg, cd, cgit) in per_file {
        let mut has_any = false;
        if cb > 0 {
            files_box_dyn.push(FileCount {
                path: p.clone(),
                count: cb,
            });
            total_box += cb;
            has_any = true;
            migration_candidates.insert(p.clone());
        }
        if cg > 0 {
            files_gitai_error.push(FileCount {
                path: p.clone(),
                count: cg,
            });
            total_gitai += cg;
            has_any = true;
            gitai_error_files.insert(p.clone());
        }
        if cd > 0 {
            files_domain_error.push(FileCount {
                path: p.clone(),
                count: cd,
            });
            total_domain += cd;
            has_any = true;
        }
        if cgit > 0 {
            files_git_error.push(FileCount {
                path: p.clone(),
                count: cgit,
            });
            total_git += cgit;
            has_any = true;
        }
        // Inconsistency: mixing Box<dyn Error> with typed errors in same file
        if cb > 0 && (cg > 0 || cd > 0 || cgit > 0) {
            inconsistent_files.insert(p.clone());
        }
        if has_any {
            any_error_files.insert(p);
        }
    }

    // Sort descending by count
    files_box_dyn.sort_by(|a, b| b.count.cmp(&a.count));
    files_gitai_error.sort_by(|a, b| b.count.cmp(&a.count));
    files_domain_error.sort_by(|a, b| b.count.cmp(&a.count));
    files_git_error.sort_by(|a, b| b.count.cmp(&a.count));

    let adoption_rate = if any_error_files.is_empty() {
        0.0
    } else {
        (gitai_error_files.len() as f64) / (any_error_files.len() as f64)
    };

    // Build sorted consistency vectors
    let mut inconsistent_files: Vec<PathBuf> = inconsistent_files.into_iter().collect();
    inconsistent_files.sort();
    let mut migration_candidates: Vec<PathBuf> = migration_candidates.into_iter().collect();
    migration_candidates.sort();

    Ok(ErrorPatternReport {
        box_dyn_error_occurrences: total_box,
        gitai_error_occurrences: total_gitai,
        domain_error_occurrences: total_domain,
        git_error_occurrences: total_git,
        files_box_dyn,
        files_gitai_error,
        files_domain_error,
        files_git_error,
        adoption_rate,
        inconsistent_files,
        migration_candidates,
    })
}

fn compute_similarity(a: &str, b: &str) -> f64 {
    // Use dissimilar diff to approximate similarity by equal chunk ratio
    let chunks = dissimilar::diff(a, b);
    let mut equal_len = 0usize;
    let total = a.len() + b.len();
    for ch in chunks {
        match ch {
            dissimilar::Chunk::Equal(s) => equal_len += s.len() * 2, // counts towards both a and b
            _ => {}
        }
    }
    if total == 0 {
        1.0
    } else {
        (equal_len as f64) / (total as f64)
    }
}

pub fn find_near_duplicates(root: &Path) -> anyhow::Result<Vec<NearDuplicatePair>> {
    use std::collections::HashMap;

    let files: Vec<PathBuf> = WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .filter(|p| is_rust_source(p))
        .collect();

    // Read contents
    let mut contents: Vec<(PathBuf, String)> = Vec::with_capacity(files.len());
    for p in files {
        if let Ok(c) = fs::read_to_string(&p) {
            contents.push((p, c));
        }
    }

    // Group by file name to prune comparisons
    let mut groups: HashMap<String, Vec<usize>> = HashMap::new();
    for (idx, (p, _)) in contents.iter().enumerate() {
        let name = p
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("")
            .to_string();
        groups.entry(name).or_default().push(idx);
    }

    // Generate candidate pairs within each group (cap sizes to avoid explosion)
    let mut candidates: Vec<(usize, usize)> = Vec::new();
    for (_name, idxs) in groups.into_iter() {
        if idxs.len() < 2 {
            continue;
        }
        let capped: Vec<usize> = if idxs.len() > 30 {
            idxs.into_iter().take(30).collect()
        } else {
            idxs
        };
        for i in 0..capped.len() {
            for j in (i + 1)..capped.len() {
                candidates.push((capped[i], capped[j]));
            }
        }
    }

    // Compute similarities in parallel with filters
    let pairs: Vec<NearDuplicatePair> = candidates
        .par_iter()
        .filter_map(|(ia, ib)| {
            let (pa, ca) = &contents[*ia];
            let (pb, cb) = &contents[*ib];
            let la = ca.len();
            let lb = cb.len();
            let max_len = la.max(lb) as f64;
            if max_len == 0.0 {
                return None;
            }
            // Skip exact duplicates (already reported via hash groups)
            if ca == cb {
                return None;
            }
            // Quick length filter
            let len_ratio = (la as f64) / (lb as f64);
            if len_ratio < 0.8 || len_ratio > 1.25 {
                return None;
            }
            let sim = compute_similarity(ca, cb);
            if sim >= 0.85 {
                Some(NearDuplicatePair {
                    a: pa.clone(),
                    b: pb.clone(),
                    similarity: (sim * 1000.0).round() / 1000.0,
                })
            } else {
                None
            }
        })
        .collect();

    // Keep top N
    let mut pairs = pairs;
    pairs.sort_by(|x, y| {
        y.similarity
            .partial_cmp(&x.similarity)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    if pairs.len() > 200 {
        pairs.truncate(200);
    }
    Ok(pairs)
}

pub fn evaluate(root: &Path) -> anyhow::Result<EvaluationSummary> {
    let duplicate_groups = scan_duplicates(root)?;
    let near_duplicates = find_near_duplicates(root)?;
    let config_check = check_config_rs(root)?;
    let error_patterns = analyze_error_patterns(root)?;
    Ok(EvaluationSummary {
        duplicate_groups,
        near_duplicates,
        config_check,
        error_patterns,
    })
}
