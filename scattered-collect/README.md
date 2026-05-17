# Scattered Collections

A crate for defining linker-managed scattered collections in Rust.

The collections come in a 'referenced' and 'unreferenced' variant. The
referenced variants allow you to access the items as `static` handles at the
declaration site, while the unreferenced variants allow you to access the items
as a slice only. The latter, unreferenced variants may be more efficient.

## Collections

 - [`ScatteredSlice`]: A collection of sized items that collected into a slice
       in an arbitrary order.
 - [`ScatteredSortedSlice`]: A collection of items that are available via slice,
       in sorted order.
 - [`ScatteredReferencedSlice`]: A collection of items collected into a slice
       (link order), with each entry wrapped as [`Ref`] so `static` items work
       on targets such as WASM.
 - [`ScatteredSortedReferencedSlice`]: A collection of sized items that are
       available both via sorted slice and via reference at the declaration
       site.
