#![deny(deprecated)]

#[allow(deprecated)]
fn main() {
    who::warn!(rustc().matches(">=1"), "explicitly suppressed warning");
}
