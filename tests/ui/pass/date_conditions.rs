#![deny(deprecated)]

who::error!(date().after("9999-12-31"), "far future deadline");

fn main() {
    who::warn!(date().after("9999-12-31T23:59:59.999Z"), "future RFC 3339 deadline");
    who::error!(!date().after("2000-01-01"), "past date-only deadline");
    who::error!(!date().after("2000-01-01 12:30"), "past time without an offset");
    who::error!(!date().after("2000-01-01 12:30:00.5 +05:30"), "past offset deadline");
    who::error!(!date().after("Sat, 1 Jan 2000 12:30:00 GMT"), "past RFC 2822 deadline");
    who::error!(!date().after("2000-01-01 12:30 America/New_York"), "past named-zone deadline");
    who::error!(!date().after("2000-01-01 UTC"), "explicit UTC date");
    who::error!(!date().after("2000-01-01 12:30 UTC"), "explicit UTC date and time");
    who::error!(
        !(date().after("2000-01-01T08:00:00-04:00") && !date().after("9999-01-01"))
            || (date().after("9999-01-01") && dependency("semver").compare("<1")),
        "date predicates compose with boolean operators and dependency predicates"
    );
}
