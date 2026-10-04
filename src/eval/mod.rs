//! Combine predicate results without hiding invalid operands.

mod context;
mod outcome;

pub use context::Context;
pub use outcome::{EvalError, Outcome, PredicateOutcome};

use crate::dsl::ast::Expr;

/// Evaluate both boolean operands and retain their evidence in source order.
pub fn evaluate(expression: &Expr, context: &mut Context) -> syn::Result<Outcome> {
    match expression {
        Expr::Predicate(predicate, span) => predicate
            .evaluate(context)
            .map(Outcome::from)
            .map_err(|error| syn::Error::new(*span, error)),
        Expr::Not(inner) => {
            let mut outcome = evaluate(inner, context)?;
            outcome.value = !outcome.value;
            Ok(outcome)
        }
        Expr::And(left, right) | Expr::Or(left, right) => {
            let mut left = evaluate(left, context)?;
            let right = evaluate(right, context)?;
            left.value = match expression {
                Expr::And(_, _) => left.value && right.value,
                _ => left.value || right.value,
            };
            left.evidence.extend(right.evidence);
            Ok(left)
        }
    }
}
