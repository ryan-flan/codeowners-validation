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
    use std::fs;
    use tempfile::tempdir;

    fn rule(pattern: &str) -> CodeOwnerRule {
        CodeOwnerRule {
            pattern: pattern.to_string(),
            original_path: pattern.to_string(),
            owners: vec!["@team".to_string()],
            glob: globset::Glob::new(&format!("**/{}", pattern)).unwrap(),
        }
    }

    #[test]
    fn detects_missing_file() {
        let tmp = tempdir().unwrap();
        let rules = vec![rule("missing.txt")];
        let result = validate_directory(tmp.path(), &rules).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].pattern, "missing.txt");
    }

    #[test]
    fn passes_existing_file() {
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("exists.txt");
        fs::write(&file_path, "content").unwrap();
        let rules = vec![rule("exists.txt")];
        let result = validate_directory(tmp.path(), &rules).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn matches_wildcard_files() {
        let tmp = tempdir().unwrap();
        fs::write(tmp.path().join("foo.md"), "docs").unwrap();
        let mut r = rule("*.md");
        r.glob = globset::Glob::new("**/*.md").unwrap();
        let rules = vec![r];
        let result = validate_directory(tmp.path(), &rules).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn detects_unmatched_wildcards() {
        let tmp = tempdir().unwrap();
        let rules = vec![rule("*.xyz")];
        let result = validate_directory(tmp.path(), &rules).unwrap();
        assert_eq!(result.len(), 1);
    }
}
