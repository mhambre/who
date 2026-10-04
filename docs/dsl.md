# DSL fields and comparisons

A field selects an external input, such as a dependency or file.
A comparison turns that input into a predicate, a condition that is true or false.
When a predicate is true, `who::warn!` emits a warning and `who::error!` fails compilation.

## dependency

`dependency("foo")` selects a Cargo package by its package name, not its local dependency alias.
Dependency predicates read the resolved graph from `Cargo.lock`.
They support direct and transitive dependencies, renamed packages, registry sources, Git sources, and path dependencies.

| Comparison | Triggers when |
| --- | --- |
| `dependency("foo").changed_from("1.2.3")` | The resolved version is not `1.2.3` |
| `dependency("foo").compare(">=2")` | The resolved version matches the requirement |

The comparison requests human review rather than proving that dependency behavior changed.
If more than one reachable package named `foo` exists, `who` reports ambiguity instead of choosing one.
Locked optional, target, build, and dev dependency edges also participate in resolution.
`who` supports workspace roots and members and does not invoke Cargo during macro expansion.

## rustc

`rustc()` selects the compiler version used during compilation.
It takes no arguments.
Its comparisons use the same version rules as dependency predicates.

| Comparison | Triggers when |
| --- | --- |
| `rustc().changed_from("1.95.0")` | The compiler version differs from `1.95.0` |
| `rustc().compare(">=1.96")` | The compiler version matches the requirement |

## msrv

`msrv()` selects the calling package's minimum supported Rust version from `[package] rust-version`.
It takes no arguments and supports `rust-version.workspace = true` inheritance from `[workspace.package]`.
It does not select the running compiler version.

| Comparison | Triggers when |
| --- | --- |
| `msrv().changed_from("1.74.0")` | The declared minimum version differs from `1.74.0` |
| `msrv().compare(">=1.80")` | The declared minimum version matches the requirement |

Cargo declarations such as `"1.74"` become `1.74.0` for comparison.
A missing declaration causes a compilation error.
Manifest changes trigger another compilation of the calling crate.

## Version rules

Exact versions must include major, minor, and patch numbers.
Semver requirements use the [`semver`](https://docs.rs/semver) crate's syntax.
`compare` triggers when the version satisfies the requirement.
Use `!` to trigger outside an allowed range.
Replace existing `.matches(...)` calls with `.compare(...)`.

```text
>=2
>=1.8, <2
^1.4
~1.4
=1.2.3
```

## file

`file("schema.proto")` selects a file.
Relative paths start at the calling package's manifest directory.
Absolute paths refer to the supplied location directly.
The file comparison uses SHA-256 over the raw contents.

| Comparison | Triggers when |
| --- | --- |
| `file("schema.proto").changed_from("sha256:...")` | The file contents differ from the expected hash |

The expected hash requires the `sha256:` prefix and 64 hexadecimal digits.
Keep the baseline in source control.
A missing or unreadable file causes a compilation error.

## date

`date()` selects the compilation time and takes no arguments.
`date().after("2027-01-01")` requests review strictly after midnight UTC on that date.
Dates and times without a timezone use UTC.

You can also include a time, numeric offset, or named timezone:
`2027-01-01 09:30`, `2027-01-01T09:30:00-05:00`, or `2027-01-01 09:30 America/New_York`.

Date triggers run during compilation.
Passing a deadline does not trigger a warning until Cargo recompiles the calling crate.
If a named-zone time is ambiguous or nonexistent during a clock change, use a numeric offset or a valid local time.

## Boolean expressions

Combine predicates with `!`, `&&`, `||`, and parentheses.
The precedence order is `!`, then `&&`, then `||`.
Both operands are evaluated, so invalid predicates still produce errors even when their boolean result cannot change the overall result.

```rust
who::error!(
    !dependency("foo").compare(">=1.8, <2")
        || rustc().compare("<1.95"),
    "Recheck this dependency/compiler combination"
);
```
