use globset::GlobSetBuilder;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::parser::CodeOwnerRule;

pub(super) fn validate_directory(
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validators::exists::validate_directory;
    use globset::Glob;
    use std::fs;
    use tempfile::tempdir;

    fn create_test_files(repo_dir: &Path, files: &[&str]) {
        for file in files {
            let file_path = repo_dir.join(file);
            let parent_dir = file_path.parent().unwrap();
            fs::create_dir_all(parent_dir).unwrap();
            fs::write(file_path, "").unwrap();
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
                pattern: "src/**/modules/api/".to_string(),
                owners: vec!["api-team".to_string()],
                original_path: "src/**/modules/api/".to_string(),
                glob: Glob::new("**/src/**/modules/api").unwrap(),
            },
            CodeOwnerRule {
                pattern: "src/**/utils/*.rs".to_string(),
                owners: vec!["utils-team-1".to_string(), "utils-team-2".to_string()],
                original_path: "src/**/utils/*.rs".to_string(),
                glob: Glob::new("**/src/**/utils/*.rs").unwrap(),
            },
            CodeOwnerRule {
                pattern: "tests/**/integration/".to_string(),
                owners: vec!["integration-team".to_string()],
                original_path: "tests/**/integration/".to_string(),
                glob: Glob::new("**/tests/**/integration").unwrap(),
            },
            CodeOwnerRule {
                pattern: "examples*/".to_string(),
                owners: vec!["examples-team".to_string()],
                original_path: "examples*/".to_string(),
                glob: Glob::new("**/examples*").unwrap(),
            },
        ]
    }

    fn assert_results(result: &[CodeOwnerRule], expected_patterns: &[&str]) {
        let mut sorted_result: Vec<&str> =
            result.iter().map(|rule| rule.pattern.as_str()).collect();
        let mut sorted_expected_patterns: Vec<&str> = expected_patterns.to_vec();

        sorted_result.sort();
        sorted_expected_patterns.sort();

        assert_eq!(sorted_result, sorted_expected_patterns);
    }
    #[test]
    fn test_check_file_exists_happy_path() {
        let repo_dir = tempdir().unwrap();
        let repo_path = repo_dir.path();
        let files = [
            "file.rs",
            "examples/example1.txt",
            "examples/example2.txt",
            "config.rs",
            "src/main/main.rs",
            "src/modules/api/api.rs",
            "src/utils/util1.rs",
            "src/utils/util2.rs",
            "tests/integration/test1.rs",
            "tests/integration/test2.rs",
            "examples1/example1.txt",
            "examples2/example2.txt",
        ];

        create_test_files(repo_path, &files);

        let rules = create_rules();

        let result = validate_directory(repo_path, rules).unwrap();

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_check_file_exists_unhappy_path() {
        let repo_dir = tempdir().unwrap();
        let repo_path = repo_dir.path();
        let files = [
            "file.txt",
            "examples/example1.txt",
            "examples/example2.txt",
            "src/main/main.txt",
            "src/modules/api/api.txt",
            "src/utils/util1.txt",
            "src/utils/util2.txt",
            "tests/unit/test1.txt",
            "tests/unit/test2.txt",
            "examples1/example1.txt",
        ];

        create_test_files(repo_path, &files);

        let rules = create_rules();

        let result = validate_directory(repo_path, rules).unwrap();

        let expected_patterns = [
            "*.rs",
            "config.rs",
            "src/**/main.rs",
            "src/**/utils/*.rs",
            "tests/**/integration/",
        ];

        assert_results(&result, &expected_patterns);
    }
}
