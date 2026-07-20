@echo off
chcp 65001 >nul
title PezMax Backend Build

echo ============================================
echo   PezMax Backend Build (Java + Maven)
echo   构建后端 JAR 用于本地部署
echo ============================================
echo.

setlocal enabledelayedexpansion

set "ROOT_DIR=%~dp0"
set "JAVA_DIR=%ROOT_DIR%\PezMax-Java"
set "DIST_DIR=%ROOT_DIR%\build\dist"

rem 检查 Java 是否可用
java -version >nul 2>nul
if %ERRORLEVEL% neq 0 (
    if exist "C:\Program Files\Java\jdk-17\bin\java.exe" (
        set "PATH=C:\Program Files\Java\jdk-17\bin;%PATH%"
        set "JAVA_HOME=C:\Program Files\Java\jdk-17"
    ) else (
        echo [ERROR] Java 17+ not found. Install JDK 17+ and set JAVA_HOME.
        exit /b 1
    )
)

if not exist "%DIST_DIR%" mkdir "%DIST_DIR%"

pushd "%JAVA_DIR%"
echo Building in: %cd%
call "%JAVA_DIR%\mvnw.cmd" clean package -DskipTests
if %ERRORLEVEL% neq 0 (
    echo [ERROR] Java backend build failed!
    popd
    exit /b 1
)
popd

rem 复制 JAR 到 dist
for /r "%JAVA_DIR%\ruoyi-admin\target" %%f in (*.jar) do (
    set "JAR_NAME=%%~nxf"
    echo !JAR_NAME! | findstr /v "sources javadoc original" >nul
    if not errorlevel 1 (
        copy "%%f" "%DIST_DIR%\ruoyi-admin.jar" >nul
        goto :jar_copied
    )
)
:jar_copied
echo [OK] Backend JAR copied to %DIST_DIR%\ruoyi-admin.jar

echo ============================================
echo   Backend build complete!
echo   Output: %DIST_DIR%\ruoyi-admin.jar
echo   Run: java -jar %DIST_DIR%\ruoyi-admin.jar
echo ============================================

endlocal
pause