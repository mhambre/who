use std::fs;
use std::path::Path;
use std::process::{Command, Output};

const EMPTY_HASH: &str = "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";

fn write(root: &Path, path: &str, contents: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
}

fn cargo(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO"))
        .arg("--offline")
        .args(args)
        .current_dir(root)
        .env("CARGO_TARGET_DIR", root.join("target"))
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .output()
        .unwrap()
}

fn succeeds(output: Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(output.status.success(), "{stderr}");
    stderr
}

fn fails(output: Output, expected: &str) {
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "expected failure: {expected}");
    assert!(
        stderr.contains(expected),
        "expected {expected:?}:\n{stderr}"
    );
}

fn macro_dependency() -> String {
    format!(
        "guards = {{ package = 'who', path = {:?} }}",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn runtime_ir(ir: &str) -> Vec<&str> {
    ir.lines()
        .filter(|line| !line.starts_with('!') && !line.starts_with(';'))
        .collect()
}

#[test]
fn workspace_graph_drift_and_zero_runtime_footprint() {
    let temporary = tempfile::tempdir().unwrap();
    let root = temporary.path();
    write(
        root,
        "Cargo.toml",
        &format!(
            r#"
[package]
name = "review-root"
version = "0.1.0"
edition = "2021"
[workspace]
members = ["app", "bridge"]
exclude = ["demo", "demo-old", "git-dep", "standalone"]
resolver = "2"
[dependencies]
{}
alias = {{ package = "demo", path = "demo" }}
"#,
            macro_dependency()
        ),
    );
    write(
        root,
        "src/lib.rs",
        "guards::error!(dependency(\"demo\").changed_from(\"1.2.3\"), \"root baseline\");",
    );
    write(
        root,
        "demo/Cargo.toml",
        "[package]\nname='demo'\nversion='1.2.3'\nedition='2021'\n",
    );
    write(root, "demo/src/lib.rs", "pub const VALUE: u32 = 42;");
    write(
        root,
        "demo-old/Cargo.toml",
        "[package]\nname='demo'\nversion='2.0.0'\nedition='2021'\n",
    );
    write(root, "demo-old/src/lib.rs", "");
    write(root, "bridge/Cargo.toml", "[package]\nname='bridge'\nversion='0.1.0'\nedition='2021'\n[dependencies]\nold={package='demo',path='../demo-old'}\n");
    write(root, "bridge/src/lib.rs", "");
    let app_manifest = format!("[package]\nname='review-app'\nversion='0.1.0'\nedition='2021'\nrust-version='1.74'\n[dependencies]\n{}\nalias={{package='demo',path='../demo'}}\n", macro_dependency());
    write(root, "app/Cargo.toml", &app_manifest);
    write(root, "app/schema.proto", "");
    let date_guards = if cfg!(feature = "date") {
        r#"guards::warn!(date().after("2000-01-01T00:00:00+05:30"), "UNIQUE_DATE_REASON");
    guards::error!(date().after("9999-12-31"), "UNIQUE_FUTURE_DATE_REASON");"#
    } else {
        ""
    };
    let guard_source = format!(
        r#"
fn main() {{
    guards::warn!(dependency("demo").changed_from("1.2.3"), "UNIQUE_DEP_REASON");
    guards::warn!(dependency("demo").compare("^1.2"), "UNIQUE_ACTIVE_REASON");
    guards::warn!(file("schema.proto").changed_from("{EMPTY_HASH}"), "UNIQUE_FILE_REASON");
    guards::warn!(!rustc().compare(">=1"), "UNIQUE_RUSTC_REASON");
    guards::warn!(msrv().compare(">=1.74"), "UNIQUE_MSRV_REASON");
    guards::error!(msrv().changed_from("1.74.0"), "UNIQUE_MSRV_BASELINE_REASON");
    guards::error!(rustc().matches("<1"), "UNIQUE_LEGACY_REASON");
    {date_guards}
    println!("{{}}", alias::VALUE);
}}
"#
    );
    write(root, "app/src/main.rs", &guard_source);
    let stderr = succeeds(cargo(root, &["check", "--workspace"]));
    assert!(
        stderr.contains("warning: use of deprecated constant"),
        "{stderr}"
    );
    assert!(stderr.contains("UNIQUE_ACTIVE_REASON"), "{stderr}");
    assert!(
        stderr.replace('\\', "/").contains("app/src/main.rs:4:"),
        "{stderr}"
    );
    if cfg!(feature = "date") {
        assert!(stderr.contains("UNIQUE_DATE_REASON"), "{stderr}");
        assert!(
            stderr.contains("expected: compilation time after 1999-12-31T18:30:00+00:00"),
            "{stderr}"
        );
        assert!(stderr.contains("resolved: compilation time"), "{stderr}");
        write(
            root,
            "app/src/main.rs",
            "fn main() { guards::error!(date().after(\"2000-01-01\"), \"past deadline\"); }",
        );
        fails(
            cargo(root, &["check", "-p", "review-app"]),
            "reason: past deadline",
        );
        write(root, "app/src/main.rs", &guard_source);
    }
    succeeds(cargo(root, &["build", "-p", "review-app"]));
    let executable_name = format!("review-app{}", std::env::consts::EXE_SUFFIX);
    let default_debug = fs::read(root.join("target/debug").join(&executable_name)).unwrap();
    for forbidden in [
        "UNIQUE_",
        "WHO_ASSUMPTION",
        "WHO_MATCHES",
        ".matches(...)",
        "sha256:",
        "schema.proto",
    ] {
        assert!(
            !default_debug
                .windows(forbidden.len())
                .any(|window| window == forbidden.as_bytes()),
            "debug executable contains {forbidden}"
        );
    }

    for release in [false, true] {
        write(root, "app/src/main.rs", &guard_source);
        let mut args = vec!["rustc", "-p", "review-app"];
        if release {
            args.push("--release");
        }
        args.extend(["--", "--emit=llvm-ir", "-Cdebuginfo=0"]);
        succeeds(cargo(root, &args));
        let profile = if release { "release" } else { "debug" };
        let ir_paths: Vec<_> = fs::read_dir(root.join(format!("target/{profile}/deps")))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|e| e == "ll"))
            .collect();
        assert!(!ir_paths.is_empty());
        for path in &ir_paths {
            let ir = fs::read_to_string(path).unwrap();
            for forbidden in [
                "UNIQUE_",
                "WHO_ASSUMPTION",
                "sha256:",
                "schema.proto",
                "1.2.3",
                "Cargo.lock",
                "requires revalidation",
                "compilation time",
                "2000-01-01",
                "9999-12-31",
            ] {
                assert!(!ir.contains(forbidden), "runtime IR contains {forbidden}");
            }
            assert!(!ir.contains("guards::"));
        }
        let executable = root
            .join(format!("target/{profile}"))
            .join(&executable_name);
        let bytes = fs::read(&executable).unwrap();
        assert!(!bytes
            .windows(b"UNIQUE_".len())
            .any(|window| window == b"UNIQUE_"));
        let output = Command::new(executable).output().unwrap();
        assert_eq!(output.stdout, b"42\n");
        let guarded_ir: Vec<_> = ir_paths
            .iter()
            .map(|path| fs::read_to_string(path).unwrap())
            .collect();
        write(
            root,
            "app/src/main.rs",
            "fn main() { println!(\"{}\", alias::VALUE); }",
        );
        succeeds(cargo(root, &args));
        for (path, guarded) in ir_paths.iter().zip(guarded_ir) {
            let plain = fs::read_to_string(path).unwrap();
            assert!(
                runtime_ir(&plain) == runtime_ir(&guarded),
                "guards changed the emitted runtime code"
            );
        }
    }

    write(
        root,
        "app/src/main.rs",
        &format!(
            "fn main() {{ guards::error!(file(\"schema.proto\").changed_from(\"{EMPTY_HASH}\"), \"file drift\"); }}"
        ),
    );
    succeeds(cargo(root, &["check", "-p", "review-app"]));

    write(
        root,
        "app/schema.proto",
        "changed without touching Rust source",
    );
    fails(
        cargo(root, &["check", "-p", "review-app"]),
        "reason: file drift",
    );

    write(
        root,
        "app/src/main.rs",
        "fn main() { guards::error!(dependency(\"demo\").changed_from(\"1.2.3\"), \"dependency drift\"); }",
    );
    succeeds(cargo(root, &["check", "-p", "review-app"]));
    write(
        root,
        "demo/Cargo.toml",
        "[package]\nname='demo'\nversion='1.2.4'\nedition='2021'\n",
    );
    fails(
        cargo(root, &["check", "-p", "review-app"]),
        "resolved: demo 1.2.4",
    );

    write(
        root,
        "app/Cargo.toml",
        &format!("{app_manifest}bridge={{path='../bridge'}}\n"),
    );
    write(
        root,
        "app/src/main.rs",
        "fn main() { guards::warn!(dependency(\"demo\").compare(\"*\"), \"ambiguous\"); }",
    );
    let output = cargo(root, &["check", "-p", "review-app"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("1.2.4 (path)") && stderr.contains("2.0.0 (path)"),
        "{stderr}"
    );
    fails(output, "The condition is ambiguous");

    write(root, "app/Cargo.toml", &format!("[package]\nname='review-app'\nversion='0.1.0'\nedition='2021'\n[dependencies]\n{}\nbridge={{path='../bridge'}}\n", macro_dependency()));
    write(
        root,
        "app/src/main.rs",
        "fn main() { guards::error!(dependency(\"demo\").changed_from(\"2.0.0\"), \"transitive baseline\"); }",
    );
    succeeds(cargo(root, &["check", "-p", "review-app"]));

    write(
        root,
        "git-dep/Cargo.toml",
        "[package]\nname='demo'\nversion='1.2.4'\nedition='2021'\n",
    );
    write(root, "git-dep/src/lib.rs", "");
    for args in [
        vec!["init", "-q"],
        vec!["add", "Cargo.toml", "src/lib.rs"],
        vec![
            "-c",
            "user.name=Fixture",
            "-c",
            "user.email=fixture@example.invalid",
            "commit",
            "-qm",
            "fixture",
        ],
    ] {
        assert!(Command::new("git")
            .args(args)
            .current_dir(root.join("git-dep"))
            .output()
            .unwrap()
            .status
            .success());
    }
    let git_url = format!(
        "file://{}{}",
        if cfg!(windows) { "/" } else { "" },
        root.join("git-dep")
            .display()
            .to_string()
            .replace('\\', "/")
    );
    write(
        root,
        "app/Cargo.toml",
        &format!("{app_manifest}git_alias={{package='demo',git='{git_url}'}}\n"),
    );
    write(
        root,
        "app/src/main.rs",
        "fn main() { guards::error!(dependency(\"demo\").changed_from(\"1.2.4\"), \"git baseline\"); }",
    );
    let fetch = Command::new(env!("CARGO"))
        .args(["fetch", "--manifest-path", "app/Cargo.toml"])
        .current_dir(root)
        .env("CARGO_TARGET_DIR", root.join("target"))
        .output()
        .unwrap();
    succeeds(fetch);
    fails(
        cargo(root, &["check", "-p", "review-app"]),
        "multiple packages",
    );
    write(root, "app/Cargo.toml", &format!("[package]\nname='review-app'\nversion='0.1.0'\nedition='2021'\n[dependencies]\n{}\ngit_alias={{package='demo',git='{git_url}'}}\n", macro_dependency()));
    succeeds(cargo(root, &["check", "-p", "review-app"]));

    let version_output = Command::new("rustc").arg("--version").output().unwrap();
    let compiler = String::from_utf8(version_output.stdout).unwrap();
    let version = compiler.split_whitespace().nth(1).unwrap();
    write(
        root,
        "app/src/main.rs",
        &format!("fn main() {{ guards::error!(rustc().changed_from({version:?}), \"compiler baseline\"); }}"),
    );
    succeeds(cargo(root, &["check", "-p", "review-app"]));

    write(root, "standalone/Cargo.toml", &format!("[package]\nname='standalone'\nversion='0.1.0'\nedition='2021'\n[dependencies]\n{}\nbridge={{path='../bridge',optional=true}}\n", macro_dependency()));
    write(
        root,
        "standalone/src/lib.rs",
        "guards::error!(dependency(\"demo\").changed_from(\"2.0.0\"), \"inactive dependency drift\");",
    );
    succeeds(cargo(
        root,
        &["check", "--manifest-path", "standalone/Cargo.toml"],
    ));
    write(
        root,
        "demo-old/Cargo.toml",
        "[package]\nname='demo'\nversion='2.0.1'\nedition='2021'\n",
    );
    fails(
        cargo(root, &["check", "--manifest-path", "standalone/Cargo.toml"]),
        "reason: inactive dependency drift",
    );
    write(
        root,
        "standalone/src/lib.rs",
        "guards::warn!(dependency(\"demo\").changed_from(\"2.0.1\"), \"standalone baseline\");",
    );
    succeeds(cargo(
        root,
        &["check", "--manifest-path", "standalone/Cargo.toml"],
    ));
    let manifest = fs::read_to_string(root.join("app/Cargo.toml")).unwrap();
    write(
        root,
        "app/Cargo.toml",
        &format!("{manifest}standalone={{path='../standalone'}}\n"),
    );
    fails(
        cargo(root, &["check", "-p", "review-app"]),
        "cannot safely use this package's development lockfile",
    );
}
