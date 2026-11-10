#!/usr/bin/env bash
# Fetch PdfCraft's optional build inputs into .build-inputs/ (gitignored), at the revisions
# upstream pins, and print the environment variables that make the build embed them:
#
#   craft-i18n  the interface translations (upstream/craft-i18n.rev); without it, English only
#   models      the text-recognition (OCR) models; without them, Scan & OCR says they aren't
#               installed. Upstream's ONNX pair from ocrs-models (SHA-256 pinned in
#               crates/ocr/src/ocrs.rs), else the older .rten pair PdfCraft also reads, from
#               PdfCraft for Android at a pinned commit. CC-BY-SA-4.0, Robert Knight.
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

models="$root/.build-inputs/models"
mkdir -p "$models"
sha() { sha256sum "$1" | cut -d' ' -f1; }
# fetch URL FILE SHA256: download unless already there with that hash; fail on a mismatch.
fetch() {
    local url="$1" file="$models/$2" want="$3"
    [ -f "$file" ] && [ "$(sha "$file")" = "$want" ] && return 0
    curl -fsSL --retry 2 -o "$file.part" "$url" 2>/dev/null || { rm -f "$file.part"; return 1; }
    if [ "$(sha "$file.part")" != "$want" ]; then
        echo "$2: unexpected SHA-256; not used" >&2
        rm -f "$file.part"
        return 1
    fi
    mv "$file.part" "$file"
}
ocrs="$root/upstream/crates/ocr/src/ocrs.rs"
det_sha="$(sed -n 's/.*DETECTION_MODEL_SHA256: &str = "\([0-9a-f]*\)".*/\1/p' "$ocrs" | head -1)"
rec_sha="$(sed -n 's/.*RECOGNITION_MODEL_SHA256: &str = "\([0-9a-f]*\)".*/\1/p' "$ocrs" | head -1)"
s3="https://ocrs-models.s3-accelerate.amazonaws.com"
android="https://raw.githubusercontent.com/artcraft-android/pdfcraft-android/c643244e0e00c1f0b746ceaff4661a91921e907e/assets/models"
if fetch "$s3/text-detection.onnx" text-detection.onnx "$det_sha" && fetch "$s3/text-recognition.onnx" text-recognition.onnx "$rec_sha"; then
    rm -f "$models"/*.rten
elif fetch "$android/text-detection.rten" text-detection.rten f15cfb56bd02c4bf478a20343986504a1f01e1665c2b3a0ad66340f054b1b5ca \
    && fetch "$android/text-recognition.rten" text-recognition.rten e484866d4cce403175bd8d00b128feb08ab42e208de30e42cd9889d8f1735a6e; then
    rm -f "$models"/*.onnx
else
    echo "the OCR models could not be fetched; the app builds without them" >&2
fi
# The licence goes with the models (CC-BY-SA-4.0 asks for it).
[ -s "$models/LICENCE-CC-BY-SA-4.0.txt" ] || curl -fsSL -o "$models/LICENCE-CC-BY-SA-4.0.txt" "$android/text-detection.rten.LICENCE.txt" || true
cat > "$models/ATTRIBUTION.txt" <<'TXT'
Text recognition (Scan & OCR) uses the ocrs models by Robert Knight, unmodified:
https://github.com/robertknight/ocrs-models (trained on HierText).
Licence: CC-BY-SA-4.0 (LICENCE-CC-BY-SA-4.0.txt).
TXT
echo "export DEXPDFCRAFT_MODELS_DIR=\"$models\""
