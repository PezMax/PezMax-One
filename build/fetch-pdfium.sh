#!/bin/bash
# fetch-pdfium.sh — Download PDFium prebuilt libraries to build/pdfium/
#
# Usage:
#   ./fetch-pdfium.sh                              Download all architectures
#   ./fetch-pdfium.sh windows-x64                  Download specific architecture only
#   ./fetch-pdfium.sh --dry-run                    Show what would be downloaded
#   ./fetch-pdfium.sh --list-versions              Show the latest available version
#
# Supported architecture tags:
#   windows-x64, windows-arm64, linux-x64, linux-arm64, macos-x64, macos-arm64
#
# Output directory structure:
#   build/pdfium/{platform}/{library}
#     windows-x64/   → pdfium.dll
#     windows-arm64/ → pdfium.dll
#     linux-x64/     → libpdfium.so
#     linux-arm64/   → libpdfium.so
#     macos-x64/     → libpdfium.dylib
#     macos-arm64/   → libpdfium.dylib

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PDFIUM_DIR="$SCRIPT_DIR/pdfium"
API_REPO="bblanchon/pdfium-binaries"
API_URL="https://api.github.com/repos/$API_REPO/releases/latest"
DL_BASE="https://github.com/$API_REPO/releases/download"

# Architecture config: tag -> (download_platform, library_name, package_format)
# Download platform names: win-x64, win-arm64, linux-x64, linux-arm64, mac-x64, mac-arm64
# Package formats: zip (Windows), tgz (Linux/macOS)
declare -A PLATFORM_MAP=(
  [windows-x64]="win-x64|pdfium.dll|tgz"
  [windows-arm64]="win-arm64|pdfium.dll|tgz"
  [linux-x64]="linux-x64|libpdfium.so|tgz"
  [linux-arm64]="linux-arm64|libpdfium.so|tgz"
  [macos-x64]="mac-x64|libpdfium.dylib|tgz"
  [macos-arm64]="mac-arm64|libpdfium.dylib|tgz"
)

# Colors
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'; CYAN='\033[0;36m'; NC='\033[0m'

log()  { echo -e "${GREEN}[pdfium]${NC} $*"; }
warn() { echo -e "${YELLOW}[WARN]${NC} $*"; }
err()  { echo -e "${RED}[ERROR]${NC} $*"; }

# ── Fetch latest version ──────────────────────────────────────────
get_latest_version() {
  if ! command -v curl &>/dev/null; then
    err "curl is required. Please install it first."
    exit 1
  fi
  local ver
  ver=$(curl -sf "$API_URL" \
    | grep '"tag_name"' \
    | sed 's/.*"tag_name": *"chromium\/\([^"]*\)".*/\1/')
  if [ -z "$ver" ]; then
    err "Cannot fetch PDFium version. Check your network connection."
    exit 1
  fi
  echo "$ver"
}

# ── Download and extract a single architecture ────────────────────
download_arch() {
  local arch_tag="$1"
  local mapping="${PLATFORM_MAP[$arch_tag]:-}"
  if [ -z "$mapping" ]; then
    err "Unsupported architecture: $arch_tag"
    echo "Supported architectures: ${!PLATFORM_MAP[*]}" | tr ' ' '\n' | sort
    exit 1
  fi

  IFS='|' read -r dl_platform lib_name pkg_fmt <<< "$mapping"
  local target_dir="$PDFIUM_DIR/$arch_tag"
  local lib_path="$target_dir/$lib_name"

  if [ "${DRY_RUN:-false}" = true ]; then
    echo "  [DRY-RUN] Would download to: $lib_path"
    return
  fi

  # Skip if exists
  if [ -f "$lib_path" ] && [ "${FORCE:-false}" = false ]; then
    log "Already exists: $lib_path (use --force to re-download)"
    return
  fi

  mkdir -p "$target_dir"

  local tmp_dir
  tmp_dir=$(mktemp -d)

  local dl_url
  if [ "$pkg_fmt" = "zip" ]; then
    dl_url="$DL_BASE/chromium/$VERSION/pdfium-$dl_platform.zip"
  else
    dl_url="$DL_BASE/chromium/$VERSION/pdfium-$dl_platform.tgz"
  fi

  log "Downloading $arch_tag ..."
  echo "   URL: $dl_url"
  curl -fL "$dl_url" -o "$tmp_dir/pdfium.$pkg_fmt" --progress-bar

  log "Extracting $arch_tag ..."
  if [ "$pkg_fmt" = "zip" ]; then
    unzip -q -o "$tmp_dir/pdfium.zip" -d "$tmp_dir/extracted"
    find "$tmp_dir/extracted" -name "$lib_name" -exec cp {} "$lib_path" \; -quit
  else
    mkdir -p "$tmp_dir/extracted"
    tar -xzf "$tmp_dir/pdfium.tgz" -C "$tmp_dir/extracted"
    find "$tmp_dir/extracted" -name "$lib_name" -exec cp {} "$lib_path" \; -quit
  fi

  rm -rf "$tmp_dir"

  if [ -f "$lib_path" ]; then
    local fsize
    fsize=$(stat -c%s "$lib_path" 2>/dev/null || stat -f%z "$lib_path" 2>/dev/null)
    log "OK $arch_tag -> $lib_path ($(numfmt --to=iec-i 2>/dev/null || echo "$fsize bytes"))"
  else
    err "Downloaded but could not find $lib_name in the archive"
    exit 1
  fi
}

# ── Main ──────────────────────────────────────────────────────────
main() {
  local targets=()

  # Parse arguments
  while [ $# -gt 0 ]; do
    case "$1" in
      --dry-run) DRY_RUN=true; shift ;;
      --force)   FORCE=true; shift ;;
      --list-versions)
        echo "Latest version: $(get_latest_version)"
        exit 0
        ;;
      -*)
        err "Unknown option: $1"
        echo "Usage: $0 [--dry-run] [--force] [architecture tags...]"
        exit 1
        ;;
      *)
        targets+=("$1")
        shift
        ;;
    esac
  done

  # Default: download all architectures
  if [ ${#targets[@]} -eq 0 ]; then
    targets=("${!PLATFORM_MAP[@]}")
  fi

  # Validate architecture tags
  for t in "${targets[@]}"; do
    if [ -z "${PLATFORM_MAP[$t]:-}" ]; then
      err "Unsupported architecture: $t"
      echo "Supported architectures: ${!PLATFORM_MAP[*]}" | tr ' ' '\n' | sort
      exit 1
    fi
  done

  # Fetch latest version
  echo ""
  echo "============================================="
  echo "  PDFium Downloader"
  echo "============================================="
  VERSION=$(get_latest_version)
  log "Latest version: chromium/$VERSION"
  echo ""

  if [ "${DRY_RUN:-false}" = true ]; then
    log "DRY-RUN mode — no files will be downloaded"
    echo ""
  fi

  for t in "${targets[@]}"; do
    download_arch "$t"
  done

  echo ""
  log "All done."
  if [ "${DRY_RUN:-false}" = false ]; then
    echo "All PDFium libraries saved to: $PDFIUM_DIR"
  fi
}

main "$@"