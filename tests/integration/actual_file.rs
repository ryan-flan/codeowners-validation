use codeowners_validation::parser::parse_codeowners_file;
use codeowners_validation::validators::duplicate_patterns::validate_duplicates;
use codeowners_validation::validators::exists::validate_directory;
use std::path::Path;
use std::time::Instant;

#[test]
#[ignore] // Run with: cargo test --release -- --ignored actual_codeowners
fn test_actual_codeowners_file() {
    // Try to use the actual CODEOWNERS file if it exists
    let codeowners_path = ".github/CODEOWNERS";

    if !Path::new(codeowners_path).exists() {
        println!(
            "Skipping test - no CODEOWNERS file found at {}",
            codeowners_path
        );
        return;
    }

    let file_size = std::fs::metadata(codeowners_path).unwrap().len();
    println!("\n=== Testing with actual CODEOWNERS file ===");
    println!("File size: {} KB ({} bytes)", file_size / 1024, file_size);

    // Measure parsing
    let start = Instant::now();
    let (rules, invalid_lines) = parse_codeowners_file(codeowners_path).unwrap();
    let parse_time = start.elapsed();

    println!("\nParsing results:");
    println!("- Time: {:?}", parse_time);
    println!("- Rules parsed: {}", rules.len());
    println!("- Invalid lines: {}", invalid_lines.len());

    // Count rule types
    let direct_rules = rules
        .iter()
        .filter(|r| !r.pattern.contains('*') && !r.pattern.contains('?'))
        .count();
    let wildcard_rules = rules.len() - direct_rules;
    let anchored_rules = rules
        .iter()
        .filter(|r| r.original_path.starts_with('/'))
        .count();

    println!("\nRule distribution:");
    println!("- Direct paths: {}", direct_rules);
    println!("- Wildcard patterns: {}", wildcard_rules);
    println!("- Anchored patterns: {}", anchored_rules);

    // Show sample of rules
    if rules.len() > 0 {
        println!("\nSample rules:");
        for (i, rule) in rules.iter().take(3).enumerate() {
            println!("  {}: {} -> {}", i + 1, rule.original_path, rule.pattern);
        }
    }

    // Measure exists validation
    let start = Instant::now();
    let missing = validate_directory(Path::new("."), &rules).unwrap();
    let exists_time = start.elapsed();

    println!("\nExists validation:");
    println!("- Time: {:?}", exists_time);
    println!("- Missing files/patterns: {}", missing.len());

    if missing.len() > 0 && missing.len() < 10 {
        println!("- Sample missing patterns:");
        for (i, rule) in missing.iter().take(5).enumerate() {
            println!("  {}: {} ({})", i + 1, rule.pattern, rule.original_path);
        }
    }

    // Measure duplicate validation
    let start = Instant::now();
    let duplicates = validate_duplicates(&rules);
    let dup_time = start.elapsed();

    println!("\nDuplicate validation:");
    println!("- Time: {:?}", dup_time);
    println!("- Duplicate patterns found: {}", duplicates.len());

    // Total time
    let total_time = parse_time + exists_time + dup_time;
    println!("\n=== Total processing time: {:?} ===", total_time);

    // Performance assertions
    assert!(
        parse_time.as_millis() < 1000,
        "Parsing took too long: {:?}",
        parse_time
    );
    assert!(
        exists_time.as_secs() < 60, // 1 minute limit
        "Exists validation took too long: {:?}",
        exists_time
    );
    assert!(
        dup_time.as_millis() < 1000,
        "Duplicate validation took too long: {:?}",
        dup_time
    );

    println!("\n✅ All performance requirements met!");
}
