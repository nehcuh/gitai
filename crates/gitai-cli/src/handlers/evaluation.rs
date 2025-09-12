//! CLI handler for evaluation subcommand

use crate::args::Command;
use gitai_evaluation::evaluate;
use std::fs;
use std::path::PathBuf;

/// Handle the `Evaluate` subcommand: run evaluation checks and print results
pub async fn handle_command(cmd: &Command) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Command::Evaluate {
        path,
        format,
        output,
    } = cmd
    {
        let root: PathBuf = path.clone();
        let summary = evaluate(&root)?;

        match format.as_str() {
            "json" => {
                let json = serde_json::to_string_pretty(&summary)?;
                if let Some(out) = output {
                    fs::write(out, json)?;
                } else {
                    println!("{}", json);
                }
            }
            _ => {
                // text
                println!("Evaluation Summary:");
                println!(
                    "- config.rs files: {}",
                    summary.config_check.config_paths.len()
                );
                for p in &summary.config_check.config_paths {
                    println!("  - {}", p.display());
                }
                println!("- duplicate groups: {}", summary.duplicate_groups.len());
                for g in &summary.duplicate_groups {
                    println!("  hash {}: {} files", g.content_hash, g.files.len());
                    for f in &g.files {
                        println!("    - {}", f.display());
                    }
                }
                if !summary.near_duplicates.is_empty() {
                    println!(
                        "- near-duplicate pairs (top {}):",
                        summary.near_duplicates.len().min(10)
                    );
                    for p in summary.near_duplicates.iter().take(10) {
                        println!(
                            "  - {:.1}% similar: {} <-> {}",
                            p.similarity * 100.0,
                            p.a.display(),
                            p.b.display()
                        );
                    }
                } else {
                    println!("- near-duplicate pairs: none above threshold");
                }
                let ep = &summary.error_patterns;
                println!("- error patterns:");
                println!(
                    "  Box<dyn Error> occurrences: {} (files: {})",
                    ep.box_dyn_error_occurrences,
                    ep.files_box_dyn.len()
                );
                println!(
                    "  GitAIError occurrences: {} (files: {})",
                    ep.gitai_error_occurrences,
                    ep.files_gitai_error.len()
                );
                println!(
                    "  DomainError occurrences: {} (files: {})",
                    ep.domain_error_occurrences,
                    ep.files_domain_error.len()
                );
                println!(
                    "  GitError occurrences: {} (files: {})",
                    ep.git_error_occurrences,
                    ep.files_git_error.len()
                );
                println!(
                    "  GitAIError adoption rate: {:.1}%",
                    ep.adoption_rate * 100.0
                );
                println!(
                    "  Inconsistent files (Box<dyn Error> mixed with typed): {}",
                    ep.inconsistent_files.len()
                );
                for p in ep.inconsistent_files.iter().take(10) {
                    println!("    - {}", p.display());
                }
                println!(
                    "  Migration candidates (Box<dyn Error> present): {}",
                    ep.migration_candidates.len()
                );
                for p in ep.migration_candidates.iter().take(10) {
                    println!("    - {}", p.display());
                }
                let show_top = |label: &str, files: &Vec<gitai_evaluation::FileCount>| {
                    let n = files.len().min(5);
                    if n > 0 {
                        println!("  Top {} files:", label);
                        for fc in files.iter().take(5) {
                            println!("    - {} ({} occurrences)", fc.path.display(), fc.count);
                        }
                    }
                };
                show_top("Box<dyn Error>", &ep.files_box_dyn);
                show_top("GitAIError", &ep.files_gitai_error);
                show_top("DomainError", &ep.files_domain_error);
                show_top("GitError", &ep.files_git_error);
            }
        }
        Ok(())
    } else {
        Err("invalid command routing".into())
    }
}
