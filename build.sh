#!/bin/bash
set -e

echo "============================================"
echo "  PezMax 全量构建脚本 (Linux)"
echo "  构建 Rust 前端 + Java 后端"
echo "============================================"
echo ""

ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"
BUILD_DIR="$ROOT_DIR/build"
DIST_DIR="$BUILD_DIR/dist"
RUST_TARGET_DIR="$BUILD_DIR/rust-target"

# 清理旧的 dist 目录
rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

# ─── 1. 构建 Rust 前端 ────────────────────────────────
echo "[1/2] 构建 Rust 前端..."

export CARGO_TARGET_DIR="$RUST_TARGET_DIR"

cd "$ROOT_DIR"

cargo build --release
echo "[OK] Rust 前端构建成功"

# 复制二进制到 dist
cp "$RUST_TARGET_DIR/release/pezmax-egui" "$DIST_DIR/pezmax-egui"
echo "[OK] 前端二进制已复制到 $DIST_DIR/pezmax-egui"

# ─── 2. 构建 Java 后端 ────────────────────────────────
echo "[2/2] 构建 Java 后端..."

cd "$ROOT_DIR/PezMax-Java"

chmod +x mvnw
./mvnw clean package -DskipTests
echo "[OK] Java 后端构建成功"

# 复制 JAR 到 dist
JAR_FILE=$(find ruoyi-admin/target -name "*.jar" ! -name "*sources*" ! -name "*javadoc*" 2>/dev/null | head -1)
if [ -n "$JAR_FILE" ]; then
    cp "$JAR_FILE" "$DIST_DIR/ruoyi-admin.jar"
    echo "[OK] 后端 JAR 已复制到 $DIST_DIR/ruoyi-admin.jar"
else
    echo "[WARN] 未找到后端 JAR 文件，请检查构建输出"
fi

# ─── 完成 ────────────────────────────────────────────
echo "============================================"
echo "  构建完成！"
echo "  输出目录: $DIST_DIR"
echo "    - pezmax-egui       (Rust 前端)"
echo "    - ruoyi-admin.jar   (Java 后端)"
echo "============================================"

cd "$ROOT_DIR"