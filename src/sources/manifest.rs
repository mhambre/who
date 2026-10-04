use std::fs;
use std::path::{Path, PathBuf};

/// Prefer an explicit workspace, then the caller itself, then an ancestor.
pub fn workspace_root(
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
pub fn read_manifest(path: &Path) -> Result<toml::Value, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("who: cannot read `{}`: {error}", path.display()))?;
    toml::from_str(&content)
        .map_err(|error| format!("who: invalid manifest `{}`: {error}", path.display()))
}
