#!/bin/sh
# crok --runner wrapper: runs one command in a vmactions VM
# Usage: vm-run.sh <ssh-host> <command>
# Piped into sh on stdin (login shell may be csh). Host paths map verbatim.
# Script-set env vars arrive inlined in the command (crok 0.9.0).
set -eu

host="$1"
case "$host" in
  dragonflybsd) os_dir=dragonfly ;;
  *) os_dir="$host" ;;
esac

cargo_home="${GITHUB_WORKSPACE}/.cache/ci/vmactions/${os_dir}/cargo-home"
path_prefix=""
if [ "$host" = "netbsd" ]; then
  path_prefix="/usr/pkg/sbin:/usr/pkg/bin:"
fi

printf '%s\n' \
  "cd '$PWD' || exit 1" \
  "export CARGO_HOME='$cargo_home'" \
  "export PATH='${path_prefix}${cargo_home}/bin':\"\$PATH\"" \
  "$2" \
  | ssh -T \
      -o ControlMaster=auto -o ControlPath=/tmp/crok-vm-%h -o ControlPersist=60 \
      "$host" sh
