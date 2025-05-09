use crate::parser::CodeOwnerRule;

pub(crate) fn validate_avoid_shadowing(rules: &[CodeOwnerRule]) -> Vec<CodeOwnerRule> {
    let mut violating_rules = Vec::new();

    for (i, rule) in rules.iter().enumerate() {
        let mut is_violating = false;

        for prev_rule in &rules[..i] {
            let matcher = prev_rule.glob.compile_matcher();
            if matcher.is_match(&rule.original_path) {
                violating_rules.push(rule.clone());
                is_violating = true;
                break;
            }
        }

        if !is_violating {
            for next_rule in &rules[i + 1..] {
                let matcher = rule.glob.compile_matcher();
                if matcher.is_match(&next_rule.original_path) {
                    violating_rules.push(rule.clone());
                    break;
                }
            }
        }
    }

    violating_rules
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::CodeOwnerRule;
    use globset::Glob;

    fn create_rules() -> Vec<CodeOwnerRule> {
        vec![
            CodeOwnerRule {
                pattern: "/build/logs/".to_string(),
                owners: vec!["octocat".to_string()],
                original_path: "/build/logs/".to_string(),
                glob: Glob::new("**/build/logs").unwrap(),
            },
            CodeOwnerRule {
                pattern: "*".to_string(),
                owners: vec!["s1".to_string()],
                original_path: "*".to_string(),
                glob: Glob::new("**/*").unwrap(),
            },
            CodeOwnerRule {
                pattern: "/script/*".to_string(),
                owners: vec!["o2".to_string()],
                original_path: "/script/*".to_string(),
                glob: Glob::new("**/script/*").unwrap(),
            },
            CodeOwnerRule {
                pattern: "/b*/logs".to_string(),
                owners: vec!["s5".to_string()],
                original_path: "/b*/logs".to_string(),
                glob: Glob::new("**/b*/logs").unwrap(),
            },
            CodeOwnerRule {
                pattern: "/b*/other".to_string(),
                owners: vec!["o1".to_string()],
                original_path: "/b*/other".to_string(),
                glob: Glob::new("**/b*/other").unwrap(),
            },
        ]
    }

    #[test]
    fn test_check_avoid_shadowing_with_shadowing() {
        let rules = create_rules();

        let result = validate_avoid_shadowing(&rules);

        assert_eq!(result.len(), 4);
        assert!(result.contains(&rules[1])); // *
        assert!(result.contains(&rules[2])); // /script/*
        assert!(result.contains(&rules[3])); // /b*/logs
        assert!(result.contains(&rules[4])); // /b*/other
    }

    #[test]
    fn test_check_avoid_shadowing_no_shadowing() {
        let rules = vec![
            CodeOwnerRule {
                pattern: "/src/main.rs".to_string(),
                owners: vec!["team-main".to_string()],
                original_path: "/src/main.rs".to_string(),
                glob: Glob::new("**/src/main.rs").unwrap(),
            },
            CodeOwnerRule {
                pattern: "/src/lib.rs".to_string(),
                owners: vec!["team-lib".to_string()],
                original_path: "/src/lib.rs".to_string(),
                glob: Glob::new("**/src/lib.rs").unwrap(),
            },
            CodeOwnerRule {
                pattern: "/tests/*".to_string(),
                owners: vec!["team-tests".to_string()],
                original_path: "/tests/*".to_string(),
                glob: Glob::new("**/tests/*").unwrap(),
            },
        ];

        let result = validate_avoid_shadowing(&rules);

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_check_avoid_shadowing_single_rule() {
        let rules = vec![CodeOwnerRule {
            pattern: "/docs/*".to_string(),
            owners: vec!["team-docs".to_string()],
            original_path: "/docs/*".to_string(),
            glob: Glob::new("**/docs/*").unwrap(),
        }];

        let result = validate_avoid_shadowing(&rules);

        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_check_avoid_shadowing_multiple_shadowing() {
        let rules = vec![
            CodeOwnerRule {
                pattern: "/src/*".to_string(),
                owners: vec!["team-src".to_string()],
                original_path: "/src/*".to_string(),
                glob: Glob::new("**/src/*").unwrap(),
            },
            CodeOwnerRule {
                pattern: "/src/main/*".to_string(),
                owners: vec!["team-main".to_string()],
                original_path: "/src/main/*".to_string(),
                glob: Glob::new("**/src/main/*").unwrap(),
            },
            CodeOwnerRule {
                pattern: "/src/main/core/*".to_string(),
                owners: vec!["team-core".to_string()],
                original_path: "/src/main/core/*".to_string(),
                glob: Glob::new("**/src/main/core/*").unwrap(),
            },
            CodeOwnerRule {
                pattern: "/src/main/core/api/*".to_string(),
                owners: vec!["team-api".to_string()],
                original_path: "/src/main/core/api/*".to_string(),
                glob: Glob::new("**/src/main/core/api/*").unwrap(),
            },
        ];

        let result = validate_avoid_shadowing(&rules);
        assert_eq!(result.len(), 4);
        assert!(result.contains(&rules[0])); // /src/*
        assert!(result.contains(&rules[1])); // /src/main/*
        assert!(result.contains(&rules[2])); // /src/main/core/*
        assert!(result.contains(&rules[3])); // /src/main/core/api/*
    }

    #[test]
    fn test_check_avoid_shadowing_shadowing_and_non_shadowing() {
        let rules = vec![
            CodeOwnerRule {
                pattern: "/src/*".to_string(),
                owners: vec!["team-src".to_string()],
                original_path: "/src/*".to_string(),
                glob: Glob::new("**/src/*").unwrap(),
            },
            CodeOwnerRule {
                pattern: "/tests/*".to_string(),
                owners: vec!["team-tests".to_string()],
                original_path: "/tests/*".to_string(),
                glob: Glob::new("**/tests/*").unwrap(),
            },
            CodeOwnerRule {
                pattern: "/src/main/*".to_string(),
                owners: vec!["team-main".to_string()],
                original_path: "/src/main/*".to_string(),
                glob: Glob::new("**/src/main/*").unwrap(),
            },
            CodeOwnerRule {
                pattern: "/docs/*".to_string(),
                owners: vec!["team-docs".to_string()],
                original_path: "/docs/*".to_string(),
                glob: Glob::new("**/docs/*").unwrap(),
            },
        ];

        let result = validate_avoid_shadowing(&rules);

        assert_eq!(result.len(), 2);
        assert!(result.contains(&rules[0])); // /src/*
        assert!(result.contains(&rules[2])); // /src/main/*
    }
}
