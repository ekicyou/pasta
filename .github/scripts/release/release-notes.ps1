#Requires -Version 7
<#
.SYNOPSIS
  1 つ前のリリースタグからリリースタグまでのコミットを分類し、リリースノート（Markdown）を作る。
.DESCRIPTION
  前のタグは、形の合うタグ（vX.Y.Z）のうち版で並べて -Tag より小さい最大のもの。無ければ全履歴を使う。
  マージコミットは除く。HEAD ではなくタグを基準にする。
  -OutFile を省くとノートを標準出力へ書く。
.EXAMPLE
  pwsh -NoProfile -File .github/scripts/release/release-notes.ps1 -Tag v0.3.7 -Repo ekicyou/pasta -OutFile notes.md
#>
param(
    [Parameter(Mandatory)][string]$Tag,
    [Parameter(Mandatory)][string]$Repo,
    [string]$OutFile,
    [string]$WorkspaceRoot = (Join-Path $PSScriptRoot '../../..')
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# 見出しの並び。type → 見出し。6 種以外の type と Conventional Commits でない件名は Maintenance へ入れる（黙って落とさない）。
$Headings = [ordered]@{
    feat     = '✨ Features'
    fix      = '🐛 Bug Fixes'
    refactor = '♻️ Refactoring'
    docs     = '📝 Documentation'
    test     = '🧪 Tests'
    chore    = '🔧 Maintenance'
}
$Fallback = 'chore'
# 形の合うタグ: 大文字小文字を区別し ASCII 数字だけ、末尾は \z。
$TagShape = '^v([0-9]+\.[0-9]+\.[0-9]+)\z'

function Fail([string]$Message) {
    [Console]::Error.WriteLine("release-notes: $Message")
    exit 1
}

if ($Tag -cnotmatch $TagShape) { Fail "タグ ``$Tag`` は ``vX.Y.Z``（数字 3 つ）の形ではない" }
$version = [version]$Matches[1]

# git の出力（日本語・絵文字の件名）を UTF-8 で読む。コンソールのコードページは終了時に戻す。
$savedEncoding = [Console]::OutputEncoding
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
try {
    git -C $WorkspaceRoot rev-parse --verify --quiet "refs/tags/$Tag" *> $null
    if ($LASTEXITCODE -ne 0) { Fail "タグ ``$Tag`` が見つからない" }

    # 1 つ前のリリースタグ
    $prev = $null
    $prevVersion = $null
    $tags = @(git -C $WorkspaceRoot tag -l 'v[0-9]*.[0-9]*.[0-9]*')
    if ($LASTEXITCODE -ne 0) { Fail 'タグの一覧を取れない' }
    foreach ($t in $tags) {
        if ($t -cnotmatch $TagShape) { continue }
        $v = [version]$Matches[1]
        if ($v -lt $version -and ($null -eq $prevVersion -or $v -gt $prevVersion)) { $prev = $t; $prevVersion = $v }
    }

    $range = if ($prev) { "$prev..$Tag" } else { $Tag }
    $subjects = @(git -C $WorkspaceRoot log $range --no-merges --format=%s)
    if ($LASTEXITCODE -ne 0) { Fail "``git log $range`` が失敗した" }
}
finally {
    [Console]::OutputEncoding = $savedEncoding
}

# 分類（git log の順 = 新しい順のまま）
$groups = @{}
foreach ($key in $Headings.Keys) { $groups[$key] = [System.Collections.Generic.List[string]]::new() }
# type と scope は大文字小文字を区別しない（Conventional Commits の規定）。
foreach ($s in $subjects) {
    $key = $Fallback
    if ($s -match '^(\w+)(?:\(([^)]*)\))?!?:\s') {
        if ($Matches[2] -eq 'spec') { continue }   # spec スコープは除く
        if ($Headings.Contains($Matches[1])) { $key = $Matches[1].ToLowerInvariant() }
    }
    $groups[$key].Add($s)
}

$lines = [System.Collections.Generic.List[string]]::new()
foreach ($key in $Headings.Keys) {
    if ($groups[$key].Count -eq 0) { continue }   # 空の見出しは出さない
    $lines.Add("## $($Headings[$key])")
    foreach ($s in $groups[$key]) { $lines.Add("- $s") }
    $lines.Add('')
}
if ($prev) { $lines.Add("**Full Changelog**: https://github.com/$Repo/compare/$prev...$Tag") }
$notes = ($lines -join "`n").TrimEnd() + "`n"

if ($OutFile) {
    $path = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OutFile)
    [IO.File]::WriteAllText($path, $notes, [Text.UTF8Encoding]::new($false))
    $counted = ($Headings.Keys | ForEach-Object { $groups[$_].Count } | Measure-Object -Sum).Sum
    Write-Output "release-notes: $Tag の前のタグ = $(if ($prev) { $prev } else { '(無し: 全履歴)' })、$counted 件 → $path"
}
else {
    Write-Output $notes
}
exit 0
