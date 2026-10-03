fn main() {
    who::error!(dependency("semver").matches("banana"), "bad requirement");
    who::error!(rustc().changed_from("1.2"), "bad exact version");
}
