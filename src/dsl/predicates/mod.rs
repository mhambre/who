//! Route predicate families without changing shared expression grammar.

#[cfg(feature = "date")]
pub mod date;
pub mod dependency;
#[cfg(feature = "file")]
pub mod file;
pub mod msrv;
#[cfg(feature = "path")]
pub mod path;
pub mod rustc;
mod version;

use proc_macro2::Span;
use syn::parse::ParseStream;
use syn::{Ident, Result};

use crate::eval::{Context, EvalError, PredicateOutcome};

pub enum Predicate {
    Dependency(dependency::Condition),
    Rustc(rustc::Condition),
    Msrv(msrv::Condition),
    #[cfg(feature = "path")]
    Path(path::Condition),
    #[cfg(feature = "file")]
    File(file::Condition),
    #[cfg(feature = "date")]
    Date(date::Condition),
}

/// Route a receiver to its parser while preserving its diagnostic span.
pub fn parse(input: ParseStream<'_>) -> Result<(Predicate, Span)> {
    let kind: Ident = input.parse()?;
    let predicate = match kind.to_string().as_str() {
        "dependency" => Predicate::Dependency(dependency::Condition::parse(input)?),
        "rustc" => Predicate::Rustc(rustc::Condition::parse(input)?),
        "msrv" => Predicate::Msrv(msrv::Condition::parse(input)?),
        #[cfg(feature = "path")]
        "path" => Predicate::Path(path::Condition::parse(input)?),
        #[cfg(not(feature = "path"))]
        "path" => return Err(disabled(input, &kind, "path")?),
        #[cfg(feature = "file")]
        "file" => Predicate::File(file::Condition::parse(input)?),
        #[cfg(feature = "date")]
        "date" => Predicate::Date(date::Condition::parse(input)?),
        #[cfg(not(feature = "file"))]
        "file" => return Err(disabled(input, &kind, "file")?),
        #[cfg(not(feature = "date"))]
        "date" => return Err(disabled(input, &kind, "date")?),
        _ => {
            let arguments;
            syn::parenthesized!(arguments in input);
            return Err(syn::Error::new(
                kind.span(),
                "expected dependency(...), file(...), rustc(), msrv(), date(), or path(...)",
            ));
        }
    };
    Ok((predicate, kind.span()))
}

#[cfg(any(not(feature = "date"), not(feature = "file"), not(feature = "path")))]
/// Retain receiver syntax errors before reporting a disabled feature.
fn disabled(input: ParseStream<'_>, kind: &Ident, feature: &str) -> Result<syn::Error> {
    let arguments;
    syn::parenthesized!(arguments in input);
    Ok(syn::Error::new(
        kind.span(),
        format!("{feature} predicates require the `{feature}` feature on the `who` dependency"),
    ))
}

impl Predicate {
    /// Expose syntax deprecations independently of predicate truth values.
    pub fn deprecated_method(&self) -> Option<Span> {
        match self {
            Self::Dependency(condition) => condition.deprecated_method(),
            Self::Rustc(condition) => condition.deprecated_method(),
            Self::Msrv(condition) => condition.deprecated_method(),
            #[cfg(feature = "path")]
            Self::Path(_) => None,
            #[cfg(feature = "file")]
            Self::File(_) => None,
            #[cfg(feature = "date")]
            Self::Date(_) => None,
        }
    }

    /// Delegate evaluation without inspecting family-specific arguments.
    pub fn evaluate(
        &self,
        context: &mut Context,
    ) -> std::result::Result<PredicateOutcome, EvalError> {
        match self {
            Self::Dependency(condition) => condition.evaluate(context),
            Self::Rustc(condition) => condition.evaluate(context),
            Self::Msrv(condition) => condition.evaluate(context),
            #[cfg(feature = "path")]
            Self::Path(condition) => condition.evaluate(context),
            #[cfg(feature = "file")]
            Self::File(condition) => condition.evaluate(context),
            #[cfg(feature = "date")]
            Self::Date(condition) => condition.evaluate(context),
        }
    }
}
