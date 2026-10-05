//! Know who changed what your code depends on.
//!
//! Add compile-time review triggers for code whose correctness, necessity, or performance depends on things the compiler cannot verify.
//!
//! Tests validate behavior. Use `who::warn!` and `who::error!` to bring code back up for review when the context around it changes.
//!
//! For example, suppose you use `OnceLock` while waiting for your minimum supported Rust version
//! to make `LazyLock` available:
//!
//! ```rust
//! use std::sync::OnceLock;
//!
//! static CONFIG: OnceLock<String> = OnceLock::new();
//!
//! fn config() -> &'static String {
//!     who::warn!(
//!         path(std::sync::LazyLock).exists(),
//!         "Check whether the MSRV now permits replacing OnceLock with LazyLock"
//!     );
//!
//!     CONFIG.get_or_init(|| "config".to_owned())
//! }
//! ```
//!
//! The current compiler's standard library determines whether the path exists. When it does, this
//! warning prompts you to check whether the project's MSRV has also advanced enough to use it.
//!
//! This brings the code back to your attention without relying on tests to fail when the alternative
//! becomes available.
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
