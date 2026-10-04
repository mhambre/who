use std::path::{Path, PathBuf};

use semver::Version;

use super::manifest::{read_manifest, workspace_root};

/// Read a package's declared MSRV and track inherited workspace inputs.
pub fn load(manifest_dir: &Path) -> Result<(Version, Vec<PathBuf>), String> {
    let path = manifest_dir.join("Cargo.toml");
    let mut tracked = vec![path.clone()];
    let manifest = read_manifest(&path)?;
    let declaration = manifest
        .get("package")
        .and_then(|package| package.get("rust-version"))
        .ok_or_else(|| {
            format!(
                "who: msrv() requires [package] rust-version in `{}`",
                path.display()
            )
        })?;
    let version = if let Some(value) = declaration.as_str() {
        parse_version(value)?
    } else if declaration.get("workspace").and_then(toml::Value::as_bool) == Some(true) {
        let root = workspace_root(manifest_dir, &manifest, &mut tracked)?;
        let path = root.join("Cargo.toml");
        let workspace = read_manifest(&path)?;
        tracked.push(path.clone());
        let value = workspace
            .get("workspace")
            .and_then(|workspace| workspace.get("package"))
            .and_then(|package| package.get("rust-version"))
            .and_then(toml::Value::as_str)
            .ok_or_else(|| {
                format!(
                    "who: msrv() requires [workspace.package] rust-version in `{}`",
                    path.display()
                )
            })?;
        parse_version(value)?
    } else {
        return Err(
            "who: rust-version must be a version string or inherit from the workspace".into(),
        );
    };
    Ok((version, tracked))
}

/// Pad Cargo's bare version to the three components semver requires.
fn parse_version(value: &str) -> Result<Version, String> {
    let parts: Vec<_> = value.split('.').collect();
    let invalid =
        || format!("who: invalid rust-version `{value}`: expected one to three numeric components");
    if parts.is_empty()
        || parts.len() > 3
        || parts
            .iter()
            .any(|part| part.is_empty() || !part.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return Err(invalid());
    }
    let mut normalized = parts.join(".");
    for _ in parts.len()..3 {
        normalized.push_str(".0");
    }
    Version::parse(&normalized).map_err(|_| invalid())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn normalizes_bare_cargo_versions() {
        for (input, expected) in [("1", "1.0.0"), ("1.74", "1.74.0"), ("1.74.1", "1.74.1")] {
            assert_eq!(
                parse_version(input).unwrap(),
                Version::parse(expected).unwrap()
            );
        }
        for input in [
            "",
            "1.",
            "1.2.3.4",
            "^1.74",
            "1.74.0-beta.1",
            "1.74.0+build",
            "01.74",
            "banana",
        ] {
            assert!(parse_version(input).is_err(), "{input}");
        }
    }

    #[test]
    fn direct_msrv_needs_no_lockfile() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("Cargo.toml");
        fs::write(&path, "[package]\nrust-version='1.74'\n").unwrap();
        let (version, tracked) = load(directory.path()).unwrap();
        assert_eq!(version, Version::new(1, 74, 0));
        assert_eq!(tracked, vec![path.clone()]);
        fs::write(&path, "[package]\nname='caller'\n").unwrap();
        assert!(load(directory.path())
            .unwrap_err()
            .contains("requires [package] rust-version"));
    }

    #[test]
    fn inherits_from_ancestor_or_explicit_workspace() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        for explicit in [false, true] {
            let workspace = if explicit {
                root.join("workspace")
            } else {
                root.to_path_buf()
            };
            let member = root.join("member");
            fs::create_dir_all(&workspace).unwrap();
            fs::create_dir_all(&member).unwrap();
            let workspace_path = workspace.join("Cargo.toml");
            fs::write(
                &workspace_path,
                "[workspace.package]\nrust-version='1.74'\n",
            )
            .unwrap();
            let suffix = if explicit {
                "workspace='../workspace'\n"
            } else {
                ""
            };
            fs::write(
                member.join("Cargo.toml"),
                format!("[package]\nrust-version.workspace=true\n{suffix}"),
            )
            .unwrap();
            let (version, tracked) = load(&member).unwrap();
            assert_eq!(version, Version::new(1, 74, 0));
            assert!(tracked.iter().any(|path| fs::canonicalize(path).unwrap()
                == fs::canonicalize(&workspace_path).unwrap()));
            fs::write(&workspace_path, "[workspace]\n").unwrap();
            assert!(load(&member)
                .unwrap_err()
                .contains("requires [workspace.package] rust-version"));
        }
    }
}
