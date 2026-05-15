//! macro-magic integration tests.
use ::macro_magic::*;

__test!(__separate[
    __brace[()],
    __brace[[]],
    __brace[{}],
]: ((1) (2) (3)) => ((1) [2] { 3 }));

__test!(__separate[
    __brace[()],
    __brace[[]],
    __brace[{}],
]: ((1) (2) (3) (4) (5)) => ((1) [2] { 3 } (4) (5)));

__test!(__separate[
    __brace[()],
    __brace[[]],
]: ((1) (2)) => ((1) [2]));

__test!(__separate[
    __brace[()],
    __brace[[]],
]: ((1) (2) (3)) => ((1) [2] (3)));

__test!(__separate[
    __brace[()],
]: ((1) (2)) => ((1) (2)));

__test!(__separate[
    __brace[()],
]: ((1)) => ((1)));
