@echo off
chcp 65001 >nul
title PezMax Build Script

echo ============================================
echo   PezMax Build Script (Windows)
echo   Build Rust frontend + Java backend
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
echo [1/2] Building Rust frontend...

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

rem ─── 2. 构建 Java 后端 ────────────────────────────────
echo [2/2] Building Java backend...

set "JAVA_DIR=%ROOT_DIR%\PezMax-Java"

rem 检查 Java 是否可用（先查 PATH，再查常见安装路径）
java -version >nul 2>nul
if %ERRORLEVEL% equ 0 goto :java_ok

if exist "C:\Program Files\Java\jdk-17\bin\java.exe" (
    set "PATH=C:\Program Files\Java\jdk-17\bin;%PATH%"
    set "JAVA_HOME=C:\Program Files\Java\jdk-17"
    echo [INFO] Found Java at C:\Program Files\Java\jdk-17
    goto :java_ok
)

if exist "C:\Program Files\Eclipse Adoptium\jdk-17\bin\java.exe" (
    set "PATH=C:\Program Files\Eclipse Adoptium\jdk-17\bin;%PATH%"
    set "JAVA_HOME=C:\Program Files\Eclipse Adoptium\jdk-17"
    echo [INFO] Found Java at C:\Program Files\Eclipse Adoptium\jdk-17
    goto :java_ok
)

goto :no_java

:java_ok

if not exist "%JAVA_DIR%\mvnw.cmd" (
    echo [ERROR] mvnw.cmd not found in %JAVA_DIR%
    exit /b 1
)

pushd "%JAVA_DIR%"
echo Building in: %cd%
call "%JAVA_DIR%\mvnw.cmd" clean package -DskipTests
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Java backend build failed!
    popd
    exit /b 1
)
popd

echo [OK] Java backend built successfully

rem 复制 JAR 到 dist
for /r "%JAVA_DIR%\ruoyi-admin\target" %%f in (*.jar) do (
    set "JAR_NAME=%%~nxf"
    echo !JAR_NAME! | findstr /v "sources javadoc" >nul
    if not errorlevel 1 (
        copy "%%f" "%DIST_DIR%\ruoyi-admin.jar" >nul
        goto :jar_copied
    )
)
:jar_copied
echo [OK] Backend JAR copied to %DIST_DIR%\ruoyi-admin.jar
goto :build_done

:no_java
echo [WARN] Java not found - skipping Java backend build.
echo [WARN] Install JDK 17+ and set JAVA_HOME to build the backend.

:build_done
echo ============================================
echo   Build complete!
echo   Output: %DIST_DIR%
echo     - pezmax-egui.exe  (Rust frontend)
echo     - ruoyi-admin.jar  (Java backend)
echo ============================================

cd /d "%ROOT_DIR%"
endlocal
pause\r