//! A set of macros to test macros.

/// Unit test for a macro.
#[macro_export]
#[doc(hidden)]
macro_rules! __test {
    ($macro:path $([$($macro_args:tt)*])?: $input:tt => $output:tt) => {
        const _: () = {
            const INPUT: &str = "Failed to evaluate input";
            {
                __perform!(
                    $input,
                    __chain[
                        $macro $([$($macro_args)*])?,
                        __stringify[],
                        __surround[prefix=[const INPUT: &str = ] suffix=[;]],
                    ]
                );

                __perform!(
                    $output,
                    __chain[
                        __stringify[],
                        __surround[prefix=[const OUTPUT: &str = ] suffix=[;]],
                    ]
                );

                if !$crate::test::const_str_eq(
                    INPUT,
                    OUTPUT,
                ) {
                    const SLICE: &[&str] = &[
                        "Input and output do not match, processed input:\n", INPUT,
                        "\n... was not equal to expected output:\n", OUTPUT,
                        "\nIn ",
                        file!()
                    ];
                    let mut out = [0; $crate::test::const_str_slice_len(SLICE)];
                    panic!("{}", $crate::test::const_str_slice_concat(SLICE, &mut out));
                }
            }
        };
    };
}

/// Calculates the length of a slice of strings.
pub const fn const_str_slice_len(s: &[&str]) -> usize {
    let mut len = 0;
    let mut i = 0;
    while i < s.len() {
        len += s[i].len();
        i += 1;
    }
    len
}

/// Concatenates a slice of strings into a single string.
pub const fn const_str_slice_concat<'a>(s: &[&str], out: &'a mut [u8]) -> &'a str {
    let mut i = 0;
    let mut j = 0;
    while i < s.len() {
        let mut len = s[i].len();
        let mut k = 0;
        while len > 0 {
            out[j] = s[i].as_bytes()[k];
            j += 1;
            k += 1;
            len -= 1;
        }
        i += 1;
    }
    match core::str::from_utf8(out) {
        Ok(s) => s,
        Err(_) => panic!("Invalid UTF-8"),
    }
}

/// Compares two strings for equality.
pub const fn const_str_eq(a: &str, b: &str) -> bool {
    let mut i = 0;
    let mut j = 0;
    while i < a.len() && j < b.len() {
        if a.as_bytes()[i] != b.as_bytes()[j] {
            return false;
        }
        i += 1;
        j += 1;
    }
    i == a.len() && j == b.len()
}
