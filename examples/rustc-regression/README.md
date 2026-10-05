# rustc-regression

This example shows a `who::warn!` guard around a workaround for [rust-lang/rust#159035](https://github.com/rust-lang/rust/issues/159035), a real LLVM/codegen miscompilation that could segfault optimized binaries when an `Option<Enum>` was processed through `Option::map(...)`.

The regression was fixed in [Rust 1.97.1](https://github.com/rust-lang/rust/blob/master/RELEASES.md#1.97.1).

## Original problem

The issue's minimized repro used an `Option::map` shape over an enum payload. In affected toolchains, LLVM could hoist loads out of the `Some(...)` path and turn a harmless `None` case into an out-of-bounds read or crash.

## Workaround shown here

Instead of:

```rust
c.map(finalize)
```

this example uses an explicit:

```rust
match c {
    Some(checksum) => Some(finalize(checksum)),
    None => None,
}
```

That keeps the payload loads inside the `Some(...)` arm and avoids the optimizer pattern discussed in the issue thread.

## Review trigger

The code uses:

```rust
who::warn!(
    rustc().compare(">=1.97.1"),
    "Rust 1.97.1 fixed rust-lang/rust#159035; recheck whether the manual match can be simplified back to Option::map"
);
```

So the release that should re-trigger review is **Rust 1.97.1** (and any newer release).

## Run

```bash
cargo build --locked
cargo run --locked
```
