#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

if ! command -v curl >/dev/null 2>&1; then
  echo "ERROR: curl is required"
  exit 1
fi
if ! command -v jq >/dev/null 2>&1; then
  echo "ERROR: jq is required"
  exit 1
fi
if ! command -v tar >/dev/null 2>&1; then
  echo "ERROR: tar is required"
  exit 1
fi
if ! command -v diff >/dev/null 2>&1; then
  echo "ERROR: diff is required"
  exit 1
fi

TMP_DIR="$(mktemp -d)"
cleanup() { rm -rf "$TMP_DIR"; }
trap cleanup EXIT

META_JSON="$TMP_DIR/metadata.json"
cargo metadata --no-deps --format-version 1 > "$META_JSON"

CRATES_TSV="$TMP_DIR/crates.tsv"

# workspace publishable crates (publish != false), by manifest_path prefix
jq -r --arg root "$ROOT/" '
  def publishable:
    (.publish == null) or ((.publish | type) == "array" and (.publish | length) > 0);
  .packages
  | map(select(publishable))
  | map(select(.manifest_path | startswith($root)))
  | sort_by(.name)
  | .[]
  | "\(.name)\t\(.version)"
' "$META_JSON" >"$CRATES_TSV"

if [ ! -s "$CRATES_TSV" ]; then
  echo "No publishable workspace crates found."
  exit 0
fi

echo "Checking publication state for workspace crates:"
while IFS=$'\t' read -r name version; do
  [ -n "${name:-}" ] || continue
  echo "  - ${name} ${version}"
done <"$CRATES_TSV"

echo
FAILED=0

while IFS=$'\t' read -r name version; do
  [ -n "${name:-}" ] || continue

  echo "::group::${name}@${version}"

  # If this exact version is not published, we consider it "bumped" and OK.
  if ! curl -fsS "https://crates.io/api/v1/crates/${name}/${version}" >/dev/null 2>&1; then
    echo "✅ ${name}@${version}: not published yet (version appears bumped)"
    echo "::endgroup::"
    continue
  fi

  echo "ℹ️  ${name}@${version}: published; verifying local package matches"

  PUBLISHED_CRATE="$TMP_DIR/${name}-${version}-published.crate"
  LOCAL_CRATE="$ROOT/target/package/${name}-${version}.crate"
  PUBLISHED_DIR="$TMP_DIR/${name}-${version}/published"
  LOCAL_DIR="$TMP_DIR/${name}-${version}/local"

  mkdir -p "$PUBLISHED_DIR" "$LOCAL_DIR"

  curl -fsSL -o "$PUBLISHED_CRATE" "https://crates.io/api/v1/crates/${name}/${version}/download"
  tar --strip-components=1 -xzf "$PUBLISHED_CRATE" -C "$PUBLISHED_DIR"

  # Build the crate tarball Cargo would publish.
  if ! cargo package -p "$name" --allow-dirty --no-verify >/dev/null; then
    echo "❌ ${name}@${version}: failed to run \`cargo package\` for this crate"
    echo
    echo "This usually means the crate is not currently publishable (e.g. a dependency version"
    echo "constraint points at a version that doesn't exist on crates.io yet)."
    echo
    echo "Fix the dependency versions (or publish dependency crates) and re-run CI."
    FAILED=1
    echo "::endgroup::"
    continue
  fi

  if [ ! -f "$LOCAL_CRATE" ]; then
    echo "ERROR: expected local package at: $LOCAL_CRATE"
    FAILED=1
    echo "::endgroup::"
    continue
  fi

  tar --strip-components=1 -xzf "$LOCAL_CRATE" -C "$LOCAL_DIR"

  DIFF_FILE="$TMP_DIR/${name}-${version}.diff"
  if diff -ru \
    --exclude=".cargo_vcs_info.json" \
    --exclude="Cargo.lock" \
    "$PUBLISHED_DIR" "$LOCAL_DIR" >"$DIFF_FILE"; then
    echo "✅ ${name}@${version}: matches published contents"
  else
    echo "❌ ${name}@${version}: local crate differs from published crate"
    echo
    echo "This usually means you changed published files without bumping the version."
    echo "Bump ${name}'s version, then re-run CI."
    echo
    echo "Diff (first 200 lines):"
    head -n 200 "$DIFF_FILE" || true
    FAILED=1
  fi

  echo "::endgroup::"
done <"$CRATES_TSV"

if [ "$FAILED" -ne 0 ]; then
  echo
  echo "ERROR: Publication state check failed."
  exit 1
fi

echo
echo "Publication state check passed."
