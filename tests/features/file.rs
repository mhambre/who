use std::{fs, process::Command};

#[cfg(not(feature = "file"))]
#[test]
fn disabled_file_diagnostic() {
    trybuild::TestCases::new().compile_fail("tests/features/file_disabled.rs");
}

#[test]
fn consumer_can_disable_and_enable_file_support() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    fs::create_dir(root.join("src")).unwrap();
    let manifest = format!(
        "[package]\nname='file-feature-test'\nversion='0.1.0'\nedition='2021'\n[workspace]\n[dependencies]\nwho={{path={:?},default-features=false}}\n",
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
    for dependency in ["sha2", "digest", "crypto-common"] {
        assert!(!lock.contains(&format!("name = {dependency:?}")));
    }

    fs::write(root.join("schema.proto"), "").unwrap();
    fs::write(root.join("src/main.rs"), "fn main() { who::error!(file(\"schema.proto\").changed_from(\"sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855\"), \"file drift\"); }").unwrap();
    let output = check();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr)
        .contains("file predicates require the `file` feature"));

    fs::write(
        root.join("Cargo.toml"),
        manifest.replace(
            "default-features=false",
            "default-features=false,features=['file']",
        ),
    )
    .unwrap();
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
}
