fn main() {
    who::error!(date().after("2026-11-01 01:30 America/New_York"), "ambiguous local time");
    who::warn!(date().after("2026-03-08 02:30 America/New_York"), "nonexistent local time");
    who::error!(date().after("2026-10-03 12:30 America/Unknown"), "invalid zone");
}
