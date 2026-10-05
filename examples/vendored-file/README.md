# Vendored file change trigger

Shows `file(...).changed_from(...)` guarding code derived from a vendored data file.

## The problem

`data/cjk-blocks.txt` is a vendored excerpt of Unicode 15.0 `Blocks.txt`. `src/main.rs` carries a hand-copied
range table (`CJK_RANGES`) derived from it. Unicode releases do change this data: Unicode 15.1
(September 2023) added CJK Unified Ideographs Extension I (`U+2EBF0..U+2EE5D`) and
widened the Extension F block end to `U+2EBEF`. The vendored file is [published by the Unicode Consortium](https://www.unicode.org/versions/Unicode15.1.0/);
see the UCD `Blocks.txt` for each release.

Nothing fails when the file is refreshed, and tests of the old table still pass, but the table silently goes stale.
The demo prints `false` for `U+2EBF0` because the table predates Extension I.

## The workaround and trigger

The table is kept by hand, guarded by:

```rust
who::warn!(
    file("data/cjk-blocks.txt").changed_from("sha256:..."),
    "data/cjk-blocks.txt changed: regenerate CJK_RANGES and update the baseline hash"
);
```

Paths resolve relative to the crate's manifest directory, and the file is tracked so edits trigger a rebuild.

## When review is triggered

Whenever the vendored file's bytes differ from the baseline, e.g. when it is replaced with Unicode 15.1 data.
After updating `CJK_RANGES`, set the baseline to the new `sha256sum data/cjk-blocks.txt`.
Use `who::error!` instead to fail the build.

## Run

```sh
cargo run --locked
```

Edit `data/cjk-blocks.txt` and rebuild to see the warning.
