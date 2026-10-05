# dependency-bug-workaround

This runnable example pins `bytes` to `1.9.0` and uses `who::warn!` to guard a tiny workaround for a past `bytes` bug.

## Original problem

`bytes` issue [#779](https://github.com/tokio-rs/bytes/issues/779), ["Preserve addresses of empty bytes in `slice`"](<https://github.com/tokio-rs/bytes/issues/779>), shows that an empty `Bytes::slice(i..)` could lose the original allocation address. Code that derived offsets from `slice(...).as_ptr()` could then underflow or panic at end-of-buffer.

The fix landed in PR [#780](https://github.com/tokio-rs/bytes/pull/780), ["Guarantee address in `slice()` for empty slices."](<https://github.com/tokio-rs/bytes/pull/780>). The `bytes` release notes for [`v1.11.0`](https://github.com/tokio-rs/bytes/releases/tag/v1.11.0) cite that PR explicitly.

## Workaround in this example

Instead of asking `bytes.slice(begin..)` for the empty tail, `tail_offset` returns `bytes.len()` directly when `begin == bytes.len()`. That keeps the logical offset stable on `bytes 1.9.0` without depending on the pre-fix empty-slice behavior.

The guard is:

```rust
who::warn!(
    dependency("bytes").changed_from("1.9.0"),
    "Recheck tokio-rs/bytes#779 and tokio-rs/bytes#780; bytes 1.11.0 guarantees addresses for empty slice() results, so this explicit end-of-buffer branch may be removable."
);
```

## Which release should re-trigger review?

The first release that should make you review this workaround is `bytes v1.11.0`, because that is where the upstream fix shipped. This example uses `changed_from("1.9.0")`, so any dependency drift from the locked baseline will warn, but `v1.11.0` is the release that directly addresses the bug.

## Run it

```bash
cargo build --locked
cargo run --locked
```
