#!/usr/bin/env bash
set -xeuo pipefail

. $(dirname "$0")/_init.sh

cargo test -p tests -- uefi::

echo "Done."
