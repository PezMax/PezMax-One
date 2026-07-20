#!/bin/bash
set -e

echo "============================================"
echo "  PezMax Build Script (Linux)"
echo "  Build Rust frontend only"
echo "  Remote backend is at http://154.8.139.48:8080"
echo "  To build backend locally, run: ./build-backend.sh"
echo "============================================"
echo ""

ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
BUILD_DIR="$ROOT_DIR/build"
DIST_DIR="$BUILD_DIR/dist"
RUST_TARGET_DIR="$BUILD_DIR/rust-target"

# Clean old dist
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

# ─── Build Rust frontend ─────────────────────────────
echo "[1/1] Building Rust frontend..."

export CARGO_TARGET_DIR="$RUST_TARGET_DIR"

cd "$ROOT_DIR"

cargo build --release
echo "[OK] Rust frontend built successfully"

# Copy binary to dist
cp "$RUST_TARGET_DIR/release/pezmax-egui" "$DIST_DIR/pezmax-egui"
echo "[OK] Frontend binary copied to $DIST_DIR/pezmax-egui"

# ─── Done ────────────────────────────────────────────
echo "============================================"
echo "  Build complete!"
echo "  Output: $DIST_DIR/pezmax-egui"
echo "============================================"