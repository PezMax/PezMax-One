#!/bin/bash
# build-linux.sh — Build PezMax One for Linux, package as .deb and .pkg.tar.zst.
#
# Usage:
#   ./build-linux.sh                # 只打 host 架构
#   ./build-linux.sh x64            # x86_64
#   ./build-linux.sh arm64          # aarch64
#   ./build-linux.sh x64 arm64      # 两者（需要 cross toolchain）
#   ./build-linux.sh all            # 同 "x64 arm64"
#
# 产物（放在 build/dist/）：
#   pezmax-one-VERSION-x64.deb, pezmax-one-VERSION-1-x86_64.pkg.tar.zst
#   pezmax-one-VERSION-arm64.deb, pezmax-one-VERSION-1-aarch64.pkg.tar.zst
#
# 安装后系统内路径：
#   /usr/bin/pezmax-one                            # wrapper（设 LD_LIBRARY_PATH）
#   /usr/lib/pezmax-one/pezmax-one                 # 真正的二进制
#   /usr/lib/pezmax-one/libpdfium.so               # 捆绑 pdfium
#   /usr/share/applications/io.github.pezmax.one.desktop
#   /usr/share/icons/hicolor/256x256/apps/io.github.pezmax.one.png
#   /usr/share/icons/hicolor/scalable/apps/io.github.pezmax.one.svg
#
# 依赖工具：ar (binutils)、tar、zstd、gzip、curl、cargo。不需要 dpkg-deb/makepkg。
# 交叉编译 arm64 需要：rustup target add aarch64-unknown-linux-gnu、
# aarch64-linux-gnu-gcc 链接器、libwayland/dbus 的 arm64 sysroot。

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PDFIUM_DIR="$SCRIPT_DIR/pdfium"
DIST_DIR="$SCRIPT_DIR/dist"
RUST_TARGET="$SCRIPT_DIR/rust-target"

APP_ID="io.github.pezmax.one"
PKG_NAME="pezmax-one"     # deb/arch 包名 + /usr/bin wrapper + /usr/lib 子目录
BIN_NAME="pezmax-one"     # Cargo 产物二进制名（对应 Cargo.toml 里 [package].name）
VERSION=$(grep -m1 '^version' "$ROOT_DIR/Cargo.toml" | sed -E 's/.*"(.*)".*/\1/')

RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; NC='\033[0m'
log()  { echo -e "${GREEN}[build]${NC} $*"; }
info() { echo -e "${CYAN}[info]${NC}  $*"; }
warn() { echo -e "${YELLOW}[warn]${NC}  $*"; }
err()  { echo -e "${RED}[ERROR]${NC} $*"; }

# ── 参数解析 ─────────────────────────────────────────────
detect_host_arch() {
  case "$(uname -m)" in
    x86_64) echo "x64" ;;
    aarch64|arm64) echo "arm64" ;;
    *) err "不支持的宿主架构: $(uname -m)"; exit 1 ;;
  esac
}

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

# arch_tag → rust_target|pdfium_platform|deb_arch|pacman_arch
arch_meta() {
  case "$1" in
    x64)   echo "x86_64-unknown-linux-gnu|linux-x64|amd64|x86_64" ;;
    arm64) echo "aarch64-unknown-linux-gnu|linux-arm64|arm64|aarch64" ;;
  esac
}

# ── 依赖检查 ────────────────────────────────────────────
need() { command -v "$1" &>/dev/null || { err "缺少工具: $1"; exit 1; }; }
need cargo; need tar; need zstd; need gzip; need ar; need curl

mkdir -p "$DIST_DIR"

# ── 交叉编译检测 ────────────────────────────────────────
# 判断当前 host 是否有对应架构的 C 链接器 + 基础 sysroot。
# 不同发行版链接器名字略有差异，逐个探测。
#
# 返回：0=有工具链；1=没有；同时把探测到的 linker 名字回写到 CROSS_LINKER 全局。
CROSS_LINKER=""
detect_cross_linker() {
  local arch="$1"
  CROSS_LINKER=""
  # host 架构不需要 cross
  local host_arch; host_arch=$(detect_host_arch)
  if [ "$arch" = "$host_arch" ]; then
    return 0
  fi

  case "$arch" in
    arm64)
      for candidate in aarch64-linux-gnu-gcc aarch64-unknown-linux-gnu-gcc aarch64-linux-musl-gcc; do
        if command -v "$candidate" &>/dev/null; then
          CROSS_LINKER="$candidate"
          return 0
        fi
      done
      ;;
    x64)
      for candidate in x86_64-linux-gnu-gcc x86_64-unknown-linux-gnu-gcc gcc; do
        if command -v "$candidate" &>/dev/null; then
          CROSS_LINKER="$candidate"
          return 0
        fi
      done
      ;;
  esac
  return 1
}

print_cross_hint() {
  local arch="$1"
  case "$arch" in
    arm64)
      cat <<EOF
      $(warn "跳过 $arch —— 缺少交叉编译工具链")
      安装建议（Arch）：
        sudo pacman -S aarch64-linux-gnu-gcc
      安装建议（Debian/Ubuntu）：
        sudo apt install gcc-aarch64-linux-gnu
      另外还需要 arm64 版本的 libwayland-client / libdbus-1 头文件 + .so，
      普通桌面发行版没有现成的 arm64 sysroot。推荐用 docker 容器交叉编译，
      或直接在 arm64 机器上跑 ./build-linux.sh。
EOF
      ;;
  esac
}

# ── 过滤能真正编译的架构 ────────────────────────────
BUILDABLE=()
SKIPPED=()
for a in "${ARCHES[@]}"; do
  if detect_cross_linker "$a"; then
    BUILDABLE+=("$a")
  else
    SKIPPED+=("$a")
  fi
done

if [ ${#BUILDABLE[@]} -eq 0 ]; then
  err "没有可用的目标架构。"
  for s in "${SKIPPED[@]}"; do print_cross_hint "$s"; done
  exit 1
fi

info "宿主架构: $(detect_host_arch)"
info "本次将构建: ${BUILDABLE[*]}"
if [ ${#SKIPPED[@]} -gt 0 ]; then
  warn "跳过（工具链缺失）: ${SKIPPED[*]}"
  for s in "${SKIPPED[@]}"; do print_cross_hint "$s"; done
fi

# ── 内置 PDFium 获取器 ─────────────────────────────────
# 直接下 bblanchon/pdfium-binaries 最新 release，落到 build/pdfium/{platform}/libpdfium.so
# 若目标文件已存在则跳过（走 --force 环境变量强制重下）。
PDFIUM_REPO="bblanchon/pdfium-binaries"
fetch_pdfium() {
  local platform="$1"  # linux-x64 / linux-arm64
  local lib_name="libpdfium.so"
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
  log "pdfium chromium/$ver"

  local url="https://github.com/$PDFIUM_REPO/releases/download/chromium/$ver/pdfium-$platform.tgz"
  local tmp; tmp=$(mktemp -d)
  log "下载 $url"
  curl -fL "$url" -o "$tmp/pdfium.tgz" --progress-bar

  log "解压..."
  mkdir -p "$dest_dir" "$tmp/extract"
  tar -xzf "$tmp/pdfium.tgz" -C "$tmp/extract"
  find "$tmp/extract" -name "$lib_name" -exec cp {} "$dest" \; -quit
  rm -rf "$tmp"
  [ -f "$dest" ] || { err "解压后未找到 $lib_name"; return 1; }
  log "pdfium 就绪: $dest"
}

# ── postinst / postrm 内容（deb + arch 共用逻辑） ─────
POSTINST_BODY='#!/bin/sh
set -e
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -q -t /usr/share/icons/hicolor || true
fi
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database -q /usr/share/applications || true
fi
exit 0'

POSTRM_BODY='#!/bin/sh
set -e
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -q -t /usr/share/icons/hicolor || true
fi
if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database -q /usr/share/applications || true
fi
exit 0'

# ── 打 .deb（手工构造 ar 归档） ───────────────────────
build_deb() {
  local stage="$1" arch_tag="$2" deb_arch="$3"
  local out="$DIST_DIR/${PKG_NAME}-${VERSION}-${arch_tag}.deb"
  local workdir; workdir=$(mktemp -d)

  log "打包 .deb → $(basename "$out")"

  ( cd "$stage" && tar --owner=root --group=root --numeric-owner -czf "$workdir/data.tar.gz" ./usr )

  local isize; isize=$(du -sk "$stage/usr" | awk '{print $1}')

  mkdir -p "$workdir/control"
  cat > "$workdir/control/control" <<EOF
Package: ${PKG_NAME}
Version: ${VERSION}
Architecture: ${deb_arch}
Maintainer: Takahashi Rinta <mc1586182829@outlook.com>
Installed-Size: ${isize}
Depends: libc6, libgcc-s1, libwayland-client0 | libwayland-client, libdbus-1-3
Recommends: plasma-workspace | libdbusmenu-glib4
Section: education
Priority: optional
Homepage: https://github.com/PezMax/PezMax-One
Description: PezMax One - 高性能试卷资源管理桌面客户端
 基于 egui + Metro Design 的试卷资源管理器，
 支持 PDF 预览、批量下载、书签、社区贡献等功能。
EOF

  echo "$POSTINST_BODY" > "$workdir/control/postinst"
  echo "$POSTRM_BODY"   > "$workdir/control/postrm"
  chmod 0755 "$workdir/control/postinst" "$workdir/control/postrm"

  ( cd "$workdir/control" && tar --owner=root --group=root --numeric-owner \
      -czf "$workdir/control.tar.gz" ./control ./postinst ./postrm )

  echo "2.0" > "$workdir/debian-binary"

  rm -f "$out"
  ( cd "$workdir" && ar rc "$out" debian-binary control.tar.gz data.tar.gz )
  rm -rf "$workdir"
  log "  → $out"
}

# ── 打 .pkg.tar.zst（手工构造，无需 makepkg） ─────────
build_arch_pkg() {
  local stage="$1" arch_tag="$2" pacman_arch="$3"
  local out="$DIST_DIR/${PKG_NAME}-${VERSION}-1-${pacman_arch}.pkg.tar.zst"
  local workdir; workdir=$(mktemp -d)

  log "打包 .pkg.tar.zst → $(basename "$out")"
  cp -r "$stage/." "$workdir/"

  local isize; isize=$(du -sb "$stage/usr" | awk '{print $1}')
  local build_date; build_date=$(date +%s)

  cat > "$workdir/.PKGINFO" <<EOF
pkgname = ${PKG_NAME}
pkgbase = ${PKG_NAME}
pkgver = ${VERSION}-1
pkgdesc = PezMax One - 高性能试卷资源管理桌面客户端
url = https://github.com/PezMax/PezMax-One
builddate = ${build_date}
packager = Takahashi Rinta <mc1586182829@outlook.com>
size = ${isize}
arch = ${pacman_arch}
license = MIT
depend = wayland
depend = dbus
optdepend = plasma-workspace: KDE Plasma Global Menu integration
EOF

  cat > "$workdir/.INSTALL" <<'EOF'
post_install() {
  if command -v gtk-update-icon-cache >/dev/null 2>&1; then
    gtk-update-icon-cache -q -t usr/share/icons/hicolor || true
  fi
  if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database -q usr/share/applications || true
  fi
}
post_upgrade() { post_install; }
post_remove()  { post_install; }
EOF

  ( cd "$workdir" && \
    tar --owner=root --group=root --numeric-owner \
        -cf - .PKGINFO .INSTALL usr | zstd -q -T0 -o "$out" )
  rm -rf "$workdir"
  log "  → $out"
}

# ── 主循环 ────────────────────────────────────────
FAILED_ARCHES=()
for ARCH in "${BUILDABLE[@]}"; do
  IFS='|' read -r RUST_TRIPLE PDF_PLATFORM DEB_ARCH PACMAN_ARCH <<< "$(arch_meta "$ARCH")"

  echo ""
  echo "============================================================"
  echo "  PezMax One Linux Build · $ARCH ($RUST_TRIPLE)"
  echo "============================================================"

  if ! rustup target list --installed 2>/dev/null | grep -q "^$RUST_TRIPLE$"; then
    warn "Rust target $RUST_TRIPLE 未安装，rustup target add ..."
    rustup target add "$RUST_TRIPLE"
  fi

  fetch_pdfium "$PDF_PLATFORM"
  PDFIUM_LIB="$PDFIUM_DIR/$PDF_PLATFORM/libpdfium.so"

  # 探测链接器并回写到 CROSS_LINKER；给 cargo 设 CARGO_TARGET_*_LINKER
  # host 架构 detect_cross_linker 返回 0 但 CROSS_LINKER 为空，跳过设置
  detect_cross_linker "$ARCH" || true
  LINKER_ENV=()
  if [ -n "$CROSS_LINKER" ] && [ "$ARCH" != "$(detect_host_arch)" ]; then
    # RUST_TRIPLE 里的连字符转下划线大写，作为 env var 名称
    ENV_NAME="CARGO_TARGET_$(echo "$RUST_TRIPLE" | tr 'a-z-' 'A-Z_')_LINKER"
    LINKER_ENV=("$ENV_NAME=$CROSS_LINKER")
    info "cross linker: $CROSS_LINKER (via $ENV_NAME)"
  fi

  log "cargo build --release --target $RUST_TRIPLE（首次冷编译约需 5-10 分钟）"
  (
    cd "$ROOT_DIR"
    env "${LINKER_ENV[@]}" CARGO_TARGET_DIR="$RUST_TARGET" \
      cargo build --release --target "$RUST_TRIPLE"
  ) || {
    err "$ARCH cargo build 失败，跳过后续打包"
    FAILED_ARCHES+=("$ARCH")
    continue
  }
  BUILT_BIN="$RUST_TARGET/$RUST_TRIPLE/release/$BIN_NAME"
  [ -x "$BUILT_BIN" ] || { err "构建产物缺失: $BUILT_BIN"; FAILED_ARCHES+=("$ARCH"); continue; }

  STAGE="$SCRIPT_DIR/stage-$ARCH"
  rm -rf "$STAGE"
  mkdir -p \
    "$STAGE/usr/bin" \
    "$STAGE/usr/lib/$PKG_NAME" \
    "$STAGE/usr/share/applications" \
    "$STAGE/usr/share/icons/hicolor/256x256/apps" \
    "$STAGE/usr/share/icons/hicolor/scalable/apps"

  install -m 0755 "$BUILT_BIN"  "$STAGE/usr/lib/$PKG_NAME/$BIN_NAME"
  install -m 0644 "$PDFIUM_LIB" "$STAGE/usr/lib/$PKG_NAME/libpdfium.so"

  # /usr/bin wrapper：pdfium-render 用 libloading 打开 libpdfium.so，
  # 需要 LD_LIBRARY_PATH 里包含它的目录
  cat > "$STAGE/usr/bin/$PKG_NAME" << WRAPPER
#!/bin/sh
export LD_LIBRARY_PATH="/usr/lib/$PKG_NAME\${LD_LIBRARY_PATH:+:\$LD_LIBRARY_PATH}"
exec /usr/lib/$PKG_NAME/$BIN_NAME "\$@"
WRAPPER
  chmod 0755 "$STAGE/usr/bin/$PKG_NAME"

  install -m 0644 "$ROOT_DIR/resources/linux/$APP_ID.desktop" \
                  "$STAGE/usr/share/applications/$APP_ID.desktop"
  install -m 0644 "$ROOT_DIR/resources/icon.png" \
                  "$STAGE/usr/share/icons/hicolor/256x256/apps/$APP_ID.png"
  install -m 0644 "$ROOT_DIR/resources/icon.svg" \
                  "$STAGE/usr/share/icons/hicolor/scalable/apps/$APP_ID.svg"

  build_deb      "$STAGE" "$ARCH" "$DEB_ARCH"
  build_arch_pkg "$STAGE" "$ARCH" "$PACMAN_ARCH"

  rm -rf "$STAGE"
done

echo ""
if [ ${#FAILED_ARCHES[@]} -gt 0 ]; then
  warn "以下架构构建失败: ${FAILED_ARCHES[*]}"
fi
if [ ${#SKIPPED[@]} -gt 0 ]; then
  warn "以下架构因工具链缺失被跳过: ${SKIPPED[*]}"
fi
log "产物列表："
ls -lh "$DIST_DIR"/*.deb "$DIST_DIR"/*.pkg.tar.zst 2>/dev/null || true
