use std::fs::File;
use std::io::{self, BufRead};

pub fn parse_codeowners_file(file_path: &str) -> io::Result<Vec<String>> {
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    let mut globs = Vec::new();

    for line in reader.lines() {
        if let Ok(line) = line {
            if !line.trim().is_empty() && !line.trim().starts_with('#') {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if let Some(glob) = parts.get(0) {
                    globs.push((*glob).to_string());
                }
            }
        }
    }

    Ok(globs)
}
