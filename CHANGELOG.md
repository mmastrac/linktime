# Changelog

All notable changes to this project will be documented in this file.

## ctor [0.13.0] - Unreleased

### Changed

- `#[ctor(priority = naked)]` is now `#[ctor(naked)]`.

### Added

- Re-added link section option for body of `#[ctor]` items (supported for Linux/Android/FreeBSD/Apple).

## ctor [0.12.0] - 2026-04-30

### Added

- Support for `#[ctor]` on `impl` items. To be valid, the `fn` must have no
  `self` parameter and must not access any generic parameters from the outer
  item.
- Added `life before main` documentation to all crates.
- `early` and `late` priority values are now supported on all platforms.

### Removed

- deprecated `dtor` feature and crate dependency from `ctor` crate (use the `dtor` crate directly).

### Fixed

- AIX uses "standard" priority values from 0 to 999, early and late (mapped to
  80000000 to 80000999).

### Changed

- If the `priority` feature is enabled, `ctor` priority sorting is now stable
  and consistent across platforms: `early`/`0`/`unspecified`, then `1 <= N <
  1000`, then `late`.
- If a `link_section` or `export_name_prefix` is specified, a `priority` value
  must not be specified (now a compiler error).
- Migrated to using the `linktime-proc-macro` crate for proc-macro support.

## dtor [0.12.0] - 2026-04-30

### Added

- Support for `#[dtor]` on `impl` items. To be valid, the `fn` must have no
  `self` parameter and must not access any generic parameters from the outer
  item.
- Added `life before main` documentation to all crates.

### Removed

- Removed support code for `ctor`'s deprecated `dtor` macros.


## ctor [0.11.1] - 2026-04-28

### Changed

- Deprecated ``dtor` macros in favor of `dtor` crate.
- Migrated to using the `linktime-proc-macro` crate for proc-macro support.

### Fixed

- Fixed some stray `dtor` references in ctor docs.

## ctor [0.11.0] - 2026-04-28
## dtor [0.11.0] - 2026-04-28
## link-section [0.11.0] - 2026-04-28

### Added

- Added `method` attribute to `dtor` macro.
- Added `link_section` and `export_name_prefix` attributes to `dtor` macro.
- AIX support for `ctor`/`dtor` crates.

### Changed

- Significant rewrite to ctor/dtor macros and documentation.
- Renamed `at_library_exit` to `at_module_exit` in `dtor` crate.
- Macro attributes and crate features are auto-documented.
- Rewrote `statics` code in `ctor` to not require `std`.

### Removed

- `cxa_atexit` feature from `dtor` crate. (appropriate method is now used per-platform)
- `export_native` feature from `dtor` crate. (natives always exported)

## ctor [0.10.1] - 2026-04-22
## dtor [0.8.1] - 2026-04-22
## link-section [0.2.1] - 2026-04-22

### Added

- Included licenses in all files.
- Bumped proc-macro dependency versions.
- `dtor` crate exports `native` module with `at_binary_exit` and `at_library_exit` functions.

### Fixed

- Fix MSRV in ctor docs.
- Various hardening fixes under Miri.
- Adding priority to `ctor`s accidentally enabled the anonymous flag.

### Changed

- `link-section` crate no longer offers `const` section pointers.
- `ctor` exports all `dtor` macros from `dtor` crate rather than reimplementing them.
