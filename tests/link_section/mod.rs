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
! LINK_SECTION: Section { name: "%{DATA}LINK_SECTION%{DATA}", start: %{BASE16NUM}, end: %{BASE16NUM}, byte_len: %{INT} }
! link_section_function
! TYPED_LINK_SECTION: TypedSection { name: "%{DATA}TYPED_LINK%{DATA}", start: %{BASE16NUM}, end: %{BASE16NUM}, len: 2, stride: 4 }
! address of TYPED_LINK_SECTION[0]: %{BASE16NUM}
! address of TYPED_LINK_SECTION[1]: %{BASE16NUM}
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
    link_section_mut,
    r#"
set RUSTFLAGS "";
cd "link_section/mut";
defer {
    $ cargo clean --quiet
}
$ cargo run --quiet
"""
item: 1
item: 2
item: 3
item: 4
item: 5
aux item: 1234
aux item: 2341
aux item: 4321
"""
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
! link-section-no-default-features:main
"#
);
