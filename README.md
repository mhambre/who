# who

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
    dependency("foo").matches(">=2.4"),
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

### Conditions

Conditions can be combined with `!`, `&&`, `||`, and parentheses.

```rust
who::error!(
    !dependency("foo").matches(">=1.8, <2")
        || rustc().matches("<1.95"),
    "Recheck this dependency/compiler combination"
);
```

Supported conditions:

| Condition | Triggers when |
| --- | --- |
| `dependency("foo").changed_from("1.2.3")` | The resolved version is not `1.2.3` |
| `dependency("foo").matches(">=2")` | The resolved version matches the requirement |
| `file("schema.proto").changed_from("sha256:...")` | The file contents differ from the expected hash |
| `rustc().changed_from("1.95.0")` | The compiler version differs from `1.95.0` |
| `rustc().matches(">=1.96")` | The compiler version matches the requirement |
| `date().after("2027-01-01")` | The compilation time is after the given date or time |

Exact versions must include major, minor, and patch numbers.

Semver requirements use the [`semver`](https://docs.rs/semver) crate's syntax:

```text
>=2
>=1.8, <2
^1.4
~1.4
=1.2.3
```

`matches` triggers when the requirement matches. Use `!` when you want to trigger outside an allowed range.

### Dependency resolution

Dependency checks use the resolved dependency graph from `Cargo.lock`.

They support:

- Direct and transitive dependencies.
- Registry, Git, and path dependencies.
- Workspace roots and members.
- Renamed dependencies.

`dependency("foo")` refers to the Cargo package name, not the local dependency alias.

If more than one reachable package named `foo` exists, `who` reports the ambiguity instead of guessing.

`who` does not invoke Cargo during macro expansion.

### File checks

File paths are relative to the calling package's manifest directory unless an absolute path is used.

File guards use SHA-256:

```rust
who::warn!(
    file("schema.proto").changed_from(
        "sha256:0123456789abcdef..."
    ),
    "Recheck schema assumptions"
);
```

The `sha256:` prefix is required.

### Date triggers

Use `date().after("2027-01-01")` to request a review after midnight UTC on that date.
Dates and times without a timezone use UTC.

You can also include a time, numeric offset, or named timezone:
`2027-01-01 09:30`, `2027-01-01T09:30:00-05:00`, or `2027-01-01 09:30 America/New_York`.

Date triggers run during compilation, so passing a deadline does not trigger a warning until Cargo recompiles the calling crate.

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
