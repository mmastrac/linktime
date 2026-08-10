#!/usr/bin/env bash
set -xeuo pipefail

. $(dirname "$0")/_init.sh

# --nocapture surfaces the guest's serial output in the CI log.
cargo test -p tests -- uefi:: --nocapture

echo "Done."
