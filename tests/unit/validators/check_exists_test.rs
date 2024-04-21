use codeowners_validation::parser::CodeOwnerRule;
use codeowners_validation::validators::check_exists;
use globset::Glob;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
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

fn assert_results(
    result: &HashMap<String, check_exists::ValidationResult>,
    expected_matches: &[bool],
) {
    assert_eq!(result.len(), 8);
    assert_eq!(result["*.rs"].matched, expected_matches[0]);
    assert_eq!(result["examples/*"].matched, expected_matches[1]);
    assert_eq!(result["config.rs"].matched, expected_matches[2]);
    assert_eq!(result["src/**/main.rs"].matched, expected_matches[3]);
    assert_eq!(result["src/**/modules/api/"].matched, expected_matches[4]);
    assert_eq!(result["src/**/utils/*.rs"].matched, expected_matches[5]);
    assert_eq!(result["tests/**/integration/"].matched, expected_matches[6]);
    assert_eq!(result["examples*/"].matched, expected_matches[7]);
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let result = check_exists::validate_directory(repo_path, rules).unwrap();

        assert_results(&result, &[true, true, true, true, true, true, true, true]);
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

        let result = check_exists::validate_directory(repo_path, rules).unwrap();

        assert_results(
            &result,
            &[false, true, false, false, true, false, false, true],
        );
    }
}
