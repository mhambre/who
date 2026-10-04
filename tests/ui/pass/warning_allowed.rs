#![deny(deprecated)]

#[allow(deprecated)]
fn main() {
    who::warn!(rustc().compare(">=1"), "explicitly suppressed warning");
}
