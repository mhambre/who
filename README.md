# who

[![Crates.io](https://img.shields.io/crates/v/who.svg)](https://crates.io/crates/who)
[![CI](https://github.com/mhambre/who/actions/workflows/ci.yml/badge.svg)](https://github.com/mhambre/who/actions/workflows/ci.yml)
![Supported operating systems](https://img.shields.io/badge/OS-Linux%20%7C%20macOS%20%7C%20Windows-lightgrey)

`who` lets you replace TODO comments that would otherwise get lost to the abyss with compile-time review triggers for code whose correctness, performance, or necessity depends on external context the compiler cannot verify, such as dependency behavior, file contents, or the compiler version.

Determine **who** should bring you back if their context changes.

## Examples

### Revisit an optimization after a compiler bug is fixed

Suppose a rustc or LLVM regression forces you to disable an optimized implementation:

```rust
fn transform(input: &[f32], output: &mut [f32]) {
    who::warn!(
        rustc().changed_from("1.95.0"),
        "Recheck rust-lang/rust#123456 and restore the SIMD path if fixed"
    );

    // SIMD path disabled because of a compiler codegen bug.
    scalar_transform(input, output);
}
```

The fallback may be completely correct and well tested. Those tests will still pass after the compiler bug is fixed.

The compiler upgrade is what should bring this code back to your attention.

### Replace dependency TODOs with review triggers

Codebases often accumulate TODOs tied to upstream bugs, missing features, temporary limits, and compatibility workarounds:

```rust
// TODO: remove this once foo#812 is fixed
let frame = Bytes::copy_from_slice(frame.as_slice());

// TODO: increase this once foo#921 supports larger reads
let len = len.min(64 * 1024);

// TODO: remove compatibility path after foo 2.4
use_compatibility_path();
```

The problem is not that these workarounds are untested. They may be completely correct.

The problem is that a TODO has no connection to the dependency change that makes it relevant again.

With `who`, that relationship becomes explicit:

```rust
who::warn!(
    dependency("foo").changed_from("3.7.2"),
    "Recheck foo#812 and restore the zero-copy path if fixed"
);

let frame = Bytes::copy_from_slice(frame.as_slice());
```

Or when you know the release boundary:

```rust
who::warn!(
    dependency("foo").compare(">=2.4"),
    "Check foo#481 and remove this compatibility path if the fix has landed"
);

use_compatibility_path();
```

Instead of leaving reminders scattered through the codebase and hoping someone notices them later, `who` resurfaces them when the dependency actually changes.

### Revisit a temporary workaround on a date

An upstream change can have a planned date even when you do not know its release version:

```rust
who::warn!(
    date().after("2027-01-01 09:00 America/New_York"),
    "Check whether the migration finished and remove this fallback"
);
```

This compares the compilation time with the deadline.
It requests a review after that instant, even if no dependency version changes.

## When to use it

Use `who` when a change outside your code should bring a specific assumption or workaround back to your attention.

Common cases include:

- Relying on undocumented or implementation-specific behavior in a dependency.
- Carrying a workaround for a dependency or compiler bug.
- Disabling an optimization until an upstream issue is fixed.
- Keeping temporary limits or compatibility paths tied to a dependency version.
- Reviewing code again when an external file or generated artifact changes.

If you can directly and deterministically test the behavior that matters, write the test.

`who` is useful when the important signal is not that the code became incorrect, but that something changed which makes the code worth reviewing again.

## Usage

Add `who` to your dependencies:

```toml
[dependencies]
who = "..."
```

Then place a warning or error next to the code it protects:

```rust
who::warn!(
    dependency("reqwest").changed_from("0.12.23"),
    "Recheck error parsing"
);

who::error!(
    file("schema.proto").changed_from("sha256:..."),
    "Revalidate the handwritten decoder"
);
```

When the condition is true:

- `who::warn!` emits a compiler warning.
- `who::error!` fails compilation.

When the condition is false, the macro leaves no runtime behavior behind.

`who` can be used inside functions or at module scope.

See the [DSL reference](docs/dsl.md) for fields, comparison methods, and boolean expressions.

See the [changelog](CHANGELOG.md) for release history.

### Warnings

Stable Rust does not currently expose general proc-macro warning diagnostics, so `who::warn!` emits its warning through a deprecated constant.

Normal Rust lint controls therefore apply.

```rust
#[allow(deprecated)]
```

can suppress the warning, while:

```text
-D warnings
```

or:

```rust
#[deny(deprecated)]
```

can turn it into an error.

`who::error!` always emits a compilation error.

## What `who` is not

`who` is not a replacement for stronger guarantees.

Use:

- The type system when the invariant can be encoded in types.
- `#[cfg]` for conditional compilation.
- Tests when behavior can be reproduced and checked directly.
- `assert!` or `debug_assert!` for runtime invariants.

Use `who` when a change in the surrounding context should bring otherwise-valid code back up for human review.

## Runtime cost

None.

The checks run during compilation and do not add runtime branches, strings, helper functions, or metadata to the final program.

## Development

Run the test suite with:

```bash
cargo test --locked
```

Check the minimum supported Rust version with:

```bash
cargo +1.74.0 check --lib --locked
```
