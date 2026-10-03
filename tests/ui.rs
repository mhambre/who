#[test]
fn compile_time_guards() {
    let cases = trybuild::TestCases::new();
    for directory in ["tests/ui/pass", "tests/ui/fail"] {
        let mut paths: Vec<_> = std::fs::read_dir(directory)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
            .filter(|path| {
                cfg!(feature = "date")
                    || !path
                        .file_name()
                        .unwrap()
                        .to_str()
                        .unwrap()
                        .starts_with("date_")
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
