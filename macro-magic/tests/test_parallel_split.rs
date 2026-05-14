use ::macro_magic::*;

__test!(__parallel[
    __brace[[]],
    __brace[{}],
]: (1) => ([1] { 1 }));

__test!(__parallel[
    __brace[()],
    __brace[[]],
    __brace[{}],
]: (1) => ((1) [1] { 1 }));

// Split is harder to test...
macro_rules! __test_split {
    ( $input:tt ) => {
        __perform!(
            ($input),
            __chain[
                __split[
                    __brace[()],
                    __brace[[]],
                ],
                __stringify[prefix=[const _: &str =] suffix=[;]],
            ]
        );
    }
}

__test_split!(1);
