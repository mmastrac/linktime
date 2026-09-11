#!/usr/bin/env bash
set -xeuo pipefail

# BorrowSanitizer (https://borrowsanitizer.com/). Unlike miri, which interprets
# MIR, bsan instruments real machine code, so the link-section/ctor tests that
# miri has to skip do run here.
#
# `cargo bsan` picks the host target itself and forwards a second `--target` if
# we pass one, so none of the commands below specify a target.

# Build the instrumented sysroot up front. It would otherwise happen lazily
# inside the first command, mixing a libstd build into that command's output.
cargo bsan setup

preinit_dir=$(mktemp -d)
cc -c -fPIC -x c -o "$preinit_dir/preinit.o" - <<'EOF'
extern void __bsan_init(void);
__attribute__((section(".preinit_array"),
               used)) static void (*bsan_preinit)(void) = __bsan_init;
EOF

host_triple=$(rustc -vV | sed -n 's/^host: //p')
export "CARGO_TARGET_$(echo "$host_triple" | tr 'a-z-' 'A-Z_')_RUSTFLAGS=-C link-arg=$preinit_dir/preinit.o"
export RUSTDOCFLAGS="-C link-arg=$preinit_dir/preinit.o"

cargo bsan test

bsan_examples=(
  ctor-advanced
  ctor-basic
  ctor-dynamic
  ctor-example
  ctor-priority
  ctor-statics
  dtor-example
  link-section-const
  link-section-dyn
  link-section-empty
  link-section-example
  link-section-movable
  link-section-movable-no-macro
  link-section-mut
  link-section-mut-no-macro
  link-section-ref
  scattered-collect-command-registration
  scattered-collect-intern-strings
  scattered-collect-iterable
  scattered-collect-map
  scattered-collect-referenced-slice
  scattered-collect-set
  scattered-collect-slice
  scattered-collect-sorted-referenced-slice
  scattered-collect-sorted-slice
)
for example in "${bsan_examples[@]}"; do
  cargo bsan run --example "$example"
done

# Crates outside the workspace.
bsan_crates=(
  tests/ctor/edition-2018
  tests/ctor/edition-2021
  tests/ctor/edition-2024
  tests/ctor/no-default-features
  tests/ctor/priority
  tests/dtor/link-section
  tests/dtor/no-default-features
  tests/link_section/basic
  tests/link_section/copied
  tests/link_section/interior_mut
  tests/link_section/mutable
  tests/link_section/no-default-features
)
for dir in "${bsan_crates[@]}"; do
  (cd "$dir" && cargo bsan run)
done
