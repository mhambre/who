//! Read external inputs independently of DSL parsing and diagnostics.

pub mod dependency;
#[cfg(feature = "file")]
pub mod file;
mod manifest;
pub mod msrv;
#[cfg(feature = "path")]
pub mod path;
pub mod rustc;
