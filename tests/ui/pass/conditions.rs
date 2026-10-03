#![deny(deprecated)]

who::error!(dependency("semver").matches("<1"), "semver should be 1.x");
who::error!(!dependency("semver").matches("^1"), "semver should be 1.x");
who::warn!(rustc().matches("<1"), "inactive warning");

fn main() {
    who::error!(
        !(rustc().matches(">=1") && (dependency("semver").matches(">=1, <2") || rustc().matches("<1"))),
        "boolean precedence",
    );
    who::error!(dependency("semver").matches("~0.1") || dependency("semver").matches("=0.1.0"), "old versions");
    who::error!(dependency("semver").matches("<1"), "semver baseline");
}
