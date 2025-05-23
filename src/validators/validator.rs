use crate::parser::CodeOwnerRule;
use crate::validators::duplicate_patterns::validate_duplicates;
use crate::validators::exists::validate_directory;
use std::path::Path;
use std::time;

#[derive(Debug, Clone, Default)]
pub struct ValidatorArgs {
    pub exists: bool,
    pub duplicate_patterns: bool,
}

impl ValidatorArgs {
    pub fn from_env(args_str: &str) -> Self {
        let mut args = ValidatorArgs::default();

        for arg in args_str.split(',') {
            match arg.trim() {
                "exists" => args.exists = true,
                "duplicate_patterns" => args.duplicate_patterns = true,
                "all" => {
                    args.exists = true;
                    args.duplicate_patterns = true;
                }
                _ => (),
            }
        }

        args
    }

    pub fn should_run_all(&self) -> bool {
        !self.exists && !self.duplicate_patterns
    }
}

type ValidatorFn = fn(&[CodeOwnerRule]) -> Vec<CodeOwnerRule>;

pub fn run_validator(
    args: &ValidatorArgs,
    rules: &[CodeOwnerRule],
) -> Vec<(String, CodeOwnerRule)> {
    let mut failed_rules = Vec::new();

    let validators: Vec<(&str, ValidatorFn)> = vec![
        ("exists", |rules| {
            let repo_dir = Path::new(".");
            match validate_directory(repo_dir, rules) {
                Ok(result) => result,
                Err(err) => {
                    eprintln!("❌ Error during 'exists' validation: {}", err);
                    Vec::new()
                }
            }
        }),
        ("duplicate_patterns", validate_duplicates),
    ];

    for (name, validator_fn) in validators {
        if args.should_run_all()
            || (name == "exists" && args.exists)
            || (name == "duplicate_patterns" && args.duplicate_patterns)
        {
            let now = time::Instant::now();
            let results = validator_fn(rules);
            let num_failures = results.len();

            for rule in results {
                failed_rules.push((name.to_string(), rule));
            }

            println!(
                "✓ {} validation completed in {:?} ({} issues found)",
                name,
                now.elapsed(),
                num_failures
            );
        }
    }

    failed_rules
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::CodeOwnerRule;

    fn rule(pattern: &str, original: &str) -> CodeOwnerRule {
        CodeOwnerRule {
            pattern: pattern.trim_matches('/').to_string(),
            original_path: original.to_string(),
            owners: vec!["@x".to_string()],
        }
    }

    #[test]
    fn runs_all_by_default() {
        let rules = vec![
            rule("missing1.txt", "missing1.txt"),
            rule("dup.txt", "dup.txt"),
            rule("dup.txt", "dup.txt"),
        ];
        let args = ValidatorArgs::default();
        let failures = run_validator(&args, &rules);
        assert!(!failures.is_empty());
    }

    #[test]
    fn runs_only_exists_when_enabled() {
        let rules = vec![rule("notfound.txt", "notfound.txt")];
        let args = ValidatorArgs {
            exists: true,
            duplicate_patterns: false,
        };
        let failures = run_validator(&args, &rules);
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].0, "exists");
    }

    #[test]
    fn runs_only_duplicates_when_enabled() {
        let rules = vec![rule("x.txt", "x.txt"), rule("x.txt", "x.txt")];
        let args = ValidatorArgs {
            exists: false,
            duplicate_patterns: true,
        };
        let failures = run_validator(&args, &rules);
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].0, "duplicate_patterns");
    }

    #[test]
    fn from_env_splits_checks() {
        let args = ValidatorArgs::from_env("exists,duplicate_patterns");
        assert!(args.exists);
        assert!(args.duplicate_patterns);
    }

    #[test]
    fn from_env_handles_all() {
        let args = ValidatorArgs::from_env("all");
        assert!(args.exists);
        assert!(args.duplicate_patterns);
    }

    #[test]
    fn from_env_handles_whitespace() {
        let args = ValidatorArgs::from_env(" exists , duplicate_patterns ");
        assert!(args.exists);
        assert!(args.duplicate_patterns);
    }

    #[test]
    fn should_run_all_when_none_specified() {
        let args = ValidatorArgs::default();
        assert!(args.should_run_all());
    }

    #[test]
    fn not_should_run_all_when_any_specified() {
        let args = ValidatorArgs {
            exists: true,
            duplicate_patterns: false,
        };
        assert!(!args.should_run_all());
    }
}
