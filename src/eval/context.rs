use std::collections::BTreeSet;
use std::env;
use std::path::PathBuf;

use proc_macro2::Span;
use semver::Version;

use super::EvalError;
use crate::sources::{dependency, msrv, rustc};

pub struct Context {
    manifest_dir: PathBuf,
    graph: Option<dependency::Graph>,
    compiler: Option<Version>,
    msrv: Option<Version>,
    #[cfg(feature = "date")]
    now: Option<chrono::DateTime<chrono::Utc>>,
    tracked: BTreeSet<PathBuf>,
}

impl Context {
    /// Capture the caller's manifest directory without loading predicate inputs.
    pub fn from_env(span: Span) -> syn::Result<Self> {
        let manifest_dir = env::var_os("CARGO_MANIFEST_DIR")
            .ok_or_else(|| syn::Error::new(span, "who: macros must be compiled through Cargo"))?
            .into();
        Ok(Self::new(manifest_dir))
    }

    fn new(manifest_dir: PathBuf) -> Self {
        Self {
            manifest_dir,
            graph: None,
            compiler: None,
            msrv: None,
            #[cfg(feature = "date")]
            now: None,
            tracked: BTreeSet::new(),
        }
    }

    /// Expose consulted files and manifests for expansion tracking.
    pub fn tracked_paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.tracked.iter()
    }

    /// Load the caller's graph once and fail closed on ambiguous package names.
    pub fn dependency(&mut self, name: &str) -> Result<Version, EvalError> {
        if self.graph.is_none() {
            let (graph, paths) = dependency::load(&self.manifest_dir)?;
            self.graph = Some(graph);
            self.tracked.extend(paths);
        }
        self.graph
            .as_ref()
            .unwrap()
            .version(name)
            .map_err(EvalError::from)
    }

    /// Query the compiler only on the first compiler predicate.
    pub fn compiler(&mut self) -> Result<Version, EvalError> {
        if self.compiler.is_none() {
            self.compiler = Some(rustc::version()?);
        }
        Ok(self.compiler.as_ref().unwrap().clone())
    }

    /// Read the declared minimum compiler version once per invocation.
    pub fn msrv(&mut self) -> Result<Version, EvalError> {
        if self.msrv.is_none() {
            let (version, paths) = msrv::load(&self.manifest_dir)?;
            self.msrv = Some(version);
            self.tracked.extend(paths);
        }
        Ok(self.msrv.as_ref().unwrap().clone())
    }

    /// Resolve paths against the caller and track successfully hashed files.
    #[cfg(feature = "file")]
    pub fn file_hash(&mut self, path: &std::path::Path) -> Result<String, EvalError> {
        let full_path = self.manifest_dir.join(path);
        let hash = crate::sources::file::hash(&full_path)?;
        self.tracked.insert(full_path);
        Ok(hash)
    }

    /// Share one UTC instant across every date predicate in this invocation.
    #[cfg(feature = "date")]
    pub fn now(&mut self) -> chrono::DateTime<chrono::Utc> {
        *self.now.get_or_insert_with(chrono::Utc::now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dsl::Guard;

    #[test]
    fn msrv_is_cached_and_its_manifest_is_tracked() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("Cargo.toml");
        std::fs::write(&path, "[package]\nrust-version='1.74'\n").unwrap();
        let mut context = Context::new(directory.path().to_path_buf());
        assert_eq!(context.msrv().unwrap(), Version::new(1, 74, 0));
        std::fs::write(&path, "[package]\nrust-version='1.80'\n").unwrap();
        assert_eq!(context.msrv().unwrap(), Version::new(1, 74, 0));
        assert_eq!(context.tracked_paths().collect::<Vec<_>>(), vec![&path]);
        let mut next = Context::new(directory.path().to_path_buf());
        assert_eq!(next.msrv().unwrap(), Version::new(1, 80, 0));
    }

    #[test]
    fn boolean_evaluation_keeps_leaf_evidence_in_source_order() {
        let mut context = Context::new(PathBuf::new());
        context.compiler = Some(Version::new(1, 95, 0));
        let guard: Guard = syn::parse_str(
            "!(rustc().compare(\"<1\") || rustc().changed_from(\"1.95.0\")) && rustc().compare(\">=1\"), \"reason\"",
        ).unwrap();
        let outcome = crate::eval::evaluate(&guard.condition, &mut context).unwrap();
        assert!(outcome.value);
        assert_eq!(outcome.evidence.len(), 3);
        assert_eq!(outcome.evidence[0].expected, "rustc matches <1");
        assert_eq!(outcome.evidence[1].expected, "rustc 1.95.0");
        assert!(!outcome.evidence[0].value);
        assert!(!outcome.evidence[1].value);
        assert!(outcome.evidence[2].value);
        assert!(context.tracked.is_empty());
    }

    #[cfg(feature = "date")]
    #[test]
    fn date_predicates_share_one_clock_read() {
        let now = crate::dsl::predicates::date::parse_instant("2026-10-03T12:30:00Z").unwrap();
        let mut context = Context::new(PathBuf::new());
        context.now = Some(now);
        let guard: Guard = syn::parse_str(
            "date().after(\"2026-10-03T12:29:59Z\") && !date().after(\"2026-10-03T12:30:00Z\"), \"strict boundary\"",
        ).unwrap();
        let outcome = crate::eval::evaluate(&guard.condition, &mut context).unwrap();
        assert!(outcome.value);
        assert!(outcome
            .evidence
            .iter()
            .all(|evidence| evidence.resolved == "compilation time 2026-10-03T12:30:00+00:00"));
        assert_eq!(context.now, Some(now));
        assert!(context.tracked.is_empty());
    }
}
