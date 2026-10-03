#![deny(deprecated)]

who::error!(dependency("semver").matches("<1"), "semver should be 1.x");
who::error!(!dependency("semver").matches("^1"), "semver should be 1.x");
who::warn!(rustc().matches("<1"), "inactive warning");
who::error!(file("../../../../tests/data/schema.proto").changed_from("sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"), "empty file");

fn main() {
    who::error!(
        !(rustc().matches(">=1") && (dependency("semver").matches(">=1, <2") || rustc().matches("<1"))),
        "boolean precedence",
    );
    who::error!(dependency("semver").matches("~0.1") || dependency("semver").matches("=0.1.0"), "old versions");
    who::warn!(dependency("sha2").changed_from("0.10.9"), "sha2 baseline");
}
