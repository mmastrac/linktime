use crok_lib::crok;

crok!(
    no_default_features,
    r#"
set RUSTFLAGS "";
cd "dtor/no-default-features";
defer {
    $ cargo clean --quiet
}
$ cargo run --quiet
! dtor-no-default-features:main
! dtor-no-default-features:dtor
"#
);

// Run link-section with and without static CRT
crok!(
    link_section_no_crt_static,
    r#"
set RUSTFLAGS "-C target-feature=-crt-static";
set CARGO_TARGET_DIR "target/link-section-no-crt-static";
cd "dtor/link-section";
defer {
    $ cargo clean --quiet
}
$ rustc -vV
%SET TARGET "$target"
*
! host: %{DATA:target}
*
$ cargo run --quiet --target $TARGET
! dtor-link-section:main
if TARGET_OS == "linux" {
    ! dtor-link-section:dtor
}
if TARGET_OS == "freebsd" {
    ! dtor-link-section:dtor
}
if TARGET_OS == "openbsd" {
    ! dtor-link-section:dtor
}
"#
);

crok!(
    link_section_crt_static,
    r#"
set RUSTFLAGS "-C target-feature=+crt-static";
set CARGO_TARGET_DIR "target/link-section-crt-static";
cd "dtor/link-section";
defer {
    $ cargo clean --quiet
}
$ rustc -vV
%SET TARGET "$target"
*
! host: %{DATA:target}
*
$ cargo run --quiet --target $TARGET
! dtor-link-section:main
if TARGET_OS == "linux" {
    ! dtor-link-section:dtor
}
if TARGET_OS == "freebsd" {
    ! dtor-link-section:dtor
}
if TARGET_OS == "windows" {
    ! dtor-link-section:dtor
}
if TARGET_OS == "openbsd" {
    ! dtor-link-section:dtor
}
"#
);
