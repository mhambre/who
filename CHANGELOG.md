# Changelog

## [0.3.0] - 2026-10-04

### Added

- `msrv()` predicates to review changes to the calling package's declared minimum supported Rust version.

### Changed

- Deprecated `.matches(...)` in favor of `.compare(...)` for version requirement predicates.

## [0.2.0] - 2026-10-03

### Added

- `date().after(...)` predicates to trigger reviews after a specified date and time.

### Changed

- Made file-hash checks an optional Cargo feature, enabled by default, so users can opt out of the `sha2` dependency.

## [0.1.0] - 2026-10-03

### Added

- Compile-time `who::warn!` and `who::error!` review triggers.
- Dependency-version and Rust compiler-version predicates.
- SHA-256 file-change predicates.
- Boolean expressions for combining predicates.
