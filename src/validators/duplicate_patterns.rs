use crate::parser::CodeOwnerRule;
use std::collections::HashSet;

pub(super) fn validate_duplicates(rules: &[CodeOwnerRule]) -> Vec<CodeOwnerRule> {
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
