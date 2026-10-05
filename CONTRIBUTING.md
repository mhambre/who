# Contributing to `who`

Thanks for helping improve `who`. Contributions should keep the documented behavior, feature combinations, and minimum supported Rust version (MSRV) in sync.

## Prerequisites and validation

Install Rust with `rustup`. The stable toolchain is used for formatting, linting, checks, and tests; Rust 1.74.0 is the MSRV. Install the components and toolchain with:

```bash
rustup toolchain install stable --profile minimal --component clippy --component rustfmt
rustup toolchain install 1.74.0 --profile minimal
```

From the repository root, run the general checks used by CI:

```bash
cargo +stable fmt --all -- --check
cargo +stable check --all-targets --locked
cargo +stable check --all-targets --locked --no-default-features
cargo +stable clippy --all-targets --locked -- -D warnings
cargo +stable clippy --all-targets --locked --no-default-features -- -D warnings
cargo +stable fetch --locked
cargo +stable test --locked
cargo +stable test --locked --no-default-features

cargo +1.74.0 check --lib --locked
cargo +1.74.0 check --lib --locked --no-default-features
```

If you add or modify a feature, check, lint, and test it as well. Replace `<feature>` with the feature being changed:

```bash
cargo +stable check --all-targets --locked --no-default-features --features <feature>
cargo +stable clippy --all-targets --locked --no-default-features --features <feature> -- -D warnings
cargo +stable test --locked --no-default-features --features <feature>
```

`scripts/ci.sh <check|clippy|test|examples|msrv>` runs the same tasks as CI sequentially across all features, each feature alone, and no features. Features are read from `Cargo.toml` and examples from `examples/*/`, so new ones need no CI edits.

Keep `Cargo.lock` in sync when changing dependencies or package versions; CI uses `--locked`.

## DSL and predicates

The shared expression grammar and AST live in `src/dsl/`; predicate families are routed through `src/dsl/predicates/mod.rs`. Put family-specific parsing and evaluation in a module under `src/dsl/predicates/`, and add its route and `Predicate` variant to the predicate module. Keep shared expression parsing and evaluation generic rather than embedding family-specific behavior there.

Predicate evaluation uses `eval::Context` for external inputs. Reuse its cached inputs, or extend the context when a family needs a new shared input. Follow existing feature gates when a predicate is optional. See [the DSL reference](docs/dsl.md) for user-facing syntax and behavior.

## Tests and documentation

Add tests for new behavior and edge cases. Use unit tests for parsing and evaluation details, and integration or UI tests in `tests/` when behavior is visible to a calling crate or compiler diagnostics. For optional predicates, check the feature-enabled and feature-disabled paths, and run the relevant feature configurations above.

Update `docs/dsl.md` when syntax or predicate behavior changes, and update the README when the change affects its overview or examples. Keep examples and documented behavior consistent with the implementation.

## Pull requests

Keep each pull request focused. Prefix its title with a Conventional Commits type, using `<type>(<optional scope>): <description>` (for example, `docs: clarify feature test guidance`). Explain the motivation and user-visible change, mention relevant feature or compatibility considerations, and list the checks you ran. Include tests and documentation updates appropriate to the change.
