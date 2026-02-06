# release.ps1 - hello-pasta .nar Package Generator
# Validates ghost distribution, creates .nar file, and shows release instructions
#
# Usage: powershell -ExecutionPolicy Bypass -File release.ps1
# Prerequisites: Run setup.bat first to generate ghost distribution

$ErrorActionPreference = 'Stop'

# --- Path Setup ---
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$WorkspaceRoot = Resolve-Path (Join-Path $ScriptDir "..\..") | Select-Object -ExpandProperty Path
$GhostDir = Join-Path $ScriptDir "ghosts\hello-pasta"
$OutputDir = $ScriptDir
$NarFileName = "hello-pasta.nar"
$NarFilePath = Join-Path $OutputDir $NarFileName

Write-Host "========================================"
Write-Host "  hello-pasta Release Packager"
Write-Host "========================================"
Write-Host ""
Write-Host "Workspace: $WorkspaceRoot"
Write-Host "Ghost Dir: $GhostDir"
Write-Host ""

# ============================================================
# Step 1: Version Check (Task 2.1)
# ============================================================
Write-Host "[1/4] Checking version..."

$CargoToml = Join-Path $WorkspaceRoot "Cargo.toml"
if (-not (Test-Path $CargoToml)) {
    Write-Host ""
    Write-Host "ERROR: Cargo.toml not found at $CargoToml" -ForegroundColor Red
    Write-Host "  Make sure to run this script from the workspace root."
    exit 1
}

$CargoContent = Get-Content $CargoToml -Raw
if ($CargoContent -match 'version\s*=\s*"([^"]+)"') {
    $Version = $Matches[1]
    $TagName = "v$Version"
}
else {
    Write-Host ""
    Write-Host "ERROR: Could not read version from Cargo.toml" -ForegroundColor Red
    exit 1
}

Write-Host "  Version: $Version"
Write-Host "  Tag:     $TagName"
Write-Host ""

# ============================================================
# Step 2: Validate Ghost Distribution (Task 2.2)
# ============================================================
Write-Host "[2/4] Validating ghost distribution..."

if (-not (Test-Path $GhostDir)) {
    Write-Host ""
    Write-Host "ERROR: Ghost directory not found at $GhostDir" -ForegroundColor Red
    Write-Host "  Run setup.bat first to generate the ghost distribution."
    exit 1
}

$RequiredFiles = @(
    "ghost\master\pasta.dll",
    "ghost\master\pasta.toml",
    "ghost\master\descript.txt",
    "install.txt",
    "updates.txt",
    "updates2.dau"
)

$RequiredDirs = @(
    "ghost\master\dic",
    "ghost\master\scripts",
    "shell\master"
)

$ValidationFailed = $false

foreach ($file in $RequiredFiles) {
    $fullPath = Join-Path $GhostDir $file
    if (-not (Test-Path $fullPath)) {
        Write-Host "  MISSING: $file" -ForegroundColor Red
        $ValidationFailed = $true
    }
}

foreach ($dir in $RequiredDirs) {
    $fullPath = Join-Path $GhostDir $dir
    if (-not (Test-Path $fullPath)) {
        Write-Host "  MISSING DIR: $dir" -ForegroundColor Red
        $ValidationFailed = $true
    }
}

# Check dic/ has .pasta files
$DicDir = Join-Path $GhostDir "ghost\master\dic"
if (Test-Path $DicDir) {
    $pastaFiles = Get-ChildItem -Path $DicDir -Filter "*.pasta" -ErrorAction SilentlyContinue
    if ($null -eq $pastaFiles -or $pastaFiles.Count -eq 0) {
        Write-Host "  MISSING: No .pasta files in ghost\master\dic\" -ForegroundColor Red
        $ValidationFailed = $true
    }
}

# Check shell/master/ has image files
$ShellDir = Join-Path $GhostDir "shell\master"
if (Test-Path $ShellDir) {
    $imageFiles = Get-ChildItem -Path $ShellDir -Filter "surface*.png" -ErrorAction SilentlyContinue
    if ($null -eq $imageFiles -or $imageFiles.Count -eq 0) {
        Write-Host "  MISSING: No surface*.png files in shell\master\" -ForegroundColor Red
        $ValidationFailed = $true
    }
}

# Check pasta.dll is not empty
$DllPath = Join-Path $GhostDir "ghost\master\pasta.dll"
if (Test-Path $DllPath) {
    $dllSize = (Get-Item $DllPath).Length
    if ($dllSize -eq 0) {
        Write-Host "  ERROR: pasta.dll is empty (0 bytes)" -ForegroundColor Red
        $ValidationFailed = $true
    }
}

if ($ValidationFailed) {
    Write-Host ""
    Write-Host "ERROR: Ghost distribution validation failed." -ForegroundColor Red
    Write-Host "  Run setup.bat first to generate a complete ghost distribution."
    exit 1
}

Write-Host "  All required files present" -ForegroundColor Green
Write-Host ""

# ============================================================
# Step 3: Create .nar File (Task 2.3)
# ============================================================
Write-Host "[3/4] Creating $NarFileName..."

$TempDir = Join-Path $ScriptDir "temp_release"
$TempGhostDir = Join-Path $TempDir "hello-pasta"
$ZipPath = Join-Path $ScriptDir "hello-pasta.zip"

# Clean up any previous temp directory
if (Test-Path $TempDir) {
    Remove-Item -Path $TempDir -Recurse -Force
}

# Clean up any previous output
if (Test-Path $ZipPath) {
    Remove-Item -Path $ZipPath -Force
}
if (Test-Path $NarFilePath) {
    Remove-Item -Path $NarFilePath -Force
}

# Create temp directory
New-Item -ItemType Directory -Path $TempGhostDir -Force | Out-Null

# Copy with robocopy, excluding profile/ directory and temp files
$robocopyArgs = @(
    $GhostDir,
    $TempGhostDir,
    "/MIR",
    "/XD", "profile",
    "/XF", "*.bak", "*.tmp",
    "/NJH", "/NJS", "/NDL", "/NC", "/NS", "/NP"
)
& robocopy @robocopyArgs | Out-Null
# robocopy returns 0-7 for success, 8+ for error
if ($LASTEXITCODE -ge 8) {
    Write-Host ""
    Write-Host "ERROR: robocopy failed with exit code $LASTEXITCODE" -ForegroundColor Red
    if (Test-Path $TempDir) { Remove-Item -Path $TempDir -Recurse -Force }
    exit 1
}

# ZIP compress
Compress-Archive -Path (Join-Path $TempGhostDir "*") -DestinationPath $ZipPath -Force

if (-not (Test-Path $ZipPath)) {
    Write-Host ""
    Write-Host "ERROR: ZIP compression failed" -ForegroundColor Red
    if (Test-Path $TempDir) { Remove-Item -Path $TempDir -Recurse -Force }
    exit 1
}

# Rename .zip to .nar
Rename-Item -Path $ZipPath -NewName $NarFileName

if (-not (Test-Path $NarFilePath)) {
    Write-Host ""
    Write-Host "ERROR: .nar rename failed" -ForegroundColor Red
    if (Test-Path $TempDir) { Remove-Item -Path $TempDir -Recurse -Force }
    exit 1
}

# Clean up temp directory
Remove-Item -Path $TempDir -Recurse -Force

$narSize = (Get-Item $NarFilePath).Length
$narSizeMB = [math]::Round($narSize / 1MB, 2)

Write-Host "  Created: $NarFilePath"
Write-Host "  Size:    $narSizeMB MB"
Write-Host ""

# ============================================================
# Step 4: Show Release Instructions (Task 2.4)
# ============================================================
Write-Host "[4/4] Release instructions"
Write-Host ""
Write-Host "========================================"
Write-Host "  .nar Package Ready!"
Write-Host "========================================"
Write-Host ""
Write-Host "  File:    $NarFilePath"
Write-Host "  Version: $Version"
Write-Host "  Tag:     $TagName"
Write-Host "  Size:    $narSizeMB MB"
Write-Host ""
Write-Host "----------------------------------------"
Write-Host "  Next Steps"
Write-Host "----------------------------------------"
Write-Host ""
Write-Host "  1. Review RELEASE.md for full instructions:"
Write-Host "     $ScriptDir\RELEASE.md"
Write-Host ""
Write-Host "  2. Create GitHub Release:"
Write-Host ""
Write-Host "     gh release create $TagName `"$NarFilePath`" --title `"hello-pasta $TagName`" --notes-file release-notes.md" -ForegroundColor Cyan
Write-Host ""
Write-Host "  3. Or with inline notes:"
Write-Host ""
Write-Host "     gh release create $TagName `"$NarFilePath`" --title `"hello-pasta $TagName`" --notes `"hello-pasta $Version alpha release`"" -ForegroundColor Cyan
Write-Host ""
Write-Host "  Tip: Consult AI with RELEASE.md template for detailed release notes."
Write-Host ""
