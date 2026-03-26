# release.ps1 - hello-pasta Build, Setup & Release Script
# Builds ghost distribution and creates .nar release package
#
# Usage:
#   powershell -ExecutionPolicy Bypass -File release.ps1
#   powershell -ExecutionPolicy Bypass -File release.ps1 -SkipSetup
#   powershell -ExecutionPolicy Bypass -File release.ps1 -SkipDllBuild
#
# Parameters:
#   -SkipSetup     Skip setup phase (steps 1-3), run release steps only
#   -SkipDllBuild  Skip DLL build step only (use existing pasta.dll)

param(
    [switch]$SkipSetup,
    [switch]$SkipDllBuild
)

$ErrorActionPreference = 'Stop'

# --- Path Setup ---
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$WorkspaceRoot = Resolve-Path (Join-Path $ScriptDir "..\..") | Select-Object -ExpandProperty Path
$GhostDir = Join-Path $ScriptDir "ghosts\hello-pasta"
$ReleaseDir = Join-Path $WorkspaceRoot "release\hello-pasta"
$NarFilePath = Join-Path $WorkspaceRoot "release\hello-pasta.nar"

Write-Host "========================================"
Write-Host "  hello-pasta Build & Release"
Write-Host "========================================"
Write-Host ""
Write-Host "Workspace:   $WorkspaceRoot"
Write-Host "Ghost Dir:   $GhostDir"
Write-Host "Release Dir: $ReleaseDir"
Write-Host "NAR File:    $NarFilePath"
if ($SkipSetup) {
    Write-Host "Mode:        Release only (setup skipped)"
}
elseif ($SkipDllBuild) {
    Write-Host "Mode:        Setup + Release (DLL build skipped)"
}
else {
    Write-Host "Mode:        Full (setup + release)"
}
Write-Host ""

# ============================================================
# Setup Phase (Steps 1-3)
# ============================================================
if ($SkipSetup) {
    Write-Host "[1/6] Building pasta.dll ................. SKIPPED" -ForegroundColor DarkGray
    Write-Host "[2/6] Generating images .................. SKIPPED" -ForegroundColor DarkGray
    Write-Host "[3/6] Copying DLL and scripts ............ SKIPPED" -ForegroundColor DarkGray
    Write-Host ""
}
else {
    # --- Step 1: Build pasta.dll (32bit) ---
    if ($SkipDllBuild) {
        Write-Host "[1/6] Building pasta.dll ................. SKIPPED" -ForegroundColor DarkGray
    }
    else {
        Write-Host "[1/6] Building pasta.dll (32bit release)..."
        Write-Host "  Target: i686-pc-windows-msvc"

        Push-Location $WorkspaceRoot
        try {
            & cargo build --release --target i686-pc-windows-msvc -p pasta_shiori --quiet
            if ($LASTEXITCODE -ne 0) {
                Write-Host ""
                Write-Host "ERROR: pasta_shiori build failed" -ForegroundColor Red
                Write-Host ""
                Write-Host "Make sure you have the i686-pc-windows-msvc target installed:"
                Write-Host "  rustup target add i686-pc-windows-msvc"
                exit 1
            }
        }
        finally {
            Pop-Location
        }
        Write-Host "  Build completed" -ForegroundColor Green
    }
    Write-Host ""

    # --- Step 2: Generate images (surface*.png + surfaces.txt) ---
    Write-Host "[2/6] Generating images..."

    Push-Location $WorkspaceRoot
    try {
        & cargo run -p pasta_sample_ghost --quiet
        if ($LASTEXITCODE -ne 0) {
            Write-Host ""
            Write-Host "ERROR: Image generation failed" -ForegroundColor Red
            exit 1
        }
    }
    finally {
        Pop-Location
    }
    Write-Host "  Images generated" -ForegroundColor Green
    Write-Host ""

    # --- Step 3: Copy pasta.dll and scripts/ ---
    Write-Host "[3/6] Copying files..."

    $MasterDir = Join-Path $GhostDir "ghost\master"
    if (-not (Test-Path $MasterDir)) {
        New-Item -ItemType Directory -Path $MasterDir -Force | Out-Null
    }

    # Copy pasta.dll
    $DllSrc = Join-Path $WorkspaceRoot "target\i686-pc-windows-msvc\release\pasta.dll"
    $DllDest = Join-Path $MasterDir "pasta.dll"

    if (-not (Test-Path $DllSrc)) {
        Write-Host ""
        Write-Host "ERROR: pasta.dll not found at $DllSrc" -ForegroundColor Red
        Write-Host "  Run without -SkipDllBuild to build it first."
        exit 1
    }

    Copy-Item -Path $DllSrc -Destination $DllDest -Force
    Write-Host "  Copied pasta.dll"

    # Copy Lua runtime scripts
    $ScriptsSrc = Join-Path $WorkspaceRoot "crates\pasta_lua\pasta_scripts"
    $ScriptsDest = Join-Path $MasterDir "pasta_scripts"

    if (-not (Test-Path $ScriptsSrc)) {
        Write-Host "  WARNING: Lua scripts not found at $ScriptsSrc" -ForegroundColor Yellow
        Write-Host "           Skipping pasta_scripts copy"
    }
    else {
        if (Test-Path $ScriptsDest) {
            Remove-Item -Path $ScriptsDest -Recurse -Force
        }
        $robocopySetupArgs = @(
            $ScriptsSrc,
            $ScriptsDest,
            "/MIR",
            "/NJH", "/NJS", "/NDL", "/NC", "/NS", "/NP"
        )
        & robocopy @robocopySetupArgs | Out-Null
        if ($LASTEXITCODE -ge 8) {
            Write-Host ""
            Write-Host "ERROR: robocopy failed copying pasta_scripts (exit code $LASTEXITCODE)" -ForegroundColor Red
            exit 1
        }
        Write-Host "  Copied pasta_scripts/"
    }

    # Copy user scripts directory (README.md only)
    $UserScriptsSrc = Join-Path $WorkspaceRoot "crates\pasta_lua\scripts"
    $UserScriptsDest = Join-Path $MasterDir "scripts"

    if (Test-Path $UserScriptsSrc) {
        $robocopyScriptsArgs = @(
            $UserScriptsSrc,
            $UserScriptsDest,
            "/MIR",
            "/NJH", "/NJS", "/NDL", "/NC", "/NS", "/NP"
        )
        & robocopy @robocopyScriptsArgs | Out-Null
        if ($LASTEXITCODE -ge 8) {
            Write-Host ""
            Write-Host "ERROR: robocopy failed copying scripts (exit code $LASTEXITCODE)" -ForegroundColor Red
            exit 1
        }
        Write-Host "  Synced scripts/ (user layer)"
    }

    # Count files
    $fileCount = (Get-ChildItem -Path $GhostDir -Recurse -File).Count
    Write-Host "  Distribution files: $fileCount"
    Write-Host ""
}

# ============================================================
# Release Phase (Steps 4-6)
# ============================================================

# --- Step 4: Run pasta_check release ---
Write-Host "[4/6] Running pasta_check release..."

Push-Location $WorkspaceRoot
try {
    & cargo run -p pasta_check --quiet -- release --target $GhostDir --release $ReleaseDir --nar $NarFilePath
    if ($LASTEXITCODE -ne 0) {
        Write-Host ""
        Write-Host "ERROR: pasta_check release failed" -ForegroundColor Red
        exit 1
    }
}
finally {
    Pop-Location
}
Write-Host "  Release package created" -ForegroundColor Green
Write-Host ""

# --- Step 5: Version Check ---
Write-Host "[5/6] Checking version..."

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

# --- Step 6: Show Release Instructions ---
$narSize = (Get-Item $NarFilePath).Length
$narSizeMB = [math]::Round($narSize / 1MB, 2)

Write-Host "[6/6] Release instructions"
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
