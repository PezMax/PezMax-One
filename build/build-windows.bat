@echo off
setlocal enabledelayedexpansion
chcp 65001 >nul 2>&1
title PezMax One Windows Build

REM ============================================
REM  build-windows.bat - Build PezMax One for Windows
REM  Output: MSI installer + ZIP portable, per architecture
REM
REM  产物命名（与客户端 auto-update pick_asset() 对齐）：
REM    pezmax-one-VERSION-x86_64.msi   pezmax-one-VERSION-aarch64.msi
REM    pezmax-one-VERSION-x86_64.zip   pezmax-one-VERSION-aarch64.zip
REM
REM  Usage:
REM    build-windows.bat            Build for host arch only
REM    build-windows.bat x64        Build for x86_64 (64-bit AMD/Intel)
REM    build-windows.bat arm64      Build for aarch64 (64-bit ARM)
REM    build-windows.bat all        Build for both x64 + arm64
REM
REM  Note: 32-bit x86 (i686) is not supported — pdfium-binaries doesn't
REM        ship a 32-bit Windows prebuilt, and modern Windows apps target 64-bit.
REM
REM  Requirements:
REM    - Rust + cargo
REM    - PowerShell (for pdfium download)
REM    - WiX Toolset v3 (for MSI packaging)
REM    - MSVC Build Tools with the target architecture components
REM
REM  Environment:
REM    FORCE_PDFIUM=1     Re-download pdfium even if cached
REM ============================================

set "SCRIPT_DIR=%~dp0"
if "!SCRIPT_DIR:~-1!"=="\" set "SCRIPT_DIR=!SCRIPT_DIR:~0,-1!"

set "ROOT_DIR=!SCRIPT_DIR!\.."
set "PDFIUM_DIR=!SCRIPT_DIR!\pdfium"
set "DIST_DIR=!SCRIPT_DIR!\dist"
set "RUST_TARGET=!SCRIPT_DIR!\rust-target"

set "BIN_NAME=pezmax-one"
set "PKG_NAME=pezmax-one"
set "PKG_REPO=bblanchon/pdfium-binaries"

REM ── 读取 Cargo.toml 版本号 ─────────────────────
REM 注意：不要在 for /f 块内用 goto :label，cmd 解析器会因 ) 未闭合报
REM   ") was unexpected at this time"。改用 "第一次匹配后不再赋值" 的模式。
REM   `findstr /b /c:"version = "` 精确匹配以 'version = ' 起首的行（Cargo.toml
REM   里只有 [package] 节的 version 行如此），tokens=3 拆出 `"1.0.0"`，%%~V 去引号。
set "VERSION="
for /f "tokens=3 delims= " %%V in ('findstr /b /c:"version = " "!ROOT_DIR!\Cargo.toml"') do (
  if not defined VERSION set "VERSION=%%~V"
)
if not defined VERSION (
  echo [ERROR] Failed to extract VERSION from Cargo.toml
  exit /b 1
)
echo [info]  Package version: !VERSION!

REM ── Detect host architecture ──────────────────
if /i "%PROCESSOR_ARCHITECTURE%"=="AMD64" (
  set "HOST_ARCH=x64"
) else if /i "%PROCESSOR_ARCHITECTURE%"=="ARM64" (
  set "HOST_ARCH=arm64"
) else (
  echo [ERROR] Unsupported host architecture: %PROCESSOR_ARCHITECTURE%
  exit /b 1
)

REM ── Parse arguments ───────────────────────────
REM 关键：cmd.exe 不允许在括号块 () 内放 :label / goto。上一版本把
REM parse_loop 放进 else (...) 里就是 ") was unexpected at this time" 报错源之二。
REM 现在把循环放在顶层，无参再走默认赋值。
set "ARCHES="

:parse_loop
if "%~1"=="" goto :parse_done
if /i "%~1"=="x64"   set "ARCHES=!ARCHES! x64"   & shift & goto :parse_loop
if /i "%~1"=="arm64" set "ARCHES=!ARCHES! arm64" & shift & goto :parse_loop
if /i "%~1"=="all"   set "ARCHES=x64 arm64"      & shift & goto :parse_loop
echo [ERROR] Unknown argument: %~1 ^(supported: x64 / arm64 / all^)
echo         Note: 32-bit x86 not supported (no pdfium prebuilt);
echo               use "x64" for 64-bit AMD, "arm64" for 64-bit ARM.
exit /b 1

:parse_done
if not defined ARCHES set "ARCHES=!HOST_ARCH!"

if not exist "!DIST_DIR!" mkdir "!DIST_DIR!"

echo [info]  Host arch: !HOST_ARCH!
echo [info]  Will build:!ARCHES!

REM ── Preflight: check MSVC toolchain for each arch ─────
REM Rust MSVC targets require the corresponding MSVC ARM64/AMD64 build tool
REM component. link.exe alone is not enough — the target toolchain has to be
REM installed via Visual Studio Installer (Individual Components > MSVC vXXX).
REM We only warn here; cargo will error clearly if a component is missing.
where link.exe >nul 2>&1
if errorlevel 1 (
  echo [warn]  link.exe not found on PATH. Run this from "Developer Command
  echo         Prompt for VS" or install "Visual Studio Build Tools" first.
)

REM ── Build each architecture ─────────────────────
set "FAILED_ARCHES="
for %%A in (!ARCHES!) do (
  set "ARCH=%%A"
  call :build_one !ARCH!
  if errorlevel 1 set "FAILED_ARCHES=!FAILED_ARCHES! !ARCH!"
)

echo.
if not "!FAILED_ARCHES!"=="" (
  echo [warn]  Failed architectures:!FAILED_ARCHES!
)
echo [info]  Artifacts:
dir /b "!DIST_DIR!\pezmax-one-*.msi" "!DIST_DIR!\pezmax-one-*.zip" 2>nul

endlocal
pause
exit /b 0

REM ============================================
REM  Subroutine: build one architecture
REM  Arg1: arch tag (x64 / arm64)
REM ============================================
:build_one
setlocal enabledelayedexpansion
set "ARCH=%~1"

if /i "!ARCH!"=="x64" (
  set "RUST_TRIPLE=x86_64-pc-windows-msvc"
  set "PDF_PLATFORM=windows-x64"
  set "CANON_ARCH=x86_64"
) else if /i "!ARCH!"=="arm64" (
  set "RUST_TRIPLE=aarch64-pc-windows-msvc"
  set "PDF_PLATFORM=windows-arm64"
  set "CANON_ARCH=aarch64"
) else (
  echo [ERROR] Invalid arch: !ARCH!
  exit /b 1
)

echo.
echo ============================================================
echo   PezMax One Windows Build - !ARCH! (!RUST_TRIPLE!)
echo ============================================================

REM ── rustup target ─────────────────────────────
rustup target list --installed 2>nul | findstr /b /c:"!RUST_TRIPLE!" >nul
if errorlevel 1 (
  echo [warn]  Rust target !RUST_TRIPLE! missing. Running rustup target add ...
  rustup target add !RUST_TRIPLE!
  if errorlevel 1 (
    echo [ERROR] rustup target add failed for !RUST_TRIPLE!
    exit /b 1
  )
)

REM ── pdfium fetch (inline) ─────────────────────
set "PDFIUM_LIB_DIR=!PDFIUM_DIR!\!PDF_PLATFORM!"
set "PDFIUM_DLL=!PDFIUM_LIB_DIR!\pdfium.dll"
set "SKIP_FETCH=0"
if exist "!PDFIUM_DLL!" if /i not "!FORCE_PDFIUM!"=="1" set "SKIP_FETCH=1"

if "!SKIP_FETCH!"=="1" (
  echo [info]  pdfium exists: !PDFIUM_DLL!
) else (
  echo [pdfium] Fetching latest version ...
  for /f "usebackq delims=" %%v in (`powershell -NoProfile -Command "try { $v = (Invoke-RestMethod 'https://api.github.com/repos/!PKG_REPO!/releases/latest').tag_name; Write-Output ($v -replace 'chromium/','') } catch { Write-Output 'ERROR' }"`) do set "PDFIUM_VER=%%v"
  if "!PDFIUM_VER!"=="ERROR" (
    echo [ERROR] Failed to query pdfium version
    exit /b 1
  )
  echo [pdfium] chromium/!PDFIUM_VER!

  set "DL_URL=https://github.com/!PKG_REPO!/releases/download/chromium/!PDFIUM_VER!/pdfium-!PDF_PLATFORM!.tgz"
  set "TMP_TGZ=%TEMP%\pdfium-!PDF_PLATFORM!.tgz"
  set "TMP_EXTRACT=%TEMP%\pdfium-!PDF_PLATFORM!-extract"

  echo [pdfium] Downloading !DL_URL!
  powershell -NoProfile -Command "Invoke-WebRequest -Uri '!DL_URL!' -OutFile '!TMP_TGZ!'"
  if errorlevel 1 (
    echo [ERROR] pdfium download failed
    exit /b 1
  )

  if exist "!TMP_EXTRACT!" rmdir /s /q "!TMP_EXTRACT!"
  mkdir "!TMP_EXTRACT!"
  tar -xzf "!TMP_TGZ!" -C "!TMP_EXTRACT!"
  if not exist "!PDFIUM_LIB_DIR!" mkdir "!PDFIUM_LIB_DIR!"
  REM 找到 pdfium.dll 并复制（不能在括号块内用 :label + goto）
  REM 用旗标模式：找到第一个后设 COPIED=1，之后跳过
  set "COPIED=0"
  for /r "!TMP_EXTRACT!" %%f in (pdfium.dll) do (
    if "!COPIED!"=="0" (
      copy /Y "%%f" "!PDFIUM_DLL!" >nul
      set "COPIED=1"
    )
  )
  del /q "!TMP_TGZ!" 2>nul
  rmdir /s /q "!TMP_EXTRACT!" 2>nul

  if not exist "!PDFIUM_DLL!" (
    echo [ERROR] pdfium.dll not found in archive
    exit /b 1
  )
  echo [pdfium] Ready: !PDFIUM_DLL!
)

REM ── cargo build ───────────────────────────────
echo [build] cargo build --release --target !RUST_TRIPLE!
cd /d "!ROOT_DIR!"
set "CARGO_TARGET_DIR=!RUST_TARGET!"
cargo build --release --target !RUST_TRIPLE!
if errorlevel 1 (
  echo [ERROR] cargo build failed for !ARCH!
  echo         If the error mentions link.exe / MSVC target, install the
  echo         "MSVC vXXX - VS 20XX C++ !ARCH! build tools" component via
  echo         Visual Studio Installer.
  exit /b 1
)

set "TARGET_RELEASE=!RUST_TARGET!\!RUST_TRIPLE!\release"
if not exist "!TARGET_RELEASE!\!BIN_NAME!.exe" (
  echo [ERROR] Built binary missing: !TARGET_RELEASE!\!BIN_NAME!.exe
  exit /b 1
)

REM Copy pdfium.dll next to the binary so cargo-wix can bundle it
copy /Y "!PDFIUM_DLL!" "!TARGET_RELEASE!\pdfium.dll" >nul
if errorlevel 1 (
  echo [ERROR] Failed to copy pdfium.dll into release dir
  exit /b 1
)

REM ── MSI (cargo wix) ───────────────────────────
set "WIX_BIN=C:\Program Files (x86)\WiX Toolset v3.14\bin"
if exist "!WIX_BIN!\candle.exe" (
  echo [msi]   WiX Toolset found: !WIX_BIN!
  set "PATH=!WIX_BIN!;!PATH!"
) else (
  echo [warn]  WiX Toolset not at default path. Install via:
  echo         choco install wixtoolset -y
  echo         or winget install WiXToolset.WiXToolset --accept-source-agreements
)
cd /d "!ROOT_DIR!"
cargo wix --no-build --target-bin-dir "!TARGET_RELEASE!" --bin-path "!WIX_BIN!" --output "!DIST_DIR!\!PKG_NAME!-!VERSION!-!CANON_ARCH!.msi" --nocapture
if errorlevel 1 (
  echo [ERROR] MSI build failed for !ARCH!
  exit /b 1
)

REM ── ZIP portable ──────────────────────────────
set "OUT_DIR=!DIST_DIR!\!PKG_NAME!-!VERSION!-!CANON_ARCH!"
if exist "!OUT_DIR!" rmdir /s /q "!OUT_DIR!"
mkdir "!OUT_DIR!"

copy /Y "!TARGET_RELEASE!\!BIN_NAME!.exe" "!OUT_DIR!\!BIN_NAME!.exe" >nul
copy /Y "!PDFIUM_DLL!" "!OUT_DIR!\pdfium.dll" >nul

set "ARCHIVE=!DIST_DIR!\!PKG_NAME!-!VERSION!-!CANON_ARCH!.zip"
if exist "!ARCHIVE!" del "!ARCHIVE!"
powershell -NoProfile -Command "Compress-Archive -Path '!OUT_DIR!\*' -DestinationPath '!ARCHIVE!'" >nul

echo [done]  MSI:  !DIST_DIR!\!PKG_NAME!-!VERSION!-!CANON_ARCH!.msi
echo [done]  ZIP:  !ARCHIVE!

endlocal
exit /b 0
