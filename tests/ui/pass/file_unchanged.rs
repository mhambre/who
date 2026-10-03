#![deny(deprecated)]

who::error!(file("../../../../tests/data/schema.proto").changed_from("sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"), "empty file");

fn main() {}
