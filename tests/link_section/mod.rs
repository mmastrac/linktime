use clitest_lib::clitest;

clitest!(
    basic,
    r#"
set RUSTFLAGS "";
cd "link_section/basic";
defer {
    $ cargo clean --quiet
}
$ cargo build --quiet
*
$ cargo run --quiet
! LINK_SECTION: Section { name: "%{DATA}LINK_SEC%{DATA}", start: %{BASE16NUM}, end: %{BASE16NUM}, byte_len: %{INT} }
! link_section_function
! TYPED_LINK_SECTION: TypedSection { name: "%{DATA}TYPED_LI%{DATA}", start: %{BASE16NUM}, end: %{BASE16NUM}, len: 2, stride: 4 }
! address of TYPED_LINK_SECTION[0]: %{BASE16NUM}
! address of TYPED_LINK_SECTION[1]: %{BASE16NUM}
! AUX_LINK_SECTION: TypedSection { name: "%{DATA}TYPED_LI%{DATA}", start: %{BASE16NUM}, end: %{BASE16NUM}, len: 1, stride: 4 }
! aux: 1234
! CODE_SECTION: TypedSection { name: "%{DATA}FN_ARRAY%{DATA}", start: %{BASE16NUM}, end: %{BASE16NUM}, len: 3, stride: 8 }
! [%{BASE16NUM}, %{BASE16NUM}, %{BASE16NUM}]
unordered {
    ! f: %{BASE16NUM}
    ! link_section_function
    ! f: %{BASE16NUM}
    ! linked_function
    ! f: %{BASE16NUM}
    ! linked_function_2
}
choice {
    ! DEBUGGABLES: [1, 2, debuggable_function]
    ! DEBUGGABLES: [debuggable_function, 2, 1]
}
"#
);

clitest!(
    copied,
    r#"
set RUSTFLAGS "";
cd "link_section/copied";
defer {
    $ cargo clean --quiet
}
$ cargo run --quiet
"""
MUTABLE: [ComplexType { static_string: "1", static_ptr: OtherType { u32: 1, u64: 2 } }, ComplexType { static_string: "2", static_ptr: OtherType { u32: 1, u64: 2 } }, ComplexType { static_string: "3", static_ptr: OtherType { u32: 1, u64: 2 } }, ComplexType { static_string: "4", static_ptr: OtherType { u32: 3, u64: 4 } }, ComplexType { static_string: "5", static_ptr: OtherType { u32: 1, u64: 2 } }]
IMMUTABLE: [ComplexType { static_string: "1", static_ptr: OtherType { u32: 1, u64: 2 } }, ComplexType {static_string: "4", static_ptr: OtherType { u32: 1, u64: 2 } }, ComplexType { static_string: "9", static_ptr: OtherType { u32: 3, u64: 4 } }]
"""
"#
);

clitest!(
    interior_mut,
    r#"
set RUSTFLAGS "";
cd "link_section/interior_mut";
defer {
    $ cargo clean --quiet
}
$ cargo run --quiet
! INTERIOR_MUT_LINK_SECTION: TypedSection { name: "%{DATA}INTERIOR%{DATA}", start: %{BASE16NUM}, end: %{BASE16NUM}, len: 2, stride: 8 }
unordered {
    ! item before: InteriorMutItem { value: 1, atomic: 1 }
    ! item after: InteriorMutItem { value: 1, atomic: 2 }
    ! item before: InteriorMutItem { value: 2, atomic: 2 }
    ! item after: InteriorMutItem { value: 2, atomic: 3 }
}
"#
);

clitest!(
    link_section_mut,
    r#"
set RUSTFLAGS "";
cd "link_section/mut";
defer {
    $ cargo clean --quiet
}
$ cargo run --quiet
! MUT_LINK_SECTION: TypedMutableSection { name: "%{DATA}MUT_LINK%{DATA}", start: %{BASE16NUM}, end: %{BASE16NUM}, len: 5, stride: 4 }
"""
item: 1
item: 2
item: 3
item: 4
item: 5
"""
! AUX_MUT_LINK_SECTION: TypedMutableSection { name: "%{DATA}MUT_LINK%{DATA}", start: %{BASE16NUM}, end: %{BASE16NUM}, len: 3, stride: 4 }
"""
aux item: 1234
aux item: 2341
aux item: 4321
"""
! MOVABLE_LINK_SECTION: TypedMovableSection { name: "%{DATA}MOVABLE%{DATA}", start: %{BASE16NUM}, end: %{BASE16NUM}, len: 4, stride: 4 }
! MOVABLE_BACKREFS: 4
"""
movable item: 10
movable item: 20
movable item: 30
movable item: 40
"""
! MOVABLE_40: 40
! MOVABLE_20: 20
! MOVABLE_10: 10
! MOVABLE_30: 30
"#
);

clitest!(
    no_default_features,
    r#"
set RUSTFLAGS "";
cd "link_section/no-default-features";
defer {
    $ cargo clean --quiet
}
$ cargo run --quiet
! link-section-no-default-features:in-section
! link-section-no-default-features:in-section-aux
! link-section-no-default-features:main
"#
);
