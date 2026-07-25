@echo off
setlocal enabledelayedexpansion
title PezMax PDFium Downloader

REM =============================================
REM  fetch-pdfium.bat — Download PDFium prebuilt binaries
REM
REM  Usage:
REM    fetch-pdfium.bat                         Download all architectures
REM    fetch-pdfium.bat windows-x64             Download specific architecture only
REM    fetch-pdfium.bat windows-x64 windows-arm64  Download multiple architectures
REM    fetch-pdfium.bat --list-versions         Show latest available version
REM    fetch-pdfium.bat --help                  Show this help
REM
REM  Supported architecture tags:
REM    windows-x64  windows-arm64  linux-x64  linux-arm64  macos-x64  macos-arm64
REM
REM  Output directory structure:
REM    build/pdfium/{platform}/{library}
REM =============================================

set "SCRIPT_DIR=%~dp0"
if "!SCRIPT_DIR:~-1!"=="\" set "SCRIPT_DIR=!SCRIPT_DIR:~0,-1!"
set "PDFIUM_DIR=!SCRIPT_DIR!\pdfium"

set "DRY_RUN=0"
set "FORCE=0"
set "TARGETS="

:parse_args
if "%~1"=="" goto :parse_done
if /i "%~1"=="--dry-run"    set "DRY_RUN=1"   & shift & goto :parse_args
if /i "%~1"=="--force"      set "FORCE=1"     & shift & goto :parse_args
if /i "%~1"=="--list-versions" goto :list_versions
if /i "%~1"=="--help"       goto :show_usage
set "TARGETS=!TARGETS! %~1" & shift & goto :parse_args
:parse_done

REM Default: download all architectures
if "!TARGETS!"=="" set "TARGETS=windows-x64 windows-arm64"

echo.
echo =============================================
echo   PDFium Downloader
echo =============================================

REM Fetch latest version
echo [pdfium] Fetching latest version...
for /f "usebackq delims=" %%v in (`powershell -NoProfile -Command "try { $v = (Invoke-RestMethod 'https://api.github.com/repos/bblanchon/pdfium-binaries/releases/latest').tag_name; Write-Output ($v -replace 'chromium/','') } catch { Write-Output 'ERROR' }"`) do set "VERSION=%%v"

if "!VERSION!"=="ERROR" (
  echo [ERROR] Cannot fetch PDFium version. Check your network connection.
  exit /b 1
)
echo [pdfium] Latest version: chromium/!VERSION!

for %%a in (!TARGETS!) do call :download_arch %%a

echo.
echo [pdfium] All done.
goto :end

REM =============================================
REM  Download a single architecture
REM =============================================
:download_arch
set "ARCH_TAG=%~1"

REM Map arch tag to download platform, library name, and package format
if /i "%ARCH_TAG%"=="windows-x64"   set "DL_PLATFORM=win-x64"   & set "LIB_NAME=pdfium.dll"    & set "PKG_FMT=tgz"   & goto :do_dl
if /i "%ARCH_TAG%"=="windows-arm64" set "DL_PLATFORM=win-arm64" & set "LIB_NAME=pdfium.dll"    & set "PKG_FMT=tgz"   & goto :do_dl
if /i "%ARCH_TAG%"=="linux-x64"     set "DL_PLATFORM=linux-x64" & set "LIB_NAME=libpdfium.so"  & set "PKG_FMT=tgz"   & goto :do_dl
if /i "%ARCH_TAG%"=="linux-arm64"   set "DL_PLATFORM=linux-arm64" & set "LIB_NAME=libpdfium.so"  & set "PKG_FMT=tgz" & goto :do_dl
if /i "%ARCH_TAG%"=="macos-x64"     set "DL_PLATFORM=mac-x64"   & set "LIB_NAME=libpdfium.dylib" & set "PKG_FMT=tgz" & goto :do_dl
if /i "%ARCH_TAG%"=="macos-arm64"   set "DL_PLATFORM=mac-arm64" & set "LIB_NAME=libpdfium.dylib" & set "PKG_FMT=tgz" & goto :do_dl

echo [ERROR] Unsupported architecture: %ARCH_TAG%
echo Supported: windows-x64 windows-arm64 linux-x64 linux-arm64 macos-x64 macos-arm64
exit /b 1

:do_dl
set "LIB_PATH=!PDFIUM_DIR!\%ARCH_TAG%\!LIB_NAME!"

if "!DRY_RUN!"=="1" (
  echo [DRY-RUN] Would download to: !LIB_PATH!
  exit /b 0
)

if exist "!LIB_PATH!" (
  if "!FORCE!"=="0" (
    echo [pdfium] Already exists: !LIB_PATH! (use --force to re-download)
    exit /b 0
  )
)

if not exist "!PDFIUM_DIR!\%ARCH_TAG%" mkdir "!PDFIUM_DIR!\%ARCH_TAG%"

set "DL_URL=https://github.com/bblanchon/pdfium-binaries/releases/download/chromium/!VERSION!/pdfium-!DL_PLATFORM!"
if "!PKG_FMT!"=="zip" ( set "DL_URL=!DL_URL!.zip" ) else ( set "DL_URL=!DL_URL!.tgz" )

echo [pdfium] Downloading %ARCH_TAG% ...
echo    URL: !DL_URL!

set "TMP_DIR=%TEMP%\pdfium_dl_%ARCH_TAG%"
if exist "!TMP_DIR!" rmdir /s /q "!TMP_DIR!"
mkdir "!TMP_DIR!"

REM Download using curl (preferred) or PowerShell (fallback)
where curl.exe >nul 2>nul
if %ERRORLEVEL% equ 0 (
  curl.exe -fL --connect-timeout 30 --max-time 120 "!DL_URL!" -o "!TMP_DIR!\pdfium.!PKG_FMT!"
) else (
  powershell -NoProfile -Command "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; (New-Object System.Net.WebClient).DownloadFile('!DL_URL!', '!TMP_DIR!\pdfium.!PKG_FMT!')"
)
if %ERRORLEVEL% neq 0 (
  echo [ERROR] Download failed for %ARCH_TAG%
  exit /b 1
)

REM Extract (all platforms use tgz format now)
pushd "!TMP_DIR!" 2>nul
tar -xzf "pdfium.tgz" 2>nul
if %ERRORLEVEL% neq 0 (
  if not exist "extracted" mkdir "extracted"
  tar -xzf "pdfium.tgz" -C "extracted" 2>nul
)
popd
if exist "!TMP_DIR!\bin\!LIB_NAME!" (
  copy /Y "!TMP_DIR!\bin\!LIB_NAME!" "!LIB_PATH!" >nul
) else (
  for /r "!TMP_DIR!" %%f in (!LIB_NAME!) do copy /Y "%%f" "!LIB_PATH!" >nul
)

:found
rmdir /s /q "!TMP_DIR!" 2>nul

if not exist "!LIB_PATH!" (
  echo [ERROR] Downloaded but could not find !LIB_NAME! in the archive
  exit /b 1
)

for %%f in ("!LIB_PATH!") do echo [pdfium] OK %ARCH_TAG% - !LIB_PATH! (%%~zf bytes)
exit /b 0

REM =============================================
REM  List versions
REM =============================================
:list_versions
powershell -NoProfile -Command "try { $v = (Invoke-RestMethod 'https://api.github.com/repos/bblanchon/pdfium-binaries/releases/latest').tag_name; Write-Host 'Latest version: ' $v } catch { Write-Host 'Failed to fetch version' }"
exit /b 0

REM =============================================
REM  Help
REM =============================================
:show_usage
echo Usage: %~nx0 [--dry-run] [--force] [architecture tags...]
echo.
echo Options:
echo   --dry-run        Show what would be downloaded without actually downloading
echo   --force          Re-download even if files already exist
echo   --list-versions  Show the latest available PDFium version
echo   --help           Show this help
echo.
echo Supported architecture tags:
echo   windows-x64  windows-arm64  linux-x64  linux-arm64  macos-x64  macos-arm64
echo.
echo Examples:
echo   %~nx0                          Download all architectures
echo   %~nx0 windows-x64              Download only Windows x64
echo   %~nx0 windows-x64 windows-arm64 Download Windows x64 and ARM64
exit /b 0

:end
endlocal