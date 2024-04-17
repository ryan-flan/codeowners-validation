extern crate globset;

use globset::{Glob, GlobSetBuilder};
use std::fs::File;
use std::io::{self, BufRead};

// Function to parse a CODEOWNERS file and extract globs
fn parse_codeowners_file(file_path: &str) -> io::Result<Vec<String>> {
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    let mut globs = Vec::new();

    for line in reader.lines() {
        if let Ok(line) = line {
            // Skip comments and empty lines
            if !line.trim().is_empty() && !line.trim().starts_with('#') {
                // Extract the glob from the line
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(glob) = parts.get(0) {
                    globs.push((*glob).to_string());
                }
            }
        }
    }

    Ok(globs)
}

// Function to validate codeowner rules using globset
fn validate_codeowner_rules_exist(globs: &[String]) -> Vec<(String, bool)> {
    let mut results = Vec::new();

    // Build a GlobSet from the list of globs
    let mut glob_set_builder = GlobSetBuilder::new();
    for glob in globs {
        if let Ok(pattern) = Glob::new(glob) {
            glob_set_builder.add(pattern);
        }
    }
    let glob_set = glob_set_builder.build().unwrap();

    // Match rules against the glob set
    for glob in globs {
        let matched = glob_set.is_match(glob);
        results.push((glob.clone(), matched));
    }

    results
}

fn main() -> io::Result<()> {
    //TODO: Allow configuration or search all common locations for CODEOWNERS
    let codeowners_file_path = ".github/CODEOWNERS";

    // Parse the CODEOWNERS file
    let globs = parse_codeowners_file(codeowners_file_path)?;

    // Validate the codeowner rules
    let results = validate_codeowner_rules_exist(&globs.clone());

    // Print the validation results
    for (rule, valid) in results {
        if valid {
            println!("Rule '{}' is valid.", rule);
        } else {
            println!("Rule '{}' is invalid.", rule);
        }
    }

    Ok(())
}
