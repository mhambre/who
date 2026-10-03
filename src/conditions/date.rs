use chrono::{DateTime, LocalResult, NaiveDate, NaiveDateTime, TimeZone, Utc};
use chrono_tz::Tz;

pub fn parse(value: &str) -> Result<DateTime<Utc>, String> {
    let value = value.trim();
    if let Ok(date) = DateTime::parse_from_rfc3339(value) {
        return Ok(date.with_timezone(&Utc));
    }
    if let Ok(date) = DateTime::parse_from_rfc2822(value) {
        return Ok(date.with_timezone(&Utc));
    }
    for format in [
        "%Y-%m-%d %H:%M:%S%.f %:z",
        "%Y-%m-%d %H:%M:%S%.f%:z",
        "%Y-%m-%dT%H:%M:%S%.f%z",
        "%Y-%m-%d %H:%M %:z",
        "%Y-%m-%d %H:%M%:z",
        "%Y-%m-%dT%H:%M%:z",
    ] {
        if let Ok(date) = DateTime::parse_from_str(value, format) {
            return Ok(date.with_timezone(&Utc));
        }
    }
    if let Some(date) = parse_local(value) {
        return Ok(date.and_utc());
    }
    if let Some((local, zone)) = value.rsplit_once(char::is_whitespace) {
        if let Some(local) = parse_local(local.trim()) {
            let zone: Tz = zone.parse().map_err(|_| {
                format!("invalid timezone `{zone}`: expected an IANA name such as America/New_York")
            })?;
            return match zone.from_local_datetime(&local) {
                LocalResult::Single(date) => Ok(date.with_timezone(&Utc)),
                LocalResult::Ambiguous(_, _) => Err(format!("ambiguous local date `{value}`: use an explicit numeric UTC offset to select one instant")),
                LocalResult::None => Err(format!("nonexistent local date `{value}`: choose a valid local time or use an explicit numeric UTC offset")),
            };
        }
    }
    Err("invalid date: expected RFC 3339, RFC 2822, YYYY-MM-DD, or YYYY-MM-DD[ T]HH:MM[:SS[.fraction]] with an optional numeric UTC offset or IANA timezone; dates without a timezone use UTC".into())
}

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

pub fn after(now: DateTime<Utc>, deadline: DateTime<Utc>) -> bool {
    now > deadline
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_and_offsets_describe_the_same_instant() {
        let expected = parse("2026-10-03T12:30:00Z").unwrap();
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
            assert_eq!(parse(value).unwrap(), expected, "{value}");
        }
    }

    #[test]
    fn date_only_means_midnight_utc() {
        assert_eq!(
            parse("2026-10-03").unwrap(),
            parse("2026-10-03T00:00:00Z").unwrap()
        );
        assert_eq!(
            parse("2026-10-03 UTC").unwrap(),
            parse("2026-10-03T00:00:00Z").unwrap()
        );
        assert_eq!(
            parse("2026-10-03 09:30 UTC").unwrap(),
            parse("2026-10-03 09:30").unwrap()
        );
    }

    #[test]
    fn comparison_is_strict_and_keeps_fractional_seconds() {
        let deadline = parse("2026-10-03T12:30:00.123456789Z").unwrap();
        assert!(!after(
            parse("2026-10-03T12:30:00.123456788Z").unwrap(),
            deadline
        ));
        assert!(!after(deadline, deadline));
        assert!(after(
            parse("2026-10-03T12:30:00.123456790Z").unwrap(),
            deadline
        ));
    }

    #[test]
    fn named_zones_apply_daylight_saving_rules() {
        assert_eq!(
            parse("2026-01-03 08:30 America/New_York").unwrap(),
            parse("2026-01-03T13:30:00Z").unwrap()
        );
        assert_eq!(
            parse("2026-07-03 08:30 America/New_York").unwrap(),
            parse("2026-07-03T12:30:00Z").unwrap()
        );
        assert_eq!(
            parse("2026-10-03 America/New_York").unwrap(),
            parse("2026-10-03T04:00:00Z").unwrap()
        );
        assert!(parse("2026-11-01 01:30 America/New_York")
            .unwrap_err()
            .contains("ambiguous local date"));
        assert!(parse("2026-03-08 02:30 America/New_York")
            .unwrap_err()
            .contains("nonexistent local date"));
        assert!(parse("2026-10-03 12:30 America/Unknown")
            .unwrap_err()
            .contains("invalid timezone"));
    }

    #[test]
    fn calendar_validation_uses_chrono() {
        assert!(parse("2024-02-29").is_ok());
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
            assert!(parse(value).is_err(), "{value}");
        }
    }
}
