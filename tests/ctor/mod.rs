use clitest_lib::clitest;

// Does not work on apple platforms
#[cfg(not(target_vendor = "apple"))]
clitest!(
    crt_static,
    r#"
set RUSTFLAGS "-C target-feature=+crt-static";
cd "ctor/crt-static";
defer {
    $ cargo clean --quiet
}
$ rustc -vV
%SET TARGET "$target"
*
! host: %{DATA:target}
*
$ cargo build --quiet --target $TARGET
*
$ cargo run --quiet --target $TARGET
! +crt-static
! main
"#
);

clitest!(
    doctest,
    r#"
set RUSTFLAGS "";
cd "ctor/doctest";
defer {
    $ cargo clean --quiet
}
$ cargo build --quiet
*
$ cargo test --doc --quiet
!
! running 1 test
! .
! test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in %{DATA}
!
"#
);

clitest!(
    edition_2018,
    r#"
set RUSTFLAGS "";
cd "ctor/edition-2018";
defer {
    $ cargo clean --quiet
}
$ cargo build --quiet
*
$ cargo run --quiet
! foo
! main
"#
);

clitest!(
    edition_2021,
    r#"
set RUSTFLAGS "";
cd "ctor/edition-2021";
defer {
    $ cargo clean --quiet
}
$ cargo build --quiet
*
$ cargo run --quiet
! foo
! main
"#
);

clitest!(
    edition_2024,
    r#"
set RUSTFLAGS "";
cd "ctor/edition-2024";
defer {
    $ cargo clean --quiet
}
$ cargo build --quiet
*
$ cargo run --quiet
! foo
! main
"#
);

clitest!(
    no_std,
    r#"
set RUSTFLAGS "";
cd "ctor/no-std";
defer {
    $ cargo clean --quiet
}
$ cargo build --quiet
*
"#
);

clitest!(
    priority,
    r#"
set RUSTFLAGS "";
cd "ctor/priority";
defer {
    $ cargo clean --quiet
}
$ cargo build --quiet
*
$ cargo run --quiet
unordered {
    ! no priority
    ! no priority
    ! early
    ! early
    ! 0
}
! 1
! 2
! 3
! 4
! 5
! 6
! 7
! 8
! 9
! 10
! late
! late
! main
"#
);

clitest!(
    system_no_crt_static,
    r#"
set RUSTFLAGS "-C target-feature=-crt-static";
set CARGO_TARGET_DIR "target/system_no_crt_static";
cd "ctor/system";
defer {
    $ cargo clean --quiet
}
$ cargo build --lib --examples --quiet
*
$ cargo run --example dylib_load --quiet
unordered {
    ! + Foo::ctor
    ! + ctor bin
}
! ++ main start
unordered {
    ! +++ ctor STATIC_INT
    ! +++ ctor lib (-crt-static)
}
unordered {
    ! -- main end
    ! --- dtor lib
}
unordered {
    ! - Foo::dtor
    ! - dtor bin
}
"#
);

// Only Windows supports +crt-static w/dylibs, but we may be able to work around
// this: https://github.com/rust-lang/rust/issues/71651#issuecomment-864265118
#[cfg(windows)]
clitest!(
    system_crt_static,
    r#"
set RUSTFLAGS "-C target-feature=+crt-static";
set CARGO_TARGET_DIR "target/system_crt_static";
cd "ctor/system";
defer {
    $ cargo clean --quiet
}
$ rustc -vV
%SET TARGET "$target"
*
! host: %{DATA:target}
*
$ cargo build --lib --examples --quiet
*
$ find $CARGO_TARGET_DIR
*
$ cargo run --example dylib_load --quiet
unordered {
    ! + Foo::ctor
    ! + ctor bin
}
! ++ main start
unordered {
    ! +++ ctor STATIC_INT
    ! +++ ctor lib (+crt-static)
}
unordered {
    ! -- main end
    ! --- dtor lib
}
unordered {
    ! - Foo::dtor
    ! - dtor bin
}
"#
);

clitest!(
    warn_unsafe,
    r#"
set RUSTFLAGS "";
set CARGO_TARGET_DIR "target/warn_yes";
cd "ctor/warn-unsafe";
defer {
    $ cargo clean --quiet
}
$ cargo build
%EXIT 101
ignore {
    !    Compiling %{DATA}
    !     Blocking waiting for file lock on package cache
    !     Blocking waiting for file lock on shared package cache
    !      Locking %{DATA} to latest compatible version%{DATA}
}
"""
error: Missing unsafe keyword in #[ctor] annotation. Use #[ctor(unsafe)]. This error can be suppressed by passing `--cfg linktime_no_fail_on_missing_unsafe` in `RUSTFLAGS` or placing this in your `config.toml` file.


       #[ctor]
       ^^^^^^^------- replace this with #[ctor(unsafe)]

"""
? ^\s*--> .*main\.rs:%{NUMBER}:1
"""
  |
5 | #[ctor]
  | ^^^^^^^
  |
  = note: this error originates in the macro `$crate::__ctor_parse_impl` which comes from the expansion of the attribute macro `ctor` (in Nightly builds, run with -Z macro-backtrace for more info)

error: Missing unsafe keyword in #[ctor] annotation. Use #[ctor(unsafe)]. This error can be suppressed by passing `--cfg linktime_no_fail_on_missing_unsafe` in `RUSTFLAGS` or placing this in your `config.toml` file.


       #[ctor]
       ^^^^^^^------- replace this with #[ctor(unsafe)]

"""
? ^\s*--> .*main\.rs:%{NUMBER}:1
"""
   |
22 | #[ctor]
   | ^^^^^^^
   |
   = note: this error originates in the macro `$crate::__ctor_parse_impl` which comes from the expansion of the attribute macro `ctor` (in Nightly builds, run with -Z macro-backtrace for more info)

error: could not compile `warn-unsafe` (bin "warn-unsafe") due to 2 previous errors
"""
"#
);

clitest!(
    warn_unsafe_disabled,
    r#"
set RUSTFLAGS "";
set CARGO_TARGET_DIR "target/warn_no";
cd "ctor/warn-unsafe";
defer {
    $ cargo clean --quiet
}
$ RUSTFLAGS="--cfg linktime_no_fail_on_missing_unsafe" cargo build
reject {
    ! warning: %{DATA}
}
*
!     Finished `dev` profile %{DATA}
"#
);

clitest!(
    no_default_features,
    r#"
set RUSTFLAGS "";
cd "ctor/no-default-features";
defer {
    $ cargo clean --quiet
}
$ cargo run --quiet
! ctor-no-default-features:ctor
! ctor-no-default-features:main
"#
);
