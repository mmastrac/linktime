#!/usr/bin/env bash
# Run the crok suite on the host, driving commands into a vmactions VM.
# Usage: vm-crok.sh <ssh-host> <target-os>  (e.g. dragonflybsd dragonfly)
set -xeuo pipefail

host="$1"
target_os="$2"

find tests -name '*.crok' -not -path '*/target/*' -not -path '*/.*' -print0 | sort -z |
  xargs -0 crok --timeout 300 --target-os "$target_os" \
    --runner "sh $(pwd)/.github/jobs/vm-run.sh $host"
