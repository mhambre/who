use semver::{Version, VersionReq};
use syn::{Ident, LitStr};

use crate::eval::PredicateOutcome;

pub enum VersionRule {
    ChangedFrom(Version),
    Compare(VersionReq),
}

impl VersionRule {
    /// Parse shared version rules with errors attached to their literals.
    pub fn parse(method: &Ident, literal: &LitStr) -> syn::Result<Self> {
        match method.to_string().as_str() {
            "changed_from" => Version::parse(&literal.value())
                .map(Self::ChangedFrom)
                .map_err(|error| {
                    syn::Error::new(literal.span(), format!("invalid exact version: {error}"))
                }),
            "compare" => VersionReq::parse(&literal.value())
                .map(Self::Compare)
                .map_err(|error| {
                    syn::Error::new(
                        literal.span(),
                        format!("invalid semver requirement: {error}"),
                    )
                }),
            _ => Err(super::super::arguments::unsupported(method)),
        }
    }

    /// Compare a resolved version and retain the expected baseline or range.
    pub fn evaluate(&self, name: &str, resolved: Version) -> PredicateOutcome {
        let (value, expected) = match self {
            Self::ChangedFrom(version) => (resolved != *version, format!("{name} {version}")),
            Self::Compare(requirement) => (
                requirement.matches(&resolved),
                format!("{name} matches {requirement}"),
            ),
        };
        PredicateOutcome {
            value,
            expected,
            resolved: format!("{name} {resolved}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_drift_and_requirement_matching_keep_distinct_semantics() {
        let version = Version::new(1, 2, 3);
        let exact = VersionRule::ChangedFrom(version.clone());
        assert!(!exact.evaluate("foo", version.clone()).value);
        assert!(exact.evaluate("foo", Version::new(1, 2, 4)).value);
        let range = VersionRule::Compare(VersionReq::parse(">=1, <2").unwrap());
        assert!(range.evaluate("foo", version).value);
        assert!(!range.evaluate("foo", Version::new(2, 0, 0)).value);
    }
}
