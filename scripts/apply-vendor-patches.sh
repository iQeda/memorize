#!/usr/bin/env bash
# Apply local patches against the vendor/anki submodule.
# Run after `git submodule update --init --recursive`.
# Idempotent: skips already-applied patches.

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
SUBMODULE="$ROOT/vendor/anki"

for patch in "$ROOT"/patches/*.patch; do
  [ -e "$patch" ] || continue
  if git -C "$SUBMODULE" apply --check --reverse "$patch" >/dev/null 2>&1; then
    echo "✓ Already applied: $(basename "$patch")"
    continue
  fi
  if git -C "$SUBMODULE" apply --check "$patch" >/dev/null 2>&1; then
    git -C "$SUBMODULE" apply "$patch"
    echo "+ Applied: $(basename "$patch")"
  else
    echo "✗ Cannot apply (rejected): $(basename "$patch")" >&2
    exit 1
  fi
done
