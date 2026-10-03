#[test]
fn compile_time_guards() {
    let cases = trybuild::TestCases::new();
    for directory in ["tests/ui/pass", "tests/ui/fail"] {
        let mut paths: Vec<_> = std::fs::read_dir(directory)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
            .filter(|path| {
                let name = path.file_name().unwrap().to_str().unwrap();
                (cfg!(feature = "date") || !name.starts_with("date_"))
                    && (cfg!(feature = "file")
                        || !(name.starts_with("file_")
                            || name == "invalid_hash.rs"
                            || name == "missing_file.rs"))
            })
            .collect();
        paths.sort();
        for path in paths {
            if directory.ends_with("pass") {
                cases.pass(path);
            } else {
                cases.compile_fail(path);
            }
        }
    }
}
