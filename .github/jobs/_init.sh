# This actually fails on older rustc
DEFAULT_TARGET=`rustc --print host-tuple || echo invalid-tuple-specify-one-explicitly`
TARGET="${TARGET:-$DEFAULT_TARGET}"

CLIPPY_LINTS=$(cat <<EOF
    -D clippy::all
    -D deprecated-safe
    -D future-incompatible
    -D keyword-idents
    -D let-underscore
    -D nonstandard-style
    -D refining-impl-trait
    -D rust-2018-compatibility
    -D rust-2018-idioms
    -D rust-2021-compatibility
    -D rust-2024-compatibility
    -D unused
    -D unsafe_code
    -D unreachable-pub
    -D missing-docs
EOF
)
