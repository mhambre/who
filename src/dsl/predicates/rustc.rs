use syn::parse::ParseStream;

use super::version::VersionRule;
use crate::dsl::arguments;
use crate::eval::{Context, EvalError, PredicateOutcome};

pub struct Condition {
    rule: VersionRule,
}

impl Condition {
    /// Parse a compiler version rule after an empty rustc() receiver.
    pub fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        arguments::empty(input)?;
        let (method, literal) = arguments::method(input)?;
        Ok(Self {
            rule: VersionRule::parse(&method, &literal)?,
        })
    }

    /// Reuse the compiler version cached for this invocation.
    pub fn evaluate(&self, context: &mut Context) -> Result<PredicateOutcome, EvalError> {
        Ok(self.rule.evaluate("rustc", context.compiler()?))
    }
}
