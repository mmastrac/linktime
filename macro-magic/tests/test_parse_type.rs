//! test path parsing.
use ::macro_magic::*;

__test!(__parse_type: (SomeType) =>
(
    type = (SomeType)
    prefix = ()
    final = SomeType
    generics = ()
));
__test!(__parse_type: (::SomeType) =>
(
    type = (:: SomeType)
    prefix = (::)
    final = SomeType
    generics = ()
));
__test!(__parse_type: (::root::SomeType) =>
(
    type = (:: root :: SomeType)
    prefix = (:: root ::)
    final = SomeType
    generics = ()
));

__test!(__parse_type: (root::SomeType) =>
(
    type = (root :: SomeType)
    prefix = (root ::)
    final = SomeType
    generics = ()
));

__test!(__parse_type: (::root::more::SomeType) =>
(
    type = (:: root:: more :: SomeType)
    prefix = (:: root:: more ::)
    final = SomeType
    generics = ()
));

__test!(__parse_type: (root::more::SomeType) =>
(
    type = (root:: more :: SomeType)
    prefix = (root:: more ::)
    final = SomeType
    generics = ()
));

__test!(__parse_type: (SomeType<T, U>) =>
(
    type = (SomeType < T, U >)
    prefix = ()
    final = SomeType
    generics = (T, U)
));

__test!(__parse_type: (::crazy::long::path_to_type::with::generics::SomeType<T, U>) =>
(
    type = (:: crazy:: long :: path_to_type :: with :: generics :: SomeType < T, U >)
    prefix = (:: crazy:: long :: path_to_type :: with :: generics ::)
    final = SomeType
    generics = (T, U)
));
