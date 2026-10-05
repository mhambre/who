# msrv polyfill example

This example shows how to keep a compatibility shim for `std::sync::LazyLock` while the package MSRV is still below Rust 1.80.

## Original problem

`LazyLock` is a convenient standard-library API for lazily initialized statics, but it was stabilized later than this example's declared MSRV. If you need the same pattern before raising `rust-version`, you still need a local compatibility layer.

- Tracking issue: <https://github.com/rust-lang/rust/issues/109736>
- Stabilization PR: <https://github.com/rust-lang/rust/pull/121377>

## Workaround

The binary uses a tiny `compat::LazyLock<T>` wrapper built on `std::sync::OnceLock`, which was already available before Rust 1.80. A `who::warn!` guard asks for review when either:

- `rust-version` is raised to `>=1.80`, or
- the active toolchain already exposes `std::sync::LazyLock`.

That warning marks the point where the local shim may no longer be necessary.

## When to re-trigger

Revisit this example after either of these changes:

1. `Cargo.toml` raises `rust-version` to `1.80` or newer.
2. A newer compiler/toolchain is used for builds and `path(std::sync::LazyLock).exists()` starts matching.

At that point, replace the local shim with `std::sync::LazyLock` directly.

## Commands

```bash
cargo build --locked
cargo run --locked
```
