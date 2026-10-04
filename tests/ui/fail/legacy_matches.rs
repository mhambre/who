fn main() {
    who::error!(dependency("semver").matches(">=1"), "use compare");
    who::error!(rustc().matches(">=1"), "use compare");
    who::error!(msrv().matches(">=1"), "use compare");
}
