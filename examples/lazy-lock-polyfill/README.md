# `std::sync::LazyLock` compatibility shim

`LazyLock` lets a static value be initialized on first use without a separate
accessor function. Before Rust 1.80, `OnceLock` could provide the same pattern,
but required more boilerplate. This example keeps a small shim for an
infallibly initialized static configuration value, which remains compatible
with the crate's Rust 1.74 MSRV.

`LazyLock` was stabilized in [Rust 1.80](https://github.com/rust-lang/rust/releases/tag/1.80.0)
by [rust-lang/rust#121377](https://github.com/rust-lang/rust/pull/121377), which
closed [tracking issue #109736](https://github.com/rust-lang/rust/issues/109736).
The example uses `path(std::sync::LazyLock).exists()` to request a review when
the item is actually available, without hard-coding a compiler version. This
requires enabling `who`'s `path` feature.

When the final API's name or path is not yet known, use a compiler-version
trigger instead:

```rust
who::warn!(
    rustc().compare(">=1.80"),
    "Review this shim for the stabilized lazy initialization API"
);
```

Unlike `path()`, this version trigger works even when the API's final location
is unknown. For `LazyLock`, its stabilization version is known, so either
trigger can work.

The shim intentionally supports only infallible function-pointer
initializers; it is not a drop-in replacement for every `LazyLock` behavior.

Run the example and its tests with:

```sh
cargo run --locked
cargo test --locked
```
