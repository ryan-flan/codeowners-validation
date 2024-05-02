use codeowners_validation::parser::{parse_codeowners_file, CodeOwnerRule};
use codeowners_validation::validators::check_exists::validate_directory;
use codeowners_validation::validators::duplicate_patterns::validate_duplicates;
use std::io;
use std::path::Path;

fn main() -> io::Result<()> {
    let codeowners_file_path = ".github/CODEOWNERS";
    let repo_dir = Path::new(".");

    // Parse the CODEOWNERS file
    let (rules, invalid_lines) = match parse_codeowners_file(codeowners_file_path) {
        Ok((rules, invalid_lines)) => (rules, invalid_lines),
        Err(e) => {
            eprintln!("Error parsing CODEOWNERS file: {}", e);
            return Err(e);
        }
    };

    // Check for invalid lines in the CODEOWNERS file
    if !invalid_lines.is_empty() {
        eprintln!("Invalid lines found in the CODEOWNERS file:");
        for invalid_line in invalid_lines {
            eprintln!(
                "Line {}: {}",
                invalid_line.line_number, invalid_line.content
            );
        }
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Invalid lines found in the CODEOWNERS file",
        ));
    }

    let mut failed_rules: Vec<(String, CodeOwnerRule)> = Vec::new();

    // Run the check_exists validation
    match validate_directory(repo_dir, rules.clone()) {
        Ok(failures) => {
            for rule in failures {
                failed_rules.push(("check_exists".to_string(), rule));
            }
        }
        Err(e) => {
            eprintln!("Error validating directory: {}", e);
            return Err(io::Error::new(io::ErrorKind::Other, e.to_string()));
        }
    }

    // Run the duplicate_patterns validation
    for rule in validate_duplicates(&rules) {
        failed_rules.push(("duplicate_patterns".to_string(), rule));
    }

    if !failed_rules.is_empty() {
        eprintln!("The following rules failed validation:");
        eprintln!();
        for (validator, rule) in &failed_rules {
            eprintln!("Validator: {}", validator);
            eprintln!("  Pattern: {}", rule.pattern);
            eprintln!("    Rule: {}", rule.original_path);
            eprintln!("    Owners: {:?}", rule.owners);
            eprintln!();
        }
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Some rules failed validation",
        ));
    }

    Ok(())
}
