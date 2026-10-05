fn main() {
    who::error!(path("std::mem").exists(), "not a Rust path");
    who::error!(path(std::mem, core::mem).exists(), "extra receiver argument");
    who::error!(path(std::vec::Vec<u8>).exists(), "generic arguments");
    who::error!(path(std::mem).exists("argument"), "empty method arguments");
    who::error!(path(std::mem).changed_from(), "unsupported method");
    who::error!(path(std::mem::size_of()).exists(), "not an expression");
}
