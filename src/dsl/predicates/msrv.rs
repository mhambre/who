use syn::parse::ParseStream;

use super::version::VersionRule;
use crate::dsl::arguments;
use crate::eval::{Context, EvalError, PredicateOutcome};

pub struct Condition {
    rule: VersionRule,
}

impl Condition {
    /// Expose the version rule's deprecated syntax location.
    pub fn deprecated_method(&self) -> Option<proc_macro2::Span> {
        self.rule.deprecated_method()
    }

    /// Parse a version rule after an empty msrv() receiver.
    pub fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        arguments::empty(input)?;
        let (method, literal) = arguments::method(input)?;
        Ok(Self {
            rule: VersionRule::parse(&method, &literal)?,
        })
    }

    /// Compare against the caller's declared minimum compiler version.
    pub fn evaluate(&self, context: &mut Context) -> Result<PredicateOutcome, EvalError> {
        Ok(self.rule.evaluate("msrv", context.msrv()?))
    }
}
