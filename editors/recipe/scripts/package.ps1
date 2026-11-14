#Requires -Version 7.4
<#
.SYNOPSIS
  配布物 release/pasta-recipe.<arch>.zip（pasta-recipe.exe と THIRD_PARTY_LICENSES.txt の 2 つ）を arch ごとに作る。
.DESCRIPTION
  -Arch は x64・arm64（既定は x64・arm64 の順で両方）。npm ci を 1 回走らせてから、arch ごとに
  その arch の zip を消す → tauri build --no-bundle --target <triple> → cargo about --target <triple> →
  画面の部品の表示（build/js-licenses.txt）を区切りの見出し付きで足す → zip → 中身が 2 つだけかの自己検査、の順に走る。
  どの段で失敗しても終了コード 1 で止まり、その arch の半端な zip を残さない。残りの arch は作らず、先に作り終えた arch の zip は残す。
  作る arch の zip だけを作り直し、もう一方の zip には触れない。release/ はリポジトリの根の下（コミットしない）。
.EXAMPLE
  pwsh -NoProfile -File editors/recipe/scripts/package.ps1
.EXAMPLE
  pwsh -NoProfile -File editors/recipe/scripts/package.ps1 -Arch arm64
#>
param(
    [string[]]$Arch = @('x64', 'arm64')
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
# 外部のコマンドが 0 以外で終われば止める（$LASTEXITCODE を段ごとに見るのと同じ）
$PSNativeCommandUseErrorActionPreference = $true

$Triples = [ordered]@{ 'x64' = 'x86_64-pc-windows-msvc'; 'arm64' = 'aarch64-pc-windows-msvc' }
$AppRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$RepoRoot = (Resolve-Path -LiteralPath (Join-Path $AppRoot '../..')).Path
$ReleaseDir = Join-Path $RepoRoot 'release'
$JsLicenses = Join-Path $AppRoot 'build/js-licenses.txt'
$NativeLicenses = Join-Path $AppRoot 'build/native-licenses.txt'
$Notice = Join-Path $AppRoot 'build/THIRD_PARTY_LICENSES.txt'
$Expected = @('pasta-recipe.exe', 'THIRD_PARTY_LICENSES.txt')

$ok = $false
$ZipPath = $null  # 作っている途中の arch の zip（失敗のときはこれだけを消す）
Push-Location -LiteralPath $AppRoot
try {
    # pwsh -File は `-Arch x64,arm64` を 1 つの文字列で渡すため、param の ValidateSet は使えない。分けてから受け付ける値と照合する。
    $Arch = @($Arch -split ',' | ForEach-Object Trim)
    $bad = @($Arch | Where-Object { $Triples.Keys -cnotcontains $_ })
    # @('') は偽になるので要素の数で見る（空の要素も弾く）
    if ($bad.Count) { throw "-Arch に受け付けない値: $(($bad | ForEach-Object { "'$_'" }) -join ', ')（受け付けるのは $($Triples.Keys -join ', ')）" }

    Write-Host '==> npm ci'
    npm ci

    foreach ($a in $Arch) {
        $triple = $Triples[$a]
        $ZipPath = Join-Path $ReleaseDir "pasta-recipe.$a.zip"
        # x64 にも --target を付けて exe の置き場を arch ごとに分ける（前の arch の exe を詰める取り違えを防ぐ）
        $ExePath = Join-Path $AppRoot "src-tauri/target/$triple/release/pasta-recipe.exe"

        Write-Host "==> [$a] 古い zip を消す"
        Remove-Item -LiteralPath $ZipPath -Force -ErrorAction SilentlyContinue
        if (Test-Path -LiteralPath $ZipPath) { throw "古い zip を消せない: $ZipPath" }

        # beforeBuildCommand（npm run build）が dist/ と build/js-licenses.txt を作り直す
        Write-Host "==> [$a] tauri build --no-bundle --target $triple"
        Remove-Item -LiteralPath $JsLicenses -Force -ErrorAction SilentlyContinue
        npx --no-install tauri build --no-bundle --target $triple
        if (-not (Test-Path -LiteralPath $ExePath)) { throw "exe ができていない: $ExePath" }
        if (-not (Test-Path -LiteralPath $JsLicenses)) { throw "画面の部品の表示ができていない: $JsLicenses" }

        Write-Host "==> [$a] cargo about generate --target $triple"
        cargo about generate -m src-tauri/Cargo.toml -c src-tauri/about.toml --target $triple --output-file $NativeLicenses src-tauri/about.hbs

        Write-Host "==> [$a] 第三者の表示をつなぐ"
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

        Write-Host "==> [$a] zip を作る"
        New-Item -ItemType Directory -Force -Path $ReleaseDir | Out-Null
        Add-Type -AssemblyName System.IO.Compression.FileSystem
        $zip = [System.IO.Compression.ZipFile]::Open($ZipPath, 'Create')
        try {
            [System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, $ExePath, $Expected[0]) | Out-Null
            [System.IO.Compression.ZipFileExtensions]::CreateEntryFromFile($zip, $Notice, $Expected[1]) | Out-Null
        } finally { $zip.Dispose() }

        Write-Host "==> [$a] 中身を確かめる"
        $zip = [System.IO.Compression.ZipFile]::OpenRead($ZipPath)
        try { $names = @($zip.Entries | ForEach-Object FullName) } finally { $zip.Dispose() }
        if (($names -join '|') -cne ($Expected -join '|')) {
            throw "zip の中身が $($Expected -join '・') の 2 つでない: $($names -join ', ')"
        }

        Write-Host "OK: $ZipPath ($((Get-Item -LiteralPath $ZipPath).Length) バイト)"
        $ZipPath = $null  # 作り終えた zip は、後の arch が失敗しても残す
    }

    $ok = $true
} catch {
    Write-Host "ERROR: $_" -ForegroundColor Red
} finally {
    Pop-Location
    Remove-Item -LiteralPath $NativeLicenses, $Notice -Force -ErrorAction SilentlyContinue
    if (-not $ok -and $ZipPath) { Remove-Item -LiteralPath $ZipPath -Force -ErrorAction SilentlyContinue }
}
if (-not $ok) { exit 1 }
