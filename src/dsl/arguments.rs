use syn::parse::ParseStream;
use syn::{parenthesized, Ident, LitStr, Result, Token};

/// Parse one receiver string and reject remaining arguments.
pub fn named(input: ParseStream<'_>) -> Result<LitStr> {
    let arguments;
    parenthesized!(arguments in input);
    let name = arguments.parse()?;
    finish_receiver(&arguments)?;
    Ok(name)
}

/// Parse an empty receiver such as rustc() or date().
pub fn empty(input: ParseStream<'_>) -> Result<()> {
    let arguments;
    parenthesized!(arguments in input);
    finish_receiver(&arguments)
}

fn finish_receiver(input: ParseStream<'_>) -> Result<()> {
    if !input.is_empty() {
        return Err(input.error("unexpected predicate argument"));
    }
    Ok(())
}

/// Parse one Rust path without accepting expressions or extra arguments.
#[cfg(feature = "path")]
pub fn path(input: ParseStream<'_>) -> Result<syn::Path> {
    let arguments;
    parenthesized!(arguments in input);
    let path = arguments.parse()?;
    finish_receiver(&arguments)?;
    Ok(path)
}

/// Parse a method with no arguments.
#[cfg(feature = "path")]
pub fn empty_method(input: ParseStream<'_>) -> Result<Ident> {
    input.parse::<Token![.]>()?;
    let method = input.parse()?;
    empty(input)?;
    Ok(method)
}

/// Parse a method name and its single string argument, retaining both spans.
pub fn method(input: ParseStream<'_>) -> Result<(Ident, LitStr)> {
    input.parse::<Token![.]>()?;
    let method = input.parse()?;
    let arguments;
    parenthesized!(arguments in input);
    let literal = arguments.parse()?;
    if !arguments.is_empty() {
        return Err(arguments.error("expected one string argument"));
    }
    Ok((method, literal))
}

/// Report unsupported methods at the method name rather than the receiver.
pub fn unsupported(method: &Ident) -> syn::Error {
    syn::Error::new(method.span(), "unsupported predicate method")
}
