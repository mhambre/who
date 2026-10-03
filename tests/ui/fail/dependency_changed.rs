fn main() {
    who::error!(dependency("semver").changed_from("0.1.0"), "Revalidate dependency assumptions");
}
