use std::fmt;

pub struct PredicateOutcome {
    pub value: bool,
    pub expected: String,
    pub resolved: String,
}

pub struct Outcome {
    pub value: bool,
    pub evidence: Vec<PredicateOutcome>,
}

impl From<PredicateOutcome> for Outcome {
    fn from(predicate: PredicateOutcome) -> Self {
        Self {
            value: predicate.value,
            evidence: vec![predicate],
        }
    }
}

/// An evaluation failure before the guard can produce a truth value.
#[derive(Debug)]
pub struct EvalError(String);

impl From<String> for EvalError {
    fn from(message: String) -> Self {
        Self(message)
    }
}

impl fmt::Display for EvalError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl std::error::Error for EvalError {}
