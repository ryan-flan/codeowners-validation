use codeowners_validation::parser::parse_codeowners_file;
use codeowners_validation::validators::duplicate_patterns::validate_duplicates;
use codeowners_validation::validators::exists::validate_directory;
use std::path::Path;
use std::time::Instant;

#[test]
#[ignore] // Run with: cargo test --release -- --ignored large_file_test
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

    // Measure memory after parsing
    if let Some(usage) = memory_stats::memory_stats() {
        println!(
            "- Memory after parsing: {} MB",
            usage.physical_mem / 1024 / 1024
        );
    }

    // Count rule types
    let direct_rules = rules
        .iter()
        .filter(|r| !r.pattern.contains('*') && !r.pattern.contains('?'))
        .count();
    let wildcard_rules = rules.len() - direct_rules;

    println!("\nRule distribution:");
    println!("- Direct paths: {}", direct_rules);
    println!("- Wildcard patterns: {}", wildcard_rules);

    // Measure exists validation
    let start = Instant::now();
    let missing = validate_directory(Path::new("."), &rules).unwrap();
    let exists_time = start.elapsed();

    println!("\nExists validation:");
    println!("- Time: {:?}", exists_time);
    println!("- Missing files/patterns: {}", missing.len());

    // Measure memory after validation
    if let Some(usage) = memory_stats::memory_stats() {
        println!(
            "- Memory after validation: {} MB",
            usage.physical_mem / 1024 / 1024
        );
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

    // Performance assertions (adjust based on your requirements)
    assert!(
        parse_time.as_millis() < 1000,
        "Parsing took too long: {:?}",
        parse_time
    );
    assert!(
        exists_time.as_secs() < 30,
        "Exists validation took too long: {:?}",
        exists_time
    );
    assert!(
        dup_time.as_millis() < 500,
        "Duplicate validation took too long: {:?}",
        dup_time
    );
}

#[test]
#[ignore] // Run with: cargo test --release -- --ignored stress_test
fn stress_test_memory() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    println!("\n=== Memory stress test ===");

    let mut file = NamedTempFile::new().unwrap();
    let mut content = String::with_capacity(450_000); // Pre-allocate ~450KB

    for i in 0..10000 {
        use std::fmt::Write as FmtWrite;
        match i % 10 {
            0..=3 => writeln!(
                &mut content,
                "/very/long/path/to/specific/file/in/repo{}.ts @team{} @team{}",
                i,
                i % 20,
                (i + 7) % 20
            )
            .unwrap(),
            4..=6 => writeln!(
                &mut content,
                "/another/deeply/nested/directory/structure{}/ @org/team-{} @user{}",
                i,
                i % 15,
                i % 30
            )
            .unwrap(),
            7..=8 => writeln!(
                &mut content,
                "**/*complex*pattern*{}*/** @team{} @team{} @team{}",
                i,
                i % 10,
                (i + 3) % 10,
                (i + 5) % 10
            )
            .unwrap(),
            _ => writeln!(&mut content, "*.ext{} @generic-owner-{}", i % 100, i % 50).unwrap(),
        }
    }

    file.write_all(content.as_bytes()).unwrap();
    file.flush().unwrap();

    let file_size = std::fs::metadata(file.path()).unwrap().len();
    println!("Generated file size: {} KB", file_size / 1024);

    // Get baseline memory
    let baseline = memory_stats::memory_stats()
        .map(|s| s.physical_mem)
        .unwrap_or(0);

    // Process the file
    let start = Instant::now();
    let (rules, _) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();

    // Create a test repo
    let repo = tempfile::tempdir().unwrap();
    for i in 0..100 {
        let path = repo.path().join(format!("test{}.txt", i));
        std::fs::write(path, "test").unwrap();
    }

    let _ = validate_directory(repo.path(), &rules).unwrap();
    let _ = validate_duplicates(&rules);

    let duration = start.elapsed();

    // Measure peak memory
    let peak = memory_stats::memory_stats()
        .map(|s| s.physical_mem)
        .unwrap_or(0);

    let memory_used_mb = (peak.saturating_sub(baseline)) as f64 / 1024.0 / 1024.0;

    println!("\nResults:");
    println!("- Processing time: {:?}", duration);
    println!("- Memory used: {:.2} MB", memory_used_mb);
    println!(
        "- Memory per rule: {:.2} KB",
        (memory_used_mb * 1024.0) / rules.len() as f64
    );

    // Assert reasonable memory usage (adjust based on your requirements)
    assert!(
        memory_used_mb < 100.0,
        "Memory usage too high: {:.2} MB for {} rules",
        memory_used_mb,
        rules.len()
    );
}
