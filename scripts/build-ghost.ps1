# build-ghost.ps1 - hello-pasta ゴースト配布物ビルドスクリプト
#
# 使用方法:
#   .\scripts\build-ghost.ps1
#
# 出力:
#   dist/hello-pasta/ に配布可能なゴーストが生成されます

param(
    [string]$OutputDir = "dist/hello-pasta",
    [switch]$SkipDllBuild,
    [switch]$Verbose
)

$ErrorActionPreference = "Stop"
$WorkspaceRoot = (Get-Item $PSScriptRoot).Parent.FullName

Write-Host "=== hello-pasta ゴーストビルド ===" -ForegroundColor Cyan
Write-Host "Workspace: $WorkspaceRoot"
Write-Host "Output: $OutputDir"
Write-Host ""

# 1. pasta_shiori.dll をビルド（32bit Windows）
if (-not $SkipDllBuild) {
    Write-Host "[1/4] pasta_shiori.dll をビルド中..." -ForegroundColor Yellow
    Push-Location $WorkspaceRoot
    try {
        cargo build --release --target i686-pc-windows-msvc -p pasta_shiori
        if ($LASTEXITCODE -ne 0) {
            throw "pasta_shiori のビルドに失敗しました"
        }
    }
    finally {
        Pop-Location
    }
    Write-Host "  -> ビルド完了" -ForegroundColor Green
}
else {
    Write-Host "[1/4] DLLビルドをスキップ" -ForegroundColor Gray
}

# 2. 出力ディレクトリを準備
Write-Host "[2/4] 出力ディレクトリを準備中..." -ForegroundColor Yellow
$DistPath = Join-Path $WorkspaceRoot $OutputDir
if (Test-Path $DistPath) {
    Remove-Item -Recurse -Force $DistPath
}

# ghosts/hello-pasta/ をコピー
$GhostSrc = Join-Path $WorkspaceRoot "crates/pasta_sample_ghost/ghosts/hello-pasta"
Copy-Item -Recurse $GhostSrc $DistPath
Write-Host "  -> テンプレートをコピー" -ForegroundColor Green

# 3. pasta.dll をコピー
Write-Host "[3/4] pasta.dll をコピー中..." -ForegroundColor Yellow
# 注: pasta_shiori クレートは [lib] name = "pasta" なので、出力は pasta.dll
$DllSrc = Join-Path $WorkspaceRoot "target/i686-pc-windows-msvc/release/pasta.dll"
$DllDest = Join-Path $DistPath "ghost/master/pasta.dll"

if (-not (Test-Path $DllSrc)) {
    throw "pasta.dll が見つかりません: $DllSrc`n32bit ビルドを実行してください: cargo build --release --target i686-pc-windows-msvc -p pasta_shiori"
}
Copy-Item $DllSrc $DllDest
Write-Host "  -> pasta.dll をコピー" -ForegroundColor Green

# 4. Lua スクリプトをコピー
Write-Host "[4/4] Lua ランタイムをコピー中..." -ForegroundColor Yellow
$ScriptsSrc = Join-Path $WorkspaceRoot "crates/pasta_lua/scripts"
$ScriptsDest = Join-Path $DistPath "ghost/master/scripts"

if (-not (Test-Path $ScriptsSrc)) {
    throw "pasta_lua scripts が見つかりません: $ScriptsSrc"
}
Copy-Item -Recurse $ScriptsSrc $ScriptsDest
Write-Host "  -> scripts/ をコピー" -ForegroundColor Green

# 完了
Write-Host ""
Write-Host "=== ビルド完了 ===" -ForegroundColor Cyan
Write-Host "配布物: $DistPath" -ForegroundColor Green
Write-Host ""
Write-Host "ディレクトリ構成:" -ForegroundColor Yellow
Get-ChildItem -Recurse $DistPath -Name | ForEach-Object {
    Write-Host "  $_"
}
