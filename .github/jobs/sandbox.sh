#!/bin/sh
# Runs commands inside a sandbox.
#
#   sandbox.sh start <sandbox>          bring it up (before the cache restore)
#   sandbox.sh provision <sandbox>      install toolchains (after it)
#   sandbox.sh up <sandbox>             start + provision, unless already up
#   sandbox.sh exec <sandbox> <command> run one command in it
#
# Commands are piped into sh on stdin.
set -eu

BSAN_IMAGE=ghcr.io/borrowsanitizer/bsan:latest
BSAN_CONTAINER=rust-ctor-bsan
BSAN_BIN=/root/.cargo/bin

mode="${1:?usage: sandbox.sh start|provision|exec <sandbox> [command]}"
sandbox="${2:?usage: sandbox.sh start|provision|exec <sandbox> [command]}"

root=$(cd "$(dirname "$0")/../.." && pwd)
# vmactions copyback path
cache="$root/.cache/ci/sandbox/$sandbox"

ssh_host() {
  case "$sandbox" in
    dragonfly) echo dragonflybsd ;;
    *) echo "$sandbox" ;;
  esac
}

vm_preamble() {
  # NetBSD's pkgsrc binaries are not in PATH.
  case "$sandbox" in
    netbsd) prefix="/usr/pkg/sbin:/usr/pkg/bin:" ;;
    *) prefix="" ;;
  esac
  printf '%s\n' \
    "cd '$PWD' || exit 1" \
    "export CARGO_HOME='$cache/cargo-home'" \
    "export PATH='$prefix$cache/cargo-home/bin':\"\$PATH\""
}

# Move HOME to cached tree.
bsan_preamble() {
  printf '%s\n' \
    "cd '$PWD' || exit 1" \
    "mkdir -p '$cache/home' || exit 1" \
    "export HOME='$cache/home'" \
    "if [ -r /opt/bsan/env.sh ]; then . /opt/bsan/env.sh; fi"
}

sandbox_exec() {
  case "$sandbox" in
    bsan)
      { bsan_preamble; printf '%s\n' "$1"; } | docker exec -i "$BSAN_CONTAINER" sh
      ;;
    *)
      # LogLevel=ERROR to hide ssh's `Permanently added ... `
      { vm_preamble; printf '%s\n' "$1"; } | ssh -T \
        -o LogLevel=ERROR \
        -o ControlMaster=auto -o ControlPath=/tmp/crok-vm-%h -o ControlPersist=60 \
        "$(ssh_host)" sh
      ;;
  esac
}

start() {
  [ "$sandbox" = bsan ] || return 0

  docker rm -f "$BSAN_CONTAINER" >/dev/null 2>&1 || true
  docker run -d --name "$BSAN_CONTAINER" \
    --volume "$root:$root" --workdir "$root" \
    --entrypoint sh "$BSAN_IMAGE" -c 'while :; do sleep 3600; done' >/dev/null

  [ -n "${GITHUB_OUTPUT:-}" ] || return 0
  key=$(docker exec "$BSAN_CONTAINER" sh -c \
    "$BSAN_BIN/rustc -vV; $BSAN_BIN/cargo bsan --version" |
    sha256sum | cut -c1-16)
  echo "toolchain=$key" >> "$GITHUB_OUTPUT"
}

provision() {
  [ "$sandbox" = bsan ] || return 0
  sandbox_exec 'sh .github/jobs/bsan.sh provision'
}

up() {
  [ "$sandbox" = bsan ] || return 0
  docker exec "$BSAN_CONTAINER" true >/dev/null 2>&1 && return 0
  start
  provision
}

case "$mode" in
  start) start ;;
  provision) provision ;;
  up) up ;;
  exec) sandbox_exec "${3:?usage: sandbox.sh exec <sandbox> <command>}" ;;
  *) echo "usage: $0 start|provision|up|exec <sandbox> [command]" >&2; exit 1 ;;
esac
