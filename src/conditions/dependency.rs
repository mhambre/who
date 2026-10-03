use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use semver::Version;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Lockfile {
    version: Option<u32>,
    #[serde(rename = "package")]
    packages: Vec<Package>,
}

#[derive(Deserialize)]
struct Package {
    name: String,
    version: String,
    source: Option<String>,
    #[serde(default)]
    dependencies: Vec<String>,
}

pub struct Graph {
    lock: Lockfile,
    reachable: BTreeSet<usize>,
}

impl Graph {
    pub fn read(path: &Path, name: &str, version: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|error| format!("who: cannot read lockfile `{}`: {error}", path.display()))?;
        Self::parse(&content, name, version)
    }

    fn parse(content: &str, name: &str, version: &str) -> Result<Self, String> {
        let lock: Lockfile =
            toml::from_str(content).map_err(|error| format!("who: invalid Cargo.lock: {error}"))?;
        if !matches!(lock.version, Some(3 | 4)) {
            return Err("who: only Cargo.lock formats 3 and 4 are supported".into());
        }
        let roots: Vec<_> = lock
            .packages
            .iter()
            .enumerate()
            .filter(|(_, p)| p.name == name && p.version == version && p.source.is_none())
            .map(|(i, _)| i)
            .collect();
        if roots.len() != 1 {
            return Err(format!("who: cannot uniquely identify caller `{name} {version}` in Cargo.lock; dependency guards require a local Cargo package"));
        }
        let root = roots[0];
        let mut graph = Self {
            lock,
            reachable: BTreeSet::new(),
        };
        let mut pending = vec![root];
        while let Some(index) = pending.pop() {
            if !graph.reachable.insert(index) {
                continue;
            }
            for edge in &graph.lock.packages[index].dependencies {
                pending.push(graph.resolve_edge(edge)?);
            }
        }
        graph.reachable.remove(&root);
        Ok(graph)
    }

    fn resolve_edge(&self, edge: &str) -> Result<usize, String> {
        let mut parts = edge.splitn(3, ' ');
        let name = parts.next().unwrap_or("");
        let version = parts.next();
        let source = parts
            .next()
            .map(|value| {
                value
                    .strip_prefix('(')
                    .and_then(|v| v.strip_suffix(')'))
                    .ok_or_else(|| "who: invalid lockfile source selector".to_owned())
            })
            .transpose()?;
        let candidates: Vec<_> = self
            .lock
            .packages
            .iter()
            .enumerate()
            .filter(|(_, p)| {
                p.name == name
                    && version.map_or(true, |v| p.version == v)
                    && source.map_or(true, |s| {
                        p.source.as_deref().is_some_and(|resolved| {
                            // Git edge selectors omit precise revisions.
                            resolved == s
                                || (s.starts_with("git+")
                                    && !s.contains('#')
                                    && resolved.split('#').next() == Some(s))
                        })
                    })
            })
            .map(|(i, _)| i)
            .collect();
        if candidates.len() > 1 && version.is_some() && source.is_none() {
            let local: Vec<_> = candidates
                .iter()
                .copied()
                .filter(|i| self.lock.packages[*i].source.is_none())
                .collect();
            if local.len() == 1 {
                return Ok(local[0]);
            }
        }
        if candidates.len() != 1 {
            return Err(format!(
                "who: lockfile dependency edge `{edge}` is missing or ambiguous"
            ));
        }
        Ok(candidates[0])
    }

    pub fn version(&self, name: &str) -> Result<Version, String> {
        let candidates: Vec<_> = self
            .reachable
            .iter()
            .map(|i| &self.lock.packages[*i])
            .filter(|p| p.name == name)
            .collect();
        if candidates.is_empty() {
            return Err(format!("who: dependency `{name}` is not in the caller's resolved lockfile graph (use the Cargo package name, not an alias)"));
        }
        if candidates.len() > 1 {
            let versions = candidates
                .iter()
                .map(|p| {
                    format!(
                        "  - {} ({})",
                        p.version,
                        p.source.as_deref().unwrap_or("path")
                    )
                })
                .collect::<Vec<_>>()
                .join("\n");
            return Err(format!("who: dependency `{name}` resolves to multiple packages:\n{versions}\nThe condition is ambiguous; optional, target, build, and dev dependencies are included."));
        }
        Version::parse(&candidates[0].version)
            .map_err(|error| format!("who: invalid locked version: {error}"))
    }
}

pub fn locate(manifest_dir: &Path) -> Result<(PathBuf, Vec<PathBuf>), String> {
    let manifest = manifest_dir.join("Cargo.toml");
    let mut tracked = vec![manifest.clone()];
    let value = read_manifest(&manifest)?;
    let explicit = value
        .get("package")
        .and_then(|p| p.get("workspace"))
        .and_then(toml::Value::as_str);
    let root = if let Some(workspace) = explicit {
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
        root
    } else if value.get("workspace").is_some() {
        manifest_dir.to_path_buf()
    } else {
        let mut root = manifest_dir.to_path_buf();
        for ancestor in manifest_dir.ancestors().skip(1) {
            let path = ancestor.join("Cargo.toml");
            if path.is_file() {
                let value = read_manifest(&path)?;
                tracked.push(path);
                if value.get("workspace").is_some() {
                    if !is_excluded(&value, ancestor, &manifest) {
                        root = ancestor.to_path_buf();
                    }
                    break;
                }
            }
        }
        root
    };
    let lock = root.join("Cargo.lock");
    if !lock.is_file() {
        return Err(format!(
            "who: no Cargo.lock at `{}`; dependency guards require a lockfile",
            lock.display()
        ));
    }
    Ok((lock, tracked))
}

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

fn read_manifest(path: &Path) -> Result<toml::Value, String> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("who: cannot read `{}`: {error}", path.display()))?;
    toml::from_str(&content)
        .map_err(|error| format!("who: invalid manifest `{}`: {error}", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn package(name: &str, version: &str, source: Option<&str>, edges: &[&str]) -> String {
        let source = source
            .map(|source| format!("source = {source:?}\n"))
            .unwrap_or_default();
        format!("[[package]]\nname = {name:?}\nversion = {version:?}\n{source}dependencies = {edges:?}\n")
    }

    #[test]
    fn graph_reaches_only_callers_dependencies() {
        let content = format!(
            "version = 4\n{}{}{}{}{}",
            package("caller", "0.1.0", None, &["bridge"]),
            package("bridge", "1.0.0", None, &["demo 1.2.3"]),
            package(
                "demo",
                "1.2.3",
                Some("registry+https://example.invalid"),
                &[]
            ),
            package("unrelated", "0.1.0", None, &["demo 2.0.0"]),
            package("demo", "2.0.0", None, &[])
        );
        let graph = Graph::parse(&content, "caller", "0.1.0").unwrap();
        assert_eq!(graph.version("demo").unwrap(), Version::new(1, 2, 3));
        assert!(graph.version("unrelated").is_err());
        assert!(graph.version("caller").is_err());
    }

    #[test]
    fn source_selectors_cycles_and_same_version_ambiguity() {
        let content = format!(
            "version = 3\n{}{}{}",
            package(
                "caller",
                "0.1.0",
                None,
                &["demo 1.0.0", "demo 1.0.0 (git+https://example.invalid)"]
            ),
            package("demo", "1.0.0", None, &["caller"]),
            package(
                "demo",
                "1.0.0",
                Some("git+https://example.invalid#abc"),
                &[]
            )
        );
        let graph = Graph::parse(&content, "caller", "0.1.0").unwrap();
        let error = graph.version("demo").unwrap_err();
        assert!(error.contains("multiple packages"));
        assert!(
            error.contains("1.0.0 (path)") && error.contains("git+https://example.invalid#abc")
        );
    }

    #[test]
    fn malformed_or_missing_lock_information_fails_closed() {
        for content in ["invalid", "version=5\npackage=[]", "version=3\npackage=[]"] {
            assert!(Graph::parse(content, "caller", "0.1.0").is_err());
        }
        let content = format!(
            "version=3\n{}",
            package("caller", "0.1.0", None, &["missing"])
        );
        assert!(Graph::parse(&content, "caller", "0.1.0")
            .err()
            .unwrap()
            .contains("missing or ambiguous"));
    }

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
