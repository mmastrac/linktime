//! macro-magic integration tests.
use ::macro_magic::*;

macro_rules! __make_tuple {
    ( @entry next=$next:path[$next_args:tt], input=($value:literal) ) => {
        $next!($next_args, (($value, $value)));
    };
    ( @entry next=$next:path[$next_args:tt], input=$input:tt $(, args=$args:tt )? ) => {
        const _: () = {
            compile_error!(concat!("Unexpected input: ", stringify!($input)));
        };
    };
}

__test!(__chain[
    __parallel[
        __emit[pub const EXPR: (u8, u8) = ],
        __make_tuple,
        __emit[;],
    ],
]: (1) => (pub const EXPR: (u8, u8) = (1, 1);));
