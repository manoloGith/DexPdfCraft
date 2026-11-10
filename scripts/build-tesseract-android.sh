#!/usr/bin/env bash
# Build Tesseract (Apache-2.0) for Android, for PdfCraft's second OCR engine, and fetch its
# language data. PdfCraft's built-in engine (ocrs) reads only English; Tesseract reads the
# languages PdfCraft allow-lists (Spanish, French, German…). PdfCraft runs it as a separate
# program; Android only runs programs from an app's native-library folder, so it ships as
# lib/<abi>/libtesseract.so (an executable, not a library).
#
#   scripts/build-tesseract-android.sh [abi…]     # default: arm64-v8a
#
# Needs ANDROID_NDK_HOME, cmake and ninja. Results (gitignored):
#   .build-inputs/tesseract/<abi>/libtesseract.so
#   .build-inputs/tessdata/*.traineddata          (DEXPDFCRAFT_TESSDATA_LANGS, see below)
#
# Sources, pinned: libpng v1.6.49 (libpng licence), Leptonica 1.85.0 (BSD-2-Clause),
# Tesseract 5.5.4 (Apache-2.0), tessdata_fast at 87416418 (Apache-2.0). zlib is Android's own.
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
ndk="${ANDROID_NDK_HOME:?set ANDROID_NDK_HOME to the NDK (29.0.14206865)}"
abis=("${@:-arm64-v8a}")
api=29
work="$root/.build-inputs/tesseract-build"
src="$work/src"
mkdir -p "$src"

LIBPNG_TAG=v1.6.49
LEPTONICA_VERSION=1.85.0
TESSERACT_TAG=5.5.4
TESSDATA_COMMIT=87416418657359cb625c412a48b6e1d6d41c29bd
# Latin-script languages PdfCraft allow-lists (fast models, about 2–4 MB each).
langs="${DEXPDFCRAFT_TESSDATA_LANGS:-eng spa fra deu ita por nld}"

clone() { # url tag dir
    [ -d "$3/.git" ] || git clone --quiet --depth 1 --branch "$2" "$1" "$3"
}
clone https://github.com/pnggroup/libpng "$LIBPNG_TAG" "$src/libpng"
clone https://github.com/tesseract-ocr/tesseract "$TESSERACT_TAG" "$src/tesseract"
if [ ! -d "$src/leptonica" ]; then
    curl -fsSL -o "$src/leptonica.tar.gz" \
        "https://github.com/DanBloomberg/leptonica/releases/download/$LEPTONICA_VERSION/leptonica-$LEPTONICA_VERSION.tar.gz"
    mkdir -p "$src/leptonica"
    tar -xzf "$src/leptonica.tar.gz" -C "$src/leptonica" --strip-components 1
fi

for abi in "${abis[@]}"; do
    prefix="$work/$abi/install"
    common=(
        -G Ninja
        -DCMAKE_TOOLCHAIN_FILE="$ndk/build/cmake/android.toolchain.cmake"
        -DANDROID_ABI="$abi" -DANDROID_PLATFORM="android-$api" -DANDROID_STL=c++_static
        -DCMAKE_BUILD_TYPE=Release -DBUILD_SHARED_LIBS=OFF
        -DCMAKE_INSTALL_PREFIX="$prefix" -DCMAKE_PREFIX_PATH="$prefix" -DCMAKE_FIND_ROOT_PATH="$prefix"
    )
    cmake -S "$src/libpng" -B "$work/$abi/libpng" "${common[@]}" \
        -DPNG_SHARED=OFF -DPNG_STATIC=ON -DPNG_TESTS=OFF -DPNG_TOOLS=OFF -DPNG_FRAMEWORK=OFF > "$work/$abi/libpng.log"
    cmake --build "$work/$abi/libpng" --target install >> "$work/$abi/libpng.log"

    cmake -S "$src/leptonica" -B "$work/$abi/leptonica" "${common[@]}" \
        -DBUILD_PROG=OFF -DSW_BUILD=OFF -DENABLE_ZLIB=ON -DENABLE_PNG=ON -DENABLE_GIF=OFF -DENABLE_JPEG=OFF \
        -DENABLE_TIFF=OFF -DENABLE_WEBP=OFF -DENABLE_OPENJPEG=OFF > "$work/$abi/leptonica.log"
    cmake --build "$work/$abi/leptonica" --target install >> "$work/$abi/leptonica.log"

    cmake -S "$src/tesseract" -B "$work/$abi/tesseract" "${common[@]}" \
        -DBUILD_TRAINING_TOOLS=OFF -DBUILD_TESTS=OFF -DGRAPHICS_DISABLED=ON -DDISABLE_ARCHIVE=ON -DDISABLE_CURL=ON \
        -DDISABLE_TIFF=ON -DOPENMP_BUILD=OFF -DENABLE_LTO=OFF -DINSTALL_CONFIGS=OFF \
        -DLeptonica_DIR="$prefix/lib/cmake/leptonica" > "$work/$abi/tesseract.log"
    cmake --build "$work/$abi/tesseract" --target tesseract >> "$work/$abi/tesseract.log"

    out="$root/.build-inputs/tesseract/$abi"
    mkdir -p "$out"
    cp "$work/$abi/tesseract/bin/tesseract" "$out/libtesseract.so" 2>/dev/null || cp "$work/$abi/tesseract/tesseract" "$out/libtesseract.so"
    "$ndk/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-strip" "$out/libtesseract.so"
    echo "built $out/libtesseract.so"
done

data="$root/.build-inputs/tessdata"
mkdir -p "$data"
for lang in $langs; do
    [ -s "$data/$lang.traineddata" ] || curl -fsSL -o "$data/$lang.traineddata" \
        "https://raw.githubusercontent.com/tesseract-ocr/tessdata_fast/$TESSDATA_COMMIT/$lang.traineddata"
done
[ -s "$data/LICENSE" ] || curl -fsSL -o "$data/LICENSE" "https://raw.githubusercontent.com/tesseract-ocr/tessdata_fast/$TESSDATA_COMMIT/LICENSE"
echo "language data: $(ls "$data" | tr '\n' ' ')"
