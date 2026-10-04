#![deny(warnings)]
#![allow(deprecated)]

who::error!(rustc().matches("<1"), "item-level alias");

fn main() {
    who::error!(dependency("semver").matches("<1"), "inactive trigger");
    who::error!(!dependency("semver").matches(">=1, <2"), "same semver semantics");
    who::error!(!(rustc().matches(">=1") && rustc().matches("*")), "combined aliases");
}
