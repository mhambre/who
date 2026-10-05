# dependency-shim

This example shows a real dependency compatibility shim guarded by `who::warn!`.

## Original problem

Before `uuid` 1.25.0, the crate exposed `uuid::serde::compact` but not a `uuid::serde::bytes` helper. That meant binary formats such as CBOR encoded a UUID as a 16-element integer array instead of a single byte string. The original report is [uuid-rs/uuid#893](https://github.com/uuid-rs/uuid/issues/893).

## Workaround in this example

`src/main.rs` defines a local `uuid_serde_bytes` module that serializes a `Uuid` as raw bytes and deserializes it back, then uses it with `#[serde(with = "uuid_serde_bytes")]`.

The program compares that shim with `uuid::serde::compact` on `uuid` 1.24.1 and shows the same size difference described in the upstream issue:

- `uuid::serde::compact`: 32-byte CBOR payload
- local `uuid_serde_bytes` shim: 17-byte CBOR payload

## Upstream fix

- Problem report: [uuid-rs/uuid#893](https://github.com/uuid-rs/uuid/issues/893)
- Fixing PR: [uuid-rs/uuid#902](https://github.com/uuid-rs/uuid/pull/902)
- Release that added the feature: [`uuid` 1.25.0](https://github.com/uuid-rs/uuid/releases/tag/1.25.0)

## Review trigger

`who::warn!` is intentionally tied to:

```rust
dependency("uuid").compare(">=1.25.0")
```

Upgrading this example to `uuid` 1.25.0 or newer should re-trigger review, because that is the point where upstream gained `uuid::serde::bytes` and this local shim may no longer be worth carrying.

## Try it

```bash
cargo build --locked
cargo run --locked
```
