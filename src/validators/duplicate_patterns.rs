use crate::parser::CodeOwnerRule;
use std::collections::HashSet;

pub(crate) fn validate_duplicates(rules: &[CodeOwnerRule]) -> Vec<CodeOwnerRule> {
    let mut pattern_set = HashSet::new();
    let mut original_path_set = HashSet::new();
    let mut duplicates = Vec::new();

    for rule in rules {
        let is_original_path_duplicate = !original_path_set.insert(&rule.original_path);
        let is_pattern_duplicate = !pattern_set.insert(&rule.pattern);

        if is_original_path_duplicate {
            duplicates.push(rule.clone());
        } else if is_pattern_duplicate {
            println!("Warning: Duplicate pattern found in the tools mutated pattern");
            println!("Please raise an issue if this seems incorrect.");
            println!("Pattern: {}", &rule.pattern);
            println!("Original: {}", &rule.original_path);
        }
    }

    duplicates
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validators::duplicate_patterns::validate_duplicates;
    use globset::Glob;

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
        ]
    }

    #[test]
    fn test_validate_duplicates_no_duplicates() {
        let rules = create_rules();

        let result = validate_duplicates(&rules);

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_validate_duplicates_with_duplicates() {
        let mut rules = create_rules();

        // Add duplicate rules
        rules.push(rules[0].clone());
        rules.push(rules[1].clone());

        let result = validate_duplicates(&rules);

        assert_eq!(result.len(), 2);
        assert!(result.contains(&rules[0]));
        assert!(result.contains(&rules[1]));
    }
}
