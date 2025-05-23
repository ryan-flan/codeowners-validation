use crate::parser::CodeOwnerRule;
use globset::{Glob, GlobSetBuilder};
use ignore::{DirEntry, WalkBuilder, WalkState};
use rustc_hash::FxHashMap;
use std::error::Error;
use std::path::Path;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

fn build_globset_with_mapping(
    rules: &[&CodeOwnerRule],
) -> Result<(globset::GlobSet, FxHashMap<usize, usize>), globset::Error> {
    let mut builder = GlobSetBuilder::new();
    let mut globset_idx_to_rule_idx = FxHashMap::default();
    let mut globset_idx = 0;

    for (rule_idx, rule) in rules.iter().enumerate() {
        let pattern = &rule.pattern;
        let is_directory = rule.original_path.ends_with('/');
        let is_anchored = rule.original_path.starts_with('/');

        match (is_anchored, is_directory) {
            (true, true) => {
                // /docs/ → match "docs" and "docs/**"
                builder.add(Glob::new(pattern)?);
                globset_idx_to_rule_idx.insert(globset_idx, rule_idx);
                globset_idx += 1;

                builder.add(Glob::new(&format!("{}/**", pattern))?);
                globset_idx_to_rule_idx.insert(globset_idx, rule_idx);
                globset_idx += 1;
            }
            (true, false) => {
                // /src/file.rs → match "src/file.rs" exactly
                builder.add(Glob::new(pattern)?);
                globset_idx_to_rule_idx.insert(globset_idx, rule_idx);
                globset_idx += 1;
            }
            (false, true) => {
                // lib/ → match "**/lib" and "**/lib/**"
                builder.add(Glob::new(&format!("**/{}", pattern))?);
                globset_idx_to_rule_idx.insert(globset_idx, rule_idx);
                globset_idx += 1;

                builder.add(Glob::new(&format!("**/{}/**", pattern))?);
                globset_idx_to_rule_idx.insert(globset_idx, rule_idx);
                globset_idx += 1;
            }
            (false, false) => {
                // *.rs → match "**/*.rs" (or just pattern if it's already a glob)
                if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
                    // Already a wildcard pattern like *.rs, **/*.md
                    builder.add(Glob::new(pattern)?);
                    globset_idx_to_rule_idx.insert(globset_idx, rule_idx);
                    globset_idx += 1;
                } else {
                    // Plain file like config.json → match "**/config.json"
                    builder.add(Glob::new(&format!("**/{}", pattern))?);
                    globset_idx_to_rule_idx.insert(globset_idx, rule_idx);
                    globset_idx += 1;
                }
            }
        }
    }

    Ok((builder.build()?, globset_idx_to_rule_idx))
}

pub fn validate_directory(
    repo_path: &Path,
    rules: &[CodeOwnerRule],
) -> Result<Vec<CodeOwnerRule>, Box<dyn Error>> {
    // OPTIMIZATION: Pre-allocate with estimated capacity
    let estimated_direct = rules.len() / 3;
    let estimated_wildcard = rules.len() - estimated_direct;

    let mut direct_rules = Vec::with_capacity(estimated_direct);
    let mut wildcard_rules = Vec::with_capacity(estimated_wildcard);

    // Separate direct and wildcard rules
    for rule in rules {
        if rule.pattern.contains('*')
            || rule.pattern.contains('?')
            || rule.pattern.contains('[')
            || rule.pattern.contains(']')
        {
            wildcard_rules.push(rule);
        } else {
            // Direct paths - but we need to check if they're anchored
            // Anchored paths are direct checks, non-anchored need glob matching
            if rule.original_path.starts_with('/') {
                direct_rules.push(rule);
            } else {
                // Non-anchored non-wildcard patterns like "main.rs" need glob matching
                wildcard_rules.push(rule);
            }
        }
    }

    // Check direct paths (fast path for anchored patterns only)
    let mut missing = Vec::new();
    for rule in direct_rules {
        let path = repo_path.join(&rule.pattern);

        if !path.exists() {
            missing.push(rule.clone());
        }
    }

    if wildcard_rules.is_empty() {
        return Ok(missing);
    }

    let (globset, idx_mapping) = build_globset_with_mapping(&wildcard_rules)?;
    let num_wildcards = wildcard_rules.len();

    // OPTIMIZATION: Use atomic array for lock-free tracking
    let matched: Arc<Vec<AtomicUsize>> =
        Arc::new((0..num_wildcards).map(|_| AtomicUsize::new(0)).collect());
    let remaining = Arc::new(AtomicUsize::new(num_wildcards));

    // OPTIMIZATION: Dynamic thread count based on workload
    let thread_count = if num_wildcards > 5000 {
        num_cpus::get().min(8) // More threads for large workloads
    } else if num_wildcards > 1000 {
        num_cpus::get().min(4) // Moderate threads
    } else {
        2 // Minimal threads for small workloads
    };

    WalkBuilder::new(repo_path)
        .standard_filters(false)
        .hidden(false) // Check hidden files too
        .git_ignore(false) // Disable for performance
        .git_global(false)
        .git_exclude(false)
        .threads(thread_count)
        .build_parallel()
        .run(|| {
            let globset = globset.clone();
            let matched = Arc::clone(&matched);
            let remaining = Arc::clone(&remaining);
            let idx_mapping = idx_mapping.clone();

            Box::new(move |entry: Result<DirEntry, ignore::Error>| {
                let dir_entry = match entry {
                    Ok(de) => de,
                    Err(_) => return WalkState::Continue,
                };

                let path = dir_entry.path();

                // Skip .git directory
                if dir_entry.file_type().is_some_and(|ft| ft.is_dir())
                    && path.file_name().is_some_and(|name| name == ".git")
                {
                    return WalkState::Skip;
                }

                // OPTIMIZATION: Early exit check
                if remaining.load(Ordering::Relaxed) == 0 {
                    return WalkState::Quit;
                }

                if let Ok(rel_path) = path.strip_prefix(repo_path) {
                    let matches = globset.matches(rel_path);
                    if !matches.is_empty() {
                        for glob_idx in matches {
                            // Map from globset index to wildcard rule index
                            if let Some(&rule_idx) = idx_mapping.get(&glob_idx) {
                                // Only decrement remaining if this is the first match
                                if matched[rule_idx].fetch_add(1, Ordering::Relaxed) == 0 {
                                    remaining.fetch_sub(1, Ordering::Relaxed);
                                }
                            }
                        }
                    }
                }

                WalkState::Continue
            })
        });

    // Collect unmatched wildcard rules
    for (idx, rule) in wildcard_rules.iter().enumerate() {
        if matched[idx].load(Ordering::Relaxed) == 0 {
            missing.push((*rule).clone());
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

    fn rule(pattern: &str, original: &str) -> CodeOwnerRule {
        CodeOwnerRule {
            pattern: pattern.trim_matches('/').to_string(),
            original_path: original.to_string(),
            owners: vec!["@team".to_string()],
        }
    }

    #[test]
    fn detects_missing_file() {
        let tmp = tempdir().unwrap();
        let rules = vec![rule("missing.txt", "missing.txt")];
        let result = validate_directory(tmp.path(), &rules).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].pattern, "missing.txt");
    }

    #[test]
    fn passes_existing_file() {
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("exists.txt");
        fs::write(&file_path, "content").unwrap();
        let rules = vec![rule("exists.txt", "exists.txt")];
        let result = validate_directory(tmp.path(), &rules).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn matches_wildcard_files() {
        let tmp = tempdir().unwrap();
        fs::write(tmp.path().join("foo.md"), "docs").unwrap();
        let rules = vec![rule("*.md", "*.md")];
        let result = validate_directory(tmp.path(), &rules).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn detects_unmatched_wildcards() {
        let tmp = tempdir().unwrap();
        let rules = vec![rule("*.xyz", "*.xyz")];
        let result = validate_directory(tmp.path(), &rules).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn handles_anchored_patterns() {
        let tmp = tempdir().unwrap();
        // Create src/main.rs
        let src_dir = tmp.path().join("src");
        fs::create_dir(&src_dir).unwrap();
        fs::write(src_dir.join("main.rs"), "fn main() {}").unwrap();

        // Anchored pattern should match
        let rules = vec![rule("src/main.rs", "/src/main.rs")];
        let result = validate_directory(tmp.path(), &rules).unwrap();
        assert!(result.is_empty());

        // Non-anchored pattern should also match
        let rules = vec![rule("main.rs", "main.rs")];
        let result = validate_directory(tmp.path(), &rules).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn handles_directory_patterns() {
        let tmp = tempdir().unwrap();
        let docs_dir = tmp.path().join("docs");
        fs::create_dir(&docs_dir).unwrap();
        fs::write(docs_dir.join("README.md"), "# Docs").unwrap();

        // Directory pattern should match
        let rules = vec![rule("docs", "docs/")];
        let result = validate_directory(tmp.path(), &rules).unwrap();
        assert!(result.is_empty());

        // Anchored directory pattern
        let rules = vec![rule("docs", "/docs/")];
        let result = validate_directory(tmp.path(), &rules).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn handles_nested_patterns() {
        let tmp = tempdir().unwrap();
        // Create nested structure
        let nested = tmp.path().join("a").join("b").join("c");
        fs::create_dir_all(&nested).unwrap();
        fs::write(nested.join("file.txt"), "content").unwrap();

        // Non-anchored should match anywhere
        let rules = vec![rule("file.txt", "file.txt")];
        let result = validate_directory(tmp.path(), &rules).unwrap();
        assert!(result.is_empty());

        // Anchored should not match nested file
        let rules = vec![rule("file.txt", "/file.txt")];
        let result = validate_directory(tmp.path(), &rules).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn handles_complex_wildcards() {
        let tmp = tempdir().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir(&src).unwrap();
        fs::write(src.join("main.test.js"), "test").unwrap();

        // Complex wildcard pattern
        let rules = vec![rule("**/*.test.js", "**/*.test.js")];
        let result = validate_directory(tmp.path(), &rules).unwrap();
        assert!(result.is_empty());
    }
}
