use codeowners_validation::parser::parse_codeowners_file;
use codeowners_validation::validators::duplicate_patterns::validate_duplicates;
use codeowners_validation::validators::exists::validate_directory;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::fmt::Write as FmtWrite;
use std::fs;
use tempfile::{tempdir, NamedTempFile};

fn create_realistic_codeowners(num_rules: usize) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    let mut content = String::new();

    // Add some comments and empty lines for realism
    writeln!(&mut content, "# CODEOWNERS file for large monorepo").unwrap();
    writeln!(&mut content, "# Generated for benchmarking\n").unwrap();

    // More realistic distribution of rule types
    for i in 0..num_rules {
        match i % 20 {
            // Specific file rules (40%)
            0..=7 => writeln!(
                &mut content,
                "/src/components/feature{}/index.ts @team{}",
                i % 100,
                i % 15
            )
            .unwrap(),

            // Directory rules (25%)
            8..=12 => {
                writeln!(&mut content, "/packages/lib{}/ @org/team{}", i % 50, i % 10).unwrap()
            }

            // Wildcard patterns (20%)
            13..=16 => match i % 4 {
                0 => writeln!(
                    &mut content,
                    "*.{} @team{}",
                    ["yml", "yaml", "json", "xml"][i % 4],
                    i % 8
                )
                .unwrap(),
                1 => writeln!(&mut content, "/docs/**/*.md @docs-team").unwrap(),
                2 => writeln!(&mut content, "**/test/** @qa-team").unwrap(),
                _ => writeln!(&mut content, "**/*.spec.ts @test-team").unwrap(),
            },

            // Complex patterns (15%)
            _ => writeln!(
                &mut content,
                "/src/**/components/**/*.tsx @frontend-team @ui-team"
            )
            .unwrap(),
        }

        // Add occasional empty lines and comments
        if i % 50 == 0 && i > 0 {
            writeln!(&mut content, "\n# Section for module {}", i / 50).unwrap();
        }
    }

    use std::io::Write;
    file.write_all(content.as_bytes()).unwrap();
    file.flush().unwrap();
    file
}

// Create a realistic repo structure
fn create_realistic_repo(scale: usize) -> tempfile::TempDir {
    let dir = tempdir().unwrap();

    // Create a structure similar to a large monorepo
    let structures = vec![
        "src/components/",
        "src/utils/",
        "src/services/",
        "packages/core/src/",
        "packages/ui/src/",
        "packages/shared/lib/",
        "docs/api/",
        "docs/guides/",
        "test/unit/",
        "test/integration/",
        ".github/workflows/",
        "scripts/build/",
        "config/",
    ];

    for (i, base) in structures.iter().enumerate() {
        for j in 0..scale {
            let file_types = vec!["ts", "tsx", "js", "jsx", "json", "md", "yml"];
            let file_type = file_types[(i + j) % file_types.len()];

            let path = format!("{}feature{}/file{}.{}", base, j % 10, j, file_type);
            let full_path = dir.path().join(&path);

            if let Some(parent) = full_path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(full_path, format!("// content for {}", path)).unwrap();
        }
    }

    dir
}

// Benchmark specifically for your 10k rules scenario
fn benchmark_10k_rules_scenario(c: &mut Criterion) {
    let mut group = c.benchmark_group("real_world_10k_rules");
    group.sample_size(20); // Smaller sample size for longer benchmarks

    // Test with 10k rules
    let file = create_realistic_codeowners(10000);
    let file_size = std::fs::metadata(file.path()).unwrap().len();
    println!("Generated CODEOWNERS file size: {} KB", file_size / 1024);

    // Create a realistic repo
    let repo = create_realistic_repo(150); // Larger repo for 10k rules

    group.bench_function("parse_10k_rules", |b| {
        b.iter(|| {
            let (rules, _) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();
            std::hint::black_box(rules);
        });
    });

    // Parse once for the validation benchmarks
    let (rules, _) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();
    println!("Parsed {} rules", rules.len());

    group.bench_function("validate_exists_10k_rules", |b| {
        b.iter(|| {
            let result = validate_directory(repo.path(), &rules).unwrap();
            std::hint::black_box(result);
        });
    });

    group.bench_function("validate_duplicates_10k_rules", |b| {
        b.iter(|| {
            let result = validate_duplicates(&rules);
            std::hint::black_box(result);
        });
    });

    group.bench_function("end_to_end_10k_rules", |b| {
        b.iter(|| {
            let (rules, _) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();
            let missing = validate_directory(repo.path(), &rules).unwrap();
            let duplicates = validate_duplicates(&rules);
            std::hint::black_box((missing, duplicates));
        });
    });

    group.finish();
}

// Benchmark to find performance cliffs
fn benchmark_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling");

    // Test scaling up to 15k to see how it handles growth beyond 10k
    for size in [1000, 5000, 10000, 15000, 20000].iter() {
        let file = create_realistic_codeowners(*size);
        let repo = create_realistic_repo(50);

        group.bench_with_input(BenchmarkId::new("parse", size), size, |b, _| {
            b.iter(|| {
                let (rules, _) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();
                std::hint::black_box(rules);
            });
        });

        let (rules, _) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();

        group.bench_with_input(BenchmarkId::new("exists", size), &rules, |b, rules| {
            b.iter(|| {
                let result = validate_directory(repo.path(), rules).unwrap();
                std::hint::black_box(result);
            });
        });
    }

    group.finish();
}

// Memory-focused benchmark for 10k rules
fn benchmark_memory_10k(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_10k");
    group.sample_size(10);

    group.bench_function("memory_usage_10k_rules", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = std::time::Duration::new(0, 0);
            let mut peak_memory = 0usize;

            for _ in 0..iters {
                let file = create_realistic_codeowners(10000);
                let repo = create_realistic_repo(150);

                // Reset for accurate measurement
                let _ = peak_memory;

                let before = memory_stats::memory_stats()
                    .map(|s| s.physical_mem)
                    .unwrap_or(0);

                let start = std::time::Instant::now();

                // Full validation pipeline
                let (rules, _) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();
                println!("Parsed {} rules", rules.len());

                let missing = validate_directory(repo.path(), &rules).unwrap();
                println!("Found {} missing files", missing.len());

                let duplicates = validate_duplicates(&rules);
                println!("Found {} duplicate patterns", duplicates.len());

                total_duration += start.elapsed();

                let after = memory_stats::memory_stats()
                    .map(|s| s.physical_mem)
                    .unwrap_or(0);

                let memory_used = after.saturating_sub(before);
                peak_memory = peak_memory.max(memory_used);

                println!(
                    "Iteration memory: {} MB (peak: {} MB)",
                    memory_used / 1024 / 1024,
                    peak_memory / 1024 / 1024
                );
            }

            total_duration
        });
    });

    group.finish();
}

// cases that might cause memory issues
fn benchmark_worst_case(c: &mut Criterion) {
    let mut group = c.benchmark_group("worst_case");
    group.sample_size(10);

    // Many wildcard patterns (these are most expensive)
    group.bench_function("10k_wildcards", |b| {
        let mut file = NamedTempFile::new().unwrap();
        let mut content = String::new();

        // Create 10000 wildcard patterns - simpler to avoid regex issues
        for i in 0..10000 {
            match i % 5 {
                0 => writeln!(&mut content, "*.log @team{}", i % 10).unwrap(),
                1 => writeln!(&mut content, "**/*.test.js @team{}", i % 10).unwrap(),
                2 => writeln!(&mut content, "src/**/*.ts @team{}", i % 10).unwrap(),
                3 => writeln!(&mut content, "*/config/* @team{}", i % 10).unwrap(),
                4 => writeln!(&mut content, "packages/*/src/*.js @team{}", i % 10).unwrap(),
                _ => unreachable!(),
            }
        }

        use std::io::Write;
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();

        let repo = create_realistic_repo(50);
        let (rules, _) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();

        b.iter(|| {
            let result = validate_directory(repo.path(), &rules).unwrap();
            std::hint::black_box(result);
        });
    });

    // Many duplicate patterns
    group.bench_function("5k_duplicates", |b| {
        let file = create_realistic_codeowners(5000);
        let (mut rules, _) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();

        // Double the rules to create 10k total with 5k duplicates
        let original_len = rules.len();
        for i in 0..original_len {
            rules.push(rules[i].clone());
        }

        b.iter(|| {
            let result = validate_duplicates(&rules);
            std::hint::black_box(result);
        });
    });

    // Deep directory structures
    group.bench_function("deep_paths", |b| {
        let mut file = NamedTempFile::new().unwrap();
        let mut content = String::new();

        // Create rules with very deep paths
        for i in 0..10000 {
            let depth = 10 + (i % 5);
            let mut path = String::new();
            for d in 0..depth {
                path.push_str(&format!("level{}/", d));
            }
            writeln!(&mut content, "/{}file{}.ts @team{}", path, i, i % 20).unwrap();
        }

        use std::io::Write;
        file.write_all(content.as_bytes()).unwrap();
        file.flush().unwrap();

        let repo = create_realistic_repo(50);
        let (rules, _) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();

        b.iter(|| {
            let result = validate_directory(repo.path(), &rules).unwrap();
            std::hint::black_box(result);
        });
    });

    group.finish();
}

// Quick benchmark for development iteration
fn benchmark_quick(c: &mut Criterion) {
    let mut group = c.benchmark_group("quick");
    group.sample_size(10);

    // Just test 10k rules end-to-end
    let file = create_realistic_codeowners(10000);
    let repo = create_realistic_repo(100);

    group.bench_function("quick_10k_end_to_end", |b| {
        b.iter(|| {
            let (rules, _) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();
            let missing = validate_directory(repo.path(), &rules).unwrap();
            let duplicates = validate_duplicates(&rules);
            std::hint::black_box((missing, duplicates));
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_10k_rules_scenario,
    benchmark_scaling,
    benchmark_memory_10k,
    benchmark_worst_case,
    benchmark_quick
);
criterion_main!(benches);
