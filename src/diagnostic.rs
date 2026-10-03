use std::path::PathBuf;

use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};

use crate::eval::Outcome;
use crate::parse::Guard;

pub enum Severity {
    Warning,
    Error,
}

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
        let message = format!(
            "who assumption requires revalidation\n{}\nreason: {}",
            outcome.details.join("\n"),
            guard.message.value()
        );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inactive_expansion_has_no_metadata() {
        let guard = syn::parse_str("rustc().matches(\"<1\"), \"SECRET_REASON\"").unwrap();
        let tokens = expand(
            &guard,
            Outcome {
                value: false,
                details: vec!["SECRET_DETAIL".into()],
            },
            std::iter::empty(),
            Severity::Warning,
        )
        .to_string();
        assert_eq!(tokens, "const _ : () = { } ;");
    }
}
