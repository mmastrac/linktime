# Scattered Collections

![Build Status](https://github.com/mmastrac/linktime/actions/workflows/rust.yml/badge.svg)

The crate is part of the [`linktime`](https://crates.io/crates/linktime) project.

| crate               |                                                         | docs                                                                                         | version                                                                                                           |
| ------------------- | ------------------------------------------------------- | -------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------- |
| `linktime`          | Convenience crate for `ctor`, `dtor` and `link-section` | [![docs.rs](https://docs.rs/linktime/badge.svg)](https://docs.rs/linktime)                   | [![crates.io](https://img.shields.io/crates/v/linktime.svg)](https://crates.io/crates/linktime)                   |
| `ctor`              | Module initialization functions before main             | [![docs.rs](https://docs.rs/ctor/badge.svg)](https://docs.rs/ctor)                           | [![crates.io](https://img.shields.io/crates/v/ctor.svg)](https://crates.io/crates/ctor)                           |
| `dtor`              | Module shutdown functions before main                   | [![docs.rs](https://docs.rs/dtor/badge.svg)](https://docs.rs/dtor)                           | [![crates.io](https://img.shields.io/crates/v/dtor.svg)](https://crates.io/crates/dtor)                           |
| `link-section`      | Linker-managed typed (slices) and untyped sections      | [![docs.rs](https://docs.rs/link-section/badge.svg)](https://docs.rs/link-section)           | [![crates.io](https://img.shields.io/crates/v/link-section.svg)](https://crates.io/crates/link-section)           |
| `scattered-collect` | Linker-managed collections: slices, sorted slices, maps | [![docs.rs](https://docs.rs/scattered-collect/badge.svg)](https://docs.rs/scattered-collect) | [![crates.io](https://img.shields.io/crates/v/scattered-collect.svg)](https://crates.io/crates/scattered-collect) |

A crate for defining linker-managed scattered collections in Rust.

The collections come in a 'referenced' and 'unreferenced' variant. The
referenced variants allow you to access the items as `static` handles at the
declaration site, while the unreferenced variants allow you to access the items
as a slice only. The latter, unreferenced variants may be more efficient.

## Zero-allocation collections

The collections are all zero-allocation. This means that they can be used in
`no-std`/`no-alloc` environments, and that they do not contribute to heap usage
whatsoever.

## Collections

- [`ScatteredSlice`]: A collection of sized items that collected into a slice in
  an arbitrary order.
- [`ScatteredSortedSlice`]: A collection of items that are available via slice,
  in sorted order.
- [`ScatteredReferencedSlice`]: A collection of items collected into a slice
  (link order), with each `static` item auto-wrapped as
  [`referenced_slice::Ref`].
- [`ScatteredSortedReferencedSlice`]: A collection of sized items that are
  available both via sorted slice and via reference at the declaration site
  (auto-wrapped as [`sorted_referenced_slice::Ref`]).
- [`ScatteredMap`]: A collection of key-value pairs that are available via
  slice, as well as indexed by key.
