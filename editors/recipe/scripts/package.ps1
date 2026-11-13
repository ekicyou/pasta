#Requires -Version 7.4
<#
.SYNOPSIS
  配布物 release/pasta-recipe.zip（pasta-recipe.exe と THIRD_PARTY_LICENSES.txt の 2 つ）を作る。
.DESCRIPTION
  古い zip を消す → npm ci → tauri build --no-bundle → cargo about → 画面の部品の表示（build/js-licenses.txt）を
  区切りの見出し付きで足す → zip → 中身が 2 つだけかの自己検査、の順に走る。
  どの段で失敗しても終了コード 1 で止まり、半端な zip を残さない。release/ はリポジトリの根の下（コミットしない）。
.EXAMPLE
  pwsh -NoProfile -File editors/recipe/scripts/package.ps1
#>
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
# 外部のコマンドが 0 以外で終われば止める（$LASTEXITCODE を段ごとに見るのと同じ）
$PSNativeCommandUseErrorActionPreference = $true

$AppRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$RepoRoot = (Resolve-Path -LiteralPath (Join-Path $AppRoot '../..')).Path
$ReleaseDir = Join-Path $RepoRoot 'release'
$ZipPath = Join-Path $ReleaseDir 'pasta-recipe.zip'
$ExePath = Join-Path $AppRoot 'src-tauri/target/release/pasta-recipe.exe'
$JsLicenses = Join-Path $AppRoot 'build/js-licenses.txt'
$NativeLicenses = Join-Path $AppRoot 'build/native-licenses.txt'
$Notice = Join-Path $AppRoot 'build/THIRD_PARTY_LICENSES.txt'
$Expected = @('pasta-recipe.exe', 'THIRD_PARTY_LICENSES.txt')

$ok = $false
Push-Location -LiteralPath $AppRoot
try {
    Write-Host '==> 古い zip を消す'
    Remove-Item -LiteralPath $ZipPath -Force -ErrorAction SilentlyContinue
    if (Test-Path -LiteralPath $ZipPath) { throw "古い zip を消せない: $ZipPath" }

    Write-Host '==> npm ci'
    npm ci

    # beforeBuildCommand（npm run build）が dist/ と build/js-licenses.txt を作り直す
    Write-Host '==> tauri build --no-bundle'
    Remove-Item -LiteralPath $JsLicenses -Force -ErrorAction SilentlyContinue
    npx --no-install tauri build --no-bundle
    if (-not (Test-Path -LiteralPath $ExePath)) { throw "exe ができていない: $ExePath" }
    if (-not (Test-Path -LiteralPath $JsLicenses)) { throw "画面の部品の表示ができていない: $JsLicenses" }

    Write-Host '==> cargo about generate'
    cargo about generate -m src-tauri/Cargo.toml -c src-tauri/about.toml --target x86_64-pc-windows-msvc --output-file $NativeLicenses src-tauri/about.hbs

    Write-Host '==> 第三者の表示をつなぐ'
    $bar = '=' * 80
    $heading = @(
        '', $bar, $bar,
        '画面の部品（エディタ・着色の部品・文法。JavaScript で exe に組み込まれている）'
        'Web UI components (editor, highlighter and grammars, bundled into the exe as JavaScript)'
        $bar, $bar, ''
    ) -join "`n"
    $text = (Get-Content -LiteralPath $NativeLicenses -Raw).TrimEnd() + "`n" + $heading + "`n" +
            (Get-Content -LiteralPath $JsLicenses -Raw)
    Set-Content -LiteralPath $Notice -Value $text -Encoding utf8NoBOM -NoNewline

    Write-Host '==> zip を作る'
    New-Item -ItemType Directory -Force -Path $ReleaseDir | Out-Null
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $zip = [System.IO.Compression.ZipFile]::Open($ZipPath, 'Create')
    try {
        [System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, $ExePath, $Expected[0]) | Out-Null
        [System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, $Notice, $Expected[1]) | Out-Null
    } finally { $zip.Dispose() }

    Write-Host '==> 中身を確かめる'
    $zip = [System.IO.Compression.ZipFile]::OpenRead($ZipPath)
    try { $names = @($zip.Entries | ForEach-Object FullName) } finally { $zip.Dispose() }
    if (($names -join '|') -cne ($Expected -join '|')) {
        throw "zip の中身が $($Expected -join '・') の 2 つでない: $($names -join ', ')"
    }

    $ok = $true
    Write-Host "OK: $ZipPath ($((Get-Item -LiteralPath $ZipPath).Length) バイト)"
} catch {
    Write-Host "ERROR: $_" -ForegroundColor Red
} finally {
    Pop-Location
    Remove-Item -LiteralPath $NativeLicenses, $Notice -Force -ErrorAction SilentlyContinue
    if (-not $ok) { Remove-Item -LiteralPath $ZipPath -Force -ErrorAction SilentlyContinue }
}
if (-not $ok) { exit 1 }
