# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.14.1] - 2026-05-04

### Changed

- `wasm32-unknown-unknown` would run dtor items on each call of an exported
  function.

## [0.14.0] - 2026-05-03

### Fixed

- WASM support now uses `atexit` on all platforms (including
  `wasm32-unknown-unknown`).
- Typos and other documentation nits.

## [0.13.1] - 2026-05-02

### Changed

- Documentation polish and typo fixes.

## [0.13.0] - 2026-05-02

### Changed

- `used_linker` feature moved to `--cfg linktime_used_linker` flag.

## [0.12.0] - 2026-04-30

### Added

- Support for `#[dtor]` on `impl` items. To be valid, the `fn` must have no
  `self` parameter and must not access any generic parameters from the outer
  item.
- Added `life before main` documentation to all crates.

### Removed

- Removed support code for `ctor`'s deprecated `dtor` macros.

## [0.11.0] - 2026-04-28

### Added

- Added `method` attribute to `dtor` macro.
- Added `link_section` and `export_name_prefix` attributes to `dtor` macro.
- AIX support for `ctor`/`dtor` crates.

### Changed

- Significant rewrite to ctor/dtor macros and documentation.
- Renamed `at_library_exit` to `at_module_exit` in `dtor` crate.
- Macro attributes and crate features are auto-documented.

### Removed

- `cxa_atexit` feature from `dtor` crate. (appropriate method is now used per-platform)
- `export_native` feature from `dtor` crate. (natives always exported)

## [0.8.1] - 2026-04-22

### Added

- Included licenses in all files.
- Bumped proc-macro dependency versions.
- `dtor` crate exports `native` module with `at_binary_exit` and `at_library_exit` functions.

### Fixed

- Various hardening fixes under Miri.
