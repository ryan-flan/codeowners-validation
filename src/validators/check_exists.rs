use globset::GlobSetBuilder;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::parser::CodeOwnerRule;

pub struct ValidationResult {
    pub matched: bool,
    pub owners: Vec<String>,
    pub original_path: String,
}

pub fn validate_directory(
    path: &Path,
    rules: Vec<CodeOwnerRule>,
) -> Result<Vec<CodeOwnerRule>, Box<dyn std::error::Error>> {
    let mut builder = GlobSetBuilder::new();
    let mut rule_map: HashMap<String, (CodeOwnerRule, bool)> = HashMap::new();

    for rule in &rules {
        builder.add(rule.glob.clone());
        rule_map.insert(rule.pattern.clone(), (rule.clone(), false));
    }

    let globset = builder.build()?;

    for entry in WalkDir::new(path)
        .follow_links(true)
        .into_iter()
        .filter_map(Result::ok)
    {
        let entry_path = entry.path();
        let normalized_path = normalize_path(entry_path);

        if globset.is_match(&normalized_path) {
            let matching_rules: Vec<_> = globset
                .matches(&normalized_path)
                .into_iter()
                .map(|index| &rules[index])
                .collect();

            for matching_rule in matching_rules {
                if let Some(result) = rule_map.get_mut(&matching_rule.pattern) {
                    result.1 = true;
                }
            }
        }

        if rule_map.values().all(|result| result.1) {
            break;
        }
    }
    let failures: Vec<CodeOwnerRule> = rule_map
        .into_iter()
        .filter(|(_, (_, is_valid))| !is_valid)
        .map(|(_, (rule, _))| rule)
        .collect();

    Ok(failures)
}

fn normalize_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    let mut normalized_path = path.to_path_buf();

    if normalized_path.to_str().unwrap().ends_with('/') {
        normalized_path.pop();
    }

    normalized_path
}
