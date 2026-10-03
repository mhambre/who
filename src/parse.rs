use proc_macro2::Span;
use semver::{Version, VersionReq};
use syn::parse::{Parse, ParseStream};
use syn::{parenthesized, Ident, LitStr, Result, Token};

use crate::ast::{Expr, Predicate};

pub struct Guard {
    pub condition: Expr,
    pub message: LitStr,
    pub span: Span,
}

impl Parse for Guard {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let span = input.span();
        let condition = parse_or(input)?;
        input.parse::<Token![,]>()?;
        let message = input.parse()?;
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }
        if !input.is_empty() {
            return Err(input.error("expected only a condition and a message string"));
        }
        Ok(Self {
            condition,
            message,
            span,
        })
    }
}

fn parse_or(input: ParseStream<'_>) -> Result<Expr> {
    let mut left = parse_and(input)?;
    while input.peek(Token![||]) {
        input.parse::<Token![||]>()?;
        left = Expr::Or(Box::new(left), Box::new(parse_and(input)?));
    }
    Ok(left)
}

fn parse_and(input: ParseStream<'_>) -> Result<Expr> {
    let mut left = parse_unary(input)?;
    while input.peek(Token![&&]) {
        input.parse::<Token![&&]>()?;
        left = Expr::And(Box::new(left), Box::new(parse_unary(input)?));
    }
    Ok(left)
}

fn parse_unary(input: ParseStream<'_>) -> Result<Expr> {
    if input.peek(Token![!]) {
        input.parse::<Token![!]>()?;
        return Ok(Expr::Not(Box::new(parse_unary(input)?)));
    }
    if input.peek(syn::token::Paren) {
        let content;
        parenthesized!(content in input);
        let expression = parse_or(&content)?;
        if !content.is_empty() {
            return Err(content.error("unexpected token in condition"));
        }
        return Ok(expression);
    }
    parse_predicate(input)
}

fn parse_predicate(input: ParseStream<'_>) -> Result<Expr> {
    let kind: Ident = input.parse()?;
    let argument;
    parenthesized!(argument in input);
    let name: Option<LitStr> = match kind.to_string().as_str() {
        "dependency" | "file" => Some(argument.parse()?),
        "rustc" => None,
        _ => {
            return Err(syn::Error::new(
                kind.span(),
                "expected dependency(...), file(...), or rustc()",
            ))
        }
    };
    if !argument.is_empty() {
        return Err(argument.error("unexpected predicate argument"));
    }
    input.parse::<Token![.]>()?;
    let method: Ident = input.parse()?;
    let value;
    parenthesized!(value in input);
    let literal: LitStr = value.parse()?;
    if !value.is_empty() {
        return Err(value.error("expected one string argument"));
    }
    let version = || {
        Version::parse(&literal.value()).map_err(|error| {
            syn::Error::new(literal.span(), format!("invalid exact version: {error}"))
        })
    };
    let requirement = || {
        VersionReq::parse(&literal.value()).map_err(|error| {
            syn::Error::new(
                literal.span(),
                format!("invalid semver requirement: {error}"),
            )
        })
    };
    let predicate = match (kind.to_string().as_str(), method.to_string().as_str()) {
        ("dependency", "changed_from") => Predicate::DependencyChangedFrom {
            name: name.unwrap().value(),
            version: version()?,
        },
        ("dependency", "matches") => Predicate::DependencyMatches {
            name: name.unwrap().value(),
            requirement: requirement()?,
        },
        ("rustc", "changed_from") => Predicate::RustcChangedFrom {
            version: version()?,
        },
        ("rustc", "matches") => Predicate::RustcMatches {
            requirement: requirement()?,
        },
        ("file", "changed_from") => {
            let hash = literal.value();
            let digest = hash.strip_prefix("sha256:").unwrap_or("");
            if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                return Err(syn::Error::new(
                    literal.span(),
                    "expected sha256: followed by 64 hexadecimal digits",
                ));
            }
            Predicate::FileChangedFrom {
                path: name.unwrap().value().into(),
                hash: hash.to_ascii_lowercase(),
            }
        }
        _ => {
            return Err(syn::Error::new(
                method.span(),
                "unsupported predicate method",
            ))
        }
    };
    Ok(Expr::Predicate(predicate, kind.span()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn precedence() {
        let guard: Guard = syn::parse_str(
            "rustc().matches(\"*\") || !rustc().matches(\"*\") && (rustc().matches(\"*\")), \"reason\",",
        )
        .unwrap();
        assert!(matches!(guard.condition, Expr::Or(_, right) if matches!(*right, Expr::And(_, _))));
    }

    #[test]
    fn rejects_rust_and_extra_arguments() {
        for text in [
            "true, \"reason\"",
            "cfg!(unix), \"reason\"",
            "rustc(\"x\").matches(\"*\"), \"r\"",
            "rustc().matches(\"*\", \"x\"), \"r\"",
            "rustc().matches(\"*\"), \"r\", false",
        ] {
            assert!(syn::parse_str::<Guard>(text).is_err(), "{text}");
        }
    }
}
