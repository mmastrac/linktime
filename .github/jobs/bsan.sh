#!/bin/sh
# BorrowSanitizer (https://borrowsanitizer.com/).
#
# Usage: bsan.sh provision   inside the container, via sandbox.sh
#        bsan.sh extra       on the host, driving commands back in
set -xe

TOOLS=/opt/bsan
CARGO=/root/.cargo/bin/cargo

# NOTE: Once upstream issue is fixed we don't need preinit.o

provision() {
  export PATH="/root/.cargo/bin:$PATH"
  mkdir -p "$TOOLS/bin"

  # __bsan_init has to run before every constructor under test, so it goes in
  # .preinit_array.
  cc -c -fPIC -x c -o "$TOOLS/preinit.o" - <<'PREINIT'
extern void __bsan_init(void);
__attribute__((section(".preinit_array"),
               used)) static void (*bsan_preinit)(void) = __bsan_init;
PREINIT

  cat > "$TOOLS/env.sh" <<ENV
export BSAN=1
export CARGO_HOME="\$HOME/.cargo"
export RUSTUP_HOME=/root/.rustup
export PATH="$TOOLS/bin:/root/.cargo/bin:\$PATH"
export SANDBOX_RUSTFLAGS="-C link-arg=$TOOLS/preinit.o"
export RUSTDOCFLAGS="-C link-arg=$TOOLS/preinit.o"
ENV

  cat > "$TOOLS/bin/cargo" <<SHIM
#!/bin/sh
set -eu
case "\${1:-}" in
  run|test) exec $CARGO bsan "\$@" ;;
esac
exec $CARGO "\$@"
SHIM
  chmod +x "$TOOLS/bin/cargo"

  . "$TOOLS/env.sh"

  # Build the instrumented libstd
  cargo bsan setup
}

extra() {
  root=$(cd "$(dirname "$0")/../.." && pwd)
  "$root/.github/jobs/sandbox.sh" exec bsan 'RUSTFLAGS="$SANDBOX_RUSTFLAGS" cargo test'

  examples=$(cargo metadata --no-deps --format-version 1 |
    jq -r '.packages[].targets[] | select(.kind[] == "example") | .name')
  for example in $examples; do
    "$root/.github/jobs/sandbox.sh" exec bsan \
      "RUSTFLAGS=\"\$SANDBOX_RUSTFLAGS\" cargo run --example $example"
  done
}

case "${1:-}" in
  provision) provision ;;
  extra) extra ;;
  *) echo "usage: $0 provision|extra" >&2; exit 1 ;;
esac
