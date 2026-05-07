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

crates_io_get() {
  # Usage: crates_io_get <url> <out_body_file> <out_headers_file> <out_stderr_file>
  # Prints HTTP status code to stdout (000 for transport error).
  local url="$1"
  local out_body="$2"
  local out_headers="$3"
  local out_stderr="$4"

  # -L: follow redirects (crates.io download endpoint returns 302 to static.crates.io)
  curl -sS -L -D "$out_headers" -o "$out_body" -w "%{http_code}" "$url" 2>"$out_stderr" || echo "000"
}

while IFS=$'\t' read -r name version; do
  [ -n "${name:-}" ] || continue

  echo "::group::${name}@${version}"

  # If this exact version is not published, we consider it "bumped" and OK.
  meta_body="$TMP_DIR/${name}-${version}.meta.body"
  meta_headers="$TMP_DIR/${name}-${version}.meta.headers"
  meta_stderr="$TMP_DIR/${name}-${version}.meta.stderr"
  meta_url="https://crates.io/api/v1/crates/${name}/${version}"

  http_code=""
  for attempt in 1 2 3; do
    http_code="$(crates_io_get "$meta_url" "$meta_body" "$meta_headers" "$meta_stderr")"
    if [ "$http_code" != "000" ] && [ "$http_code" -lt 500 ]; then
      break
    fi
    sleep 1
  done

  if [ "$http_code" = "404" ]; then
    echo "✅ ${name}@${version}: not published yet (version appears bumped)"
    echo "::endgroup::"
    continue
  fi

  if [ "$http_code" = "000" ] || [ "$http_code" -ge 500 ] || [ "$http_code" -lt 200 ] || [ "$http_code" -ge 300 ]; then
    echo "❌ ${name}@${version}: failed to query crates.io (${http_code})"
    echo
    echo "URL: $meta_url"
    echo
    echo "--- curl stderr ---"
    cat "$meta_stderr" || true
    echo "--- headers ---"
    cat "$meta_headers" || true
    echo "--- body (first 200 lines) ---"
    head -n 200 "$meta_body" || true
    FAILED=1
    echo "::endgroup::"
    continue
  fi

  echo "ℹ️  ${name}@${version}: published; verifying local package matches"

  PUBLISHED_CRATE="$TMP_DIR/${name}-${version}-published.crate"
  LOCAL_CRATE="$ROOT/target/package/${name}-${version}.crate"
  PUBLISHED_DIR="$TMP_DIR/${name}-${version}/published"
  LOCAL_DIR="$TMP_DIR/${name}-${version}/local"

  mkdir -p "$PUBLISHED_DIR" "$LOCAL_DIR"

  download_body="$TMP_DIR/${name}-${version}.download.body"
  download_headers="$TMP_DIR/${name}-${version}.download.headers"
  download_stderr="$TMP_DIR/${name}-${version}.download.stderr"
  download_url="https://crates.io/api/v1/crates/${name}/${version}/download"

  dl_code=""
  for attempt in 1 2 3; do
    dl_code="$(crates_io_get "$download_url" "$download_body" "$download_headers" "$download_stderr")"
    if [ "$dl_code" != "000" ] && [ "$dl_code" -lt 500 ]; then
      break
    fi
    sleep 1
  done

  if [ "$dl_code" != "200" ]; then
    echo "❌ ${name}@${version}: failed to download published crate (${dl_code})"
    echo
    echo "URL: $download_url"
    echo
    echo "--- curl stderr ---"
    cat "$download_stderr" || true
    echo "--- headers ---"
    cat "$download_headers" || true
    echo "--- body (first 50 lines) ---"
    head -n 50 "$download_body" || true
    FAILED=1
    echo "::endgroup::"
    continue
  fi

  mv "$download_body" "$PUBLISHED_CRATE"
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
