mod common;

use codeowners_validation::parser::parse_codeowners_file;
use codeowners_validation::validators::duplicate_patterns::validate_duplicates;
use codeowners_validation::validators::exists::validate_directory;
use common::*;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::{Duration, Instant};

const MAX_RUNTIME: Duration = Duration::from_secs(60);
const CI_MEMORY_LIMIT_MB: f64 = 500.0;

// Measure memory efficiency per rule
fn benchmark_memory_per_rule(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_per_rule");
    group.sample_size(10);

    for size in [1000, 5000, 10000, 20000].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter_custom(|iters| {
                let mut total_duration = Duration::new(0, 0);
                let mut memory_samples = Vec::new();

                for _ in 0..iters {
                    let file = create_realistic_codeowners(size);
                    let repo = create_realistic_repo(100);

                    let before = memory_stats::memory_stats()
                        .map(|s| s.physical_mem)
                        .unwrap_or(0);

                    let start = Instant::now();

                    let (rules, _) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();
                    let _ = validate_directory(repo.path(), &rules).unwrap();
                    let _ = validate_duplicates(&rules);

                    total_duration += start.elapsed();

                    let after = memory_stats::memory_stats()
                        .map(|s| s.physical_mem)
                        .unwrap_or(0);

                    let memory_used = after.saturating_sub(before);
                    let kb_per_rule = (memory_used / 1024) as f64 / size as f64;
                    memory_samples.push(kb_per_rule);
                }

                let avg_kb_per_rule =
                    memory_samples.iter().sum::<f64>() / memory_samples.len() as f64;
                println!("{} rules: {:.2} KB per rule", size, avg_kb_per_rule);

                total_duration
            });
        });
    }

    group.finish();
}

// CI environment simulation with memory constraints
fn benchmark_ci_memory_limits(c: &mut Criterion) {
    let mut group = c.benchmark_group("ci_memory_limits");
    group.sample_size(10);

    group.bench_function("ci_20k_rules", |b| {
        b.iter_custom(|iters| {
            let mut total_duration = Duration::new(0, 0);
            let mut max_memory_mb: f64 = 0.0;
            let mut passed_tests = 0;

            for _ in 0..iters {
                let file = create_realistic_codeowners(20000);
                let repo = create_realistic_repo(200);

                let before = memory_stats::memory_stats()
                    .map(|s| s.physical_mem)
                    .unwrap_or(0);

                let start = Instant::now();

                // Full validation pipeline
                let (rules, _) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();
                let missing = validate_directory(repo.path(), &rules).unwrap();
                let duplicates = validate_duplicates(&rules);

                let elapsed = start.elapsed();
                total_duration += elapsed;

                let after = memory_stats::memory_stats()
                    .map(|s| s.physical_mem)
                    .unwrap_or(0);

                let memory_mb = (after.saturating_sub(before)) as f64 / 1024.0 / 1024.0;
                max_memory_mb = max_memory_mb.max(memory_mb);

                let passed = elapsed < MAX_RUNTIME && memory_mb < CI_MEMORY_LIMIT_MB;
                if passed {
                    passed_tests += 1;
                }

                println!(
                    "CI Test: {} | Time: {:?} | Memory: {:.2} MB | Missing: {} | Dupes: {}",
                    if passed { "✅ PASS" } else { "❌ FAIL" },
                    elapsed,
                    memory_mb,
                    missing.len(),
                    duplicates.len()
                );
            }

            println!(
                "\nCI Success Rate: {}/{} ({:.0}%)",
                passed_tests,
                iters,
                (passed_tests as f64 / iters as f64) * 100.0
            );
            println!("Peak Memory: {:.2} MB", max_memory_mb);

            total_duration
        });
    });

    group.finish();
}

// Memory vs Speed trade-off analysis
fn benchmark_optimization_modes(c: &mut Criterion) {
    let mut group = c.benchmark_group("optimization_modes");
    group.sample_size(10);

    // Test different thread counts to see memory/speed trade-off
    for thread_count in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::new("threads", thread_count),
            thread_count,
            |b, &threads| {
                b.iter_custom(|iters| {
                    let mut total_duration = Duration::new(0, 0);

                    // Temporarily set thread count via environment variable
                    std::env::set_var("CODEOWNERS_THREADS", threads.to_string());

                    for _ in 0..iters {
                        let file = create_realistic_codeowners(10000);
                        let repo = create_realistic_repo(150);

                        let before = memory_stats::memory_stats()
                            .map(|s| s.physical_mem)
                            .unwrap_or(0);

                        let start = Instant::now();

                        let (rules, _) =
                            parse_codeowners_file(file.path().to_str().unwrap()).unwrap();
                        let _ = validate_directory(repo.path(), &rules).unwrap();

                        let elapsed = start.elapsed();
                        total_duration += elapsed;

                        let after = memory_stats::memory_stats()
                            .map(|s| s.physical_mem)
                            .unwrap_or(0);

                        let memory_mb = (after.saturating_sub(before)) as f64 / 1024.0 / 1024.0;

                        println!(
                            "{} threads: {:.3}s, {:.2} MB",
                            threads,
                            elapsed.as_secs_f64(),
                            memory_mb
                        );
                    }

                    std::env::remove_var("CODEOWNERS_THREADS");
                    total_duration
                });
            },
        );
    }

    group.finish();
}

// Peak memory usage for different patterns
fn benchmark_pattern_memory_impact(c: &mut Criterion) {
    let mut group = c.benchmark_group("pattern_memory_impact");
    group.sample_size(10);

    // Compare memory usage of different pattern types
    let patterns: Vec<(&str, Box<dyn Fn(usize) -> tempfile::NamedTempFile>)> = vec![
        (
            "direct_paths",
            Box::new(|n| {
                let mut file = tempfile::NamedTempFile::new().unwrap();
                let mut content = String::new();
                use std::fmt::Write as FmtWrite;
                for i in 0..n {
                    writeln!(&mut content, "/src/file{}.rs @team", i).unwrap();
                }
                use std::io::Write;
                file.write_all(content.as_bytes()).unwrap();
                file.flush().unwrap();
                file
            }),
        ),
        ("wildcards", Box::new(create_wildcard_heavy_codeowners)),
        ("deep_paths", Box::new(create_deep_path_codeowners)),
    ];

    for (name, create_fn) in patterns {
        group.bench_function(name, |b| {
            b.iter_custom(|iters| {
                let mut total_duration = Duration::new(0, 0);

                for _ in 0..iters {
                    let file = create_fn(5000);
                    let repo = create_realistic_repo(50);

                    let before = memory_stats::memory_stats()
                        .map(|s| s.physical_mem)
                        .unwrap_or(0);

                    let start = Instant::now();

                    let (rules, _) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();
                    let _ = validate_directory(repo.path(), &rules).unwrap();

                    total_duration += start.elapsed();

                    let after = memory_stats::memory_stats()
                        .map(|s| s.physical_mem)
                        .unwrap_or(0);

                    let memory_mb = (after.saturating_sub(before)) as f64 / 1024.0 / 1024.0;
                    println!("{} pattern type: {:.2} MB", name, memory_mb);
                }

                total_duration
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_memory_per_rule,
    benchmark_ci_memory_limits,
    benchmark_optimization_modes,
    benchmark_pattern_memory_impact
);
criterion_main!(benches);
