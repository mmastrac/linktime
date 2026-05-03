#![recursion_limit = "150"]
use ::macro_magic::*;

__declare_features!(
    my_macro: my_macro_parse;

    /// Enable support for the standard library. This is required for static
    /// ctor variables, but not for functions.
    std {
        feature: "std";
    };
    /// Marks a ctor/dtor as unsafe.
    unsafe {
        attr: [(unsafe) => (unsafe)];
    };
    priority {
        attr: [(priority = $priority_value:literal) => ($priority_value)];
        example: "priority = N";
        validate: [($numeric:literal), (early), (late)];
    };
    used_linker {
        attr: [(used(linker)) => (used_linker)];
    };
    /// Make the ctor function anonymous.
    anonymous {
        attr: [(anonymous) => (anonymous)];
    };
);

__test!(__split_meta:
    (#[my_macro] fn foo() { /* ... */ }) => 
    ((#[my_macro]) (fn foo() { /* ... */ })));
__test!(__split_meta:
    (#[my_macro] unsafe fn foo() { /* ... */ }) => 
    ((#[my_macro]) (unsafe fn foo() { /* ... */ })));
__test!(__split_meta:
    (#[my_macro] #[other_macro] fn foo() { /* ... */ }) => 
    ((#[my_macro] #[other_macro]) (fn foo() { /* ... */ })));

__test!(__parse_item[my_macro_parse]:
(
    #[my_macro(unsafe, priority = 1)]
    fn foo() { /* ... */ }
) =>
(
    features = (std = std : default, unsafe = unsafe : value, priority = 1 : value, used_linker = (): default, anonymous = (): default,),
    self = (unsafe, priority = 1),
    meta = (),
    item = (fn foo() { /* ... */ })
));

__test!(__parse_item[my_macro_parse]:
(
    #[other]
    #[my_macro(unsafe)]
    #[doc]
    fn foo() { /* ... */ }
) =>
(
    features = (std = std : default, unsafe = unsafe : value, priority = (): default, used_linker = (): default, anonymous = (): default,),
    self = (unsafe),
    meta = (#[other] #[doc]),
    item = (fn foo() { /* ... */ })
));

__test!(__parse_item[my_macro_parse]:
(
    #[other]
    #[my_macro(unsafe, used(linker))]
    #[doc]
    fn foo() { /* ... */ }
) =>
(
    features = (std = std : default, unsafe = unsafe : value, priority = (): default, used_linker = used_linker : value, anonymous = (): default,),
    self = (unsafe, used(linker)),
    meta = (#[other] #[doc]),
    item = (fn foo() { /* ... */ })
));
