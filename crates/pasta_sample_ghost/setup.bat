@echo off
REM setup.bat - hello-pasta Sample Ghost Setup
REM Double-click this file to setup the ghost

setlocal enabledelayedexpansion

REM Get paths
set SCRIPT_DIR=%~dp0
set WORKSPACE_ROOT=%SCRIPT_DIR%..\..
set GHOST_DIR=%SCRIPT_DIR%ghosts\hello-pasta

echo ========================================
echo   hello-pasta Sample Ghost Setup
echo ========================================
echo.
echo Workspace: %WORKSPACE_ROOT%
echo Ghost Dir: %GHOST_DIR%
echo.

REM Step 1: Generate surface images
echo [1/3] Generating surface images...
cargo run --bin generate-surfaces
if errorlevel 1 (
    echo.
    echo WARNING: Surface image generation failed
    echo          Images will be generated on first test run
)
echo.

REM Step 2: Build pasta_shiori.dll (32bit Windows)
echo [2/3] Building pasta.dll...
echo   Target: i686-pc-windows-msvc (32bit Windows)

cd /d %WORKSPACE_ROOT%
cargo build --release --target i686-pc-windows-msvc -p pasta_shiori
if errorlevel 1 (
    echo.
    echo ERROR: pasta_shiori build failed
    echo.
    pause
    exit /b 1
)
echo   Build completed
echo.

REM Step 3: Copy pasta.dll and scripts/
echo [3/3] Copying files...

REM pasta.dll
set DLL_SRC=%WORKSPACE_ROOT%\target\i686-pc-windows-msvc\release\pasta.dll
set DLL_DEST=%GHOST_DIR%\ghost\master\pasta.dll

if not exist "%DLL_SRC%" (
    echo.
    echo ERROR: pasta.dll not found
    echo Path: %DLL_SRC%
    echo.
    echo Please build with:
    echo   cargo build --release --target i686-pc-windows-msvc -p pasta_shiori
    echo.
    pause
    exit /b 1
)

copy /Y "%DLL_SRC%" "%DLL_DEST%" >nul
echo   Copied pasta.dll

REM Lua runtime
set SCRIPTS_SRC=%WORKSPACE_ROOT%\crates\pasta_lua\scripts
set SCRIPTS_DEST=%GHOST_DIR%\ghost\master\scripts

if not exist "%SCRIPTS_SRC%" (
    echo.
    echo ERROR: pasta_lua scripts not found
    echo Path: %SCRIPTS_SRC%
    echo.
    pause
    exit /b 1
)

if exist "%SCRIPTS_DEST%" (
    rmdir /S /Q "%SCRIPTS_DEST%"
)
xcopy /E /I /Y /Q "%SCRIPTS_SRC%" "%SCRIPTS_DEST%" >nul
echo   Copied scripts/

REM Complete
echo.
echo ========================================
echo   Setup Complete!
echo ========================================
echo.
echo Distribution: %GHOST_DIR%
echo.
echo Next steps:
echo   1. cargo test -p pasta_sample_ghost
echo   2. Install the above folder to SSP
echo.
pause
