#!/bin/bash
# build-macos.sh — Build PezMax for macOS
# Output: tar.gz archive with executable + libpdfium.dylib + launcher script

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PDFIUM_DIR="$SCRIPT_DIR/pdfium"
DIST_DIR="$SCRIPT_DIR/dist"
RUST_TARGET="$SCRIPT_DIR/rust-target"

# Detect architecture
HOST_ARCH=$(uname -m)
case "$HOST_ARCH" in
  x86_64) ARCH_TAG="x64";   PDF_PLATFORM="macos-x64"  ;;
  arm64)  ARCH_TAG="arm64"; PDF_PLATFORM="macos-arm64" ;;
  *) echo "[ERROR] Unsupported architecture: $HOST_ARCH"; exit 1 ;;
esac

PDFIUM_LIB_DIR="$PDFIUM_DIR/$PDF_PLATFORM"
PDFIUM_LIB="$PDFIUM_LIB_DIR/libpdfium.dylib"

echo "============================================"
echo "  PezMax macOS Build ($PDF_PLATFORM)"
echo "============================================"

# Download pdfium if missing
if [ ! -f "$PDFIUM_LIB" ]; then
  echo "[pdfium] Prebuilt library not found, downloading..."
  "$SCRIPT_DIR/fetch-pdfium.sh" "$PDF_PLATFORM"
  if [ ! -f "$PDFIUM_LIB" ]; then
    echo "[ERROR] pdfium download failed"
    exit 1
  fi
  echo "[pdfium] Cached to: $PDFIUM_LIB"
else
  echo "[pdfium] Using existing library: $PDFIUM_LIB"
fi

# Build Rust
echo "[build] cargo build --release ..."
cd "$ROOT_DIR"
CARGO_TARGET_DIR="$RUST_TARGET" cargo build --release
echo "[build] Build successful"

# Assemble dist directory
OUT_DIR="$DIST_DIR/pezmax-macos-$ARCH_TAG"
rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

cp "$RUST_TARGET/release/pezmax-egui" "$OUT_DIR/pezmax-egui"
cp "$PDFIUM_LIB" "$OUT_DIR/libpdfium.dylib"
chmod +x "$OUT_DIR/pezmax-egui"

# Create launcher script: pdfium-render uses libloading to load libpdfium.dylib
# at runtime. DYLD_LIBRARY_PATH must include the directory containing it.
# Note: SIP restricts DYLD_* for system processes, but user-signed apps are unaffected.
cat > "$OUT_DIR/pezmax.sh" << 'LAUNCHER'
#!/bin/bash
DIR="$(cd "$(dirname "$0")" && pwd)"
exec env DYLD_LIBRARY_PATH="$DIR${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}" "$DIR/pezmax-egui" "$@"
LAUNCHER
chmod +x "$OUT_DIR/pezmax.sh"

# Package as tar.gz
ARCHIVE="$DIST_DIR/pezmax-macos-$ARCH_TAG.tar.gz"
cd "$DIST_DIR"
tar -czf "$ARCHIVE" "pezmax-macos-$ARCH_TAG/"

echo ""
echo "============================================"
echo "  Build complete"
echo "  Output: $ARCHIVE"
echo "  Run: ./pezmax.sh"
echo "============================================"