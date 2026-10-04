use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use semver::Version;
use serde::Deserialize;

#[derive(Deserialize)]
struct Lockfile {
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

struct Selector<'a> {
    name: &'a str,
    version: Option<&'a str>,
    source: Option<&'a str>,
}

impl<'a> Selector<'a> {
    /// Decode Cargo's optional version and parenthesized source selectors.
    fn parse(edge: &'a str) -> Result<Self, String> {
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
        Ok(Self {
            name,
            version,
            source,
        })
    }

    /// Git edge selectors can omit the precise revision recorded on a package.
    fn matches(&self, package: &Package) -> bool {
        package.name == self.name
            && self
                .version
                .map_or(true, |version| package.version == version)
            && self.source.map_or(true, |source| {
                package.source.as_deref().is_some_and(|resolved| {
                    resolved == source
                        || (source.starts_with("git+")
                            && !source.contains('#')
                            && resolved.split('#').next() == Some(source))
                })
            })
    }
}

/// Require exactly one local caller package before following dependency edges.
fn caller_index(lock: &Lockfile, name: &str, version: &str) -> Result<usize, String> {
    let roots: Vec<_> = lock
        .packages
        .iter()
        .enumerate()
        .filter(|(_, package)| {
            package.name == name && package.version == version && package.source.is_none()
        })
        .map(|(index, _)| index)
        .collect();
    if roots.len() != 1 {
        return Err(format!("who: cannot uniquely identify caller `{name} {version}` in Cargo.lock; dependency guards require a local Cargo package"));
    }
    Ok(roots[0])
}

impl Graph {
    /// Read a lockfile and identify the local caller before traversing its graph.
    pub fn read(path: &Path, name: &str, version: &str) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|error| format!("who: cannot read lockfile `{}`: {error}", path.display()))?;
        Self::parse(&content, name, version)
    }

    /// Walk reachable packages, including inactive edges recorded by Cargo.
    fn parse(content: &str, name: &str, version: &str) -> Result<Self, String> {
        let lock: Lockfile =
            toml::from_str(content).map_err(|error| format!("who: invalid Cargo.lock: {error}"))?;
        if !matches!(lock.version, Some(3 | 4)) {
            return Err("who: only Cargo.lock formats 3 and 4 are supported".into());
        }
        let root = caller_index(&lock, name, version)?;
        let mut graph = Self {
            lock,
            reachable: BTreeSet::new(),
        };
        graph.traverse(root)?;
        Ok(graph)
    }

    /// Visit cycles once and exclude the caller from its own dependencies.
    fn traverse(&mut self, root: usize) -> Result<(), String> {
        let mut pending = vec![root];
        while let Some(index) = pending.pop() {
            if !self.reachable.insert(index) {
                continue;
            }
            for edge in &self.lock.packages[index].dependencies {
                pending.push(self.resolve_edge(edge)?);
            }
        }
        self.reachable.remove(&root);
        Ok(())
    }

    /// Match name, version, and source selectors without choosing ambiguous edges.
    fn resolve_edge(&self, edge: &str) -> Result<usize, String> {
        let selector = Selector::parse(edge)?;
        let candidates: Vec<_> = self
            .lock
            .packages
            .iter()
            .enumerate()
            .filter(|(_, package)| selector.matches(package))
            .map(|(i, _)| i)
            .collect();
        if candidates.len() > 1 && selector.version.is_some() && selector.source.is_none() {
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

    /// Resolve a package name and report every competing package when ambiguous.
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
}
