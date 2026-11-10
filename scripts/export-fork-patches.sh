#!/usr/bin/env bash
# Write the commits of upstream/'s `dexpdfcraft/android` branch (on top of the upstream commit the
# superproject pins) to patches/upstream/, replacing what is there.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"
base="$(git ls-tree HEAD upstream | awk '{print $3}')"
rm -f patches/upstream/*.patch
mkdir -p patches/upstream
git -C upstream format-patch --quiet --zero-commit --no-signature "$base"..dexpdfcraft/android -o "$root/patches/upstream"
ls patches/upstream
