use chrono::{DateTime, LocalResult, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;
use syn::parse::ParseStream;

use crate::dsl::arguments;
use crate::eval::{Context, EvalError, PredicateOutcome};

pub struct Condition {
    deadline: DateTime<Utc>,
}

impl Condition {
    /// Parse an after deadline without reading the compilation clock.
    pub fn parse(input: ParseStream<'_>) -> syn::Result<Self> {
        arguments::empty(input)?;
        let (method, literal) = arguments::method(input)?;
        if method != "after" {
            return Err(arguments::unsupported(&method));
        }
        let deadline = parse_instant(&literal.value())
            .map_err(|message| syn::Error::new(literal.span(), message))?;
        Ok(Self { deadline })
    }

    /// Compare the deadline with the invocation's single cached UTC instant.
    pub fn evaluate(&self, context: &mut Context) -> Result<PredicateOutcome, EvalError> {
        let now = context.now();
        Ok(PredicateOutcome {
            value: after(now, self.deadline),
            expected: format!("compilation time after {}", self.deadline.to_rfc3339()),
            resolved: format!("compilation time {}", now.to_rfc3339()),
        })
    }
}

/// Normalize supported date formats to UTC. Unzoned inputs already mean UTC.
pub fn parse_instant(value: &str) -> Result<DateTime<Utc>, String> {
    let value = value.trim();
    if let Ok(date) = DateTime::parse_from_rfc3339(value) {
        return Ok(date.with_timezone(&Utc));
    }
    if let Ok(date) = DateTime::parse_from_rfc2822(value) {
        return Ok(date.with_timezone(&Utc));
    }
    if let Some(date) = parse_offset(value) {
        return Ok(date);
    }
    if let Some(date) = parse_local(value) {
        return Ok(date.and_utc());
    }
    if let Some((local, zone)) = value.rsplit_once(char::is_whitespace) {
        if let Some(local) = parse_local(local.trim()) {
            return resolve_zone(local, zone, value);
        }
    }
    Err("invalid date: expected RFC 3339, RFC 2822, YYYY-MM-DD, or YYYY-MM-DD[ T]HH:MM[:SS[.fraction]] with an optional numeric UTC offset or IANA timezone; dates without a timezone use UTC".into())
}

/// Parse explicit numeric offsets before interpreting an input as local time.
fn parse_offset(value: &str) -> Option<DateTime<Utc>> {
    [
        "%Y-%m-%d %H:%M:%S%.f %:z",
        "%Y-%m-%d %H:%M:%S%.f%:z",
        "%Y-%m-%dT%H:%M:%S%.f%z",
        "%Y-%m-%d %H:%M %:z",
        "%Y-%m-%d %H:%M%:z",
        "%Y-%m-%dT%H:%M%:z",
    ]
    .iter()
    .find_map(|format| {
        DateTime::parse_from_str(value, format)
            .ok()
            .map(|date| date.with_timezone(&Utc))
    })
}

/// Resolve IANA rules without guessing across daylight-saving gaps or folds.
fn resolve_zone(local: NaiveDateTime, zone: &str, value: &str) -> Result<DateTime<Utc>, String> {
    let zone: Tz = zone.parse().map_err(|_| {
        format!("invalid timezone `{zone}`: expected an IANA name such as America/New_York")
    })?;
    match zone.from_local_datetime(&local) {
        LocalResult::Single(date) => Ok(date.with_timezone(&Utc)),
        LocalResult::Ambiguous(_, _) => Err(format!("ambiguous local date `{value}`: use an explicit numeric UTC offset to select one instant")),
        LocalResult::None => Err(format!("nonexistent local date `{value}`: choose a valid local time or use an explicit numeric UTC offset")),
    }
}

/// Parse local calendar fields, treating date-only forms as midnight.
fn parse_local(value: &str) -> Option<NaiveDateTime> {
    for format in [
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-%m-%d %H:%M",
        "%Y-%m-%dT%H:%M",
    ] {
        if let Ok(date) = NaiveDateTime::parse_from_str(value, format) {
            return Some(date);
        }
    }
    NaiveDate::parse_from_str(value, "%Y-%m-%d")
        .ok()?
        .and_hms_opt(0, 0, 0)
}

/// Exclude equality when deciding whether the deadline passed.
pub fn after(now: DateTime<Utc>, deadline: DateTime<Utc>) -> bool {
    now > deadline
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_and_offsets_describe_the_same_instant() {
        let expected = parse_instant("2026-10-03T12:30:00Z").unwrap();
        for value in [
            "2026-10-03T08:30:00-04:00",
            "2026-10-03T18:00:00+05:30",
            "Sat, 3 Oct 2026 08:30:00 -0400",
            "Sat, 3 Oct 2026 12:30:00 GMT",
            "2026-10-03 08:30:00 -04:00",
            "2026-10-03 08:30:00-0400",
            "2026-10-03T08:30:00-0400",
            "2026-10-03 08:30 -04:00",
            "2026-10-03T08:30-0400",
            "2026-10-03 12:30:00",
            "2026-10-03T12:30",
            "2026-10-03T08:30:00 America/New_York",
            "2026-10-03 13:30 Europe/London",
            "2026-10-03 18:00 Asia/Kolkata",
            "2026-10-03 12:30 UTC",
            " 2026-10-03T12:30:00Z ",
        ] {
            assert_eq!(parse_instant(value).unwrap(), expected, "{value}");
        }
    }

    #[test]
    fn date_only_means_midnight_utc() {
        assert_eq!(
            parse_instant("2026-10-03").unwrap(),
            parse_instant("2026-10-03T00:00:00Z").unwrap()
        );
        assert_eq!(
            parse_instant("2026-10-03 UTC").unwrap(),
            parse_instant("2026-10-03T00:00:00Z").unwrap()
        );
        assert_eq!(
            parse_instant("2026-10-03 09:30 UTC").unwrap(),
            parse_instant("2026-10-03 09:30").unwrap()
        );
    }

    #[test]
    fn comparison_is_strict_and_keeps_fractional_seconds() {
        let deadline = parse_instant("2026-10-03T12:30:00.123456789Z").unwrap();
        assert!(!after(
            parse_instant("2026-10-03T12:30:00.123456788Z").unwrap(),
            deadline
        ));
        assert!(!after(deadline, deadline));
        assert!(after(
            parse_instant("2026-10-03T12:30:00.123456790Z").unwrap(),
            deadline
        ));
    }

    #[test]
    fn named_zones_apply_daylight_saving_rules() {
        assert_eq!(
            parse_instant("2026-01-03 08:30 America/New_York").unwrap(),
            parse_instant("2026-01-03T13:30:00Z").unwrap()
        );
        assert_eq!(
            parse_instant("2026-07-03 08:30 America/New_York").unwrap(),
            parse_instant("2026-07-03T12:30:00Z").unwrap()
        );
        assert_eq!(
            parse_instant("2026-10-03 America/New_York").unwrap(),
            parse_instant("2026-10-03T04:00:00Z").unwrap()
        );
        assert!(parse_instant("2026-11-01 01:30 America/New_York")
            .unwrap_err()
            .contains("ambiguous local date"));
        assert!(parse_instant("2026-03-08 02:30 America/New_York")
            .unwrap_err()
            .contains("nonexistent local date"));
        assert!(parse_instant("2026-10-03 12:30 America/Unknown")
            .unwrap_err()
            .contains("invalid timezone"));
    }

    #[test]
    fn calendar_validation_uses_chrono() {
        assert!(parse_instant("2024-02-29").is_ok());
        for value in [
            "",
            "tomorrow",
            "10/03/2026",
            "2026-02-29",
            "2026-13-01",
            "2026-10-03T25:00:00Z",
            "2026-10-03T12:30:00+25:00",
            "2026-10-03 trailing text",
        ] {
            assert!(parse_instant(value).is_err(), "{value}");
        }
    }
}
