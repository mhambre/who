fn main() {
    who::error!(date().after("2026-02-29"), "invalid calendar date");
    who::warn!(date().after("tomorrow"), "relative dates are not supported");
    who::error!(date().after("2026-10-03T12:00:00+25:00"), "invalid timezone offset");
}
