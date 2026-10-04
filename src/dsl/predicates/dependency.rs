use syn::parse::ParseStream;

use super::version::VersionRule;
use crate::dsl::arguments;
use crate::eval::{Context, EvalError, PredicateOutcome};

pub struct Condition {
    name: String,
    rule: VersionRule,
}

impl Condition {
    /// Expose the version rule's deprecated syntax location.
    pub fn deprecated_method(&self) -> Option<proc_macro2::Span> {
        self.rule.deprecated_method()
    }

    /// Parse a Cargo package name and its version rule without reading Cargo state.
    pub fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        let name = arguments::named(input)?.value();
        let (method, literal) = arguments::method(input)?;
        Ok(Self {
            name,
            rule: VersionRule::parse(&method, &literal)?,
        })
    }

    /// Resolve an unambiguous package through the invocation's cached graph.
    pub fn evaluate(&self, context: &mut Context) -> Result<PredicateOutcome, EvalError> {
        Ok(self
            .rule
            .evaluate(&self.name, context.dependency(&self.name)?))
    }
}
