@echo off
REM setup.bat - hello-pasta Full Setup Script
REM Double-click this file to build and setup the ghost distribution
REM
REM Optimized workflow:
REM   1. Build pasta.dll (32bit) - heavy operation first
REM   2. Run pasta_sample_ghost - generate ghost files
REM   3. Copy pasta.dll and scripts/
REM   4. Finalize - generate updates2.dau and updates.txt

setlocal enabledelayedexpansion

REM Get paths
set SCRIPT_DIR=%~dp0
set WORKSPACE_ROOT=%SCRIPT_DIR%..\..
set GHOST_DIR=%SCRIPT_DIR%ghosts\hello-pasta

echo ========================================
echo   hello-pasta Full Setup
echo ========================================
echo.
echo Workspace: %WORKSPACE_ROOT%
echo Ghost Dir: %GHOST_DIR%
echo.

cd /d %WORKSPACE_ROOT%

REM Step 1: Build pasta_shiori.dll (32bit Windows release) - FIRST
echo [1/4] Building pasta.dll (32bit release)...
echo   Target: i686-pc-windows-msvc
cargo build --release --target i686-pc-windows-msvc -p pasta_shiori --quiet
if errorlevel 1 (
    echo.
    echo ERROR: pasta_shiori build failed
    echo.
    echo Make sure you have the i686-pc-windows-msvc target installed:
    echo   rustup target add i686-pc-windows-msvc
    echo.
    pause
    exit /b 1
)
echo   Build completed
echo.

REM Step 2: Run pasta_sample_ghost (generate ghost files + images)
echo [2/4] Generating ghost distribution...
cargo run -p pasta_sample_ghost --quiet
if errorlevel 1 (
    echo.
    echo ERROR: Ghost generation failed
    echo.
    pause
    exit /b 1
)
echo   Ghost files generated
echo.

REM Step 3: Copy pasta.dll and scripts/
echo [3/4] Copying files...

REM Ensure destination directory exists
if not exist "%GHOST_DIR%\ghost\master" (
    mkdir "%GHOST_DIR%\ghost\master"
)

REM Copy pasta.dll
set DLL_SRC=%WORKSPACE_ROOT%\target\i686-pc-windows-msvc\release\pasta.dll
set DLL_DEST=%GHOST_DIR%\ghost\master\pasta.dll

if not exist "%DLL_SRC%" (
    echo.
    echo ERROR: pasta.dll not found at %DLL_SRC%
    echo.
    pause
    exit /b 1
)

copy /Y "%DLL_SRC%" "%DLL_DEST%" >nul
echo   Copied pasta.dll

REM Copy Lua runtime scripts
set SCRIPTS_SRC=%WORKSPACE_ROOT%\crates\pasta_lua\scripts
set SCRIPTS_DEST=%GHOST_DIR%\ghost\master\scripts

if not exist "%SCRIPTS_SRC%" (
    echo.
    echo WARNING: Lua scripts not found at %SCRIPTS_SRC%
    echo          Skipping scripts copy
) else (
    if exist "%SCRIPTS_DEST%" (
        rmdir /S /Q "%SCRIPTS_DEST%"
    )
    xcopy /E /I /Y /Q "%SCRIPTS_SRC%" "%SCRIPTS_DEST%" >nul
    echo   Copied scripts/
)
echo.

REM Step 4: Finalize - generate updates2.dau and updates.txt
echo [4/4] Generating update files...
cargo run -p pasta_sample_ghost --quiet -- --finalize
if errorlevel 1 (
    echo.
    echo ERROR: Finalize failed
    echo.
    pause
    exit /b 1
)
echo   Update files generated

REM Count files
set /a FILE_COUNT=0
for /R "%GHOST_DIR%" %%f in (*) do set /a FILE_COUNT+=1

REM Complete
echo.
echo ========================================
echo   Setup Complete!
echo ========================================
echo.
echo   Distribution: %GHOST_DIR%
echo   Files:        %FILE_COUNT%
echo.
echo Generated update files:
echo   - updates2.dau (SSP binary format)
echo   - updates.txt  (SSP text format)
echo.
echo Next steps:
echo   1. Run tests: cargo test -p pasta_sample_ghost
echo   2. Install the ghost folder to SSP
echo.
