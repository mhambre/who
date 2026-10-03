fn main() {
    who::error!(date("2026-01-01").after("2026-02-01"), "date takes no arguments");
    who::error!(date().before("2026-01-01"), "only after is supported");
    who::error!(date().after("2026-01-01", "UTC"), "after takes one string");
}
