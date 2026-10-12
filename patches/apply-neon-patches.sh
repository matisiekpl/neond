#!/usr/bin/env sh
# Applies patches/neon/*.patch to the neon submodule, in file name order. Idempotent: a stack that is
# already applied is skipped, and a patch that neither applies nor is already applied fails the build
# (e.g. after bumping the neon submodule), so it never gets silently dropped.
#
# Several patches may touch the same file, so once one of them is applied the earlier ones no longer
# reverse-apply cleanly. The applied stack is therefore recorded in a marker file inside neon/.
set -eu

patches_dir="$(cd "$(dirname "$0")/neon" && pwd)"
neon_dir="${1:-$(dirname "$0")/../neon}"
cd "$neon_dir"

marker=.neond-patches-applied
stack="$(cat "$patches_dir"/*.patch 2>/dev/null | sha256sum | cut -d' ' -f1)"
if [ -f "$marker" ] && [ "$(cat "$marker")" = "$stack" ]; then
  echo "neon patches already applied"
  exit 0
fi

for patch_file in "$patches_dir"/*.patch; do
  [ -e "$patch_file" ] || continue
  name="$(basename "$patch_file")"
  if patch -p1 -R --dry-run --force --silent < "$patch_file" >/dev/null 2>&1; then
    echo "neon patch already applied: $name"
  elif patch -p1 --forward --dry-run --batch --silent < "$patch_file" >/dev/null 2>&1; then
    patch -p1 --forward --batch --silent < "$patch_file"
    echo "applied neon patch: $name"
  else
    echo "neon patch does not apply: $name" >&2
    exit 1
  fi
done
echo "$stack" > "$marker"
