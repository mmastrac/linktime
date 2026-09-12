# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Added UEFI support (`*-unknown-uefi`), run via `dtor::run_destructors`.

## [1.0.6] - 2026-08-09

### Changed

- Bump `linktime-proc-macro` dependency to 0.2.3.

### Documentation

- Documented the `crate_path` override for re-exporting the macro from another
  crate.

## [1.0.5] - 2026-05-30

### Changed

- Bump `link-section` dependency to 0.18.1.
- Fixed WASM import module for `atexit` to support Rust 1.96.

## [1.0.4] - 2026-05-28

### Changed

- Bump `link-section` dependency to 0.18.0.
- Improve error messages for bad attribute values.

## [1.0.3] - 2026-05-16

### Fixed

- `#[dtor]` requires significantly less macro recursion.

## [1.0.2] - 2026-05-08

### Added

- Support for UEFI targets (`.fini_array` matching gnu-efi)
- Fallback to `.fini_array` for unsupported targets (with a warning)

### Changed

- `target_vendor = "pc"` is now `target_os = "windows"`. This should be a
  strictly increasing set of support for windows targets.

## [1.0.1] - 2026-05-07

### Changed

- OpenBSD support uses `method = at_module_exit` rather than `method = linker`
  by default due to C library inconsistencies. When using `method = linker`,
  the `link_section` is now `.dtors` rather than `.fini_array`.
- Documentation improvements.

## [1.0.0] - 2026-05-03

### Changed

- Stabilized (yay!). Functionally identical to 0.14.1. Thanks to all the
  contributors who helped along the way! Please file any upgrade issues in
  <https://github.com/mmastrac/linktime/issues>.
- For those upgrading from earlier versions, the major changes for you to note
  are:
    - `#[dtor(unsafe)]` is now required for `#[dtor]` items. If you are building a
      binary, you can use `RUSTFLAGS="--cfg linktime_no_fail_on_missing_unsafe"`
      (or alternatively, specify this in your `config.toml` file) to bypass the
      error.
    - For those re-exporting `dtor` from their own crates: the
      `dtor::declarative::dtor!` macro should be preferred over
      `#[dtor(crate_path = ...)]`. The latter form will continue to work, but
      the declarative macro is far more stable for most use cases. See
      <https://docs.rs/dtor/latest/dtor/declarative/macro.dtor.html> for more
      details.

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
