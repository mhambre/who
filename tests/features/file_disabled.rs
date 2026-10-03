fn main() {
    who::warn!(file("schema.proto").changed_from("sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"), "file feature is disabled");
}
