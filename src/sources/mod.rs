//! Read external inputs independently of DSL parsing and diagnostics.

pub mod dependency;
#[cfg(feature = "file")]
pub mod file;
pub mod rustc;
