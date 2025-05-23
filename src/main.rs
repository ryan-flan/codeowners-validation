use clap::Parser;
use codeowners_validation::parser::parse_codeowners_file;
use codeowners_validation::validators::validator::{run_validator, ValidatorArgs};
use std::{io, path::Path};

#[derive(Parser, Debug)]
#[command(name = "codeowners-validation")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Comma-separated list of checks: exists, duplicate_patterns
    #[arg(long, env = "INPUT_CHECKS", default_value = "all")]
    checks: String,

    /// Path to CODEOWNERS file
    #[arg(long, default_value = ".github/CODEOWNERS")]
    path: String,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let validator_args = ValidatorArgs::from_env(&cli.checks);
    let path = Path::new(&cli.path);

    if !path.exists() {
        eprintln!("❌ CODEOWNERS file not found at {:?}", path);
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("CODEOWNERS file not found at {:?}", path),
        ));
    }

    let (rules, invalid_lines) = match parse_codeowners_file(&cli.path) {
        Ok((rules, invalid_lines)) => (rules, invalid_lines),
        Err(e) => {
            eprintln!("❌ Error parsing CODEOWNERS file: {}", e);
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to parse CODEOWNERS file: {}", e),
            ));
        }
    };

    if !invalid_lines.is_empty() {
        eprintln!("⚠️  Invalid lines found:");
        for line in invalid_lines {
            eprintln!(" - Line {}: {}", line.line_number, line.content);
        }
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid lines found in the CODEOWNERS file",
        ));
    }

    let failed_rules = run_validator(&validator_args, &rules);

    if !failed_rules.is_empty() {
        eprintln!("❌ The following rules failed:\n");
        for (validator, rule) in &failed_rules {
            eprintln!("Validator: {}", validator);
            eprintln!("  Pattern: {}", rule.pattern);
            eprintln!("    Rule: {}", rule.original_path);
            eprintln!("    Owners: {:?}", rule.owners);
            eprintln!();
        }

        return Err(io::Error::other("Some rules failed validation"));
    }

    println!("✅ CODEOWNERS validation passed.");
    Ok(())
}
