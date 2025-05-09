use crate::parser::CodeOwnerRule;
use globset::{Glob, GlobSetBuilder};
use ignore::{DirEntry, WalkBuilder, WalkState};
use std::error::Error;
use std::path::Path;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

fn build_globset(rules: &[CodeOwnerRule]) -> Result<globset::GlobSet, globset::Error> {
    let mut builder = GlobSetBuilder::new();
    for rule in rules {
        let mut pattern = rule.pattern.clone();
        if pattern.starts_with('/') {
            pattern.remove(0); // Remove leading slash for anchored patterns
        } else {
            pattern = format!("**/{}", pattern); // Match anywhere if not anchored
        }
        if pattern.ends_with('/') {
            pattern.pop(); // Remove trailing slash first
            builder.add(Glob::new(&pattern)?); // Match the directory itself
            builder.add(Glob::new(&format!("{}/**", pattern))?); // Match contents recursively
        } else {
            builder.add(Glob::new(&pattern)?);
        }
    }
    builder.build()
}

pub fn validate_directory(
    repo_path: &Path,
    rules: &[CodeOwnerRule],
) -> Result<Vec<CodeOwnerRule>, Box<dyn Error>> {
    let (direct_rules, wildcard_rules): (Vec<CodeOwnerRule>, Vec<CodeOwnerRule>) =
        rules.iter().cloned().partition(|rule| {
            !rule.pattern.contains('*')
                && !rule.pattern.contains('?')
                && !rule.pattern.contains('[')
                && !rule.pattern.contains(']')
        });

    let mut missing = Vec::new();
    for rule in &direct_rules {
        let full_path = repo_path.join(&rule.pattern);
        if !full_path.exists() {
            missing.push(rule.clone());
        }
    }

    let globset = build_globset(&wildcard_rules)?;
    let num_wildcards = wildcard_rules.len();
    if num_wildcards == 0 {
        return Ok(missing);
    }

    let matched = Arc::new(
        (0..num_wildcards)
            .map(|_| AtomicUsize::new(0))
            .collect::<Vec<_>>()
            .into_boxed_slice(),
    );
    let remaining = Arc::new(AtomicUsize::new(num_wildcards));

    WalkBuilder::new(repo_path)
        .standard_filters(false)
        .build_parallel()
        .run(|| {
            let globset = globset.clone();
            let matched = Arc::clone(&matched);
            let remaining = Arc::clone(&remaining);
            Box::new(move |entry: Result<DirEntry, ignore::Error>| {
                let dir_entry = match entry {
                    Ok(de) => de,
                    Err(_) => return WalkState::Continue,
                };
                let path = dir_entry.path();
                if dir_entry.file_type().map_or(false, |ft| ft.is_dir())
                    && path.file_name().map_or(false, |name| name == ".git")
                {
                    return WalkState::Skip;
                }
                if remaining.load(Ordering::Relaxed) == 0 {
                    return WalkState::Quit;
                }
                let rel_path = path.strip_prefix(repo_path).unwrap_or(path);
                if globset.is_match(rel_path) {
                    for idx in globset.matches(rel_path) {
                        let already = matched[idx].fetch_add(1, Ordering::Relaxed);
                        if already == 0 {
                            remaining.fetch_sub(1, Ordering::Relaxed);
                        }
                    }
                }
                WalkState::Continue
            })
        });

    for (i, rule) in wildcard_rules.into_iter().enumerate() {
        if matched[i].load(Ordering::Relaxed) == 0 {
            missing.push(rule);
        }
    }
    Ok(missing)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::CodeOwnerRule;
    use globset::Glob;
    use std::fs;
    use tempfile::tempdir;

    fn create_test_files(repo_dir: &Path, files: &[&str]) {
        for file in files {
            let file_path = repo_dir.join(file);
            let parent_dir = file_path.parent().unwrap();
            fs::create_dir_all(parent_dir).unwrap();
            if file.ends_with('/') {
                fs::create_dir_all(file_path).unwrap();
            } else {
                fs::write(file_path, b"test").unwrap();
            }
        }
    }

    fn create_rules() -> Vec<CodeOwnerRule> {
        vec![
            CodeOwnerRule {
                pattern: "*.rs".to_string(),
                owners: vec!["rust-team".to_string()],
                original_path: "*.rs".to_string(),
                glob: Glob::new("**/*.rs").unwrap(),
            },
            CodeOwnerRule {
                pattern: "examples/*".to_string(),
                owners: vec!["examples-team".to_string()],
                original_path: "examples/*".to_string(),
                glob: Glob::new("**/examples/*").unwrap(),
            },
            CodeOwnerRule {
                pattern: "config.rs".to_string(),
                owners: vec!["config-team".to_string()],
                original_path: "config.rs".to_string(),
                glob: Glob::new("**/config.rs").unwrap(),
            },
            CodeOwnerRule {
                pattern: "src/**/main.rs".to_string(),
                owners: vec!["main-team".to_string()],
                original_path: "src/**/main.rs".to_string(),
                glob: Glob::new("**/src/**/main.rs").unwrap(),
            },
            CodeOwnerRule {
                pattern: "tests/**/integration/".to_string(),
                owners: vec!["integration-team".to_string()],
                original_path: "tests/**/integration/".to_string(),
                glob: Glob::new("**/tests/**/integration").unwrap(),
            },
        ]
    }

    #[test]
    fn test_validate_directory_happy_path() {
        let repo_dir = tempdir().unwrap();
        let repo_path = repo_dir.path();
        create_test_files(
            repo_path,
            &[
                "src/main/main.rs",
                "config.rs",
                "examples/example1.txt",
                "tests/integration/",
            ],
        );
        let rules = create_rules();
        let result = validate_directory(repo_path, &rules).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_validate_directory_unhappy_path() {
        let repo_dir = tempdir().unwrap();
        let repo_path = repo_dir.path();
        create_test_files(repo_path, &["src/main/main.rs", "examples/example1.txt"]);
        let rules = create_rules();
        let result = validate_directory(repo_path, &rules).unwrap();
        assert_eq!(result.len(), 2);
    }
}
