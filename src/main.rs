use codeowners_validation::parser::parse_codeowners_file;
use codeowners_validation::validators::validator::{run_validator, ValidatorArgs};
use std::{env, io, path::Path};

fn main() -> io::Result<()> {
    let validator_args = match env::var("INPUT_CHECKS") {
        Ok(args_str) => ValidatorArgs::from_env(&args_str),
        Err(_) => ValidatorArgs::default(),
    };

    let codeowners_file_path = ".github/CODEOWNERS";
    let path = Path::new(codeowners_file_path);

    if !path.exists() {
        eprintln!("❌ ERROR: CODEOWNERS file not found at {:?}", path);
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("CODEOWNERS file not found at {:?}", path),
        ));
    }

    let (rules, invalid_lines) = match parse_codeowners_file(codeowners_file_path) {
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
        eprintln!("⚠️  Invalid lines found in the CODEOWNERS file:");
        for invalid_line in invalid_lines {
            eprintln!(
                " - Line {}: {}",
                invalid_line.line_number, invalid_line.content
            );
        }
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid lines found in the CODEOWNERS file",
        ));
    }

    let failed_rules = run_validator(&validator_args, &rules);

    if !failed_rules.is_empty() {
        eprintln!("❌ The following rules failed validation:\n");
        for (validator, rule) in &failed_rules {
            eprintln!("Validator: {}", validator);
            eprintln!("  Pattern: {}", rule.pattern);
            eprintln!("    Rule: {}", rule.original_path);
            eprintln!("    Owners: {:?}", rule.owners);
            eprintln!();
        }

        return Err(io::Error::new(
            io::ErrorKind::Other,
            "Some rules failed validation",
        ));
    }

    println!("✅ CODEOWNERS validation passed.");
    Ok(())
}
