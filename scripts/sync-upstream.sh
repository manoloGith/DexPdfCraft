#!/usr/bin/env bash
# Bring the fork (upstream/, branch dexpdfcraft/android of manoloGith/pdfcraft) up to date with
# storytold/pdfcraft: rebase the branch onto upstream's main (or a given commit), check that this
# workspace's [patch.crates-io] still mirrors upstream's, and reseed Cargo.lock from upstream's.
#
#   scripts/sync-upstream.sh            # onto the tip of storytold/pdfcraft main
#   scripts/sync-upstream.sh <commit>   # onto a given commit
#
# Afterwards: build and test, push the branch to the fork
# (git -C upstream push --force-with-lease origin dexpdfcraft/android) and commit the new
# submodule commit here.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

if [ -n "$(git -C upstream status --porcelain)" ]; then
    echo "upstream/ has uncommitted changes; commit or stash them first" >&2
    exit 1
fi
git -C upstream remote get-url storytold >/dev/null 2>&1 || git -C upstream remote add storytold https://github.com/storytold/pdfcraft
git -C upstream fetch --quiet storytold main
target="${1:-storytold/main}"
git -C upstream checkout --quiet dexpdfcraft/android
git -C upstream rebase --quiet "$target"
echo "dexpdfcraft/android is now on $(git -C upstream log -1 --format='%h %s' "$target")"

# Upstream's [patch.crates-io] entries, with path patches moved under upstream/.
extract() { sed -n '/^\[patch\.crates-io\]/,/^\[/p' "$1" | grep -E '^[a-z_-]+ *=' | sed 's/ *#.*//'; }
want="$(extract upstream/Cargo.toml | sed 's#path = "vendor/#path = "upstream/vendor/#' | sort)"
have="$(extract Cargo.toml | sort)"
if [ "$want" != "$have" ]; then
    echo "Cargo.toml's [patch.crates-io] no longer mirrors upstream's; update it:" >&2
    diff <(echo "$have") <(echo "$want") >&2 || true
    exit 1
fi

# Start from upstream's lock file so the engine builds with the versions upstream tests.
cp upstream/Cargo.lock Cargo.lock
cargo update --workspace --quiet
echo "Cargo.lock reseeded from upstream; build with: cargo check -p dexpdfcraft"
