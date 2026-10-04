mod graph;
mod workspace;

use std::path::{Path, PathBuf};
use std::{env, fs};

pub use graph::Graph;
use workspace::locate;

/// Load only a caller-owned lockfile and return every input Cargo must track.
pub fn load(manifest_dir: &Path) -> Result<(Graph, Vec<PathBuf>), String> {
    if env::var_os("CARGO_RESOLVER_LOCKFILE_PATH").is_some() {
        return Err("who: custom resolver lockfile paths are not supported".into());
    }
    let (path, mut manifests) = locate(manifest_dir)?;
    require_caller_workspace(&path)?;
    let name = env::var("CARGO_PKG_NAME")
        .map_err(|error| format!("who: missing package name: {error}"))?;
    let version = env::var("CARGO_PKG_VERSION")
        .map_err(|error| format!("who: missing package version: {error}"))?;
    let graph = Graph::read(&path, &name, &version)?;
    manifests.push(path);
    Ok((graph, manifests))
}

/// Reject development lockfiles when another workspace consumes the caller.
fn require_caller_workspace(lockfile: &Path) -> Result<(), String> {
    let compiler_dir = env::current_dir()
        .map_err(|error| format!("who: cannot identify compiler directory: {error}"))?;
    let root = fs::canonicalize(lockfile.parent().unwrap())
        .map_err(|error| format!("who: cannot identify lockfile directory: {error}"))?;
    let compiler_dir = fs::canonicalize(compiler_dir)
        .map_err(|error| format!("who: cannot identify compiler directory: {error}"))?;
    if compiler_dir != root {
        return Err("who: the caller's workspace differs from the compiler's working directory; dependency guards cannot safely use this package's development lockfile when another workspace consumes it".into());
    }
    Ok(())
}
