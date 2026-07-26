@echo off
setlocal enabledelayedexpansion
title PezMax One Windows Build

REM ============================================================
REM  build-windows.bat -- Build PezMax One for Windows
REM  Output: MSI installer + ZIP portable, per architecture
REM
REM  Naming (aligned with auto-update pick_asset()):
REM    pezmax-one-VERSION-x86_64.msi   pezmax-one-VERSION-aarch64.msi
REM    pezmax-one-VERSION-x86_64.zip   pezmax-one-VERSION-aarch64.zip
REM
REM  Usage:
REM    build-windows.bat              Build for host arch
REM    build-windows.bat x64          Build x86_64 (64-bit AMD/Intel)
REM    build-windows.bat arm64        Build aarch64 (64-bit ARM)
REM    build-windows.bat x64 arm64    Build both
REM    build-windows.bat all          Same as "x64 arm64"
REM
REM  Requirements:
REM    - Rust + cargo
REM    - PowerShell (for pdfium download)
REM    - WiX Toolset v3 (for MSI packaging)
REM    - MSVC Build Tools (for MSVC mode, optional for GNU)
REM
REM  Environment:
REM    FORCE_PDFIUM=1   Re-download pdfium even if cached
REM ============================================================

set "SCRIPT_DIR=%~dp0"
if "!SCRIPT_DIR:~-1!"=="\" set "SCRIPT_DIR=!SCRIPT_DIR:~0,-1!"

set "ROOT_DIR=!SCRIPT_DIR!\.."
set "PDFIUM_DIR=!SCRIPT_DIR!\pdfium"
set "DIST_DIR=!SCRIPT_DIR!\dist"
set "RUST_TARGET=!SCRIPT_DIR!\rust-target"

set "BIN_NAME=pezmax-one"
set "PKG_NAME=pezmax-one"
set "PKG_REPO=bblanchon/pdfium-binaries"

REM ---- Read version from Cargo.toml ----
set "VERSION="
for /f "tokens=3 delims= " %%V in ('findstr /b /c:"version = " "!ROOT_DIR!\Cargo.toml"') do (
  if not defined VERSION set "VERSION=%%~V"
)
if not defined VERSION (
  echo [ERROR] Failed to extract VERSION from Cargo.toml
  exit /b 1
)
echo [info]  Package version: !VERSION!

REM ---- Detect host architecture ----
if /i "%PROCESSOR_ARCHITECTURE%"=="AMD64" (
  set "HOST_ARCH=x64"
) else if /i "%PROCESSOR_ARCHITECTURE%"=="ARM64" (
  set "HOST_ARCH=arm64"
) else (
  echo [ERROR] Unsupported host architecture: %PROCESSOR_ARCHITECTURE%
  exit /b 1
)

REM ---- Parse arguments ----
set "ARCHES="
:parse_loop
if "%~1"=="" goto :parse_done
if /i "%~1"=="x64"   set "ARCHES=!ARCHES! x64"   & shift & goto :parse_loop
if /i "%~1"=="arm64" set "ARCHES=!ARCHES! arm64" & shift & goto :parse_loop
if /i "%~1"=="all"   set "ARCHES=x64 arm64"      & shift & goto :parse_loop
echo [ERROR] Unknown argument: %~1 ^(supported: x64 / arm64 / all^)
exit /b 1
:parse_done
if not defined ARCHES set "ARCHES=!HOST_ARCH!"

echo [info]  Host arch: !HOST_ARCH!
echo [info]  Will build:!ARCHES!

if not exist "!DIST_DIR!" mkdir "!DIST_DIR!"

REM ---- Detect MSVC availability ----
REM Check for real MSVC link.exe (not Git's /usr/bin/link.exe)
set "MSVC_AVAILABLE=0"
where cl.exe >nul 2>&1
if not errorlevel 1 (
  set "MSVC_AVAILABLE=1"
) else (
  REM Try to activate VS 2022 dev environment
  set "VSWHERE=C:\Program Files (x86)\Microsoft Visual Studio\Installer\vswhere.exe"
  if exist "!VSWHERE!" (
    for /f "usebackq delims=" %%p in (`"!VSWHERE!" -latest -property installationPath`) do (
      set "VS_INSTALL=%%p"
    )
    if defined VS_INSTALL (
      call "!VS_INSTALL!\VC\Auxiliary\Build\vcvars64.bat" >nul 2>&1
      if not errorlevel 1 (
        where cl.exe >nul 2>&1
        if not errorlevel 1 set "MSVC_AVAILABLE=1"
      )
    )
  )
)

REM ---- Detect LLVM MinGW (for ARM64 cross-compilation without MSVC) ----
set "LLVM_MINGW_BIN="
set "LLVM_MINGW_AVAILABLE=0"

REM Check known installation paths
if exist "C:\Program Files\LLVM-MinGW\bin\aarch64-w64-mingw32-clang.exe" (
  set "LLVM_MINGW_BIN=C:\Program Files\LLVM-MinGW\bin"
  set "LLVM_MINGW_AVAILABLE=1"
)
if "!LLVM_MINGW_AVAILABLE!"=="0" if exist "C:\Program Files (x86)\LLVM-MinGW\bin\aarch64-w64-mingw32-clang.exe" (
  set "LLVM_MINGW_BIN=C:\Program Files (x86)\LLVM-MinGW\bin"
  set "LLVM_MINGW_AVAILABLE=1"
)

REM Check winget installation path
if "!LLVM_MINGW_AVAILABLE!"=="0" (
  set "WINGET_DIR=!LOCALAPPDATA!\Microsoft\WinGet\Packages\MartinStorsjo.LLVM-MinGW.UCRT_Microsoft.Winget.Source_8wekyb3d8bbwe"
  if exist "!WINGET_DIR!\" (
    for /d %%d in ("!WINGET_DIR!\llvm-mingw-*-ucrt-x86_64") do (
      if exist "%%d\bin\aarch64-w64-mingw32-clang.exe" (
        set "LLVM_MINGW_BIN=%%d\bin"
        set "LLVM_MINGW_AVAILABLE=1"
      )
    )
  )
)

REM Also check PATH
if "!LLVM_MINGW_AVAILABLE!"=="0" (
  where aarch64-w64-mingw32-clang.exe >nul 2>&1
  if not errorlevel 1 (
    for /f "delims=" %%p in ('where aarch64-w64-mingw32-clang.exe') do (
      set "LLVM_MINGW_BIN=%%~dpp"
      set "LLVM_MINGW_AVAILABLE=1"
    )
  )
)

REM ---- Filter buildable architectures ----
REM Like build-linux.sh: detect toolchain availability, skip gracefully.
set "BUILDABLE="
set "SKIPPED="
for %%A in (!ARCHES!) do (
  call :check_arch %%A
)
set "ARCHES=!BUILDABLE!"

if not defined ARCHES (
  echo [ERROR] No buildable architectures.
  if defined SKIPPED echo [warn]  Skipped:!SKIPPED!
  pause
  exit /b 1
)

if defined SKIPPED echo [warn]  Skipped (toolchain missing):!SKIPPED!

REM ---- Build each architecture ----
set "FAILED_ARCHES="
for %%A in (!ARCHES!) do (
  call :build_one %%A
  if errorlevel 1 set "FAILED_ARCHES=!FAILED_ARCHES! %%A"
)

echo.
if defined FAILED_ARCHES (
  echo [warn]  Failed architectures:!FAILED_ARCHES!
)
echo [info]  Artifacts:
dir /b "!DIST_DIR!\!PKG_NAME!-*.msi" "!DIST_DIR!\!PKG_NAME!-*.zip" 2>nul

endlocal
pause
exit /b 0

REM ============================================================
REM  Subroutine: check if an architecture is buildable
REM  Sets BUILDABLE / SKIPPED from parent scope
REM ============================================================
:check_arch
setlocal enabledelayedexpansion
set "A=%~1"

if /i "!A!"=="x64" (
  if "!MSVC_AVAILABLE!"=="1" (
    set "TRIPLE=x86_64-pc-windows-msvc"
  ) else (
    set "TRIPLE=x86_64-pc-windows-gnu"
  )
  REM Check if the default toolchain has this target installed
  rustup target list --installed 2>nul | findstr /b /c:"!TRIPLE!" >nul
  if errorlevel 1 (
    REM Also check filesystem (manual install)
    if exist "!USERPROFILE!\.rustup\toolchains\stable-x86_64-pc-windows-gnu\lib\rustlib\!TRIPLE!\" (
      endlocal
      set "BUILDABLE=!BUILDABLE! x64"
      exit /b 0
    )
    REM Target not installed. Try to install.
    echo [info]  Rust target !TRIPLE! not installed. Running rustup target add ...
    rustup target add !TRIPLE! 2>nul
    if errorlevel 1 (
      endlocal
      set "SKIPPED=!SKIPPED! x64(!TRIPLE!)"
      exit /b 0
    )
  )
  endlocal
  set "BUILDABLE=!BUILDABLE! x64"
  exit /b 0
)

if /i "!A!"=="arm64" (
  if "!MSVC_AVAILABLE!"=="1" (
    set "TRIPLE=aarch64-pc-windows-msvc"
  ) else if "!LLVM_MINGW_AVAILABLE!"=="1" (
    set "TRIPLE=aarch64-pc-windows-gnullvm"
  ) else (
    endlocal
    set "SKIPPED=!SKIPPED! arm64(no-toolchain)"
    exit /b 0
  )
  REM Check if target is installed (rustup list + filesystem fallback)
  rustup target list --installed 2>nul | findstr /b /c:"!TRIPLE!" >nul
  if errorlevel 1 (
    if exist "!USERPROFILE!\.rustup\toolchains\stable-x86_64-pc-windows-gnu\lib\rustlib\!TRIPLE!\" (
      endlocal
      set "BUILDABLE=!BUILDABLE! arm64"
      exit /b 0
    )
    echo [info]  Rust target !TRIPLE! not installed. Running rustup target add ...
    rustup target add !TRIPLE! 2>nul
    if errorlevel 1 (
      endlocal
      set "SKIPPED=!SKIPPED! arm64(!TRIPLE!)"
      exit /b 0
    )
  )
  endlocal
  set "BUILDABLE=!BUILDABLE! arm64"
  exit /b 0
)
  rustup target list --installed 2>nul | findstr /b /c:"!TRIPLE!" >nul
  if errorlevel 1 (
    echo [info]  Rust target !TRIPLE! not installed. Running rustup target add ...
    rustup target add !TRIPLE! 2>nul
    if errorlevel 1 (
      endlocal
      set "SKIPPED=!SKIPPED! arm64(!TRIPLE!)"
      exit /b 0
    )
  )
  endlocal
  set "BUILDABLE=!BUILDABLE! arm64"
  exit /b 0
)

endlocal
exit /b 0

REM ============================================================
REM  Subroutine: build one architecture
REM  Arg1: arch tag (x64 / arm64)
REM ============================================================
:build_one
setlocal enabledelayedexpansion
set "ARCH=%~1"

if /i "!ARCH!"=="x64" (
  if "!MSVC_AVAILABLE!"=="1" (
    set "RUST_TRIPLE=x86_64-pc-windows-msvc"
  ) else (
    set "RUST_TRIPLE=x86_64-pc-windows-gnu"
  )
  set "PDF_PLATFORM=windows-x64"
  set "CANON_ARCH=x86_64"
) else if /i "!ARCH!"=="arm64" (
  if "!MSVC_AVAILABLE!"=="1" (
    set "RUST_TRIPLE=aarch64-pc-windows-msvc"
  ) else if "!LLVM_MINGW_AVAILABLE!"=="1" (
    set "RUST_TRIPLE=aarch64-pc-windows-gnullvm"
  ) else (
    echo [ERROR] No toolchain for ARM64. Install LLVM-MinGW or MSVC ARM64 tools.
    exit /b 1
  )
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

REM ---- rustup target (check filesystem too for manual installs) ----
set "RUSTLIB_DIR=!USERPROFILE!\.rustup\toolchains\stable-x86_64-pc-windows-gnu\lib\rustlib"
rustup target list --installed 2>nul | findstr /b /c:"!RUST_TRIPLE!" >nul
if errorlevel 1 (
  if exist "!RUSTLIB_DIR!\!RUST_TRIPLE!\" (
    echo [info]  Rust target !RUST_TRIPLE! found in filesystem.
  ) else (
    echo [warn]  Rust target !RUST_TRIPLE! missing. Running rustup target add ...
    rustup target add !RUST_TRIPLE!
    if errorlevel 1 (
      echo [ERROR] rustup target add failed for !RUST_TRIPLE!
      echo         Your rustup mirror may be missing this component.
      exit /b 1
    )
  )
)

REM ---- pdfium fetch ----
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

REM ---- cargo build ----
cd /d "!ROOT_DIR!"
set "CARGO_TARGET_DIR=!RUST_TARGET!"

REM Set linker for LLVM MinGW cross-compilation
if "!RUST_TRIPLE!"=="aarch64-pc-windows-gnullvm" (
  if defined LLVM_MINGW_BIN (
    set "CARGO_TARGET_AARCH64_PC_WINDOWS_GNULLVM_LINKER=!LLVM_MINGW_BIN!\aarch64-w64-mingw32-clang.exe"
    set "PATH=!LLVM_MINGW_BIN!;!PATH!"
    echo [info]  Cross linker: !CARGO_TARGET_AARCH64_PC_WINDOWS_GNULLVM_LINKER!
  )
)

echo [build]  cargo build --release --target !RUST_TRIPLE!
cargo build --release --target !RUST_TRIPLE!
if errorlevel 1 (
  echo [ERROR] cargo build failed for !ARCH!
  exit /b 1
)

set "TARGET_RELEASE=!RUST_TARGET!\!RUST_TRIPLE!\release"
if not exist "!TARGET_RELEASE!\!BIN_NAME!.exe" (
  echo [ERROR] Built binary missing: !TARGET_RELEASE!\!BIN_NAME!.exe
  exit /b 1
)

REM Copy pdfium.dll next to the binary
copy /Y "!PDFIUM_DLL!" "!TARGET_RELEASE!\pdfium.dll" >nul
if errorlevel 1 (
  echo [ERROR] Failed to copy pdfium.dll into release dir
  exit /b 1
)

REM ---- MSI (cargo wix) ----
set "WIX_BIN=C:\Program Files (x86)\WiX Toolset v3.14\bin"
if exist "!WIX_BIN!\candle.exe" (
  echo [msi]   WiX Toolset found: !WIX_BIN!
  set "PATH=!WIX_BIN!;!PATH!"
) else (
  echo [warn]  WiX Toolset not at default path. Install via:
  echo         choco install wixtoolset -y
  goto :skip_msi
)
cd /d "!ROOT_DIR!"
if "!CANON_ARCH!"=="aarch64" (
  cargo wix --no-build --target-bin-dir "!TARGET_RELEASE!" --bin-path "!WIX_BIN!" --output "!DIST_DIR!\!PKG_NAME!-!VERSION!-!CANON_ARCH!.msi" --nocapture -C "-arch" -C "arm64"
) else (
  cargo wix --no-build --target-bin-dir "!TARGET_RELEASE!" --bin-path "!WIX_BIN!" --output "!DIST_DIR!\!PKG_NAME!-!VERSION!-!CANON_ARCH!.msi" --nocapture
)
if errorlevel 1 (
  echo [ERROR] MSI build failed for !ARCH!
  exit /b 1
)
:skip_msi

REM ---- ZIP portable ----
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