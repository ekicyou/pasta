# Build WASM module for pasta_lsp
# Usage: powershell -File scripts/build-wasm.ps1
#
# This script builds the pasta_lsp crate as a WASM module using wasm-pack
# and places the output in editors/vscode/wasm/ for bundling into the VSIX.

param(
    [switch]$Release,
    [switch]$Clean
)

$ErrorActionPreference = "Stop"

# ---- Resolve paths (absolute, independent of working directory) ----
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$VscodeDir = Split-Path -Parent $ScriptDir          # editors/vscode
$EditorsDir = Split-Path -Parent $VscodeDir           # editors
$ProjectRoot = Split-Path -Parent $EditorsDir          # repo root
$WasmOutDir = Join-Path $VscodeDir "wasm"
$CratePath = Join-Path (Join-Path $ProjectRoot "crates") "pasta_lsp"

Write-Host ""
Write-Host "=== Pasta WASM Build ===" -ForegroundColor Cyan
Write-Host "  Project root : $ProjectRoot"
Write-Host "  Crate        : $CratePath"
Write-Host "  Output dir   : $WasmOutDir"
Write-Host ""

# ---- Prerequisites check ----
$wasm_pack = Get-Command wasm-pack -ErrorAction SilentlyContinue
if (-not $wasm_pack) {
    Write-Host "[ERROR] wasm-pack not found. Install via: cargo install wasm-pack" -ForegroundColor Red
    exit 1
}
Write-Host "[OK] wasm-pack: $($wasm_pack.Source)" -ForegroundColor Green

# Check wasm32 target
$targets = rustup target list --installed 2>&1
if ($targets -notmatch "wasm32-unknown-unknown") {
    Write-Host "[INFO] Adding wasm32-unknown-unknown target..." -ForegroundColor Yellow
    rustup target add wasm32-unknown-unknown
    if ($LASTEXITCODE -ne 0) {
        Write-Host "[ERROR] Failed to add wasm32 target" -ForegroundColor Red
        exit 1
    }
}
Write-Host "[OK] wasm32-unknown-unknown target installed" -ForegroundColor Green

# ---- Clean (optional) ----
if ($Clean -and (Test-Path $WasmOutDir)) {
    Write-Host "[CLEAN] Removing $WasmOutDir ..." -ForegroundColor Yellow
    Remove-Item -Recurse -Force $WasmOutDir
}

# ---- Ensure output directory exists ----
if (-not (Test-Path $WasmOutDir)) {
    New-Item -ItemType Directory -Path $WasmOutDir -Force | Out-Null
    Write-Host "[OK] Created output directory" -ForegroundColor Green
}

# ---- Build ----
$buildMode = if ($Release) { "--release" } else { "--dev" }
Write-Host ""
Write-Host "[BUILD] wasm-pack build --target nodejs $buildMode" -ForegroundColor Cyan

Push-Location $ProjectRoot
try {
    if ($Release) {
        wasm-pack build --target nodejs --out-dir $WasmOutDir --out-name pasta_lsp_wasm $CratePath --release
    }
    else {
        wasm-pack build --target nodejs --out-dir $WasmOutDir --out-name pasta_lsp_wasm $CratePath --dev
    }

    if ($LASTEXITCODE -ne 0) {
        Write-Host "[ERROR] wasm-pack build failed (exit code: $LASTEXITCODE)" -ForegroundColor Red
        exit $LASTEXITCODE
    }
}
finally {
    Pop-Location
}

# ---- Verify output ----
$wasmFile = Join-Path $WasmOutDir "pasta_lsp_wasm_bg.wasm"
$jsFile = Join-Path $WasmOutDir "pasta_lsp_wasm.js"

if (-not (Test-Path $wasmFile)) {
    Write-Host "[ERROR] WASM file not found: $wasmFile" -ForegroundColor Red
    exit 1
}
if (-not (Test-Path $jsFile)) {
    Write-Host "[ERROR] JS bindings not found: $jsFile" -ForegroundColor Red
    exit 1
}

$wasmSize = (Get-Item $wasmFile).Length
$wasmSizeKB = [math]::Round($wasmSize / 1024, 1)
Write-Host ""
Write-Host "=== Build Complete ===" -ForegroundColor Green
Write-Host "  WASM : $wasmFile ($wasmSizeKB KB)"
Write-Host "  JS   : $jsFile"
Write-Host ""

# ---- Cleanup unnecessary files ----
$packageJson = Join-Path $WasmOutDir "package.json"
$gitignore = Join-Path $WasmOutDir ".gitignore"
if (Test-Path $packageJson) { Remove-Item $packageJson -Force }
if (Test-Path $gitignore) { Remove-Item $gitignore -Force }

Write-Host "[OK] WASM build pipeline completed successfully!" -ForegroundColor Green
