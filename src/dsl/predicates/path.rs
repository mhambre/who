use quote::ToTokens;
use syn::ext::IdentExt;
use syn::parse::ParseStream;
use syn::{PathArguments, Result};

use crate::dsl::arguments;
use crate::eval::{Context, EvalError, PredicateOutcome};

pub struct Condition {
    path: String,
}

impl Condition {
    /// Accept import paths to sysroot items, including modules and macros.
    pub fn parse(input: ParseStream<'_>) -> Result<Self> {
        let path = arguments::path(input)?;
        let root = &path.segments.first().unwrap().ident;
        match root.unraw().to_string().as_str() {
            "std" | "core" | "alloc" | "proc_macro" => {}
            "crate" | "self" | "super" => return Err(syn::Error::new(
                root.span(),
                "who: path() cannot probe crate/self/super paths. Use a std, core, alloc, or proc_macro path",
            )),
            _ => return Err(syn::Error::new(
                root.span(),
                "who: external-crate path probing is not implemented. Use a std, core, alloc, or proc_macro path",
            )),
        }
        for segment in &path.segments {
            if !matches!(segment.arguments, PathArguments::None) {
                return Err(syn::Error::new_spanned(
                    segment,
                    "who: path() requires an import path without generic arguments",
                ));
            }
        }
        let method = arguments::empty_method(input)?;
        if method != "exists" {
            return Err(arguments::unsupported(&method));
        }
        Ok(Self {
            path: path.to_token_stream().to_string().replace(' ', ""),
        })
    }

    /// Ask rustc whether a public import compiles in the caller's context.
    pub fn evaluate(
        &self,
        context: &mut Context,
    ) -> std::result::Result<PredicateOutcome, EvalError> {
        let value = context.path_exists(&self.path)?;
        Ok(PredicateOutcome {
            value,
            expected: format!("{} is importable", self.path),
            resolved: format!(
                "{} is {}",
                self.path,
                if value {
                    "importable"
                } else {
                    "not importable"
                }
            ),
        })
    }
}
