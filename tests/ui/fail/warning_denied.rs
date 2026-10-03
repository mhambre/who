#![deny(deprecated)]

fn main() {
    who::warn!(dependency("sha2").changed_from("0.1.0"), "Revalidate hash assumptions");
}
