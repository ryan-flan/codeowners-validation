use std::fmt::Write as FmtWrite;
use std::fs;
use tempfile::{tempdir, NamedTempFile};

pub fn create_realistic_codeowners(num_rules: usize) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    let mut content = String::new();

    writeln!(&mut content, "# CODEOWNERS file for large monorepo").unwrap();
    writeln!(&mut content, "# Generated for benchmarking\n").unwrap();

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

        if i % 50 == 0 && i > 0 {
            writeln!(&mut content, "\n# Section for module {}", i / 50).unwrap();
        }
    }

    use std::io::Write;
    file.write_all(content.as_bytes()).unwrap();
    file.flush().unwrap();
    file
}

pub fn create_realistic_repo(scale: usize) -> tempfile::TempDir {
    let dir = tempdir().unwrap();

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
            let file_types = ["ts", "tsx", "js", "jsx", "json", "md", "yml"];
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

pub fn create_wildcard_heavy_codeowners(num_rules: usize) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    let mut content = String::new();

    writeln!(&mut content, "# Wildcard-heavy CODEOWNERS").unwrap();

    for i in 0..num_rules {
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
    file
}

pub fn create_deep_path_codeowners(num_rules: usize) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    let mut content = String::new();

    writeln!(&mut content, "# Deep path CODEOWNERS").unwrap();

    for i in 0..num_rules {
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
    file
}
