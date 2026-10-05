//! Know who changed what your code depends on.
//!
//! Add compile-time review triggers for code whose correctness, necessity, or performance depends on things the compiler cannot verify.
//!
//! Tests validate behavior. Use `who::warn!` and `who::error!` to bring code back up for review when the context around it changes.
//!
//! For example, suppose a compatibility shim can be replaced by a newer standard-library API:
//!
//! ```rust,ignore
//! who::warn!(
//!     path(std::sync::LazyLock).exists(),
//!     "Review this OnceLock shim and replace it with std::sync::LazyLock"
//! );
//! ```
//!
//! The shim can be correct and well tested, but those tests will not tell you when the newer API becomes available.
//! The `path` feature brings this code back to your attention when `std::sync::LazyLock` is importable.
//!
//! ## DSL reference
//!
//! | Function | Feature | `exists` | `changed_from` | `compare` | `after` |
//! | --- | --- | :---: | :---: | :---: | :---: |
//! | `path(std::sync::LazyLock)` | `path` | ✓ | | | |
//! | `rustc()` | built-in | | ✓ | ✓ | |
//! | `msrv()` | built-in | | ✓ | ✓ | |
//! | `dependency("name")` | built-in | | ✓ | ✓ | |
//! | `file("path")` | `file` | | ✓ | | |
//! | `date()` | `date` | | | | ✓ |
//!
//! *All feature-gated DSL fields are enabled by default.*
//! Conditions can combine with `&&`, `||`, and `!`; parentheses group multiple conditions.
//! Precedence is `!` > `&&` > `||`.
//!
//! ```rust
//! who::warn!(
//!     rustc().changed_from("1.95.0") && !msrv().compare(">=1.74"),
//!     "Recheck the compiler workaround"
//! );
//! ```

mod diagnostic;
mod dsl;
mod eval;
mod sources;

use proc_macro::TokenStream;

/// Warn when a condition holds. Respects the caller's `deprecated` lint level.
#[proc_macro]
pub fn warn(input: TokenStream) -> TokenStream {
    expand(input, diagnostic::Severity::Warning)
}

/// Fail compilation when a condition holds.
#[proc_macro]
pub fn error(input: TokenStream) -> TokenStream {
    expand(input, diagnostic::Severity::Error)
}

/// Parse, evaluate, and expand a guard without emitting runtime machinery.
fn expand(input: TokenStream, severity: diagnostic::Severity) -> TokenStream {
    let result = syn::parse::<dsl::Guard>(input).and_then(|guard| {
        let mut context = eval::Context::from_env(guard.span)?;
        let outcome = eval::evaluate(&guard.condition, &mut context)?;
        Ok(diagnostic::expand(
            &guard,
            outcome,
            context.tracked_paths(),
            severity,
        ))
    });
    result
        .unwrap_or_else(|error| error.into_compile_error())
        .into()
}
