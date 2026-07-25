@echo off
setlocal enabledelayedexpansion
title PezMax Windows Build Script

REM ============================================
REM  build-windows.bat — Build PezMax for Windows
REM  Output: MSI installer + ZIP portable archive
REM ============================================

set "SCRIPT_DIR=%~dp0"
if "!SCRIPT_DIR:~-1!"=="\" set "SCRIPT_DIR=!SCRIPT_DIR:~0,-1!"

set "ROOT_DIR=!SCRIPT_DIR!\.."
set "PDFIUM_DIR=!SCRIPT_DIR!\pdfium"
set "DIST_DIR=!SCRIPT_DIR!\dist"
set "RUST_TARGET=!SCRIPT_DIR!\rust-target"

REM Detect architecture
if /i "%PROCESSOR_ARCHITECTURE%"=="AMD64" (
  set "ARCH_TAG=x64"
  set "PDF_PLATFORM=windows-x64"
) else if /i "%PROCESSOR_ARCHITECTURE%"=="ARM64" (
  set "ARCH_TAG=arm64"
  set "PDF_PLATFORM=windows-arm64"
) else (
  echo [ERROR] Unsupported architecture: %PROCESSOR_ARCHITECTURE%
  exit /b 1
)

set "PDFIUM_LIB_DIR=!PDFIUM_DIR!\!PDF_PLATFORM!"
set "PDFIUM_DLL=!PDFIUM_LIB_DIR!\pdfium.dll"

echo ============================================
echo   PezMax Windows Build (!PDF_PLATFORM!)
echo   Output: MSI installer
echo ============================================

REM Download pdfium if missing
if not exist "!PDFIUM_DLL!" (
  echo [pdfium] Prebuilt library not found, downloading...
  call "!SCRIPT_DIR!\fetch-pdfium.bat" !PDF_PLATFORM!
  if not exist "!PDFIUM_DLL!" (
    echo [ERROR] pdfium download failed
    exit /b 1
  )
  echo [pdfium] Cached to: !PDFIUM_DLL!
) else (
  echo [pdfium] Using existing library: !PDFIUM_DLL!
)

REM Build Rust
echo [build] cargo build --release ...
cd /d "!ROOT_DIR!"
set "CARGO_TARGET_DIR=!RUST_TARGET!"
cargo build --release
if %ERRORLEVEL% neq 0 (
  echo [ERROR] cargo build failed
  exit /b 1
)
echo [build] Build successful

REM Copy pdfium.dll to target directory for cargo wix to bundle
echo [msi] Copying pdfium.dll to target directory...
copy /Y "!PDFIUM_DLL!" "!RUST_TARGET!\release\pdfium.dll" >nul
if %ERRORLEVEL% neq 0 (
  echo [ERROR] Failed to copy pdfium.dll
  exit /b 1
)
echo [msi] pdfium.dll ready

REM Build MSI installer
echo [msi] Building MSI installer...
if not exist "!DIST_DIR!" mkdir "!DIST_DIR!"

cd /d "!ROOT_DIR!"
set "WIX_BIN=C:\Program Files (x86)\WiX Toolset v3.14\bin"
if exist "!WIX_BIN!\candle.exe" (
  echo [msi] WiX Toolset found at: !WIX_BIN!
  set "PATH=!WIX_BIN!;!PATH!"
) else (
  echo [WARN] WiX Toolset not found at default path
)
cargo wix --no-build --target-bin-dir "!RUST_TARGET!\release" --bin-path "!WIX_BIN!" --output "!DIST_DIR!\PezMax-!ARCH_TAG!.msi" --nocapture
if %ERRORLEVEL% neq 0 (
  echo [ERROR] MSI build failed
  echo.
  echo Hint: Install WiX Toolset v3 first (run as Administrator):
  echo   choco install wixtoolset -y
  echo   or
  echo   winget install WiXToolset.WiXToolset --accept-source-agreements
  exit /b 1
)

REM Also create a ZIP portable archive
set "OUT_DIR=!DIST_DIR!\pezmax-windows-!ARCH_TAG!"
if exist "!OUT_DIR!" rmdir /s /q "!OUT_DIR!"
mkdir "!OUT_DIR!"

copy /Y "!RUST_TARGET!\release\pezmax-egui.exe" "!OUT_DIR!\pezmax-egui.exe" >nul
copy /Y "!PDFIUM_DLL!" "!OUT_DIR!\pdfium.dll" >nul

set "ARCHIVE=!DIST_DIR!\pezmax-windows-!ARCH_TAG!.zip"
if exist "!ARCHIVE!" del "!ARCHIVE!"
powershell -NoProfile -Command "Compress-Archive -Path '!OUT_DIR!\*' -DestinationPath '!ARCHIVE!'" >nul

echo.
echo ============================================
echo   Build complete
echo.
echo   MSI installer:  !DIST_DIR!\PezMax-!ARCH_TAG!.msi
echo   Portable ZIP:   !ARCHIVE!
echo   Run directly:   !OUT_DIR!\pezmax-egui.exe
echo ============================================

endlocal
pause