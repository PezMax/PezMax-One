@echo off
chcp 65001 >nul
title PezMax Build Script

echo ============================================
echo   PezMax 全量构建脚本 (Windows)
echo   构建 Rust 前端 + Java 后端
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

rem ─── 1. 构建 Rust 前端 ────────────────────────────────
echo [1/2] 构建 Rust 前端...

set "CARGO_TARGET_DIR=%RUST_TARGET_DIR%"

cd /d "%ROOT_DIR%"

cargo build --release
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Rust 前端构建失败！
    exit /b 1
)

echo [OK] Rust 前端构建成功

rem 复制二进制到 dist
copy "%RUST_TARGET_DIR%\release\pezmax-egui.exe" "%DIST_DIR%\pezmax-egui.exe" >nul
echo [OK] 前端二进制已复制到 %DIST_DIR%\pezmax-egui.exe

rem ─── 2. 构建 Java 后端 ────────────────────────────────
echo [2/2] 构建 Java 后端...

cd /d "%ROOT_DIR%\PezMax-Java"

call mvnw.cmd clean package -DskipTests
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Java 后端构建失败！
    exit /b 1
)

echo [OK] Java 后端构建成功

rem 复制 JAR 到 dist（取第一个非 sources/javadoc 的 jar）
for /r "ruoyi-admin\target" %%f in (*.jar) do (
    set "JAR_NAME=%%~nxf"
    echo !JAR_NAME! | findstr /v "sources javadoc" >nul
    if not errorlevel 1 (
        copy "%%f" "%DIST_DIR%\ruoyi-admin.jar" >nul
        goto :jar_copied
    )
)
:jar_copied

echo [OK] 后端 JAR 已复制到 %DIST_DIR%\ruoyi-admin.jar

rem ─── 完成 ────────────────────────────────────────────
echo ============================================
echo   构建完成！
echo   输出目录: %DIST_DIR%
echo     - pezmax-egui.exe  (Rust 前端)
echo     - ruoyi-admin.jar  (Java 后端)
echo ============================================

cd /d "%ROOT_DIR%"
endlocal
pause