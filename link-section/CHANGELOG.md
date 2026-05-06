# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.15.0] - 2026-05-06

### Added

- Support for `const` items in link sections.
- WASM now requires `const` items, and uses `ctor`-like initialization to copy
  data to a contiguous section. To access link-section slices in WASM in
  constructor functions, make sure to use `priority = 1`.
- Zero-sized types are no longer used in `extern`s. Windows now uses a
  non-zero-sized alignment marker to align the start and end of the section.
  Other LLVM/GCC platforms use a `u8`.
- `link-section` is now `no_std`-compatible.

## [0.14.0] - 2026-05-04

### Changed

- WASM targets now use an extern `read_custom_section` function to read custom
  sections.

## [0.13.1] - 2026-05-02

### Changed

- Documentation polish and typo fixes.

## [0.13.0] - 2026-05-02

### Changed

- `used_linker` feature moved to `--cfg linktime_used_linker` flag.
- On macOS, `fn` items are placed in a `__TEXT,__text,regular,pure_instructions`
  section (fixes a linker warning in nightly).

## [0.11.0] - 2026-04-28

### Changed

- Macro attributes and crate features are auto-documented.

## [0.2.1] - 2026-04-22

### Added

- Included licenses in all files.
- Bumped proc-macro dependency versions.

### Changed

- `link-section` crate no longer offers `const` section pointers.
