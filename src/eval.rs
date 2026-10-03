use std::collections::BTreeSet;
use std::path::PathBuf;
use std::{env, fs};

use proc_macro2::Span;
use semver::Version;

use crate::ast::{Expr, Predicate};
#[cfg(feature = "file")]
use crate::conditions::file;
use crate::conditions::{dependency, rustc};

pub struct Outcome {
    pub value: bool,
    pub details: Vec<String>,
}

#[cfg(all(test, feature = "date"))]
mod tests {
    use super::*;

    #[test]
    fn date_predicates_share_one_clock_read() {
        let now = crate::conditions::date::parse("2026-10-03T12:30:00Z").unwrap();
        let mut context = Context {
            manifest_dir: PathBuf::new(),
            graph: None,
            compiler: None,
            now: Some(now),
            tracked: BTreeSet::new(),
        };
        let guard: crate::parse::Guard = syn::parse_str(
            "date().after(\"2026-10-03T12:29:59Z\") && !date().after(\"2026-10-03T12:30:00Z\"), \"strict boundary\"",
        ).unwrap();
        let outcome = context.evaluate(&guard.condition).unwrap();
        assert!(outcome.value);
        assert!(outcome.details.iter().all(
            |details| details.contains("resolved: compilation time 2026-10-03T12:30:00+00:00")
        ));
        assert_eq!(context.now, Some(now));
        assert!(context.tracked.is_empty());
    }
}

pub struct Context {
    manifest_dir: PathBuf,
    graph: Option<dependency::Graph>,
    compiler: Option<Version>,
    #[cfg(feature = "date")]
    now: Option<chrono::DateTime<chrono::Utc>>,
    tracked: BTreeSet<PathBuf>,
}

impl Context {
    pub fn from_env(span: Span) -> syn::Result<Self> {
        let manifest_dir = env::var_os("CARGO_MANIFEST_DIR")
            .ok_or_else(|| syn::Error::new(span, "who: macros must be compiled through Cargo"))?
            .into();
        Ok(Self {
            manifest_dir,
            graph: None,
            compiler: None,
            #[cfg(feature = "date")]
            now: None,
            tracked: BTreeSet::new(),
        })
    }

    pub fn tracked_paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.tracked.iter()
    }

    pub fn evaluate(&mut self, expression: &Expr) -> syn::Result<Outcome> {
        match expression {
            Expr::Predicate(predicate, span) => self
                .predicate(predicate)
                .map_err(|message| syn::Error::new(*span, message)),
            Expr::Not(inner) => {
                let mut outcome = self.evaluate(inner)?;
                outcome.value = !outcome.value;
                Ok(outcome)
            }
            Expr::And(left, right) | Expr::Or(left, right) => {
                // Evaluate both sides so invalid guards cannot hide behind boolean operators.
                let mut left = self.evaluate(left)?;
                let right = self.evaluate(right)?;
                left.value = match expression {
                    Expr::And(_, _) => left.value && right.value,
                    _ => left.value || right.value,
                };
                left.details.extend(right.details);
                Ok(left)
            }
        }
    }

    fn dependency(&mut self, name: &str) -> Result<Version, String> {
        if self.graph.is_none() {
            if env::var_os("CARGO_RESOLVER_LOCKFILE_PATH").is_some() {
                return Err("who: custom resolver lockfile paths are not supported".into());
            }
            let (path, manifests) = dependency::locate(&self.manifest_dir)?;
            let compiler_dir = env::current_dir()
                .map_err(|error| format!("who: cannot identify compiler directory: {error}"))?;
            let root = fs::canonicalize(path.parent().unwrap())
                .map_err(|error| format!("who: cannot identify lockfile directory: {error}"))?;
            let compiler_dir = fs::canonicalize(compiler_dir)
                .map_err(|error| format!("who: cannot identify compiler directory: {error}"))?;
            if compiler_dir != root {
                return Err("who: the caller's workspace differs from the compiler's working directory; dependency guards cannot safely use this package's development lockfile when another workspace consumes it".into());
            }
            let name = env::var("CARGO_PKG_NAME")
                .map_err(|error| format!("who: missing package name: {error}"))?;
            let version = env::var("CARGO_PKG_VERSION")
                .map_err(|error| format!("who: missing package version: {error}"))?;
            self.graph = Some(dependency::Graph::read(&path, &name, &version)?);
            self.tracked.insert(path);
            self.tracked.extend(manifests);
        }
        self.graph.as_ref().unwrap().version(name)
    }

    fn compiler(&mut self) -> Result<Version, String> {
        if self.compiler.is_none() {
            self.compiler = Some(rustc::version()?);
        }
        Ok(self.compiler.as_ref().unwrap().clone())
    }

    fn predicate(&mut self, predicate: &Predicate) -> Result<Outcome, String> {
        let (value, expected, resolved) = match predicate {
            Predicate::DependencyChangedFrom { name, version } => {
                let resolved = self.dependency(name)?;
                (
                    resolved != *version,
                    format!("{name} {version}"),
                    format!("{name} {resolved}"),
                )
            }
            Predicate::DependencyMatches { name, requirement } => {
                let resolved = self.dependency(name)?;
                (
                    requirement.matches(&resolved),
                    format!("{name} matches {requirement}"),
                    format!("{name} {resolved}"),
                )
            }
            Predicate::RustcChangedFrom { version } => {
                let resolved = self.compiler()?;
                (
                    resolved != *version,
                    format!("rustc {version}"),
                    format!("rustc {resolved}"),
                )
            }
            Predicate::RustcMatches { requirement } => {
                let resolved = self.compiler()?;
                (
                    requirement.matches(&resolved),
                    format!("rustc matches {requirement}"),
                    format!("rustc {resolved}"),
                )
            }
            #[cfg(feature = "file")]
            Predicate::FileChangedFrom { path, hash } => {
                let full_path = self.manifest_dir.join(path);
                let resolved = file::hash(&full_path)?;
                self.tracked.insert(full_path);
                (
                    resolved != *hash,
                    format!("{} {hash}", path.display()),
                    format!("{} {resolved}", path.display()),
                )
            }
            #[cfg(feature = "date")]
            Predicate::DateAfter { deadline } => {
                let now = *self.now.get_or_insert_with(chrono::Utc::now);
                (
                    crate::conditions::date::after(now, *deadline),
                    format!("compilation time after {}", deadline.to_rfc3339()),
                    format!("compilation time {}", now.to_rfc3339()),
                )
            }
        };
        Ok(Outcome {
            value,
            details: vec![format!(
                "expected: {expected}\nresolved: {resolved}\npredicate: {value}"
            )],
        })
    }
}
