# CODEOWNERS Validation

[![Crates.io](https://img.shields.io/crates/v/codeowners-validation?style=flat-square)](https://crates.io/crates/codeowners-validation)
[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/ryan-flan/codeowners-validation/ci.yml?style=flat-square)](https://github.com/ryan-flan/codeowners-validation/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg?style=flat-square)](https://opensource.org/licenses/MIT)
[![GitHub release (latest by date)](https://img.shields.io/github/v/release/ryan-flan/codeowners-validation?style=flat-square)](https://github.com/ryan-flan/codeowners-validation/releases)

A high-performance CODEOWNERS validator optimized for large repositories and monorepos. Written in Rust for maximum performance and minimal memory usage.

## Features

### ✅ Active Checks
- **File/Directory Existence**: Validates that all paths in CODEOWNERS exist in the repository
- **Duplicate Pattern Detection**: Identifies duplicate ownership patterns

### 🚧 Planned Features
- Validate file ownership coverage (detect unowned files)
- Verify GitHub owners exist and have repository access
- Comprehensive syntax validation
- Custom check configurations

## Performance

Optimized for large CODEOWNERS files:
- Handles 10,000+ rules efficiently
- Minimal memory footprint
- Parallel file system traversal
- Early exit optimization for better performance

## Installation

### As a GitHub Action

Add to your workflow (`.github/workflows/codeowners-validation.yml`):

```yaml
name: CODEOWNERS Validation

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  validate-codeowners:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run CODEOWNERS Validation
        uses: ryan-flan/codeowners-validation@latest  # Use latest version
        with:
          checks: |
            exists
            duplicate_patterns
```

### As a CLI Tool (Coming Soon)

```bash
# Install via cargo
cargo install codeowners-validation

# Run validation
codeowners-validation --path .github/CODEOWNERS
```

## Configuration

### Action Inputs

| Input | Description | Default | Required |
|-------|-------------|---------|----------|
| `checks` | Comma-separated list of checks to run | `all` | No |
| `path` | Path to CODEOWNERS file | `.github/CODEOWNERS` | No |

### Available Checks

- `exists` - Validate all referenced files/directories exist
- `duplicate_patterns` - Find duplicate ownership patterns
- `all` - Run all available checks (default)

### Action Outputs

| Output | Description |
|--------|-------------|
| `result` | Validation result: `success` or `failure` |
| `errors` | Detailed error messages (if any) |

## Usage Examples

### Basic Validation

```yaml
- uses: ryan-flan/codeowners-validation@v0.4.4
```

### Specific Checks Only

```yaml
- uses: ryan-flan/codeowners-validation@v0.4.4
  with:
    checks: exists
```

### Custom CODEOWNERS Location

```yaml
- uses: ryan-flan/codeowners-validation@v0.4.4
  with:
    path: docs/CODEOWNERS
    checks: exists,duplicate_patterns
```

## Development

### Prerequisites

- Rust 1.70+ 
- Cargo

### Building

```bash
# Clone the repository
git clone https://github.com/ryan-flan/codeowners-validation
cd codeowners-validation

# Build release version
cargo build --release

# Run tests
cargo test

# Run benchmarks
cargo bench
```

### Performance Testing

The project includes comprehensive benchmarks for large CODEOWNERS files:

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench --bench benchmark 10k_rules
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Areas for Contribution

1. Additional validation checks
2. Performance optimizations
3. Better error messages
4. Documentation improvements
5. Test coverage

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by the need for fast CODEOWNERS validation in large monorepos
- Built with [Rust](https://www.rust-lang.org/) for performance
- Uses [globset](https://crates.io/crates/globset) for efficient pattern matching

---

### See Also

- [GitHub CODEOWNERS Documentation](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-code-owners)
- [Crates.io Package](https://crates.io/crates/codeowners-validation)
- [GitHub Marketplace](https://github.com/marketplace/actions/validate-codeowners)
