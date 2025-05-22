use crate::parser::CodeOwnerRule;
use std::collections::HashSet;

pub(crate) fn validate_duplicates(rules: &[CodeOwnerRule]) -> Vec<CodeOwnerRule> {
    let mut pattern_set = HashSet::new();
    let mut original_path_set = HashSet::new();
    let mut duplicates = Vec::new();

    for rule in rules {
        let is_original_path_duplicate = !original_path_set.insert(&rule.original_path);
        let is_pattern_duplicate = !pattern_set.insert(&rule.pattern);

        if is_original_path_duplicate || is_pattern_duplicate {
            duplicates.push(rule.clone());
            // Only print a warning if it is the normalized duplicate.
            if !is_original_path_duplicate && is_pattern_duplicate {
                println!("Warning: Duplicate pattern found in the tools mutated pattern");
                println!("Please raise an issue if this seems incorrect.");
                println!("Pattern: {}", &rule.pattern);
                println!("Original: {}", &rule.original_path);
            }
        }
    }

    duplicates
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::CodeOwnerRule;
    use globset::Glob;

    fn rule(pattern: &str) -> CodeOwnerRule {
        CodeOwnerRule {
            pattern: pattern.to_string(),
            original_path: pattern.to_string(),
            owners: vec!["@owner".to_string()],
            glob: Glob::new(&format!("**/{}", pattern)).unwrap(),
        }
    }

    #[test]
    fn no_duplicates() {
        let rules = vec![rule("a.txt"), rule("b.txt")];
        let result = validate_duplicates(&rules);
        assert!(result.is_empty());
    }

    #[test]
    fn detects_exact_duplicates() {
        let rules = vec![rule("src/"), rule("src/")];
        let result = validate_duplicates(&rules);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].pattern, "src/");
    }

    #[test]
    fn normalized_duplicates_detected() {
        let mut a = rule("/docs");
        a.pattern = "docs".to_string(); // simulate normalization ("/docs" turns into "docs")
        let b = rule("docs");
        let result = validate_duplicates(&[a, b]);
        assert_eq!(result.len(), 1);
    }
}
