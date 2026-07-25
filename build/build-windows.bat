@echo off
chcp 65001 >nul
setlocal enabledelayedexpansion
title PezMax Windows 构建脚本

set "SCRIPT_DIR=%~dp0"
rem 去掉末尾反斜杠
if "!SCRIPT_DIR:~-1!"=="\" set "SCRIPT_DIR=!SCRIPT_DIR:~0,-1!"

set "ROOT_DIR=!SCRIPT_DIR!\.."
set "PDFIUM_DIR=!SCRIPT_DIR!\pdfium"
set "DIST_DIR=!SCRIPT_DIR!\dist"
set "RUST_TARGET=!SCRIPT_DIR!\rust-target"

rem ── 检测架构 ──────────────────────────────────────────────────
if /i "%PROCESSOR_ARCHITECTURE%"=="AMD64" (
  set "ARCH_TAG=x64"
  set "PDF_PLATFORM=windows-x64"
  set "PDF_DL_NAME=win-x64"
) else if /i "%PROCESSOR_ARCHITECTURE%"=="ARM64" (
  set "ARCH_TAG=arm64"
  set "PDF_PLATFORM=windows-arm64"
  set "PDF_DL_NAME=win-arm64"
) else (
  echo [ERROR] 不支持的架构: %PROCESSOR_ARCHITECTURE%
  exit /b 1
)

set "PDFIUM_LIB_DIR=!PDFIUM_DIR!\!PDF_PLATFORM!"
set "PDFIUM_DLL=!PDFIUM_LIB_DIR!\pdfium.dll"

echo ============================================
echo   PezMax Windows 构建脚本 (!PDF_PLATFORM!)
echo ============================================

rem ── 下载 pdfium（缺失时自动获取）────────────────────────────
if not exist "!PDFIUM_DLL!" (
  echo [pdfium] 未找到预编译库，从 GitHub 下载...

  rem 获取最新版本号
  for /f "usebackq delims=" %%v in (`powershell -NoProfile -Command ^
    "(Invoke-RestMethod 'https://api.github.com/repos/bblanchon/pdfium-binaries/releases/latest').tag_name -replace 'chromium/',''"`) do (
    set "PDFIUM_VER=%%v"
  )
  if "!PDFIUM_VER!"=="" (
    echo [ERROR] 无法获取 pdfium 版本，请检查网络
    exit /b 1
  )
  echo [pdfium] 版本: chromium/!PDFIUM_VER!  平台: !PDF_DL_NAME!

  if not exist "!PDFIUM_LIB_DIR!" mkdir "!PDFIUM_LIB_DIR!"

  rem 下载并解压
  powershell -NoProfile -Command ^
    "$url = 'https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%%2F!PDFIUM_VER!/pdfium-!PDF_DL_NAME!.zip'; ^
     $tmp = Join-Path $env:TEMP 'pdfium_dl'; ^
     New-Item -ItemType Directory -Force -Path $tmp | Out-Null; ^
     Invoke-WebRequest $url -OutFile (Join-Path $tmp 'pdfium.zip'); ^
     Expand-Archive (Join-Path $tmp 'pdfium.zip') -DestinationPath (Join-Path $tmp 'extracted') -Force; ^
     Copy-Item (Join-Path $tmp 'extracted\bin\pdfium.dll') '!PDFIUM_DLL!'; ^
     Remove-Item $tmp -Recurse -Force"
  if %ERRORLEVEL% neq 0 (
    echo [ERROR] pdfium 下载失败
    exit /b 1
  )
  echo [pdfium] 已缓存到 !PDFIUM_DLL!
) else (
  echo [pdfium] 使用已有库: !PDFIUM_DLL!
)

rem ── 构建 Rust ──────────────────────────────────────────────
echo [build] cargo build --release ...
cd /d "!ROOT_DIR!"
set "CARGO_TARGET_DIR=!RUST_TARGET!"
cargo build --release
if %ERRORLEVEL% neq 0 (
  echo [ERROR] cargo build 失败
  exit /b 1
)
echo [build] 构建成功

rem ── 组装 dist 目录 ────────────────────────────────────────
set "OUT_DIR=!DIST_DIR!\pezmax-windows-!ARCH_TAG!"
if exist "!OUT_DIR!" rmdir /s /q "!OUT_DIR!"
mkdir "!OUT_DIR!"

copy /Y "!RUST_TARGET!\release\pezmax-egui.exe" "!OUT_DIR!\pezmax-egui.exe" >nul
copy /Y "!PDFIUM_DLL!" "!OUT_DIR!\pdfium.dll" >nul

rem Windows 下 DLL 与 exe 同目录时会自动被找到，无需启动脚本

rem ── 打包为 zip ────────────────────────────────────────────
set "ARCHIVE=!DIST_DIR!\pezmax-windows-!ARCH_TAG!.zip"
if exist "!ARCHIVE!" del "!ARCHIVE!"
powershell -NoProfile -Command ^
  "Compress-Archive -Path '!OUT_DIR!\*' -DestinationPath '!ARCHIVE!'"
if %ERRORLEVEL% neq 0 (
  echo [ERROR] 打包失败
  exit /b 1
)

echo.
echo ============================================
echo   构建完成
echo   输出: !ARCHIVE!
echo   直接运行 pezmax-egui.exe 即可（pdfium.dll 须在同目录）
echo ============================================

endlocal
pause
