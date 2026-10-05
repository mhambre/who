# `cfg_accessible` path workaround

`std::sync::LazyLock` became stable in Rust 1.80.0. This example keeps a small
`OnceLock`-backed shim for older compilers and uses `who` to request review when
`std::sync::LazyLock` becomes importable. The warning is a prompt to check the
supported compiler range and remove the shim; it does not alter the selected
implementation automatically.

Run the example with the stable toolchain:

```sh
cargo run --locked
```

On Rust 1.80 or newer, compilation emits the `who` review warning and the
program prints `Hello from the compatibility shim`. On older compilers, the
path predicate is false, so the shim is used without a warning.

## Nightly `cfg_accessible`

The second, optional mode demonstrates the currently implemented spelling,
`#[cfg_accessible(path)]`, which enables the review item only when rustc can
resolve the path:

```sh
cargo +nightly run --locked --features nightly-cfg-accessible
```

This mode requires nightly because it uses `#![feature(cfg_accessible)]`.
`cfg_accessible` is unstable and incomplete; unresolved paths can be
indeterminate, so this sample only probes the known-present standard-library
path on Rust 1.80 or newer. The stable command above remains runnable without
this feature.

## Upstream status and research

- [`std::sync::LazyLock`](https://doc.rust-lang.org/std/sync/struct.LazyLock.html)
  is stable since Rust 1.80.0.
- [RFC 2523](https://github.com/rust-lang/rfcs/pull/2523) proposed
  `#[cfg(accessible(path))]`.
- The [tracking issue rust-lang/rust#64797](https://github.com/rust-lang/rust/issues/64797)
  remains open and marked implementation-incomplete. Its implementation,
  documentation, and stabilization checklist items are unresolved; there is
  no stabilization release to target.
- The merged [rust-lang/rust#69870](https://github.com/rust-lang/rust/pull/69870)
  added the limited `#[cfg_accessible(path)]` attribute form. The later attempt
  to implement the RFC's `#[cfg(accessible(path))]` form,
  [rust-lang/rust#137113](https://github.com/rust-lang/rust/pull/137113), was
  closed without merging.

Consequently, this example treats `cfg_accessible` as an optional nightly
demonstration, not as a stable compatibility mechanism. The `who` path
predicate remains the review trigger and works on stable Rust.
