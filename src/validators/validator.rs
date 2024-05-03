use crate::parser::CodeOwnerRule;
use crate::validators::check_exists::validate_directory;
use crate::validators::duplicate_patterns::validate_duplicates;
use std::path::Path;
use std::time;

#[derive(Debug, Clone, Default)]
pub struct ValidatorArgs {
    pub check_exists: bool,
    pub duplicate_patterns: bool,
}

impl ValidatorArgs {
    pub fn from_env(args_str: &str) -> Self {
        let mut args = ValidatorArgs::default();

        for arg in args_str.split(',') {
            match arg.trim() {
                "check_exists" => args.check_exists = true,
                "duplicate_patterns" => args.duplicate_patterns = true,
                _ => (),
            }
        }

        args
    }

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
            let now = time::Instant::now();
            for rule in validator_fn(rules) {
                failed_rules.push((name.to_string(), rule));
            }
            println!("{} validation run in {:?}", name, now.elapsed());
        }
    }

    failed_rules
}
