#![deny(deprecated)]

who::error!(dependency("semver").compare("<1"), "semver should be 1.x");
who::error!(!dependency("semver").compare("^1"), "semver should be 1.x");
who::warn!(rustc().compare("<1"), "inactive warning");

fn main() {
    who::error!(
        !(rustc().compare(">=1") && (dependency("semver").compare(">=1, <2") || rustc().compare("<1"))),
        "boolean precedence",
    );
    who::error!(dependency("semver").compare("~0.1") || dependency("semver").compare("=0.1.0"), "old versions");
    who::error!(dependency("semver").compare("<1"), "semver baseline");
}
