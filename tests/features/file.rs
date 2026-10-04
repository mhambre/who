use std::fs;

use crate::support::cargo_project::CargoProject;

#[cfg(not(feature = "file"))]
#[test]
fn disabled_file_diagnostic() {
    trybuild::TestCases::new().compile_fail("tests/features/file_disabled.rs");
}

#[test]
fn consumer_can_disable_and_enable_file_support() {
    let project = CargoProject::new("file-feature-test");
    let root = project.root();
    let check = || project.check();

    fs::write(root.join("src/main.rs"), "fn main() { who::error!(!rustc().matches(\">=1\"), \"compiler condition remains available\"); }").unwrap();
    let output = check();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let lock = fs::read_to_string(root.join("Cargo.lock")).unwrap();
    for dependency in ["sha2", "digest", "crypto-common"] {
        assert!(!lock.contains(&format!("name = {dependency:?}")));
    }

    fs::write(root.join("schema.proto"), "").unwrap();
    fs::write(root.join("src/main.rs"), "fn main() { who::error!(file(\"schema.proto\").changed_from(\"sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\"), \"file drift\"); }").unwrap();
    let output = check();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("file predicates require the `file` feature"));

    project.features(&["file"]);
    let output = check();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let lock = fs::read_to_string(root.join("Cargo.lock")).unwrap();
    assert!(lock.contains("name = \"sha2\""));
    assert!(!lock.contains("name = \"chrono\""));

    fs::write(root.join("schema.proto"), "changed").unwrap();
    let output = check();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("reason: file drift"));

    fs::remove_file(root.join("schema.proto")).unwrap();
    let output = check();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("cannot read file"), "{stderr}");
    assert!(stderr.contains("schema.proto"), "{stderr}");
}
