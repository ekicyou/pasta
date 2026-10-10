# release.ps1 - hello-pasta Build, Setup & Release Script
# Builds ghost distribution and creates .nar and pasta.dll.zip release packages
# (also run by the release CI: .github/workflows/release.yml)
#
# Usage (PowerShell 7):
#   pwsh -File release.ps1
#   pwsh -File release.ps1 -SkipSetup
#   pwsh -File release.ps1 -SkipDllBuild
#
# Parameters:
#   -SkipSetup     Skip setup phase (steps 1-2), run release steps only
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
$DllZipPath = Join-Path $WorkspaceRoot "release\pasta.dll.zip"

Write-Host "========================================"
Write-Host "  hello-pasta Build & Release"
Write-Host "========================================"
Write-Host ""
Write-Host "Workspace:   $WorkspaceRoot"
Write-Host "Ghost Dir:   $GhostDir"
Write-Host "Release Dir: $ReleaseDir"
Write-Host "NAR File:    $NarFilePath"
Write-Host "DLL Zip:     $DllZipPath"
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
# Setup Phase (Steps 1-2)
# ============================================================
if ($SkipSetup) {
    Write-Host "[1/6] Building pasta.dll ................. SKIPPED" -ForegroundColor DarkGray
    Write-Host "[2/6] Copying DLL and scripts ............ SKIPPED" -ForegroundColor DarkGray
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

    # --- Step 2: Copy pasta.dll and scripts/ ---
    Write-Host "[2/6] Copying files..."

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

    # Generate third-party license notices for pasta.dll (about.toml / about.hbs)
    if (-not (Get-Command cargo-about -ErrorAction SilentlyContinue)) {
        Write-Host ""
        Write-Host "ERROR: cargo-about not found. Install via: cargo install cargo-about" -ForegroundColor Red
        exit 1
    }
    $LicensesDest = Join-Path $MasterDir "THIRD_PARTY_LICENSES.txt"
    Push-Location $WorkspaceRoot
    try {
        & cargo about generate --output-file $LicensesDest about.hbs
        if ($LASTEXITCODE -ne 0) {
            Write-Host ""
            Write-Host "ERROR: cargo about generate failed" -ForegroundColor Red
            exit 1
        }
    }
    finally {
        Pop-Location
    }
    Write-Host "  Generated THIRD_PARTY_LICENSES.txt"

    # Note: pasta_scripts is no longer bundled into master. The Lua framework
    # runtime is embedded in pasta.dll and self-deployed at runtime
    # (Phase 2.5 self-deploy -> profile/pasta/pasta_scripts).

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
# Release Phase (Steps 3-6)
# ============================================================

# --- Step 3: Run pasta_check release ---
Write-Host "[3/6] Running pasta_check release..."

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

# --- Step 4: Create pasta.dll.zip (pasta.dll + THIRD_PARTY_LICENSES.txt) ---
Write-Host "[4/6] Creating pasta.dll.zip..."

$ZipSources = @(
    (Join-Path $WorkspaceRoot "target\i686-pc-windows-msvc\release\pasta.dll"),
    (Join-Path $GhostDir "ghost\master\THIRD_PARTY_LICENSES.txt")
)
Compress-Archive -Path $ZipSources -DestinationPath $DllZipPath -Force

Add-Type -AssemblyName System.IO.Compression.FileSystem
$zip = [System.IO.Compression.ZipFile]::OpenRead($DllZipPath)
try {
    $zipEntries = @($zip.Entries | ForEach-Object { $_.FullName } | Sort-Object)
}
finally {
    $zip.Dispose()
}
$expectedEntries = @("pasta.dll", "THIRD_PARTY_LICENSES.txt") | Sort-Object
if (($zipEntries -join "|") -ne ($expectedEntries -join "|")) {
    Write-Host ""
    Write-Host "ERROR: pasta.dll.zip must contain exactly: $($expectedEntries -join ', ')" -ForegroundColor Red
    Write-Host "  Actual: $($zipEntries -join ', ')"
    exit 1
}
Write-Host "  Created pasta.dll.zip ($($zipEntries -join ', '))" -ForegroundColor Green
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

# --- Step 6: Size Check & Release Instructions ---
$narSize = (Get-Item $NarFilePath).Length
$narSizeMB = [math]::Round($narSize / 1MB, 2)

Write-Host "[6/6] Size check and release instructions"

$NarMaxBytes = 7MB  # 7,340,032 bytes
if ($narSize -gt $NarMaxBytes) {
    Write-Host ""
    Write-Host "ERROR: hello-pasta.nar is $narSizeMB MB ($narSize bytes), over the 7 MB limit ($NarMaxBytes bytes)" -ForegroundColor Red
    exit 1
}
Write-Host ""
Write-Host "========================================"
Write-Host "  Release Packages Ready!"
Write-Host "========================================"
Write-Host ""
Write-Host "  NAR:     $NarFilePath ($narSizeMB MB)"
Write-Host "  DLL Zip: $DllZipPath"
Write-Host "  Version: $Version"
Write-Host "  Tag:     $TagName"
Write-Host ""
Write-Host "----------------------------------------"
Write-Host "  Next Steps"
Write-Host "----------------------------------------"
Write-Host ""
Write-Host "  release/ の成果物は動作確認用で、コミットしない。"
Write-Host "  公開はリリースタグ（$TagName）の push で CI（.github/workflows/release.yml）が行う。"
Write-Host "  詳細は RELEASE.md:"
Write-Host "     $ScriptDir\RELEASE.md"
Write-Host ""
