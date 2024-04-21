use globset::GlobSetBuilder;
use rayon::iter::ParallelIterator;
use rayon::prelude::ParallelSlice;
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::parser::CodeOwnerRule;

#[derive(Clone)]
pub struct ValidationResult {
    pub matched: bool,
    pub owners: Vec<String>,
    pub original_path: String,
}

pub fn validate_directory(
    path: &Path,
    rules: Vec<CodeOwnerRule>,
) -> Result<HashMap<String, ValidationResult>, Box<dyn std::error::Error>> {
    let mut builder = GlobSetBuilder::new();
    let mut rule_map = HashMap::new();
    let mut all_patterns = HashSet::new();

    for rule in &rules {
        builder.add(rule.glob.clone());
        all_patterns.insert(rule.pattern.clone());
        rule_map.insert(
            rule.pattern.clone(),
            ValidationResult {
                matched: false,
                owners: rule.owners.clone(),
                original_path: rule.original_path.clone(),
            },
        );
    }

    let globset = builder.build()?;

    let entries: Vec<_> = WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(Result::ok)
        .collect();

    // 5 chunks
    let chunk_size = (entries.len() + 4) / 5;

    let result: HashMap<String, ValidationResult> = entries
        .par_chunks(chunk_size)
        .map(|chunk| {
            let mut chunk_rule_map = rule_map.clone();
            let mut matched_patterns = HashSet::new();

            for entry in chunk {
                let normalized_path = normalize_path(entry.path());

                if globset.is_match(&normalized_path) {
                    let matching_rules: Vec<_> = globset
                        .matches(&normalized_path)
                        .into_iter()
                        .map(|index| &rules[index])
                        .collect();

                    for matching_rule in matching_rules {
                        if let Some(result) = chunk_rule_map.get_mut(&matching_rule.pattern) {
                            if !result.matched {
                                result.matched = true;
                                matched_patterns.insert(matching_rule.pattern.clone());
                            }
                        }
                    }

                    if matched_patterns == all_patterns {
                        break;
                    }
                }
            }

            chunk_rule_map
        })
        .reduce(
            || HashMap::new(),
            |mut acc, chunk_rule_map| {
                for (pattern, result) in chunk_rule_map {
                    acc.entry(pattern)
                        .and_modify(|existing_result| {
                            existing_result.matched |= result.matched;
                        })
                        .or_insert(result);
                }
                acc
            },
        );

    Ok(result)
}

fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    let mut normalized_path = path.to_path_buf();

    if normalized_path.to_str().unwrap().ends_with('/') {
        normalized_path.pop();
    }

    normalized_path
}
