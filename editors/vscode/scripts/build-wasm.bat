@echo off
REM Build WASM module for pasta_lsp
REM Usage: scripts\build-wasm.bat [--release] [--clean]
REM
REM This is a convenience wrapper that calls build-wasm.ps1

setlocal

set "SCRIPT_DIR=%~dp0"
set "PS_SCRIPT=%SCRIPT_DIR%build-wasm.ps1"

REM Parse arguments
set "PS_ARGS="
:parse_args
if "%~1"=="" goto :run
if /i "%~1"=="--release" set "PS_ARGS=%PS_ARGS% -Release"
if /i "%~1"=="--clean"   set "PS_ARGS=%PS_ARGS% -Clean"
shift
goto :parse_args

:run
echo === Pasta WASM Build (bat wrapper) ===
echo Calling: powershell -ExecutionPolicy Bypass -File "%PS_SCRIPT%"%PS_ARGS%
echo.

powershell -ExecutionPolicy Bypass -File "%PS_SCRIPT%"%PS_ARGS%

if %ERRORLEVEL% neq 0 (
    echo.
    echo [ERROR] WASM build failed with exit code %ERRORLEVEL%
    exit /b %ERRORLEVEL%
)

echo.
echo [OK] WASM build completed successfully!
exit /b 0
