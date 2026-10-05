# `std::sync::LazyLock` compatibility shim

`LazyLock` lets a static value be initialized on first use without a separate
accessor function. Before Rust 1.80, `OnceLock` could provide the same pattern,
but required more boilerplate. This example keeps a small shim for an
infallibly initialized static configuration value, which remains compatible
with the crate's Rust 1.74 MSRV.

`LazyLock` was stabilized in [Rust 1.80](https://github.com/rust-lang/rust/releases/tag/1.80.0)
by [rust-lang/rust#121377](https://github.com/rust-lang/rust/pull/121377), which
closed [tracking issue #109736](https://github.com/rust-lang/rust/issues/109736).
Compiling with Rust 1.80 or newer triggers a review to replace the shim with
`std::sync::LazyLock`.

The shim intentionally supports only infallible function-pointer
initializers; it is not a drop-in replacement for every `LazyLock` behavior.

Run the example and its tests with:

```sh
cargo run --locked
cargo test --locked
```
