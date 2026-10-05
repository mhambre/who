# Bound image dimensions before decoding

[`image` issue #1507](https://github.com/image-rs/image/issues/1507) reports a
truncated PNM header in `image` 0.23.14 that requested an enormous decoded
buffer. [A maintainer clarified](https://github.com/image-rs/image/issues/1507#issuecomment-878404984)
that the crash was caused by the decoder and `load_from_memory` not enforcing
size limits, rather than a buffer overflow.

This example checks the dimensions before decoding and rejects images above an
application-specific pixel budget. It pins `image` 0.23.14 to reproduce the
affected version. `image` 0.24.0 introduced configurable decoder limits for
rudimentary protection against resource exhaustion
([release notes](https://github.com/image-rs/image/blob/v0.24.9/CHANGES.md#version-0240));
the generic loading path reserves the decoded size against those limits
([implementation](https://github.com/image-rs/image/blob/v0.24.9/src/io/free_functions.rs)).

The `who::warn!` trigger asks for review whenever the resolved `image` version
changes from 0.23.14. An upgrade, particularly to 0.24.0 or later, is a reason
to check whether upstream limits are sufficient for the application's policy;
it does not by itself prove that the local pixel budget is unnecessary.

Run the example and its regression tests with:

```sh
cargo run --locked --manifest-path examples/image-size-limit/Cargo.toml
cargo test --locked --manifest-path examples/image-size-limit/Cargo.toml
```
