use codeowners_validation::parser::parse_codeowners_file;
use codeowners_validation::validators::check_exists::validate_directory;
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

    let results = match validate_directory(repo_dir, rules) {
        Ok(results) => results,
        Err(e) => {
            eprintln!("Error validating directory: {}", e);
            return Err(io::Error::new(io::ErrorKind::Other, e.to_string()));
        }
    };

    // Check if any files failed the validation
    let mut failed_files = Vec::new();
    for result in results {
        failed_files.push(result);
    }

    // If there are failed files, print them nicely to stdout
    if !failed_files.is_empty() {
        println!("The following files failed the check_exists validation:");
        for rule in failed_files {
            println!("  Pattern: {}", rule.pattern);
            println!("    Rule: {}", rule.original_path);
            println!("    Owners: {:?}", rule.owners);
            println!();
        }
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "Some files failed the check_exists validation",
        ));
    }

    Ok(())
}
