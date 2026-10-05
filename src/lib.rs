//! Know who changed what your code depends on.
//!
//! Add compile-time review triggers for code whose correctness, necessity, or performance depends on things the compiler cannot verify.
//!
//! Tests validate behavior. Use `who::warn!` and `who::error!` to bring code back up for review when the context around it changes.
//!
//! For example, suppose a compiler bug forces you to disable an optimized implementation:
//!
//! ```rust
//! # fn scalar_transform(input: &[f32], output: &mut [f32]) { output.copy_from_slice(input); }
//! fn transform(input: &[f32], output: &mut [f32]) {
//!     who::warn!(
//!         rustc().changed_from("1.95.0"),
//!         "Recheck rust-lang/rust#123456 and restore the SIMD path if fixed"
//!     );
//!
//!     // SIMD path disabled because of a compiler codegen bug.
//!     scalar_transform(input, output);
//! }
//! ```
//!
//! The scalar path may be completely correct and well tested.
//! Those tests will still pass after the compiler bug is fixed.
//!
//! When the compiler version changes, `who` brings this code back to your attention so the workaround does not silently become permanent.
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
//! The `date` and `file` features are enabled by default; `path` is opt-in.
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
