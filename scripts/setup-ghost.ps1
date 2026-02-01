# setup-ghost.ps1 - hello-pasta ゴースト配布物セットアップスクリプト
#
# 使用方法:
#   .\scripts\setup-ghost.ps1
#
# 機能:
#   - pasta.dll をビルドして ghosts/hello-pasta/ghost/master/ に配置
#   - Lua ランタイムを ghosts/hello-pasta/ghost/master/scripts/ に配置
#
# 注意:
#   - .pasta と .png ファイルは build.rs で自動生成されます（cargo build/test 時）
#   - このスクリプトは pasta.dll と scripts/ の配置のみを担当

param(
    [switch]$SkipDllBuild,
    [switch]$Verbose
)

$ErrorActionPreference = "Stop"
$WorkspaceRoot = (Get-Item $PSScriptRoot).Parent.FullName
$GhostDir = Join-Path $WorkspaceRoot "crates/pasta_sample_ghost/ghosts/hello-pasta"

Write-Host "=== hello-pasta ゴーストセットアップ ===" -ForegroundColor Cyan
Write-Host "Workspace: $WorkspaceRoot"
Write-Host "Ghost Dir: $GhostDir"
Write-Host ""

# 1. pasta_shiori.dll をビルド（32bit Windows）
if (-not $SkipDllBuild) {
    Write-Host "[1/2] pasta.dll をビルド中..." -ForegroundColor Yellow
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
    Write-Host "[1/2] DLLビルドをスキップ" -ForegroundColor Gray
}

# 2. pasta.dll をコピー
Write-Host "[2/2] pasta.dll & Lua ランタイムをコピー中..." -ForegroundColor Yellow

# pasta.dll
$DllSrc = Join-Path $WorkspaceRoot "target/i686-pc-windows-msvc/release/pasta.dll"
$DllDest = Join-Path $GhostDir "ghost/master/pasta.dll"

if (-not (Test-Path $DllSrc)) {
    throw "pasta.dll が見つかりません: $DllSrc`n32bit ビルドを実行してください: cargo build --release --target i686-pc-windows-msvc -p pasta_shiori"
}
Copy-Item $DllSrc $DllDest -Force
Write-Host "  -> pasta.dll をコピー" -ForegroundColor Green

# Lua ランタイム
$ScriptsSrc = Join-Path $WorkspaceRoot "crates/pasta_lua/scripts"
$ScriptsDest = Join-Path $GhostDir "ghost/master/scripts"

if (-not (Test-Path $ScriptsSrc)) {
    throw "pasta_lua scripts が見つかりません: $ScriptsSrc"
}
if (Test-Path $ScriptsDest) {
    Remove-Item -Recurse -Force $ScriptsDest
}
Copy-Item -Recurse $ScriptsSrc $ScriptsDest -Force
Write-Host "  -> scripts/ をコピー" -ForegroundColor Green

# 完了
Write-Host ""
Write-Host "=== セットアップ完了 ===" -ForegroundColor Cyan
Write-Host "配布物: $GhostDir" -ForegroundColor Green
Write-Host ""
Write-Host "次のステップ:" -ForegroundColor Yellow
Write-Host "  1. cargo test -p pasta_sample_ghost で動的ファイル生成確認"
Write-Host "  2. $GhostDir をSSPにインストール"
