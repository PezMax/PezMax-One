#!/bin/bash
set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PDFIUM_DIR="$SCRIPT_DIR/pdfium"
DIST_DIR="$SCRIPT_DIR/dist"
RUST_TARGET="$SCRIPT_DIR/rust-target"

# ── 检测架构 ─────────────────────────────────────────────────
HOST_ARCH=$(uname -m)
case "$HOST_ARCH" in
  x86_64) ARCH_TAG="x64";   PDF_PLATFORM="mac-x64";   DIR_TAG="macos-x64"   ;;
  arm64)  ARCH_TAG="arm64"; PDF_PLATFORM="mac-arm64";  DIR_TAG="macos-arm64" ;;
  *) echo "[ERROR] 不支持的架构: $HOST_ARCH"; exit 1 ;;
esac

PDFIUM_LIB_DIR="$PDFIUM_DIR/$DIR_TAG"
PDFIUM_LIB="$PDFIUM_LIB_DIR/libpdfium.dylib"

echo "============================================"
echo "  PezMax macOS 构建脚本 ($PDF_PLATFORM)"
echo "============================================"

# ── 下载 pdfium（缺失时自动获取）───────────────────────────
if [ ! -f "$PDFIUM_LIB" ]; then
  echo "[pdfium] 未找到预编译库，从 GitHub 下载..."
  if ! command -v curl &>/dev/null; then
    echo "[ERROR] 需要 curl，请先安装"; exit 1
  fi
  PDFIUM_VER=$(curl -sf "https://api.github.com/repos/bblanchon/pdfium-binaries/releases/latest" \
    | grep '"tag_name"' | grep -oE '[0-9]+' | head -1)
  if [ -z "$PDFIUM_VER" ]; then
    echo "[ERROR] 无法获取 pdfium 版本，请检查网络"; exit 1
  fi
  echo "[pdfium] 版本: chromium/$PDFIUM_VER  平台: $PDF_PLATFORM"
  TMP=$(mktemp -d)
  curl -fL "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F${PDFIUM_VER}/pdfium-${PDF_PLATFORM}.tgz" \
    -o "$TMP/pdfium.tgz"
  tar -xzf "$TMP/pdfium.tgz" -C "$TMP"
  mkdir -p "$PDFIUM_LIB_DIR"
  cp "$TMP/lib/libpdfium.dylib" "$PDFIUM_LIB"
  rm -rf "$TMP"
  echo "[pdfium] 已缓存到 $PDFIUM_LIB"
else
  echo "[pdfium] 使用已有库: $PDFIUM_LIB"
fi

# ── 构建 Rust ─────────────────────────────────────────────
echo "[build] cargo build --release ..."
cd "$ROOT_DIR"
CARGO_TARGET_DIR="$RUST_TARGET" cargo build --release
echo "[build] 构建成功"

# ── 组装 dist 目录 ────────────────────────────────────────
OUT_DIR="$DIST_DIR/pezmax-macos-$ARCH_TAG"
rm -rf "$OUT_DIR"
mkdir -p "$OUT_DIR"

cp "$RUST_TARGET/release/pezmax-egui" "$OUT_DIR/pezmax-egui"
cp "$PDFIUM_LIB" "$OUT_DIR/libpdfium.dylib"
chmod +x "$OUT_DIR/pezmax-egui"

# 启动脚本：pdfium-render 通过 libloading 动态加载 .dylib，
# DYLD_LIBRARY_PATH 使其能从同目录找到 libpdfium.dylib。
# 注意：SIP 对系统进程限制 DYLD_*，用户签名 app 不受影响。
cat > "$OUT_DIR/pezmax.sh" << 'LAUNCHER'
#!/bin/bash
DIR="$(cd "$(dirname "$0")" && pwd)"
exec env DYLD_LIBRARY_PATH="$DIR${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}" "$DIR/pezmax-egui" "$@"
LAUNCHER
chmod +x "$OUT_DIR/pezmax.sh"

# ── 打包为 tar.gz ─────────────────────────────────────────
ARCHIVE="$DIST_DIR/pezmax-macos-$ARCH_TAG.tar.gz"
cd "$DIST_DIR"
tar -czf "$ARCHIVE" "pezmax-macos-$ARCH_TAG/"

echo ""
echo "============================================"
echo "  构建完成"
echo "  输出: $ARCHIVE"
echo "  运行: ./pezmax.sh"
echo "============================================"
