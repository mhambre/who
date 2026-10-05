use std::fs;
use std::process::Output;

use crate::support::cargo_project::CargoProject;

fn succeeds(output: Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(output.status.success(), "{stderr}");
    stderr
}

#[cfg(not(feature = "path"))]
#[test]
fn disabled_path_diagnostic() {
    trybuild::TestCases::new().compile_fail("tests/features/path_disabled.rs");
}

#[test]
fn consumer_can_disable_and_enable_path_probing() {
    let project = CargoProject::new("path-feature-test");
    let source = r#"
fn main() {
    who::error!(!path(core::mem::size_of).exists(), "existing function");
    who::error!(path(core::who_missing).exists(), "missing item");
    who::error!(!path(alloc::vec::Vec).exists(), "alloc type");
    who::error!(!path(proc_macro::TokenStream).exists(), "proc-macro type");
    who::error!(!path(std::println).exists(), "macro");
}
"#;
    let root = project.root();
    fs::write(root.join("src/main.rs"), source).unwrap();
    let output = project.check();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("path predicates require the `path` feature"));
    project.features(&["path"]);
    let manifest_path = root.join("Cargo.toml");
    let manifest = fs::read_to_string(&manifest_path).unwrap();
    for edition in ["2015", "2018", "2021", "2024"] {
        fs::write(
            &manifest_path,
            manifest.replace("edition='2021'", &format!("edition='{edition}'")),
        )
        .unwrap();
        let source = if edition == "2015" {
            format!("extern crate who;\nextern crate core;\n{source}")
        } else {
            source.to_string()
        };
        fs::write(root.join("src/main.rs"), source).unwrap();
        succeeds(project.check());
    }
    fs::write(&manifest_path, &manifest).unwrap();
    fs::write(
        root.join("src/lib.rs"),
        "#![no_std]\nwho::error!(!path(core::mem).exists(), \"no_std core import\");\n",
    )
    .unwrap();
    fs::remove_file(root.join("src/main.rs")).unwrap();
    succeeds(project.check());

    fs::write(
        root.join("src/lib.rs"),
        "who::warn!(path(core::mem::size_of).exists(), \"review function availability\");\n",
    )
    .unwrap();
    let stderr = succeeds(project.check());
    assert!(stderr.contains("review function availability"), "{stderr}");
    assert!(
        stderr.contains("expected: core::mem::size_of is importable"),
        "{stderr}"
    );
    assert!(
        stderr.contains("resolved: core::mem::size_of is importable"),
        "{stderr}"
    );
    succeeds(project.run(&["doc", "--no-deps"]));
}
