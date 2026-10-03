use std::{fs, process::Command};

#[cfg(not(feature = "date"))]
#[test]
fn disabled_date_diagnostic() {
    trybuild::TestCases::new().compile_fail("tests/features/date_disabled.rs");
}

#[test]
fn consumer_can_disable_and_enable_date_support() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    fs::create_dir(root.join("src")).unwrap();
    let manifest = format!(
        "[package]\nname='date-feature-test'\nversion='0.1.0'\nedition='2021'\n[workspace]\n[dependencies]\nwho={{path={:?},default-features=false}}\n",
        env!("CARGO_MANIFEST_DIR"),
    );
    fs::write(root.join("Cargo.toml"), &manifest).unwrap();
    let check = || {
        Command::new(env!("CARGO"))
            .args(["check", "--offline"])
            .current_dir(root)
            .env("CARGO_TARGET_DIR", root.join("target"))
            .env_remove("RUSTFLAGS")
            .env_remove("CARGO_ENCODED_RUSTFLAGS")
            .output()
            .unwrap()
    };

    fs::write(root.join("src/main.rs"), "fn main() { who::error!(!rustc().matches(\">=1\"), \"compiler condition remains available\"); }").unwrap();
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

    fs::write(
        root.join("Cargo.toml"),
        manifest.replace(
            "default-features=false",
            "default-features=false,features=['date']",
        ),
    )
    .unwrap();
    let output = check();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}
