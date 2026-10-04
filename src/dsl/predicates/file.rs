use std::path::PathBuf;

use syn::parse::ParseStream;
use syn::LitStr;

use crate::dsl::arguments;
use crate::eval::{Context, EvalError, PredicateOutcome};

pub struct Condition {
    path: PathBuf,
    hash: String,
}

impl Condition {
    /// Parse a file path and SHA-256 baseline without reading the file.
    pub fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let path = arguments::named(input)?.value().into();
        let (method, literal) = arguments::method(input)?;
        if method != "changed_from" {
            return Err(arguments::unsupported(&method));
        }
        Ok(Self {
            path,
            hash: parse_hash(&literal)?,
        })
    }

    /// Hash the current file and register its path for rebuild tracking.
    pub fn evaluate(&self, context: &mut Context) -> Result<PredicateOutcome, EvalError> {
        let resolved = context.file_hash(&self.path)?;
        Ok(PredicateOutcome {
            value: resolved != self.hash,
            expected: format!("{} {}", self.path.display(), self.hash),
            resolved: format!("{} {resolved}", self.path.display()),
        })
    }
}

/// Validate the algorithm prefix and normalize hexadecimal case.
fn parse_hash(literal: &LitStr) -> syn::Result<String> {
    let hash = literal.value();
    let digest = hash.strip_prefix("sha256:").unwrap_or("");
    if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(syn::Error::new(
            literal.span(),
            "expected sha256: followed by 64 hexadecimal digits",
        ));
    }
    Ok(hash.to_ascii_lowercase())
}
