#!/usr/bin/env bash
# Fetch PdfCraft's optional build inputs into .build-inputs/ (gitignored), at the revisions
# upstream pins, and print the environment variables that make the build embed them:
#
#   craft-i18n  the interface translations (upstream/craft-i18n.rev); without it, English only
#
#   eval "$(scripts/fetch-build-inputs.sh)"
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
dir="$root/.build-inputs/craft-i18n"
rev="$(tr -d '[:space:]' < "$root/upstream/craft-i18n.rev")"
if [ "$(git -C "$dir" rev-parse HEAD 2>/dev/null || true)" != "$rev" ]; then
    rm -rf "$dir"
    git init --quiet "$dir"
    git -C "$dir" fetch --quiet --depth 1 https://github.com/storytold/craft-i18n "$rev"
    git -C "$dir" checkout --quiet FETCH_HEAD
fi
echo "export CRAFT_I18N_DIR=\"$dir\""
