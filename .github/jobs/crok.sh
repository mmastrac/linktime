#!/usr/bin/env bash
# Runs the crok suite for a sandbox via tunnel
set -xeuo pipefail

sandbox="${SANDBOX:?}"
root=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)

# bsan's container is linux.
case "$sandbox" in
  bsan) target_os=linux ;;
  *) target_os="$sandbox" ;;
esac

find tests -name '*.crok' -not -name '_*' -not -path '*/target/*' -not -path '*/.*' -print0 | sort -z |
  xargs -0 crok --timeout 300 --target-os "$target_os" \
    --runner "sh $root/.github/jobs/sandbox.sh exec $sandbox"

# TODO: doctests shouldn't live here
case "$sandbox" in
  freebsd|openbsd) "$root/.github/jobs/sandbox.sh" exec "$sandbox" 'cargo test --doc' ;;
  bsan) "$root/.github/jobs/bsan.sh" extra ;;
esac
