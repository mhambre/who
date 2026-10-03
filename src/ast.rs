#[cfg(feature = "file")]
use std::path::PathBuf;

use proc_macro2::Span;
use semver::{Version, VersionReq};

pub enum Expr {
    Predicate(Predicate, Span),
    Not(Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Or(Box<Expr>, Box<Expr>),
}

pub enum Predicate {
    DependencyChangedFrom {
        name: String,
        version: Version,
    },
    DependencyMatches {
        name: String,
        requirement: VersionReq,
    },
    #[cfg(feature = "file")]
    FileChangedFrom {
        path: PathBuf,
        hash: String,
    },
    RustcChangedFrom {
        version: Version,
    },
    RustcMatches {
        requirement: VersionReq,
    },
    #[cfg(feature = "date")]
    DateAfter {
        deadline: chrono::DateTime<chrono::Utc>,
    },
}
