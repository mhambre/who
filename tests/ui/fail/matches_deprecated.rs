#![deny(deprecated)]

fn main() {
    who::error!(dependency("semver").matches("<1"), "inactive trigger");
    who::warn!(!(rustc().matches(">=1") || rustc().compare("<1")), "inactive trigger");
}
