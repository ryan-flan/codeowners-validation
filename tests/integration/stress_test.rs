use codeowners_validation::parser::parse_codeowners_file;
use codeowners_validation::validators::duplicate_patterns::validate_duplicates;
use codeowners_validation::validators::exists::validate_directory;
use std::fmt::Write as FmtWrite;
use std::io::Write;
use std::time::Instant;
use tempfile::NamedTempFile;

#[test]
#[ignore] // Run with: cargo test --release -- --ignored stress_test_10k
fn stress_test_10k_rules() {
    println!("\n=== 10k rules stress test ===");

    // Create a large CODEOWNERS file
    let mut file = NamedTempFile::new().unwrap();
    let mut content = String::with_capacity(600_000); // ~600KB for 10k rules

    // Generate 10k rules with realistic patterns
    for i in 0..10_000 {
        match i % 10 {
            0..=3 => writeln!(
                &mut content,
                "/src/components/feature{}/index.ts @team{} @team{}",
                i,
                i % 20,
                (i + 7) % 20
            )
            .unwrap(),
            4..=6 => writeln!(
                &mut content,
                "/packages/module{}/ @org/team-{} @user{}",
                i,
                i % 15,
                i % 30
            )
            .unwrap(),
            7..=8 => writeln!(
                &mut content,
                "**/pattern{}.* @team{} @team{}",
                i,
                i % 10,
                (i + 3) % 10
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
    println!("Parsed {} rules", rules.len());

    // Create a test repo with some files
    let repo = tempfile::tempdir().unwrap();
    for i in 0..100 {
        let path = repo.path().join(format!("test{}.txt", i));
        std::fs::write(path, "test").unwrap();
    }

    let missing = validate_directory(repo.path(), &rules).unwrap();
    let duplicates = validate_duplicates(&rules);

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
    println!("- Missing patterns: {}", missing.len());
    println!("- Duplicate patterns: {}", duplicates.len());

    // Assert performance requirements
    assert!(
        duration.as_secs() < 60,
        "Processing took too long: {:?} (limit: 1 minute)",
        duration
    );

    assert!(
        memory_used_mb < 150.0,
        "Memory usage too high: {:.2} MB for {} rules (limit: 150MB)",
        memory_used_mb,
        rules.len()
    );

    println!("\n✅ 10k rules stress test passed!");
}

#[test]
#[ignore] // Run with: cargo test --release -- --ignored stress_test_wildcards
fn stress_test_heavy_wildcards() {
    println!("\n=== Heavy wildcards stress test ===");

    // Create CODEOWNERS with mostly wildcard patterns
    let mut file = NamedTempFile::new().unwrap();
    let mut content = String::new();

    for i in 0..5000 {
        match i % 5 {
            0 => writeln!(
                &mut content,
                "**/*.test.{} @test-team",
                ["js", "ts", "jsx", "tsx"][i % 4]
            )
            .unwrap(),
            1 => writeln!(
                &mut content,
                "src/**/feature{}/**/*.* @feature-team-{}",
                i % 20,
                i % 5
            )
            .unwrap(),
            2 => writeln!(&mut content, "*/module{}/* @module-owner-{}", i % 10, i % 3).unwrap(),
            3 => writeln!(&mut content, "**/*pattern*{}* @pattern-team", i).unwrap(),
            _ => writeln!(
                &mut content,
                "*.{} @ext-team-{}",
                ["log", "tmp", "bak", "cache"][i % 4],
                i % 2
            )
            .unwrap(),
        }
    }

    file.write_all(content.as_bytes()).unwrap();
    file.flush().unwrap();

    println!("Testing {} wildcard-heavy rules", 5000);

    let start = Instant::now();
    let (rules, _) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();

    // Create a more complex repo structure
    let repo = tempfile::tempdir().unwrap();
    let dirs = [
        "src/features/auth",
        "src/features/api",
        "test/unit",
        "test/integration",
    ];
    for dir in &dirs {
        std::fs::create_dir_all(repo.path().join(dir)).unwrap();
        for i in 0..10 {
            let path = repo.path().join(dir).join(format!("file{}.ts", i));
            std::fs::write(path, "content").unwrap();
        }
    }

    let _ = validate_directory(repo.path(), &rules).unwrap();
    let duration = start.elapsed();

    println!("Wildcard validation completed in {:?}", duration);

    assert!(
        duration.as_secs() < 30,
        "Wildcard validation too slow: {:?}",
        duration
    );

    println!("\n✅ Heavy wildcards test passed!");
}
