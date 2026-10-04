use std::fs;

use crate::support::cargo_project::CargoProject;

#[cfg(not(feature = "date"))]
#[test]
fn disabled_date_diagnostic() {
    trybuild::TestCases::new().compile_fail("tests/features/date_disabled.rs");
}

#[test]
fn consumer_can_disable_and_enable_date_support() {
    let project = CargoProject::new("date-feature-test");
    let root = project.root();
    let check = || project.check();

    fs::write(root.join("src/main.rs"), "fn main() { who::error!(!rustc().compare(\">=1\"), \"compiler condition remains available\"); }").unwrap();
    let output = check();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let lock = fs::read_to_string(root.join("Cargo.lock")).unwrap();
    assert!(!lock.contains("name = \"chrono\""));
    assert!(!lock.contains("name = \"chrono-tz\""));

    fs::write(
        root.join("src/main.rs"),
        "fn main() { who::error!(date().after(\"9999-01-01\"), \"future deadline\"); }",
    )
    .unwrap();
    let output = check();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("date predicates require the `date` feature"));

    project.features(&["date"]);
    let output = check();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
