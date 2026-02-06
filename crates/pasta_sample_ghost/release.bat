@echo off
REM release.bat - hello-pasta .nar Package Generator
REM Double-click this file AFTER running setup.bat to create hello-pasta.nar
REM
REM Workflow:
REM   1. Validate ghost distribution files
REM   2. Package as .nar (ZIP with .nar extension)
REM   3. Show release instructions

setlocal

echo ========================================
echo   hello-pasta Release Packager
echo ========================================
echo.

powershell.exe -ExecutionPolicy Bypass -File "%~dp0release.ps1"

if errorlevel 1 (
    echo.
    echo ERROR: Release packaging failed
    echo.
    exit /b 1
)

echo.
