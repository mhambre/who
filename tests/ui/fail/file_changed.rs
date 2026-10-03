fn main() {
    who::error!(file("../../../../tests/data/schema.proto").changed_from("sha256:0000000000000000000000000000000000000000000000000000000000000000"), "Revalidate decoder");
}
