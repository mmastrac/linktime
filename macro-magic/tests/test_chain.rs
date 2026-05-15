//! macro-magic integration tests.
use ::macro_magic::*;

macro_rules! __parse_chain_input {
    ( @entry next=$next:path[$next_args:tt], input=({
        $($name:ident : $value:expr;)*
    }) ) => {
        $next ! ( $next_args, ( $(pub const $name: u8 = $value;)* ) );
    };
    ( @entry next=$next:path[$next_args:tt], input=$input:tt $(, args=$args:tt )? ) => {
        const _: () = { compile_error!(concat!("Unexpected input: ", stringify!($input))); };
    };
}

__test!(__chain[
    __debug,
    __parse_chain_input,
    __debug,
]: ({
    INPUT: 1;
    OUTPUT: 2;
}) => (
    pub const INPUT : u8 = 1;
    pub const OUTPUT : u8 = 2;
));
