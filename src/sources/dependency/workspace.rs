use std::fs;
use std::path::{Path, PathBuf};

/// Find the caller's workspace lockfile and collect the consulted manifests.
pub fn locate(manifest_dir: &Path) -> Result<(PathBuf, Vec<PathBuf>), String> {
    let manifest = manifest_dir.join("Cargo.toml");
    let mut tracked = vec![manifest.clone()];
    let value = read_manifest(&manifest)?;
    let root = workspace_root(manifest_dir, &value, &mut tracked)?;
    let lock = root.join("Cargo.lock");
    if !lock.is_file() {
        return Err(format!(
            "who: no Cargo.lock at `{}`; dependency guards require a lockfile",
            lock.display()
        ));
    }
    Ok((lock, tracked))
}

/// Prefer an explicit workspace, then the caller itself, then an ancestor.
fn workspace_root(
    manifest_dir: &Path,
    value: &toml::Value,
    tracked: &mut Vec<PathBuf>,
) -> Result<PathBuf, String> {
    let explicit = value
        .get("package")
        .and_then(|p| p.get("workspace"))
        .and_then(toml::Value::as_str);
    if let Some(workspace) = explicit {
        let root = manifest_dir.join(workspace);
        let path = root.join("Cargo.toml");
        let value = read_manifest(&path)?;
        if value.get("workspace").is_none() {
            return Err(format!(
                "who: `{}` does not define a workspace",
                path.display()
            ));
        }
        tracked.push(path);
        Ok(root)
    } else if value.get("workspace").is_some() {
        Ok(manifest_dir.to_path_buf())
    } else {
        ancestor_workspace(manifest_dir, tracked)
    }
}

/// Stop at the nearest workspace and keep excluded callers independent.
fn ancestor_workspace(manifest_dir: &Path, tracked: &mut Vec<PathBuf>) -> Result<PathBuf, String> {
    let manifest = manifest_dir.join("Cargo.toml");
    for ancestor in manifest_dir.ancestors().skip(1) {
        let path = ancestor.join("Cargo.toml");
        if path.is_file() {
            let value = read_manifest(&path)?;
            tracked.push(path);
            if value.get("workspace").is_some() {
                return Ok(if is_excluded(&value, ancestor, &manifest) {
                    manifest_dir.to_path_buf()
                } else {
                    ancestor.to_path_buf()
                });
            }
        }
    }
    Ok(manifest_dir.to_path_buf())
}

/// Apply workspace exclusion unless the caller is also explicitly listed.
fn is_excluded(manifest: &toml::Value, root: &Path, caller: &Path) -> bool {
    let workspace = &manifest["workspace"];
    let contains = |key| {
        workspace
            .get(key)
            .and_then(toml::Value::as_array)
            .is_some_and(|paths| {
                paths
                    .iter()
                    .filter_map(toml::Value::as_str)
                    .any(|path| caller.starts_with(root.join(path)))
            })
    };
    contains("exclude") && !contains("members")
}

/// Read a manifest with path-aware errors for workspace discovery.
fn read_manifest(path: &Path) -> Result<toml::Value, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("who: cannot read `{}`: {error}", path.display()))?;
    toml::from_str(&content)
        .map_err(|error| format!("who: invalid manifest `{}`: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_explicit_workspace_outside_ancestor_tree() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        fs::create_dir_all(root.join("workspace")).unwrap();
        fs::create_dir_all(root.join("member")).unwrap();
        fs::write(
            root.join("workspace/Cargo.toml"),
            "[workspace]\nmembers=['../member']\n",
        )
        .unwrap();
        fs::write(root.join("workspace/Cargo.lock"), "version=3\n").unwrap();
        fs::write(
            root.join("member/Cargo.toml"),
            "[package]\nname='caller'\nversion='0.1.0'\nworkspace='../workspace'\n",
        )
        .unwrap();
        let (lock, tracked) = locate(&root.join("member")).unwrap();
        assert_eq!(
            fs::canonicalize(lock).unwrap(),
            fs::canonicalize(root.join("workspace/Cargo.lock")).unwrap()
        );
        assert_eq!(tracked.len(), 2);
    }

    #[test]
    fn excluded_package_uses_its_own_lockfile() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path();
        fs::create_dir_all(root.join("excluded")).unwrap();
        fs::write(
            root.join("Cargo.toml"),
            "[workspace]\nexclude=['excluded']\n",
        )
        .unwrap();
        fs::write(root.join("Cargo.lock"), "version=3\n").unwrap();
        fs::write(
            root.join("excluded/Cargo.toml"),
            "[package]\nname='caller'\nversion='0.1.0'\n",
        )
        .unwrap();
        fs::write(root.join("excluded/Cargo.lock"), "version=3\n").unwrap();
        let (lock, _) = locate(&root.join("excluded")).unwrap();
        assert_eq!(lock, root.join("excluded/Cargo.lock"));
    }
}
