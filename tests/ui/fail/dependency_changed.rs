fn main() {
    who::error!(dependency("sha2").changed_from("0.1.0"), "Revalidate hash assumptions");
}
