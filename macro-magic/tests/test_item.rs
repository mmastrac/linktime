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
    #[my_macro]
    fn foo() { /* ... */ }
) =>
(
    features = (std = std : default, unsafe = (): default, priority = (): default, used_linker = (): default, anonymous = (): default,),
    self = (my_macro),
    meta = (),
    item = (fn foo() { /* ... */ })
));

__test!(__parse_item[my_macro_parse]:
(
    #[my_macro]
    pub fn foo() { /* ... */ }
) =>
(
    features = (std = std : default, unsafe = (): default, priority = (): default, used_linker = (): default, anonymous = (): default,),
    self = (my_macro),
    meta = (),
    item = (pub fn foo() { /* ... */ })
));

__test!(__parse_item[my_macro_parse]:
(
    #[my_macro(unsafe, priority = 1)]
    fn foo() { /* ... */ }
) =>
(
    features = (std = std : default, unsafe = unsafe : value, priority = 1 : value, used_linker = (): default, anonymous = (): default,),
    self = (my_macro (unsafe, priority = 1)),
    meta = (),
    item = (fn foo() { /* ... */ })
));

__test!(__parse_item[my_macro_parse]:
(
    #[my_macro(unsafe)]
    #[doc = ""]
    fn foo() { /* ... */ }
) =>
(
    features = (std = std : default, unsafe = unsafe : value, priority = (): default, used_linker = (): default, anonymous = (): default,),
    self = (my_macro (unsafe)),
    meta = (#[doc = ""]),
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
    self = (my_macro(unsafe)),
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
    self = (my_macro (unsafe, used(linker))),
    meta = (#[other] #[doc]),
    item = (fn foo() { /* ... */ })
));

__declare_features!(
    section: section_type_parse;
    other {
        attr: [(other = $value:tt) => ($value)];
        example: "other = N";
        validate: [($numeric:literal)];
    };
    /// One of `untyped`, `typed`, `reference`, or `movable` (same choices as
    /// `#[section(...)]`).
    section_type {
        attr: [
            ($(untyped)? $(typed)? $(reference)? $(movable)?) => ($section_type)
        ];
        example: "untyped | typed | reference | movable";
        validate: [(untyped), (typed), (reference), (movable)];
    };
);

__test!(__parse_item[section_type_parse]:
(
    #[section(typed)]
    fn foo() { /* ... */ }
) =>
(
    features = (other = (): default, section_type = typed : value,),
    self = (section (typed)),
    meta = (),
    item = (fn foo() { /* ... */ })
));

__test!(__parse_item[section_type_parse]:
(
    #[section(typed, other = 1)]
    fn foo() { /* ... */ }
) =>
(
    features = (other = 1: value, section_type = typed : value,),
    self = (section (typed, other = 1)),
    meta = (),
    item = (fn foo() { /* ... */ })
));

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
