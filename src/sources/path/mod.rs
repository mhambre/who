mod context;
mod probe;

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

use context::Compilation;

type Results = HashMap<(Compilation, String), bool>;
static RESULTS: OnceLock<Mutex<Results>> = OnceLock::new();

/// Cache metadata probes by path and the active compilation's complete context.
pub fn exists(path: &str) -> Result<bool, String> {
    let compilation = Compilation::current()?;
    let mut results = RESULTS
        .get_or_init(Mutex::default)
        .lock()
        .map_err(|_| "who: path probe cache is unavailable")?;
    cached_probe(&mut results, &compilation, path, probe::compile)
}

/// Check the environment before interpreting a failed import as absence.
fn cached_probe(
    results: &mut Results,
    compilation: &Compilation,
    path: &str,
    mut compile: impl FnMut(&Compilation, &str) -> Result<(bool, String), String>,
) -> Result<bool, String> {
    let key = (compilation.clone(), path.to_owned());
    if let Some(value) = results.get(&key) {
        return Ok(*value);
    }
    let baseline = (compilation.clone(), String::new());
    if let Entry::Vacant(entry) = results.entry(baseline) {
        let (success, diagnostics) = compile(compilation, "#![no_std]\n")?;
        if !success {
            return Err(format!(
                "who: cannot compile a baseline path probe in the caller's context:\n{diagnostics}"
            ));
        }
        entry.insert(true);
    }
    let source = import_source(path);
    let (value, _) = compile(compilation, &source)?;
    results.insert(key, value);
    Ok(value)
}

/// Alias the sysroot crate to avoid implicit-import collisions in Rust 2015.
fn import_source(path: &str) -> String {
    let path = path.trim_start_matches("::");
    let (root, tail) = path.split_once("::").unwrap_or((path, ""));
    let import = if tail.is_empty() {
        "__who_sysroot".into()
    } else {
        format!("__who_sysroot::{tail}")
    };
    format!("#![no_std]\npub extern crate {root} as __who_sysroot;\npub use {import} as __who_path_import;\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compilation(target: &str) -> Compilation {
        Compilation {
            compiler: "rustc".into(),
            directory: ".".into(),
            arguments: vec!["--target".into(), target.into()],
        }
    }

    #[test]
    fn caches_positive_and_negative_results_only_with_identical_context() {
        let mut results = Results::new();
        let mut calls = 0;
        let mut compile = |_: &Compilation, source: &str| {
            calls += 1;
            Ok((!source.contains("Missing"), String::new()))
        };
        for _ in 0..2 {
            assert!(cached_probe(
                &mut results,
                &compilation("one"),
                "std::sync::LazyLock",
                &mut compile
            )
            .unwrap());
            assert!(!cached_probe(
                &mut results,
                &compilation("one"),
                "std::Missing",
                &mut compile
            )
            .unwrap());
        }
        assert!(cached_probe(
            &mut results,
            &compilation("two"),
            "std::sync::LazyLock",
            &mut compile
        )
        .unwrap());
        assert_eq!(calls, 5);
    }

    #[test]
    fn baseline_failure_is_an_error_and_is_not_cached() {
        let mut results = Results::new();
        let error = cached_probe(&mut results, &compilation("broken"), "core::mem", |_, _| {
            Ok((false, "target missing".into()))
        })
        .unwrap_err();
        assert!(error.contains("baseline path probe"));
        assert!(results.is_empty());
    }

    #[test]
    fn compiler_errors_are_not_cached_as_missing_paths() {
        let mut results = Results::new();
        let context = compilation("one");
        let error = cached_probe(&mut results, &context, "core::mem", |_, source| {
            if source.contains("pub use") {
                Err("compiler could not launch".into())
            } else {
                Ok((true, String::new()))
            }
        })
        .unwrap_err();
        assert_eq!(error, "compiler could not launch");
        assert!(!results.contains_key(&(context.clone(), "core::mem".into())));
        assert!(
            cached_probe(&mut results, &context, "core::mem", |_, _| Ok((
                true,
                String::new()
            )))
            .unwrap()
        );
    }

    #[test]
    fn cache_identity_includes_all_resolution_inputs() {
        let mut results = Results::new();
        let original = compilation("one");
        let mut variants = vec![original.clone(); 6];
        variants[0].compiler = "other-rustc".into();
        variants[1].directory = "other-directory".into();
        variants[2]
            .arguments
            .extend(["--edition".into(), "2024".into()]);
        variants[3]
            .arguments
            .extend(["--cfg".into(), "custom".into()]);
        variants[4]
            .arguments
            .extend(["--sysroot".into(), "/other-sysroot".into()]);
        variants[5]
            .arguments
            .extend(["--extern".into(), "core=/other-core.rmeta".into()]);
        let mut calls = 0;
        let mut compile = |_: &Compilation, _: &str| {
            calls += 1;
            Ok((true, String::new()))
        };
        for context in std::iter::once(&original).chain(variants.iter()) {
            for _ in 0..2 {
                assert!(cached_probe(&mut results, context, "core::mem", &mut compile).unwrap());
            }
        }
        assert_eq!(calls, 14);
    }
}
