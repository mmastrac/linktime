# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Fixed

- Spurious `duplicate hash found` panic on control-byte collisions.

## [0.22.0] - 2026-08-09

### Added

- The `#[scatter]` / `#[gather]` proc-macro forms now accept a
  `crate_path = <path>` override, allowing the macros to be re-exported from
  another crate without a direct dependency on `scattered-collect`. Documented
  re-exporting via both the declarative forms and `crate_path`.

### Changed

- Bumped MSRV to 1.89.
- Bumped `ctor`, `link-section`, `linktime` and `linktime-proc-macro`
  dependencies.

## [0.21.3] - 2026-07-06

- Bumped `link-section` dependency to 0.19.0 to fix WASM LTO issues.

## [0.21.2] - 2026-06-26

### Fixes

- `ScatteredMap` and `ScatteredSet` use type aliases from the `gather`'d
  collection rather than re-exporting the `gather` macro's type. This fixes
  issues using $crate in collection types.
- `ScatteredIterable` cross-module fixes.

## [0.21.1] - 2026-06-26

### Changed

- `ScatteredMap` entries can be const-computed

### Fixed

- `ScatteredMap` type-mismatch error messages should be much clearer.

## [0.21.0] - 2026-06-25

### Changed

- `ScatteredMap` now implements `get` rather than `find` using `Borrow<Q>` instead of `&Q`.
- `ConstHash` now implements `hash` for more string-like types.

### Fixed

- #[allow(...)] no longer breaks #[scatter] attribute parsing.
