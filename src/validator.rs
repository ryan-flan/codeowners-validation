use globset::{Glob, GlobSetBuilder};

pub fn validate_codeowner_rules(globs: &[String]) -> Vec<(String, bool)> {
    let mut results = Vec::new();

    let mut glob_set_builder = GlobSetBuilder::new();
    for glob in globs {
        if let Ok(pattern) = Glob::new(glob) {
            glob_set_builder.add(pattern);
        }
    }
    let glob_set = glob_set_builder.build().unwrap();

    for glob in globs {
        let matched = glob_set.is_match(glob);
        results.push((glob.clone(), matched));
    }

    results
}
