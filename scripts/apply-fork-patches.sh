#!/usr/bin/env bash
# Rebuild the fork's `dexpdfcraft/android` branch in `upstream/`: the upstream commit the
# superproject pins, plus the patches in patches/upstream/ (git am, in order).
#
# Until the fork exists on GitHub (see docs/fork.md), this is how a fresh clone, CI and Gradle get
# PdfCraft with the Android changes. Once the submodule points at the fork, it is no longer needed.
#
#   scripts/apply-fork-patches.sh           # refuses if upstream/ has uncommitted changes
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

base="$(git ls-tree HEAD upstream | awk '{print $3}')"
if [ -z "$base" ]; then
    echo "upstream/ is not a submodule of this checkout" >&2
    exit 1
fi
if [ -n "$(git -C upstream status --porcelain)" ]; then
    echo "upstream/ has uncommitted changes; commit them on dexpdfcraft/android and export them" >&2
    echo "with scripts/export-fork-patches.sh first" >&2
    exit 1
fi
git -C upstream checkout --quiet -B dexpdfcraft/android "$base"
shopt -s nullglob
patches=("$root"/patches/upstream/*.patch)
if [ ${#patches[@]} -gt 0 ]; then
    git -C upstream -c user.name="DexPdfCraft" -c user.email="dexpdfcraft@users.noreply.github.com" \
        am --quiet --committer-date-is-author-date "${patches[@]}"
fi
echo "upstream/ is at $(git -C upstream log -1 --format='%h %s') (${#patches[@]} patch(es) on ${base:0:9})"
