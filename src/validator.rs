use crate::parser::CodeOwnerRule;
use crate::validators::check_exists::validate_directory;
use crate::validators::duplicate_patterns::validate_duplicates;
use clap::{ArgAction, Args};
use std::path::Path;

#[derive(Args, Debug, Clone, Default)]
pub struct ValidatorArgs {
    /// Run the check_exists validator
    #[arg(long, action = ArgAction::SetTrue, default_value_t = false)]
    pub check_exists: bool,

    /// Run the duplicate_patterns validator
    #[arg(long, action = ArgAction::SetTrue, default_value_t = false)]
    pub duplicate_patterns: bool,
}

impl ValidatorArgs {
    pub fn should_run_all(&self) -> bool {
        !self.check_exists && !self.duplicate_patterns
    }
}

pub fn run_validator(
    args: &ValidatorArgs,
    rules: &[CodeOwnerRule],
) -> Vec<(String, CodeOwnerRule)> {
    let mut failed_rules = Vec::new();

    let validators: Vec<(&str, fn(&[CodeOwnerRule]) -> Vec<CodeOwnerRule>)> = vec![
        ("check_exists", |rules| {
            let repo_dir = Path::new(".");
            validate_directory(repo_dir, rules.to_vec()).unwrap_or_default()
        }),
        ("duplicate_patterns", validate_duplicates),
    ];

    for (name, validator_fn) in validators {
        if args.should_run_all()
            || (name == "check_exists" && args.check_exists)
            || (name == "duplicate_patterns" && args.duplicate_patterns)
        {
            for rule in validator_fn(rules) {
                failed_rules.push((name.to_string(), rule));
            }
        }
    }

    failed_rules
}
