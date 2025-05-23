use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct CodeOwnerRule {
    pub pattern: String, // Normalized pattern (no leading/trailing /)
    pub owners: Vec<String>,
    pub original_path: String, // Original path from file (with / if present)
}

pub struct InvalidLine {
    pub line_number: usize,
    pub content: String,
}

pub fn parse_codeowners_file(
    file_path: &str,
) -> io::Result<(Vec<CodeOwnerRule>, Vec<InvalidLine>)> {
    let file = File::open(file_path)?;
    let reader = BufReader::with_capacity(64 * 1024, file);

    let mut rules = Vec::with_capacity(1000);
    let mut invalid_lines = Vec::new();

    for (line_number, line_result) in reader.lines().enumerate() {
        if let Ok(line) = line_result {
            let trimmed_line = line.trim();
            if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
                // Skip empty lines and comments
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let pattern = parts[0].trim_matches('/').to_string();
            let owners = parts[1..].iter().map(|s| s.to_string()).collect();
            let original_path = parts[0].to_string();

            // Basic validation - ensure pattern is not empty after trimming
            if pattern.is_empty() {
                let invalid_line = InvalidLine {
                    line_number: line_number + 1,
                    content: line,
                };
                invalid_lines.push(invalid_line);
                continue;
            }

            // Check for invalid glob patterns
            if validate_pattern(&pattern, &original_path).is_err() {
                let invalid_line = InvalidLine {
                    line_number: line_number + 1,
                    content: line,
                };
                invalid_lines.push(invalid_line);
                continue;
            }

            let rule = CodeOwnerRule {
                pattern,
                owners,
                original_path,
            };

            rules.push(rule);
        }
    }

    rules.shrink_to_fit();
    invalid_lines.shrink_to_fit();

    Ok((rules, invalid_lines))
}

// Validate that the pattern can be turned into valid globs
fn validate_pattern(pattern: &str, original_path: &str) -> Result<(), &'static str> {
    use globset::Glob;

    let is_anchored = original_path.starts_with('/');
    let is_directory = original_path.ends_with('/');

    // Test that we can create the necessary globs
    match (is_anchored, is_directory) {
        (true, true) => {
            // /docs/ → need to create "docs" and "docs/**"
            Glob::new(pattern).map_err(|_| "invalid pattern")?;
            Glob::new(&format!("{}/**", pattern)).map_err(|_| "invalid pattern")?;
        }
        (true, false) => {
            // /src/file.rs → need to create "src/file.rs"
            Glob::new(pattern).map_err(|_| "invalid pattern")?;
        }
        (false, true) => {
            // lib/ → need to create "**/lib" and "**/lib/**"
            Glob::new(&format!("**/{}", pattern)).map_err(|_| "invalid pattern")?;
            Glob::new(&format!("**/{}/**", pattern)).map_err(|_| "invalid pattern")?;
        }
        (false, false) => {
            // *.rs or file.txt → need to create pattern or "**/pattern"
            if pattern.contains('*') || pattern.contains('?') || pattern.contains('[') {
                Glob::new(pattern).map_err(|_| "invalid pattern")?;
            } else {
                Glob::new(&format!("**/{}", pattern)).map_err(|_| "invalid pattern")?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::write;
    use tempfile::NamedTempFile;

    fn with_temp_codeowners(content: &str) -> NamedTempFile {
        let file = NamedTempFile::new().unwrap();
        write(file.path(), content).unwrap();
        file
    }

    #[test]
    fn parses_valid_lines() {
        let file = with_temp_codeowners("src/lib.rs @alice\n");
        let (rules, invalids) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].pattern, "src/lib.rs");
        assert_eq!(rules[0].owners, vec!["@alice"]);
        assert_eq!(rules[0].original_path, "src/lib.rs");
        assert!(invalids.is_empty());
    }

    #[test]
    fn ignores_comments_and_blanks() {
        let file = with_temp_codeowners("# comment\n\nsrc/main.rs @bob\n");
        let (rules, _) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(rules.len(), 1);
        assert_eq!(rules[0].owners, vec!["@bob"]);
    }

    #[test]
    fn detects_invalid_glob() {
        let file = with_temp_codeowners("docs/[ @bad\n");
        let (_, invalids) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(invalids.len(), 1);
        assert!(invalids[0].content.contains("docs/["));
    }

    #[test]
    fn trims_leading_trailing_slashes() {
        let file = with_temp_codeowners("/foo/ @team\n");
        let (rules, _) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(rules[0].pattern, "foo");
        assert_eq!(rules[0].original_path, "/foo/");
    }

    #[test]
    fn parses_multiple_owners() {
        let file = with_temp_codeowners("src/ @alice @bob\n");
        let (rules, _) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(rules[0].owners, vec!["@alice", "@bob"]);
    }

    #[test]
    fn handles_wildcard_patterns() {
        let file = with_temp_codeowners("*.md @docs-team\n**/*.rs @rust-team\n");
        let (rules, invalids) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(rules.len(), 2);
        assert!(invalids.is_empty());
        assert_eq!(rules[0].pattern, "*.md");
        assert_eq!(rules[1].pattern, "**/*.rs");
    }

    #[test]
    fn handles_anchored_patterns() {
        let file = with_temp_codeowners("/README.md @docs\n/src/ @dev\n");
        let (rules, _) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].pattern, "README.md");
        assert_eq!(rules[0].original_path, "/README.md");
        assert_eq!(rules[1].pattern, "src");
        assert_eq!(rules[1].original_path, "/src/");
    }

    #[test]
    fn rejects_empty_pattern() {
        let file = with_temp_codeowners("/ @team\n");
        let (rules, invalids) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();
        assert_eq!(rules.len(), 0);
        assert_eq!(invalids.len(), 1);
    }
}
