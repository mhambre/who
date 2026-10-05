# Vendored file guard example

This example shows how to guard hand-written code that was copied from a vendored or generated file.

## Problem

`is_vendored_python_keyword` is a hand-written `matches!` table derived from `vendor/python-keywords.txt`.
If the vendored file changes later, tests for the surrounding code may still pass, so the handwritten mirror can quietly drift out of sync.

## Workaround

`who::warn!` watches the vendored file:

```rust
who::warn!(
    file("vendor/python-keywords.txt").changed_from("sha256:60812d3ca89d1be613e875bb24ccccbcb4c57d255b91cba1ac194b5348cb1f7d"),
    "Review is_vendored_python_keyword; it was handwritten from vendor/python-keywords.txt"
);
```

That hash is the current SHA-256 of `vendor/python-keywords.txt`, so `cargo build --locked` stays quiet until the file changes.
Once it changes, the build emits a warning that points back at the handwritten code derived from it.

## Upstream source

`vendor/python-keywords.txt` is a tiny vendored subset of CPython's keyword list from:

- <https://github.com/python/cpython/blob/v3.12.0/Lib/keyword.py>

This example keeps only six entries to stay minimal.

## Run it

```bash
cargo build --locked
cargo run --locked
```

Expected output:

```text
Vendored subset matches: assert, True
```

## Update the baseline after an intentional file change

1. Edit `vendor/python-keywords.txt`.
2. Recompute the hash:

   ```bash
   sha256sum vendor/python-keywords.txt
   ```

3. Replace the `sha256:...` string in `src/main.rs` with the new digest.
4. Rebuild with `cargo build --locked` and review `is_vendored_python_keyword`.
