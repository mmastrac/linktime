# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
