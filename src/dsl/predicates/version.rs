use proc_macro2::Span;
use semver::{Version, VersionReq};
use syn::{Ident, LitStr};

use crate::eval::PredicateOutcome;

pub enum VersionRule {
    ChangedFrom(Version),
    Compare(VersionReq),
    DeprecatedMatches(VersionReq, Span),
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
            "compare" | "matches" => VersionReq::parse(&literal.value())
                .map(|requirement| {
                    if method == "matches" {
                        Self::DeprecatedMatches(requirement, method.span())
                    } else {
                        Self::Compare(requirement)
                    }
                })
                .map_err(|error| {
                    syn::Error::new(
                        literal.span(),
                        format!("invalid semver requirement: {error}"),
                    )
                }),
            _ => Err(super::super::arguments::unsupported(method)),
        }
    }

    /// Retain the deprecated spelling's location for expansion diagnostics.
    pub fn deprecated_method(&self) -> Option<Span> {
        match self {
            Self::DeprecatedMatches(_, span) => Some(*span),
            _ => None,
        }
    }

    /// Compare a resolved version and retain the expected baseline or range.
    pub fn evaluate(&self, name: &str, resolved: Version) -> PredicateOutcome {
        let (value, expected) = match self {
            Self::ChangedFrom(version) => (resolved != *version, format!("{name} {version}")),
            Self::Compare(requirement) | Self::DeprecatedMatches(requirement, _) => (
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

    #[test]
    fn deprecated_alias_keeps_comparison_semantics() {
        let literal = LitStr::new(">=1, <2", Span::call_site());
        let alias =
            VersionRule::parse(&Ident::new("matches", Span::call_site()), &literal).unwrap();
        let comparison =
            VersionRule::parse(&Ident::new("compare", Span::call_site()), &literal).unwrap();
        assert!(alias.deprecated_method().is_some());
        assert!(comparison.deprecated_method().is_none());
        for version in [
            Version::new(0, 9, 0),
            Version::new(1, 2, 3),
            Version::new(2, 0, 0),
        ] {
            let old = alias.evaluate("foo", version.clone());
            let new = comparison.evaluate("foo", version);
            assert_eq!(old.value, new.value);
            assert_eq!(old.expected, new.expected);
            assert_eq!(old.resolved, new.resolved);
        }
    }
}
