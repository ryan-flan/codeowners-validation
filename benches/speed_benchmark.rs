mod common;

use codeowners_validation::parser::parse_codeowners_file;
use codeowners_validation::validators::duplicate_patterns::validate_duplicates;
use codeowners_validation::validators::exists::validate_directory;
use common::*;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};

// Benchmark parsing performance at different scales
fn benchmark_parsing_speed(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing_speed");

    for size in [1000, 5000, 10000, 15000, 20000].iter() {
        let file = create_realistic_codeowners(*size);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let (rules, _) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();
                std::hint::black_box(rules);
            });
        });
    }

    group.finish();
}

// Benchmark exists validation performance
fn benchmark_exists_validation_speed(c: &mut Criterion) {
    let mut group = c.benchmark_group("exists_validation_speed");

    // Test with different repo sizes to see impact
    for (rule_count, repo_size) in [(1000, 50), (5000, 100), (10000, 150), (15000, 200)].iter() {
        let file = create_realistic_codeowners(*rule_count);
        let repo = create_realistic_repo(*repo_size);
        let (rules, _) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();

        group.bench_with_input(BenchmarkId::new("rules", rule_count), &rules, |b, rules| {
            b.iter(|| {
                let result = validate_directory(repo.path(), rules).unwrap();
                std::hint::black_box(result);
            });
        });
    }

    group.finish();
}

// Benchmark duplicate detection performance
fn benchmark_duplicate_detection_speed(c: &mut Criterion) {
    let mut group = c.benchmark_group("duplicate_detection_speed");

    for size in [1000, 5000, 10000, 20000].iter() {
        // Create rules with varying duplicate percentages
        let file = create_realistic_codeowners(*size / 2);
        let (mut rules, _) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();

        // Add duplicates (50% duplicate rate)
        let original_len = rules.len();
        for i in 0..original_len {
            rules.push(rules[i].clone());
        }

        group.bench_with_input(BenchmarkId::new("total_rules", size), &rules, |b, rules| {
            b.iter(|| {
                let result = validate_duplicates(rules);
                std::hint::black_box(result);
            });
        });
    }

    group.finish();
}

// Benchmark end-to-end performance (the metric users care about)
fn benchmark_end_to_end_speed(c: &mut Criterion) {
    let mut group = c.benchmark_group("end_to_end_speed");

    for size in [5000, 10000, 15000, 20000].iter() {
        let file = create_realistic_codeowners(*size);
        let repo = create_realistic_repo(150);

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let (rules, _) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();
                let missing = validate_directory(repo.path(), &rules).unwrap();
                let duplicates = validate_duplicates(&rules);
                std::hint::black_box((missing, duplicates));
            });
        });
    }

    group.finish();
}

// Benchmark worst-case scenarios
fn benchmark_pathological_cases(c: &mut Criterion) {
    let mut group = c.benchmark_group("pathological_cases");
    group.sample_size(10); // Fewer samples for slow cases

    // Many wildcards (stress test glob matching)
    group.bench_function("10k_wildcards", |b| {
        let file = create_wildcard_heavy_codeowners(10000);
        let repo = create_realistic_repo(50);
        let (rules, _) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();

        b.iter(|| {
            let result = validate_directory(repo.path(), &rules).unwrap();
            std::hint::black_box(result);
        });
    });

    // Deep paths (stress test path operations)
    group.bench_function("deep_paths_10k", |b| {
        let file = create_deep_path_codeowners(10000);
        let repo = create_realistic_repo(50);
        let (rules, _) = parse_codeowners_file(file.path().to_str().unwrap()).unwrap();

        b.iter(|| {
            let result = validate_directory(repo.path(), &rules).unwrap();
            std::hint::black_box(result);
        });
    });

    group.finish();
}

// Quick smoke test for development
fn benchmark_quick_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("quick_check");
    group.sample_size(10);

    let file = create_realistic_codeowners(10000);
    let repo = create_realistic_repo(100);

    group.bench_function("10k_rules_baseline", |b| {
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
    benchmark_parsing_speed,
    benchmark_exists_validation_speed,
    benchmark_duplicate_detection_speed,
    benchmark_end_to_end_speed,
    benchmark_pathological_cases,
    benchmark_quick_check
);
criterion_main!(benches);
