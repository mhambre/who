fn main() {
    who::warn!(path(std::sync::LazyLock).exists(), "requires path probing");
}
