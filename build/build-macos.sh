#!/bin/bash
# build-macos.sh — Build PezMax One for macOS, package as tar.gz.
#
# Usage:
#   ./build-macos.sh            # 只打宿主架构
#   ./build-macos.sh x64        # x86_64
#   ./build-macos.sh arm64      # aarch64（Apple Silicon）
#   ./build-macos.sh all        # 两者都打
#
# 交叉编译说明：macOS 上 Xcode SDK 本身覆盖两种架构，只需 rustup target 到位，
# 无需额外的 C 链接器工具链。因此 x64 host 打 arm64（或反向）通常直接可用。
# .app bundle 打包由 Task #4 处理；本脚本产出仍是裸二进制 + 启动器 + tar.gz。

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PDFIUM_DIR="$SCRIPT_DIR/pdfium"
DIST_DIR="$SCRIPT_DIR/dist"
RUST_TARGET="$SCRIPT_DIR/rust-target"

BIN_NAME="pezmax-one"

# Colors
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; NC='\033[0m'
log()  { echo -e "${GREEN}[build]${NC} $*"; }
info() { echo -e "${CYAN}[info]${NC}  $*"; }
warn() { echo -e "${YELLOW}[warn]${NC}  $*"; }
err()  { echo -e "${RED}[ERROR]${NC} $*"; }

detect_host_arch() {
  case "$(uname -m)" in
    x86_64) echo "x64" ;;
    arm64|aarch64) echo "arm64" ;;
    *) err "不支持的宿主架构: $(uname -m)"; exit 1 ;;
  esac
}

# 参数解析
ARCHES=()
if [ $# -eq 0 ]; then
  ARCHES=("$(detect_host_arch)")
else
  for a in "$@"; do
    case "$a" in
      x64|arm64) ARCHES+=("$a") ;;
      all)       ARCHES=(x64 arm64) ;;
      *) err "未知架构: $a（支持 x64 / arm64 / all）"; exit 1 ;;
    esac
  done
fi

# arch_tag → rust_target|pdfium_platform
arch_meta() {
  case "$1" in
    x64)   echo "x86_64-apple-darwin|macos-x64" ;;
    arm64) echo "aarch64-apple-darwin|macos-arm64" ;;
  esac
}

need() { command -v "$1" &>/dev/null || { err "缺少工具: $1"; exit 1; }; }
need cargo; need curl; need tar

mkdir -p "$DIST_DIR"

# ── 内置 pdfium 获取器 ─────────────────────────────────
PDFIUM_REPO="bblanchon/pdfium-binaries"
fetch_pdfium() {
  local platform="$1"  # macos-x64 / macos-arm64
  local lib_name="libpdfium.dylib"
  local dest_dir="$PDFIUM_DIR/$platform"
  local dest="$dest_dir/$lib_name"

  if [ -f "$dest" ] && [ "${FORCE_PDFIUM:-0}" != "1" ]; then
    info "pdfium 已存在: $dest（FORCE_PDFIUM=1 强制重下）"
    return 0
  fi

  log "拉取 pdfium 版本号..."
  local ver
  ver=$(curl -sf "https://api.github.com/repos/$PDFIUM_REPO/releases/latest" \
        | grep '"tag_name"' \
        | sed 's/.*"tag_name": *"chromium\/\([^"]*\)".*/\1/')
  [ -n "$ver" ] || { err "无法获取 pdfium 版本"; return 1; }

  local url="https://github.com/$PDFIUM_REPO/releases/download/chromium/$ver/pdfium-$platform.tgz"
  local tmp; tmp=$(mktemp -d)
  log "下载 chromium/$ver ($platform)"
  curl -fL "$url" -o "$tmp/pdfium.tgz" --progress-bar

  mkdir -p "$dest_dir" "$tmp/extract"
  tar -xzf "$tmp/pdfium.tgz" -C "$tmp/extract"
  find "$tmp/extract" -name "$lib_name" -exec cp {} "$dest" \; -quit
  rm -rf "$tmp"
  [ -f "$dest" ] || { err "解压后未找到 $lib_name"; return 1; }
  log "pdfium 就绪: $dest"
}

# ── 计划打印 ────────────────────────────────────────────
HOST_ARCH=$(detect_host_arch)
info "宿主架构: $HOST_ARCH"
info "本次将构建: ${ARCHES[*]}"

# ── 主循环 ─────────────────────────────────────────────
FAILED_ARCHES=()
for ARCH in "${ARCHES[@]}"; do
  IFS='|' read -r RUST_TRIPLE PDF_PLATFORM <<< "$(arch_meta "$ARCH")"

  echo ""
  echo "============================================================"
  echo "  PezMax One macOS Build · $ARCH ($RUST_TRIPLE)"
  echo "============================================================"

  # rustup target 检查
  if ! rustup target list --installed 2>/dev/null | grep -q "^$RUST_TRIPLE$"; then
    warn "Rust target $RUST_TRIPLE 未安装，rustup target add ..."
    rustup target add "$RUST_TRIPLE"
  fi

  fetch_pdfium "$PDF_PLATFORM"
  PDFIUM_LIB="$PDFIUM_DIR/$PDF_PLATFORM/libpdfium.dylib"

  log "cargo build --release --target $RUST_TRIPLE"
  (
    cd "$ROOT_DIR"
    CARGO_TARGET_DIR="$RUST_TARGET" cargo build --release --target "$RUST_TRIPLE"
  ) || {
    err "$ARCH cargo build 失败，跳过打包"
    FAILED_ARCHES+=("$ARCH")
    continue
  }

  BUILT_BIN="$RUST_TARGET/$RUST_TRIPLE/release/$BIN_NAME"
  [ -x "$BUILT_BIN" ] || { err "构建产物缺失: $BUILT_BIN"; FAILED_ARCHES+=("$ARCH"); continue; }

  # 组装 dist 目录
  OUT_DIR="$DIST_DIR/pezmax-one-macos-$ARCH"
  rm -rf "$OUT_DIR"; mkdir -p "$OUT_DIR"

  cp "$BUILT_BIN"    "$OUT_DIR/$BIN_NAME"
  cp "$PDFIUM_LIB"   "$OUT_DIR/libpdfium.dylib"
  chmod +x "$OUT_DIR/$BIN_NAME"

  # 启动器：pdfium-render 用 libloading 加载 libpdfium.dylib，需要
  # DYLD_LIBRARY_PATH 指向它。（SIP 不影响用户签名的应用。）
  cat > "$OUT_DIR/pezmax-one.sh" << 'LAUNCHER'
#!/bin/bash
DIR="$(cd "$(dirname "$0")" && pwd)"
exec env DYLD_LIBRARY_PATH="$DIR${DYLD_LIBRARY_PATH:+:$DYLD_LIBRARY_PATH}" "$DIR/pezmax-one" "$@"
LAUNCHER
  chmod +x "$OUT_DIR/pezmax-one.sh"

  ARCHIVE="$DIST_DIR/pezmax-one-macos-$ARCH.tar.gz"
  (cd "$DIST_DIR" && tar -czf "$ARCHIVE" "$(basename "$OUT_DIR")/")
  log "  → $ARCHIVE"
done

echo ""
if [ ${#FAILED_ARCHES[@]} -gt 0 ]; then
  warn "以下架构构建失败: ${FAILED_ARCHES[*]}"
fi
log "产物列表："
ls -lh "$DIST_DIR"/pezmax-one-macos-*.tar.gz 2>/dev/null || true
