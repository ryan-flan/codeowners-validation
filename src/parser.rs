use globset::Glob;
use std::fs::File;
use std::io::{self, BufRead};

#[derive(Eq, PartialEq, Clone)]
pub struct CodeOwnerRule {
    pub pattern: String,
    pub owners: Vec<String>,
    pub original_path: String,
    pub glob: Glob,
}

pub struct InvalidLine {
    pub line_number: usize,
    pub content: String,
}

pub fn parse_codeowners_file(
    file_path: &str,
) -> io::Result<(Vec<CodeOwnerRule>, Vec<InvalidLine>)> {
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    let mut rules = Vec::new();
    let mut invalid_lines = Vec::new();

    for (line_number, line_result) in reader.lines().enumerate() {
        if let Ok(line) = line_result {
            let trimmed_line = line.trim();
            if trimmed_line.is_empty() || trimmed_line.starts_with('#') {
                // Skip empty lines and comments
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            let pattern = parts[0].trim_matches('/').to_string();
            let owners = parts[1..].iter().map(|s| s.to_string()).collect();
            let original_path = parts[0].to_string();

            let glob = match Glob::new(&format!("**{}", pattern)) {
                Ok(glob) => glob,
                Err(_) => {
                    let invalid_line = InvalidLine {
                        line_number: line_number + 1,
                        content: line,
                    };
                    invalid_lines.push(invalid_line);
                    continue;
                }
            };

            let rule = CodeOwnerRule {
                pattern,
                owners,
                original_path,
                glob,
            };

            rules.push(rule);
        }
    }

    Ok((rules, invalid_lines))
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
}
