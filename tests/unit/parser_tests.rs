#[cfg(test)]
mod tests {
    use codeowners_validation::parser::{parse_codeowners_file, CodeOwnerRule};
    use globset::Glob;
    use std::fs;
    use tempfile::TempDir;

    fn setup_test_codeowners_file(content: &str) -> TempDir {
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        let codeowners_file_path = temp_dir.path().join("CODEOWNERS");
        fs::write(&codeowners_file_path, content).expect("Failed to write temp file");
        temp_dir
    }

    #[test]
    fn test_parse_codeowners_file() {
        let codeowners_content = "\
            # Sample CODEOWNERS file\n\
            *.rs @src-team\n\
            src/**/*.rs @src-team\n\
            tests/*.rs @test-team\n\
            ";
        let temp_dir = setup_test_codeowners_file(codeowners_content);
        let codeowners_file_path = temp_dir.path().join("CODEOWNERS");

        let (rules, invalid_lines) =
            parse_codeowners_file(codeowners_file_path.to_str().unwrap()).unwrap();

        let expected_rules = vec![
            CodeOwnerRule {
                pattern: "*.rs".to_string(),
                owners: vec!["@src-team".to_string()],
                original_path: "*.rs".to_string(),
                glob: Glob::new("**/*.rs").unwrap(),
            },
            CodeOwnerRule {
                pattern: "src/**/*.rs".to_string(),
                owners: vec!["@src-team".to_string()],
                original_path: "src/**/*.rs".to_string(),
                glob: Glob::new("**/src/**/*.rs").unwrap(),
            },
            CodeOwnerRule {
                pattern: "tests/*.rs".to_string(),
                owners: vec!["@test-team".to_string()],
                original_path: "tests/*.rs".to_string(),
                glob: Glob::new("**/tests/*.rs").unwrap(),
            },
        ];
        assert_eq!(rules.len(), expected_rules.len());
        for (rule, expected_rule) in rules.iter().zip(expected_rules.iter()) {
            assert_eq!(rule.pattern, expected_rule.pattern);
            assert_eq!(rule.owners, expected_rule.owners);
            assert_eq!(rule.original_path, expected_rule.original_path);
        }
        assert!(invalid_lines.is_empty());
    }
}
