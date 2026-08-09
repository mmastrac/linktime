#!/bin/sh
# Runs inside vmactions BSD VMs (plain sh; BSD base systems have no bash).
# Usage: vmactions.sh provision|doctest (crok suite runs host-side, see vm-crok.sh)
set -xe

mode="$1"
os=$(uname -s)

case "$os" in
  FreeBSD) os_dir=freebsd ;;
  OpenBSD) os_dir=openbsd ;;
  NetBSD) os_dir=netbsd ;;
  DragonFly) os_dir=dragonfly ;;
  *) echo "unknown OS: $os"; exit 1 ;;
esac

if [ "$os" = "NetBSD" ]; then
  export PATH="/usr/pkg/sbin:/usr/pkg/bin:/usr/sbin:/usr/bin:${PATH}"
fi

# Under the workspace so vmactions copyback + actions/cache persist it.
export CARGO_HOME="$(pwd)/.cache/ci/vmactions/${os_dir}/cargo-home"
mkdir -p "${CARGO_HOME}/bin"
export PATH="${CARGO_HOME}/bin:${PATH}"

provision() {
  case "$os" in
    FreeBSD)
      # pkg's rustc lags stable, use rustup. Fetch the binary directly
      # (sh.rustup.rs needs curl/wget). RUSTUP_HOME stays VM-local so the
      # synced cache dir stays small.
      if ! command -v rustup >/dev/null 2>&1; then
        fetch -o /tmp/rustup-init https://static.rust-lang.org/rustup/dist/x86_64-unknown-freebsd/rustup-init
        chmod +x /tmp/rustup-init
        /tmp/rustup-init -y --profile minimal --default-toolchain none --no-modify-path
      fi
      # Installs stable into the fresh VM even when rustup came from the cache.
      rustup default stable
      ;;
    OpenBSD)
      # rustup has no OpenBSD dist; pkg_add only
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
}

# macrotest/compile-fail need cargo-expand, tier-1 only
run_doctests() {
  cargo test --doc
}

case "$mode" in
  provision) provision ;;
  doctest) run_doctests ;;
  *) echo "usage: $0 provision|doctest"; exit 1 ;;
esac
