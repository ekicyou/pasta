@echo off
REM release.bat - hello-pasta Build & Release Script
REM Double-click this file to build ghost distribution and create hello-pasta.nar
REM
REM Workflow:
REM   1-3. Build pasta.dll, generate images, copy DLL/scripts
REM   4-6. pasta_check release, version check, release instructions
REM
REM Options (passed through to release.ps1):
REM   -SkipSetup     Skip setup phase (steps 1-3)
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
