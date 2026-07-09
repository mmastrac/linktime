//! Custom (`harness = false`) test runner for the `.crok` files under this
//! crate. Every `.crok` file is a test.

// Miri is unsupported.
fn main() {
    #[cfg(not(miri))]
    harness::run();
}

#[cfg(not(miri))]
mod harness {
    use std::collections::VecDeque;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;

    pub(crate) fn run() {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let list = args.iter().any(|a| a == "--list");
        let filters: Vec<String> = args.into_iter().filter(|a| !a.starts_with('-')).collect();

        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut tests = Vec::new();
        discover(root, root, &mut tests);
        tests.sort();
        tests.retain(|(name, _)| filters.is_empty() || filters.iter().any(|f| name.contains(f)));

        if list {
            for (name, _) in &tests {
                println!("{name}: test");
            }
            return;
        }

        let total = tests.len();
        println!(
            "\nrunning {total} test{}",
            if total == 1 { "" } else { "s" }
        );

        let jobs = std::thread::available_parallelism().map_or(1, |n| n.get());
        let queue = Mutex::new(tests.into_iter().collect::<VecDeque<_>>());
        let failures = Mutex::new(Vec::new());

        std::thread::scope(|scope| {
            for _ in 0..jobs {
                scope.spawn(|| {
                    while let Some((name, path)) = queue.lock().unwrap().pop_front() {
                        match crok_lib::try_run_file_captured(&path) {
                            Ok(_) => println!("test {name} ... ok"),
                            Err(e) => {
                                println!("test {name} ... FAILED");
                                failures.lock().unwrap().push((name, e));
                            }
                        }
                    }
                });
            }
        });

        let failures = failures.into_inner().unwrap();
        println!();
        if failures.is_empty() {
            println!("test result: ok. {total} passed; 0 failed\n");
        } else {
            println!("failures:\n");
            for (name, e) in &failures {
                println!("---- {name} ----\n{}\n{}", e.error, e.output);
            }
            println!(
                "test result: FAILED. {} passed; {} failed\n",
                total - failures.len(),
                failures.len()
            );
            std::process::exit(1);
        }
    }

    /// Recursively collect `(test-name, path)` for every `.crok` file, skipping
    /// `target/` and hidden directories.
    fn discover(dir: &Path, root: &Path, out: &mut Vec<(String, PathBuf)>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if path.is_dir() {
                if name != "target" && !name.starts_with('.') {
                    discover(&path, root, out);
                }
            } else if path.extension().and_then(|e| e.to_str()) == Some("crok") {
                out.push((test_name(&path, root), path));
            }
        }
    }

    fn test_name(path: &Path, root: &Path) -> String {
        let rel = path.strip_prefix(root).unwrap_or(path);
        let comps: Vec<String> = rel
            .iter()
            .map(|c| c.to_string_lossy().into_owned())
            .collect();
        let (file, dirs) = comps.split_last().expect("path has a file name");
        let stem = file.strip_suffix(".crok").unwrap_or(file);

        let mut parts: Vec<String> = dirs.iter().map(|d| sanitize(d)).collect();
        if stem == "test" {
            // The directory path already names the test.
        } else if let Some(variant) = stem.strip_prefix("test-") {
            parts.push(sanitize(variant));
        } else {
            parts.push(sanitize(stem));
        }
        parts.join("::")
    }

    fn sanitize(s: &str) -> String {
        s.chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
            .collect()
    }
}
