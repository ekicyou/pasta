@echo off
REM release.bat - hello-pasta Build & Release Script
REM Double-click this file to build ghost distribution and create hello-pasta.nar and pasta.dll.zip
REM Outputs in release/ are for local checks only (not committed). Publishing is done by
REM the release CI (.github/workflows/release.yml) when a vX.Y.Z tag is pushed.
REM
REM Workflow:
REM   1-2. Build pasta.dll, copy DLL/scripts
REM   3-6. pasta_check release, create pasta.dll.zip, version check, size check and release instructions
REM
REM Options (passed through to release.ps1):
REM   -SkipSetup     Skip setup phase (steps 1-2)
REM   -SkipDllBuild  Skip DLL build step only

setlocal

echo ========================================
echo   hello-pasta Build ^& Release
echo ========================================
echo.

powershell.exe -ExecutionPolicy Bypass -File "%~dp0crates\pasta_sample_ghost\release.ps1" %*

if errorlevel 1 (
    echo.
    echo ERROR: Release packaging failed
    echo.
    exit /b 1
)

echo.
