use crate::parser::CodeOwnerRule;
use rustc_hash::FxHashSet;

pub fn validate_duplicates(rules: &[CodeOwnerRule]) -> Vec<CodeOwnerRule> {
    let mut pattern_set = FxHashSet::default();
    let mut original_path_set = FxHashSet::default();

    pattern_set.reserve(rules.len());
    original_path_set.reserve(rules.len());

    let mut duplicates = Vec::new();

    for rule in rules {
        let is_original_path_duplicate = !original_path_set.insert(&rule.original_path);
        let is_pattern_duplicate = !pattern_set.insert(&rule.pattern);

        if is_original_path_duplicate || is_pattern_duplicate {
            duplicates.push(rule.clone());
            // Only print a warning if it is the normalized duplicate.
            if !is_original_path_duplicate && is_pattern_duplicate {
                println!("Warning: Duplicate pattern found in normalized pattern");
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

    fn rule(pattern: &str, original: &str) -> CodeOwnerRule {
        CodeOwnerRule {
            pattern: pattern.trim_matches('/').to_string(),
            original_path: original.to_string(),
            owners: vec!["@owner".to_string()],
        }
    }

    #[test]
    fn no_duplicates() {
        let rules = vec![rule("a.txt", "a.txt"), rule("b.txt", "b.txt")];
        let result = validate_duplicates(&rules);
        assert!(result.is_empty());
    }

    #[test]
    fn detects_exact_duplicates() {
        let rules = vec![rule("src", "src/"), rule("src", "src/")];
        let result = validate_duplicates(&rules);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].pattern, "src");
    }

    #[test]
    fn normalized_duplicates_detected() {
        // /docs and docs normalize to the same pattern
        let rules = vec![rule("docs", "/docs"), rule("docs", "docs")];
        let result = validate_duplicates(&rules);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].pattern, "docs");
    }

    #[test]
    fn different_slash_variations() {
        // These all normalize to "src/lib"
        let rules = vec![
            rule("src/lib", "/src/lib/"),
            rule("src/lib", "src/lib"),
            rule("src/lib", "/src/lib"),
        ];
        let result = validate_duplicates(&rules);
        // First one is not a duplicate, second and third are
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn original_path_duplicates() {
        // Same original path is always a duplicate
        let rules = vec![rule("docs", "docs/"), rule("docs", "docs/")];
        let result = validate_duplicates(&rules);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn wildcard_duplicates() {
        let rules = vec![rule("*.md", "*.md"), rule("*.md", "*.md")];
        let result = validate_duplicates(&rules);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn complex_pattern_duplicates() {
        let rules = vec![
            rule("**/*.test.js", "**/*.test.js"),
            rule("src/*.rs", "src/*.rs"),
            rule("**/*.test.js", "**/*.test.js"), // duplicate
        ];
        let result = validate_duplicates(&rules);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].pattern, "**/*.test.js");
    }
}
