fn main() {
    who::error!(file("schema.proto").changed_from("sha256:abc123"), "bad hash");
}
