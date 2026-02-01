# setup.ps1 - hello-pasta サンプルゴースト セットアップ
#
# 使用方法: このファイルをダブルクリック、または PowerShell で実行
#
# 機能:
#   1. pasta_shiori.dll (32bit) をビルド
#   2. ghosts/hello-pasta/ghost/master/ に pasta.dll として配置
#   3. crates/pasta_lua/scripts/ を ghosts/hello-pasta/ghost/master/scripts/ にコピー

param(
    [switch]$SkipDllBuild
)

$ErrorActionPreference = "Stop"

# スクリプトのディレクトリ（crates/pasta_sample_ghost）から workspace ルートへ
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$WorkspaceRoot = (Get-Item $ScriptDir).Parent.Parent.FullName
$GhostDir = Join-Path $ScriptDir "ghosts\hello-pasta"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  hello-pasta サンプルゴースト セットアップ" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Workspace: $WorkspaceRoot" -ForegroundColor Gray
Write-Host "Ghost Dir: $GhostDir" -ForegroundColor Gray
Write-Host ""

# Step 1: pasta_shiori.dll をビルド（32bit Windows）
if (-not $SkipDllBuild) {
    Write-Host "[1/2] pasta.dll をビルド中..." -ForegroundColor Yellow
    Write-Host "  ターゲット: i686-pc-windows-msvc (32bit Windows)" -ForegroundColor Gray
    
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
    Write-Host "  ✓ ビルド完了" -ForegroundColor Green
}
else {
    Write-Host "[1/2] DLL ビルドをスキップ" -ForegroundColor Gray
}

Write-Host ""

# Step 2: pasta.dll と scripts/ をコピー
Write-Host "[2/2] ファイルをコピー中..." -ForegroundColor Yellow

# pasta.dll
$DllSrc = Join-Path $WorkspaceRoot "target\i686-pc-windows-msvc\release\pasta.dll"
$DllDest = Join-Path $GhostDir "ghost\master\pasta.dll"

if (-not (Test-Path $DllSrc)) {
    Write-Host ""
    Write-Host "エラー: pasta.dll が見つかりません" -ForegroundColor Red
    Write-Host "パス: $DllSrc" -ForegroundColor Red
    Write-Host ""
    Write-Host "以下のコマンドでビルドしてください:" -ForegroundColor Yellow
    Write-Host "  cargo build --release --target i686-pc-windows-msvc -p pasta_shiori" -ForegroundColor White
    Read-Host "Enter キーを押して終了"
    exit 1
}

Copy-Item $DllSrc $DllDest -Force
Write-Host "  ✓ pasta.dll をコピー" -ForegroundColor Green

# Lua ランタイム
$ScriptsSrc = Join-Path $WorkspaceRoot "crates\pasta_lua\scripts"
$ScriptsDest = Join-Path $GhostDir "ghost\master\scripts"

if (-not (Test-Path $ScriptsSrc)) {
    Write-Host ""
    Write-Host "エラー: pasta_lua scripts が見つかりません" -ForegroundColor Red
    Write-Host "パス: $ScriptsSrc" -ForegroundColor Red
    Read-Host "Enter キーを押して終了"
    exit 1
}

if (Test-Path $ScriptsDest) {
    Remove-Item -Recurse -Force $ScriptsDest
}
Copy-Item -Recurse $ScriptsSrc $ScriptsDest -Force
Write-Host "  ✓ scripts/ をコピー" -ForegroundColor Green

# 完了
Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  セットアップ完了！" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "配布物の場所:" -ForegroundColor Yellow
Write-Host "  $GhostDir" -ForegroundColor White
Write-Host ""
Write-Host "次のステップ:" -ForegroundColor Yellow
Write-Host "  1. cargo test -p pasta_sample_ghost で動作確認" -ForegroundColor White
Write-Host "  2. 上記フォルダを SSP にインストール" -ForegroundColor White
Write-Host ""

# ダブルクリック実行時は自動終了しないよう待機
if ($MyInvocation.InvocationName -ne "&") {
    Read-Host "Enter キーを押して終了"
}
