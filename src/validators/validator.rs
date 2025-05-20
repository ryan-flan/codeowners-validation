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
                _ => (),
            }
        }

        args
    }

    pub fn should_run_all(&self) -> bool {
        !self.exists && !self.duplicate_patterns
    }
}

pub fn run_validator(
    args: &ValidatorArgs,
    rules: &[CodeOwnerRule],
) -> Vec<(String, CodeOwnerRule)> {
    let mut failed_rules = Vec::new();

    let validators: Vec<(&str, fn(&[CodeOwnerRule]) -> Vec<CodeOwnerRule>)> = vec![
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
            for rule in results {
                failed_rules.push((name.to_string(), rule));
            }
            println!("{} validation run in {:?}", name, now.elapsed());
        }
    }

    failed_rules
}
