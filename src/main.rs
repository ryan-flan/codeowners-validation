mod parser;
mod validator;

use std::io;

fn main() -> io::Result<()> {
    //TODO: Allow configuration or search all common locations for CODEOWNERS
    let codeowners_file_path = ".github/CODEOWNERS";

    // Parse the CODEOWNERS file
    let globs = parser::parse_codeowners_file(codeowners_file_path)?;

    // Validate the codeowner rules
    let results = validator::validate_codeowner_rules(&globs);

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
