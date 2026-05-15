//! macro-magic integration tests.
use ::macro_magic::*;

__test!(__for_each[
    __surround[prefix=[BEFORE ] suffix=[AFTER ]]
]: (
    (1 ) (2 ) (3 )
) => (BEFORE (1) AFTER BEFORE (2) AFTER BEFORE (3) AFTER));

__test!(__for_each[
    __surround[prefix=[BEFORE ] suffix=[AFTER ]]
]: (
    [1 ] [2 ] [3 ]
) => (BEFORE [1] AFTER BEFORE [2] AFTER BEFORE [3] AFTER));

__test!(__for_each[
    __surround[prefix=[BEFORE ] suffix=[AFTER ]]
]: (
    1 2 3
) => (BEFORE 1 AFTER BEFORE 2 AFTER BEFORE 3 AFTER));
