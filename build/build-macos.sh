#!/bin/bash
# build-macos.sh — Build PezMax One for macOS as .app bundle.
#
# Usage:
#   ./build-macos.sh            # 只打宿主架构
#   ./build-macos.sh x64        # x86_64
#   ./build-macos.sh arm64      # aarch64（Apple Silicon）
#   ./build-macos.sh all        # 两者都打
#   ./build-macos.sh universal  # 单个 universal binary（x64 + arm64 fat）
#
# 产物（放在 build/dist/）：
#   PezMax One-1.0.0-{x64,arm64,universal}.dmg-like tarball（tar.gz 包住 .app bundle）
#     └─ PezMax One.app/Contents/{MacOS/*, Resources/*.icns, Info.plist, PkgInfo}
#
# 交叉编译说明：macOS 上 Xcode SDK 覆盖两种架构，只需 rustup target 到位。
# universal binary 用 `lipo` 合并两个架构的产物。
#
# 依赖：cargo、curl、tar、iconutil（macOS 内置）、sips（macOS 内置）、lipo（universal 时）
# 只能在 macOS 上运行（iconutil / sips / lipo 是 macOS 独有）。

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PDFIUM_DIR="$SCRIPT_DIR/pdfium"
DIST_DIR="$SCRIPT_DIR/dist"
RUST_TARGET="$SCRIPT_DIR/rust-target"

BIN_NAME="pezmax-one"
APP_NAME="PezMax One"
APP_ID="io.github.pezmax.one"
VERSION=$(grep -m1 '^version' "$ROOT_DIR/Cargo.toml" | sed -E 's/.*"(.*)".*/\1/')

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

# ── 参数解析 ────────────────────────────────────────────
BUILD_UNIVERSAL=0
ARCHES=()
if [ $# -eq 0 ]; then
  ARCHES=("$(detect_host_arch)")
else
  for a in "$@"; do
    case "$a" in
      x64|arm64) ARCHES+=("$a") ;;
      all)       ARCHES=(x64 arm64) ;;
      universal) BUILD_UNIVERSAL=1; ARCHES=(x64 arm64) ;;
      *) err "未知架构: $a（支持 x64 / arm64 / all / universal）"; exit 1 ;;
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
if [ "$(uname -s)" != "Darwin" ]; then
  err "此脚本只能在 macOS 上运行（依赖 iconutil / sips / lipo）"
  exit 1
fi
need iconutil; need sips
if [ "$BUILD_UNIVERSAL" = "1" ]; then need lipo; fi

mkdir -p "$DIST_DIR"

# ── 内置 pdfium 获取器 ─────────────────────────────────
PDFIUM_REPO="bblanchon/pdfium-binaries"
fetch_pdfium() {
  local platform="$1"  # macos-x64 / macos-arm64
  local lib_name="libpdfium.dylib"
  local dest_dir="$PDFIUM_DIR/$platform"
  local dest="$dest_dir/$lib_name"

  if [ -f "$dest" ] && [ "${FORCE_PDFIUM:-0}" != "1" ]; then
    info "pdfium 已存在: $dest"
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

# ── 从 icon.png 生成 icon.icns ─────────────────────────
# 需要 icon.png 至少 512×512；小于 512 的会被 sips 放大导致模糊。
# 缓存到 build/icon.icns 避免每次重跑。
build_icns() {
  local out="$1"
  if [ -f "$out" ] && [ "${FORCE_ICON:-0}" != "1" ]; then
    return 0
  fi
  log "生成 icon.icns..."
  local src_png="$ROOT_DIR/resources/icon.png"
  [ -f "$src_png" ] || { err "缺少 $src_png"; return 1; }
  local iconset; iconset=$(mktemp -d)/AppIcon.iconset
  mkdir -p "$iconset"
  for size in 16 32 128 256 512; do
    sips -z $size $size "$src_png" --out "$iconset/icon_${size}x${size}.png" >/dev/null
    local d=$((size * 2))
    sips -z $d $d "$src_png" --out "$iconset/icon_${size}x${size}@2x.png" >/dev/null
  done
  iconutil -c icns "$iconset" -o "$out"
  rm -rf "$(dirname "$iconset")"
  log "  → $out"
}

# ── 组装 .app bundle ─────────────────────────────────
# $1: binary source path
# $2: pdfium dylib source path
# $3: output arch tag（用于 dist 目录命名）
# $4: 已存在的 icon.icns 路径
build_app_bundle() {
  local bin_src="$1" pdf_src="$2" arch_tag="$3" icns_src="$4"
  local app_dir="$DIST_DIR/${APP_NAME}-${arch_tag}.app"

  log "组装 .app bundle → $app_dir"
  rm -rf "$app_dir"
  mkdir -p "$app_dir/Contents/MacOS" "$app_dir/Contents/Resources"

  install -m 0755 "$bin_src" "$app_dir/Contents/MacOS/$BIN_NAME"
  install -m 0644 "$pdf_src" "$app_dir/Contents/MacOS/libpdfium.dylib"
  install -m 0644 "$icns_src" "$app_dir/Contents/Resources/AppIcon.icns"

  # Info.plist
  cat > "$app_dir/Contents/Info.plist" <<EOF
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleDisplayName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleExecutable</key>
    <string>${BIN_NAME}</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundleIdentifier</key>
    <string>${APP_ID}</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>${VERSION}</string>
    <key>CFBundleVersion</key>
    <string>${VERSION}</string>
    <key>LSMinimumSystemVersion</key>
    <string>11.0</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
    <key>NSHumanReadableCopyright</key>
    <string>© Takahashi Rinta</string>
</dict>
</plist>
EOF

  # PkgInfo（可选，某些 Finder 版本要）
  printf 'APPL????' > "$app_dir/Contents/PkgInfo"

  # 打成 tar.gz 便于下载分发（macOS Finder 双击 .tar.gz 也能解出 .app）
  local archive="$DIST_DIR/pezmax-one-macos-${arch_tag}.tar.gz"
  ( cd "$DIST_DIR" && tar -czf "$archive" "$(basename "$app_dir")" )
  log "  → $archive"
}

# ── 计划打印 ────────────────────────────────────────────
HOST_ARCH=$(detect_host_arch)
info "宿主架构: $HOST_ARCH"
if [ "$BUILD_UNIVERSAL" = "1" ]; then
  info "本次将构建: universal（x64 + arm64 → lipo 合并）"
else
  info "本次将构建: ${ARCHES[*]}"
fi

# ── 图标只生成一次 ─────────────────────────────────────
ICNS="$SCRIPT_DIR/AppIcon.icns"
build_icns "$ICNS"

# ── 主循环：编译每个架构 ─────────────────────────────
FAILED_ARCHES=()
declare -A BUILT_BINS      # arch → binary path
declare -A BUILT_PDFIUM    # arch → dylib path
for ARCH in "${ARCHES[@]}"; do
  IFS='|' read -r RUST_TRIPLE PDF_PLATFORM <<< "$(arch_meta "$ARCH")"

  echo ""
  echo "============================================================"
  echo "  PezMax One macOS Build · $ARCH ($RUST_TRIPLE)"
  echo "============================================================"

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
    err "$ARCH cargo build 失败，跳过"
    FAILED_ARCHES+=("$ARCH")
    continue
  }

  BUILT_BIN="$RUST_TARGET/$RUST_TRIPLE/release/$BIN_NAME"
  [ -x "$BUILT_BIN" ] || { err "构建产物缺失: $BUILT_BIN"; FAILED_ARCHES+=("$ARCH"); continue; }
  BUILT_BINS[$ARCH]="$BUILT_BIN"
  BUILT_PDFIUM[$ARCH]="$PDFIUM_LIB"

  # 非 universal 模式：每个 arch 直接出一个 .app
  if [ "$BUILD_UNIVERSAL" != "1" ]; then
    build_app_bundle "$BUILT_BIN" "$PDFIUM_LIB" "$ARCH" "$ICNS"
  fi
done

# ── universal 模式：lipo 合并 ────────────────────────
if [ "$BUILD_UNIVERSAL" = "1" ] && [ ${#FAILED_ARCHES[@]} -eq 0 ]; then
  echo ""
  echo "============================================================"
  echo "  Building universal (x64 + arm64 → lipo)"
  echo "============================================================"
  UNIVERSAL_BIN="$SCRIPT_DIR/universal-$BIN_NAME"
  UNIVERSAL_PDFIUM="$SCRIPT_DIR/universal-libpdfium.dylib"

  lipo -create -output "$UNIVERSAL_BIN" \
    "${BUILT_BINS[x64]}" "${BUILT_BINS[arm64]}"
  lipo -create -output "$UNIVERSAL_PDFIUM" \
    "${BUILT_PDFIUM[x64]}" "${BUILT_PDFIUM[arm64]}"
  log "lipo 合并完成"

  build_app_bundle "$UNIVERSAL_BIN" "$UNIVERSAL_PDFIUM" "universal" "$ICNS"

  rm -f "$UNIVERSAL_BIN" "$UNIVERSAL_PDFIUM"
fi

echo ""
if [ ${#FAILED_ARCHES[@]} -gt 0 ]; then
  warn "以下架构构建失败: ${FAILED_ARCHES[*]}"
fi
log "产物列表："
ls -lh "$DIST_DIR"/pezmax-one-macos-*.tar.gz 2>/dev/null || true
