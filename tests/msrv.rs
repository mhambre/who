#[path = "support/mod.rs"]
mod support;

use std::fs;
use std::process::Output;

use support::cargo_project::CargoProject;

fn diagnostics(output: Output, success: bool) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert_eq!(output.status.success(), success, "{stderr}");
    stderr
}

#[test]
fn declared_and_inherited_msrv_track_manifest_changes() {
    let project = CargoProject::new("msrv-consumer");
    let path = project.root().join("Cargo.toml");
    let original = fs::read_to_string(&path).unwrap();
    let direct = original.replace("edition='2021'", "edition='2021'\nrust-version='1.74'");
    fs::write(&path, &direct).unwrap();
    fs::write(
        project.root().join("src/main.rs"),
        r#"
fn main() {
    who::error!(msrv().changed_from("1.74.0"), "revisit compatibility");
    who::error!(!msrv().compare(">=1.74, <1.75"), "supported minimum");
    who::warn!(msrv().compare("=1.74.0"), "minimum version review");
    who::error!(msrv().matches("<1"), "inactive alias");
    who::error!(!msrv().matches(">=1.74, <1.75"), "same semver semantics");
}
"#,
    )
    .unwrap();
    let output = diagnostics(project.check(), true);
    assert!(output.contains("minimum version review"), "{output}");
    assert!(output.contains("resolved: msrv 1.74.0"), "{output}");
    assert!(
        output.contains(".matches(...) is deprecated; use .compare(...) instead"),
        "{output}"
    );
    fs::write(
        &path,
        direct.replace("rust-version='1.74'", "rust-version='1.75'"),
    )
    .unwrap();
    let output = diagnostics(project.check(), false);
    assert!(output.contains("revisit compatibility"), "{output}");
    assert!(output.contains("resolved: msrv 1.75.0"), "{output}");

    let member = project.root().join("member");
    fs::create_dir_all(member.join("src")).unwrap();
    fs::copy(
        project.root().join("src/main.rs"),
        member.join("src/main.rs"),
    )
    .unwrap();
    let member_manifest = original.replace("[workspace]", "").replace(
        "edition='2021'",
        "edition='2021'\nrust-version.workspace=true",
    );
    fs::write(member.join("Cargo.toml"), member_manifest).unwrap();
    let workspace =
        "[workspace]\nmembers=['member']\nresolver='2'\n[workspace.package]\nrust-version='1.74'\n";
    fs::write(&path, workspace).unwrap();
    diagnostics(project.check(), true);
    fs::write(&path, workspace.replace("'1.74'", "'1.75'")).unwrap();
    let output = diagnostics(project.check(), false);
    assert!(output.contains("resolved: msrv 1.75.0"), "{output}");

    fs::write(&path, original).unwrap();
    let output = diagnostics(project.check(), false);
    assert!(
        output.contains("msrv() requires [package] rust-version"),
        "{output}"
    );
}
