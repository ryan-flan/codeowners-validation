#[cfg(test)]
mod tests {
    use codeowners_validation::parser;

    #[test]
    fn test_parse_codeowners_file() {
        // Create a temporary CODEOWNERS file for testing
        let codeowners_content = "\
            # Sample CODEOWNERS file\n\
            *.rs @src-team\n\
            src/**/*.rs @src-team\n\
            tests/*.rs @test-team\n\
            ";
        let temp_dir = tempfile::tempdir().expect("Failed to create temp directory");
        let codeowners_file_path = temp_dir.path().join("CODEOWNERS");
        std::fs::write(&codeowners_file_path, codeowners_content)
            .expect("Failed to write temp file");

        // Parse the CODEOWNERS file
        let globs = parser::parse_codeowners_file(codeowners_file_path.to_str().unwrap()).unwrap();

        // Assert that the parsed globs are correct
        let expected_globs = vec!["*.rs", "src/**/*.rs", "tests/*.rs"];
        assert_eq!(globs, expected_globs);
    }
}
