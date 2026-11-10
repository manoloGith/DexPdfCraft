#!/usr/bin/env bash
# Move the `upstream/` submodule (storytold/pdfcraft) to another commit and check that this
# workspace's `[patch.crates-io]` still mirrors upstream's.
#
#   scripts/sync-upstream.sh            # the tip of upstream's main
#   scripts/sync-upstream.sh <commit>   # a given commit
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
cd "$root"

target="${1:-origin/main}"
git -C upstream fetch --quiet origin
git -C upstream checkout --quiet "$target"
echo "upstream is now at $(git -C upstream log -1 --format='%h %s')"

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
