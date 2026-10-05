# unstable-feature

This example shows a stable-Rust workaround for `let` chains.

## Original problem

Before Rust 1.88, code like this required the unstable `let_chains` feature:

```rust
if let Some(raw) = std::env::args().nth(1)
    && let Ok(port) = raw.parse::<u16>()
    && port > 0
{
    println!("selected port: {port}");
}
```

That syntax was later stabilized for the 2024 edition in:

- tracking issue: <https://github.com/rust-lang/rust/issues/53667>
- stabilization PR: <https://github.com/rust-lang/rust/pull/132833>

The tracking issue explicitly notes that `let_chains` was stabilized in Rust 1.88.0 for edition 2024.

## Workaround shown here

`src/main.rs` keeps the crate on edition 2021 and uses nested `if let` blocks instead of a `let` chain.
That keeps the example runnable on stable Rust without relying on unstable syntax.

At the workaround site, `who::warn!` uses:

```rust
rustc().compare(">=1.88")
```

to re-trigger review once the compiler is new enough to support the stabilized feature.

## When to re-trigger

The warning is a reminder to revisit the workaround when both of these are true:

1. the compiler is Rust 1.88 or newer, and
2. you are ready to move the crate to edition 2024.

Then the nested `if let` blocks can be simplified back into a `let` chain.

## Try it

```bash
cargo build --locked
cargo run --locked -- 8080
```
