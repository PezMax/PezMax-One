@echo off
chcp 65001 >nul
title PezMax Build Script

echo ============================================
echo   PezMax Build Script (Windows)
echo   构建 Rust 前端
echo   远程后端 http://154.8.139.48:8080 已在运行
echo   如需本地构建后端，执行 build-backend.bat
echo ============================================
echo.

setlocal enabledelayedexpansion

set "ROOT_DIR=%~dp0"
set "BUILD_DIR=%ROOT_DIR%build"
set "DIST_DIR=%BUILD_DIR%\dist"
set "RUST_TARGET_DIR=%BUILD_DIR%\rust-target"

rem 清理旧的 dist 目录
if exist "%DIST_DIR%" rmdir /s /q "%DIST_DIR%"
mkdir "%DIST_DIR%"

rem ─── 构建 Rust 前端 ──────────────────────────────────
echo [1/1] Building Rust frontend...

set "CARGO_TARGET_DIR=%RUST_TARGET_DIR%"

cd /d "%ROOT_DIR%"

cargo build --release
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Rust frontend build failed!
    exit /b 1
)

echo [OK] Rust frontend built successfully

rem 复制二进制到 dist
copy "%RUST_TARGET_DIR%\release\pezmax-egui.exe" "%DIST_DIR%\pezmax-egui.exe" >nul
echo [OK] Frontend binary copied to %DIST_DIR%\pezmax-egui.exe

rem ─── 完成 ────────────────────────────────────────────
echo ============================================
echo   Build complete!
echo   Output: %DIST_DIR%\pezmax-egui.exe
echo ============================================

cd /d "%ROOT_DIR%"
endlocal
pause