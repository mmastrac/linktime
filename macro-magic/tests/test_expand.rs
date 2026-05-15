//! macro-magic integration tests.
use ::macro_magic::*;

__test!(__expand[
    __brace[[]],
]: ((1)) => ((1)[1]));

__test!(__expand[
    __brace[[]],
]: ((1) (2) (3)) => ((1) [1] (2) (3)));
