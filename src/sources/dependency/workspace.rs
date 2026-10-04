use std::path::{Path, PathBuf};

use crate::sources::manifest::{read_manifest, workspace_root};

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

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
