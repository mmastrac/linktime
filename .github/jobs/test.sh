#!/usr/bin/env bash
set -xeuo pipefail

. $(dirname "$0")/_init.sh

# ============================================================================
# On Windows/MSVC, link-section emits per-item COMDAT sections named
# `.data$a/$b/$c` whose attributes differ from the default `.data`, so the
# linker prints `LNK4078: multiple '.data' sections found with different
# attributes`. Rust's on-by-default `linker_messages` lint surfaces that, and
# cargo *replays* the cached warning ahead of program output — which pollutes
# the stdout the crok suites (link_section::*, dtor::link_section::*) match
# against, failing them.
#
# The real fix belongs in link-section: give those `.data$` sections matching
# attributes so MSVC stops warning. Until then we tell link.exe to ignore
# LNK4078 via its `LINK` env var.
# ============================================================================
case "${TARGET:-}" in
  *windows*) export LINK="/IGNORE:4078" ;;
esac

RUSTFLAGS='-D warnings' cargo test --no-fail-fast --target "$TARGET"
