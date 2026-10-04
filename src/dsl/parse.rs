use syn::parse::{Parse, ParseStream};
use syn::{parenthesized, Result, Token};

use super::ast::{Expr, Guard};
use super::predicates;

impl Parse for Guard {
    /// Parse a guard, its message, and an optional trailing comma.
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

/// Parse OR after higher-precedence operands, preserving left associativity.
fn parse_or(input: ParseStream<'_>) -> Result<Expr> {
    let mut left = parse_and(input)?;
    while input.peek(Token![||]) {
        input.parse::<Token![||]>()?;
        left = Expr::Or(Box::new(left), Box::new(parse_and(input)?));
    }
    Ok(left)
}

/// Parse AND before OR while allowing unary expressions on either side.
fn parse_and(input: ParseStream<'_>) -> Result<Expr> {
    let mut left = parse_unary(input)?;
    while input.peek(Token![&&]) {
        input.parse::<Token![&&]>()?;
        left = Expr::And(Box::new(left), Box::new(parse_unary(input)?));
    }
    Ok(left)
}

/// Bind NOT most tightly and recurse into parenthesized expressions.
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
    let (predicate, span) = predicates::parse(input)?;
    Ok(Expr::Predicate(predicate, span))
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
