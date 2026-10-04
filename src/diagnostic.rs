use std::path::PathBuf;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};

use crate::dsl::Guard;
use crate::eval::Outcome;

pub enum Severity {
    Warning,
    Error,
}

/// Emit tracked inputs and diagnostics inside a compile-time-only constant.
pub fn expand<'a>(
    guard: &Guard,
    outcome: Outcome,
    paths: impl Iterator<Item = &'a PathBuf>,
    severity: Severity,
) -> TokenStream {
    let span = guard.span;
    let tracked = paths.map(|path| {
        let path = path.to_string_lossy().into_owned();
        // include_bytes makes Cargo track inputs without retaining their contents at runtime.
        quote! { let _ = ::core::include_bytes!(#path); }
    });
    let diagnostic = if outcome.value {
        let message = format_message(guard, &outcome);
        match severity {
            Severity::Error => quote_spanned! {span=> ::core::compile_error!(#message); },
            Severity::Warning => quote_spanned! {span=>
                #[deprecated(note = #message)]
                const WHO_ASSUMPTION_REQUIRES_REVALIDATION: () = ();
                let _ = WHO_ASSUMPTION_REQUIRES_REVALIDATION;
            },
        }
    } else {
        TokenStream::new()
    };
    quote_spanned! {span=>
        const _: () = {
            #(#tracked)*
            #diagnostic
        };
    }
}

/// Render leaf evidence in source order without altering its truth values.
fn format_message(guard: &Guard, outcome: &Outcome) -> String {
    let details = outcome
        .evidence
        .iter()
        .map(|evidence| {
            format!(
                "expected: {}\nresolved: {}\npredicate: {}",
                evidence.expected, evidence.resolved, evidence.value,
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "who assumption requires revalidation\n{details}\nreason: {}",
        guard.message.value()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inactive_expansion_has_no_metadata() {
        let guard = syn::parse_str("rustc().compare(\"<1\"), \"SECRET_REASON\"").unwrap();
        let tokens = expand(
            &guard,
            Outcome {
                value: false,
                evidence: vec![crate::eval::PredicateOutcome {
                    value: false,
                    expected: "SECRET_DETAIL".into(),
                    resolved: "SECRET_DETAIL".into(),
                }],
            },
            std::iter::empty(),
            Severity::Warning,
        )
        .to_string();
        assert_eq!(tokens, "const _ : () = { } ;");
    }
}
