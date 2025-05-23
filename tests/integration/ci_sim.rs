use codeowners_validation::parser::parse_codeowners_file;
use codeowners_validation::validators::duplicate_patterns::validate_duplicates;
use codeowners_validation::validators::exists::validate_directory;
use std::time::Instant;

#[test]
#[ignore] // Run with: cargo test --release -- --ignored ci_simulation
fn ci_environment_constraints() {
    println!("\n=== CI Environment Simulation ===");
    println!("Testing with CI constraints:");
    println!("- Memory limit: 500MB");
    println!("- Time limit: 60 seconds");
    println!("- Large repository simulation");

    const CI_MEMORY_LIMIT_MB: f64 = 500.0;
    const CI_TIME_LIMIT_SECS: u64 = 60;

    // Create a realistic CODEOWNERS file
    let file = create_ci_test_codeowners(15_000); // 15k rules

    // Create a realistic repository structure
    let repo = create_ci_test_repo();

    // Get baseline memory
    let baseline = memory_stats::memory_stats()
        .map(|s| s.physical_mem)
        .unwrap_or(0);

    let start = Instant::now();

    // Full validation pipeline
    let (rules, invalid) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();
    assert_eq!(
        invalid.len(),
        0,
        "Generated file should have no invalid lines"
    );

    let missing = validate_directory(repo.path(), &rules).unwrap();
    let duplicates = validate_duplicates(&rules);

    let elapsed = start.elapsed();

    // Check memory usage
    let peak = memory_stats::memory_stats()
        .map(|s| s.physical_mem)
        .unwrap_or(0);
    let memory_mb = (peak.saturating_sub(baseline)) as f64 / 1024.0 / 1024.0;

    println!("\nCI Simulation Results:");
    println!("- Rules processed: {}", rules.len());
    println!("- Processing time: {:?}", elapsed);
    println!("- Memory used: {:.2} MB", memory_mb);
    println!("- Missing patterns: {}", missing.len());
    println!("- Duplicate patterns: {}", duplicates.len());

    // Assertions
    assert!(
        elapsed.as_secs() <= CI_TIME_LIMIT_SECS,
        "CI time limit exceeded: {:?} > {}s",
        elapsed,
        CI_TIME_LIMIT_SECS
    );

    assert!(
        memory_mb <= CI_MEMORY_LIMIT_MB,
        "CI memory limit exceeded: {:.2} MB > {} MB",
        memory_mb,
        CI_MEMORY_LIMIT_MB
    );

    println!("\n✅ CI simulation passed - tool is probably suitable for CI environments!");
}

#[test]
#[ignore] // Run with: cargo test --release -- --ignored github_actions_simulation
fn github_actions_runner_test() {
    println!("\n=== GitHub Actions Runner Simulation ===");
    println!("Simulating github-hosted runner constraints:");
    println!("- 2-core CPU");
    println!("- 7 GB RAM (but we should use much less)");
    println!("- Ubuntu latest");

    // Set thread count to simulate GitHub Actions
    std::env::set_var("CODEOWNERS_THREADS", "2");

    // Run a realistic workload
    let file = create_ci_test_codeowners(20_000); // 20k rules - stress test
    let repo = create_ci_test_repo();

    let start = Instant::now();
    let (rules, _) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();
    let _ = validate_directory(repo.path(), &rules).unwrap();
    let _ = validate_duplicates(&rules);
    let elapsed = start.elapsed();

    std::env::remove_var("CODEOWNERS_THREADS");

    println!("GitHub Actions simulation completed in {:?}", elapsed);

    assert!(
        // Not the actual limit but I don't think this job should ever take more than 2 min unless
        // you have an insane file
        elapsed.as_secs() < 120, // 2 minute limit for GitHub Actions
        "Too slow for GitHub Actions: {:?}",
        elapsed
    );

    println!("\n✅ GitHub Actions simulation passed!");
}

// Helper functions
fn create_ci_test_codeowners(num_rules: usize) -> tempfile::NamedTempFile {
    use std::fmt::Write as FmtWrite;
    use std::io::Write;

    let mut file = tempfile::NamedTempFile::new().unwrap();
    let mut content = String::with_capacity(num_rules * 60);

    writeln!(&mut content, "# CI Test CODEOWNERS - {} rules", num_rules).unwrap();

    for i in 0..num_rules {
        match i % 20 {
            0..=7 => writeln!(
                &mut content,
                "/src/module{}/file.ts @team{}",
                i % 100,
                i % 10
            )
            .unwrap(),
            8..=11 => writeln!(&mut content, "/lib/package{}/ @org/team{}", i % 50, i % 5).unwrap(),
            12..=15 => writeln!(
                &mut content,
                "*.{} @team{}",
                ["yml", "json", "md", "txt"][i % 4],
                i % 3
            )
            .unwrap(),
            16..=18 => writeln!(&mut content, "**/*.test.ts @test-team").unwrap(),
            _ => writeln!(&mut content, "/docs/**/*.md @docs-team").unwrap(),
        }
    }

    file.write_all(content.as_bytes()).unwrap();
    file.flush().unwrap();
    file
}

fn create_ci_test_repo() -> tempfile::TempDir {
    let repo = tempfile::tempdir().unwrap();

    // Create a realistic directory structure
    let dirs = [
        "src/module1",
        "src/module2",
        "src/module3",
        "lib/package1",
        "lib/package2",
        "docs/api",
        "docs/guides",
        "test/unit",
        "test/integration",
        ".github/workflows",
    ];

    for dir in &dirs {
        std::fs::create_dir_all(repo.path().join(dir)).unwrap();
    }

    // Create some files
    let files = [
        "src/module1/file.ts",
        "src/module2/file.ts",
        "docs/api/README.md",
        "test/unit/example.test.ts",
        ".github/workflows/ci.yml",
        "package.json",
        "README.md",
    ];

    for file in &files {
        let path = repo.path().join(file);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        std::fs::write(path, "test content").unwrap();
    }

    repo
}
