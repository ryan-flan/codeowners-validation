use codeowners_validation::parser::CodeOwnerRule;
use codeowners_validation::validators::check_exists;
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

#[cfg(test)]
mod tests {
    use super::*;
    use globset::Glob;

    #[test]
    fn test_check_file_exists_valid() {
        let repo_dir = tempdir().unwrap();
        let repo_path = repo_dir.path();
        let files = [
            "file1.txt",
            "dir1/file2.txt",
            "dir2/file3.txt",
            "dir2/file4.txt",
        ];

        create_test_files(repo_path, &files);

        let rules = vec![
            CodeOwnerRule {
                pattern: "file1.txt".to_string(),
                owners: vec!["owner1".to_string()],
                original_path: "file1.txt".to_string(),
                glob: Glob::new("**file1.txt").unwrap(),
            },
            CodeOwnerRule {
                pattern: "dir1/file2.txt".to_string(),
                owners: vec!["owner2".to_string()],
                original_path: "dir1/file2.txt".to_string(),
                glob: Glob::new("**dir1/file2.txt").unwrap(),
            },
            CodeOwnerRule {
                pattern: "dir2/*.txt".to_string(),
                owners: vec!["owner3".to_string()],
                original_path: "dir2/*.txt".to_string(),
                glob: Glob::new("**dir2/*.txt").unwrap(),
            },
        ];

        let result = check_exists::validate_directory(repo_path, rules).unwrap();

        assert_eq!(result.len(), 3);
        assert!(result.values().all(|r| r.matched));
    }

    #[test]
    fn test_check_file_exists_invalid() {
        let repo_dir = tempdir().unwrap();
        let repo_path = repo_dir.path();
        let files = ["dir1/file2.txt", "dir2/file3.txt", "dir2/file4.txt"];

        create_test_files(repo_path, &files);

        let rules = vec![
            CodeOwnerRule {
                pattern: "file1.txt".to_string(),
                owners: vec!["owner1".to_string()],
                original_path: "file1.txt".to_string(),
                glob: Glob::new("**file1.txt").unwrap(),
            },
            CodeOwnerRule {
                pattern: "dir1/file2.txt".to_string(),
                owners: vec!["owner2".to_string()],
                original_path: "dir1/file2.txt".to_string(),
                glob: Glob::new("**dir1/file2.txt").unwrap(),
            },
            CodeOwnerRule {
                pattern: "dir2/*.txt".to_string(),
                owners: vec!["owner3".to_string()],
                original_path: "dir2/*.txt".to_string(),
                glob: Glob::new("**dir2/*.txt").unwrap(),
            },
            CodeOwnerRule {
                pattern: "nonexistent.txt".to_string(),
                owners: vec!["owner4".to_string()],
                original_path: "nonexistent.txt".to_string(),
                glob: Glob::new("**nonexistent.txt").unwrap(),
            },
        ];

        let result = check_exists::validate_directory(repo_path, rules).unwrap();

        assert_eq!(result.len(), 4);
        assert!(!result["file1.txt"].matched);
        assert!(result["dir1/file2.txt"].matched);
        assert!(result["dir2/*.txt"].matched);
        assert!(!result["nonexistent.txt"].matched);
    }
}
