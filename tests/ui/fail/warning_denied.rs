#![deny(deprecated)]

fn main() {
    who::warn!(dependency("semver").changed_from("0.1.0"), "Revalidate dependency assumptions");
}
