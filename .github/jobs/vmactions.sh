#!/bin/sh
# Provisions a vmactions BSD VM.
set -xe

os=$(uname -s)

case "$os" in
  FreeBSD) sandbox=freebsd ;;
  OpenBSD) sandbox=openbsd ;;
  NetBSD) sandbox=netbsd ;;
  DragonFly) sandbox=dragonfly ;;
  *) echo "unknown OS: $os"; exit 1 ;;
esac

if [ "$os" = "NetBSD" ]; then
  export PATH="/usr/pkg/sbin:/usr/pkg/bin:/usr/sbin:/usr/bin:${PATH}"
fi

# Under the CI workspace
export CARGO_HOME="$(pwd)/.cache/ci/sandbox/${sandbox}/cargo-home"
mkdir -p "${CARGO_HOME}/bin"
export PATH="${CARGO_HOME}/bin:${PATH}"

case "$os" in
  FreeBSD)
    # pkg's rustc lags stable, use rustup.
    if ! command -v rustup >/dev/null 2>&1; then
      fetch -o /tmp/rustup-init https://static.rust-lang.org/rustup/dist/x86_64-unknown-freebsd/rustup-init
      chmod +x /tmp/rustup-init
      /tmp/rustup-init -y --profile minimal --default-toolchain none --no-modify-path
    fi
    rustup default stable
    ;;
  OpenBSD)
    # No rustup in OpenBSD dist, use pkg_add
    if ! command -v cargo >/dev/null 2>&1; then
      pkg_add rust
    fi
    ;;
  NetBSD)
    if [ -x /usr/pkg/bin/pkgin ]; then
      /usr/pkg/bin/pkgin update
      /usr/pkg/bin/pkgin -y install rust
    else
      export PKG_PATH="https://cdn.NetBSD.org/pub/pkgsrc/packages/NetBSD/amd64/10.1/All"
      /usr/sbin/pkg_add rust
    fi
    ;;
  DragonFly)
    pkg install -y rust
    ;;
esac

cargo fetch
