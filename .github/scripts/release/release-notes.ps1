#Requires -Version 7
<#
.SYNOPSIS
  1 つ前のリリースタグからリリースタグまでのコミットを分類し、詳しいリリースノート（Markdown）を作る。
.DESCRIPTION
  前のタグは、形の合うタグ（vX.Y.Z）のうち版で並べて -Tag より小さい最大のもの。無ければ全履歴を使う。
  範囲の終わりは -Head があればそれ、無ければ -Tag。-Head を指定すると、タグが在ることを求めない（タグを打つ前の手元の確認用）。
  マージコミットは除く。件名から種類・スコープ・`!` の接頭辞を外し、内部の作業は載せない。
  -OutFile を省くとノートを標準出力へ書く。
  -BodyFile を指定すると、見どころのファイル（doc/release-notes/{Tag}.md）の後に、詳しいリリースノートへのリンクと件数の行を足して、リリースの本文として書く。
  失敗したときは、どのファイルも書かない。
.EXAMPLE
  pwsh -NoProfile -File .github/scripts/release/release-notes.ps1 -Tag v0.3.7 -Repo ekicyou/pasta -OutFile notes.md -BodyFile body.md
.EXAMPLE
  pwsh -NoProfile -File .github/scripts/release/release-notes.ps1 -Tag v0.3.9 -Repo ekicyou/pasta -Head origin/main
#>
param(
    [Parameter(Mandatory)][string]$Tag,
    [Parameter(Mandatory)][string]$Repo,
    [string]$Head,
    [string]$OutFile,
    [string]$BodyFile,
    [string]$WorkspaceRoot = (Join-Path $PSScriptRoot '../../..')
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# 見出しの並び。breaking は `!` の付いた件名、other は 5 種以外の種類と決まった形でない件名（黙って落とさない）。
$Headings = [ordered]@{
    breaking = '⚠️ 動きが変わる変更'
    feat     = '✨ 新機能'
    fix      = '🐛 不具合の修正'
    refactor = '♻️ リファクタリング'
    docs     = '📝 ドキュメント'
    test     = '🧪 テスト'
    other    = '🔧 そのほか'
}
# 内部の作業のスコープ（種類は問わない）。ほかに `kiro-` で始まるスコープ・種類 spec・chore(release) も内部の作業。
# 決まりを変えるときは doc/release-notes/README.md の「内部の作業」と合わせる。
$InternalScopes = @('spec', 'discovery', 'roadmap', 'steering', 'skills', 'cc-sdd', 'release-workflow')
# 形の合うタグ: 大文字小文字を区別し ASCII 数字だけ、末尾は \z。
$TagShape = '^v([0-9]+\.[0-9]+\.[0-9]+)\z'

function Fail([string]$Message) {
    [Console]::Error.WriteLine("release-notes: $Message")
    exit 1
}

# 内部の作業か（種類とスコープは大文字小文字を区別しない。-eq・-in・-like は既定で区別しない）
function Test-Internal([string]$Type, [string]$Scope) {
    ($Type -eq 'spec') -or ($Scope -in $InternalScopes) -or ($Scope -like 'kiro-*') -or ($Type -eq 'chore' -and $Scope -eq 'release')
}

if ($Tag -cnotmatch $TagShape) { Fail "タグ ``$Tag`` は ``vX.Y.Z``（数字 3 つ）の形ではない" }
$version = [version]$Matches[1]
$end = if ($Head) { $Head } else { $Tag }

# 見どころのファイル（-BodyFile のときだけ）。何かを書き出す前に確かめる。
if ($BodyFile) {
    $hlPath = Join-Path $WorkspaceRoot "doc/release-notes/$Tag.md"
    if (-not (Test-Path -LiteralPath $hlPath -PathType Leaf)) { Fail "見どころのファイル ``doc/release-notes/$Tag.md`` が無い" }
    # 末尾の空白だけを落とす。改行は LF にそろえる（ほかは書き換えない）
    $highlights = ([IO.File]::ReadAllText($hlPath) -replace "`r`n", "`n").TrimEnd()
    if (-not $highlights) { Fail "見どころのファイル ``doc/release-notes/$Tag.md`` が空" }
}

# git の出力（日本語・絵文字の件名）を UTF-8 で読む。コンソールのコードページは終了時に戻す。
$savedEncoding = [Console]::OutputEncoding
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)
try {
    if (-not $Head) {
        git -C $WorkspaceRoot rev-parse --verify --quiet "refs/tags/$Tag" *> $null
        if ($LASTEXITCODE -ne 0) { Fail "タグ ``$Tag`` が見つからない" }
    }

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

    $range = if ($prev) { "$prev..$end" } else { $end }
    $subjects = @(git -C $WorkspaceRoot log $range --no-merges --format=%s --)
    if ($LASTEXITCODE -ne 0) { Fail "``git log $range`` が失敗した" }
}
finally {
    [Console]::OutputEncoding = $savedEncoding
}

# 件名の扱い（git log の順 = 新しい順のまま）。件名ごとに上から当てる。
$groups = @{}
foreach ($key in $Headings.Keys) { $groups[$key] = [System.Collections.Generic.List[string]]::new() }
foreach ($s in $subjects) {
    # 1. 決まった形でない → そのまま「そのほか」
    # `!` はスコープの後（正しい形）でも前（`feat!(scope):` の書き間違い）でも受け付ける
    if ($s -notmatch '^(\w+)(!)?(?:\(([^)]*)\))?(!)?:\s+(.+)$') { $groups['other'].Add($s); continue }
    $type = $Matches[1].ToLowerInvariant()
    $scope = $Matches[3]
    $breaking = [bool]$Matches[2] -or [bool]$Matches[4]
    $text = $Matches[5]   # 本文だけを載せる。末尾の (#83) は本文の一部なので残る
    # 2. 内部の作業 → 載せない
    if (Test-Internal $type $scope) { continue }
    # 3. `!` → 動きが変わる変更、4. 種類の見出し（5 種以外は「そのほか」）
    $key = if ($breaking) { 'breaking' } elseif ($type -in 'feat', 'fix', 'refactor', 'docs', 'test') { $type } else { 'other' }
    $groups[$key].Add($text)
}
$counted = ($Headings.Keys | ForEach-Object { $groups[$_].Count } | Measure-Object -Sum).Sum

$lines = [System.Collections.Generic.List[string]]::new()
$lines.Add("# pasta $Tag の詳しいリリースノート")
$lines.Add('')
if ($counted -eq 0) {
    $lines.Add('載せる変更はありません。')
    $lines.Add('')
}
foreach ($key in $Headings.Keys) {
    if ($groups[$key].Count -eq 0) { continue }   # 空の見出しは出さない
    $lines.Add("## $($Headings[$key])")
    foreach ($s in $groups[$key]) { $lines.Add("- $s") }
    $lines.Add('')
}
# -Head を指定しても、リンクの終わりは -Tag と書く
if ($prev) { $lines.Add("**Full Changelog**: https://github.com/$Repo/compare/$prev...$Tag") }
$notes = ($lines -join "`n").TrimEnd() + "`n"

$utf8 = [Text.UTF8Encoding]::new($false)
if ($BodyFile) {
    $asset = "release-notes-$Tag.md"
    $body = "$highlights`n`n---`n`nすべての変更（$counted 件）は、Assets の [$asset](https://github.com/$Repo/releases/download/$Tag/$asset) にまとめてありますわ。`n"
    $bodyPath = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($BodyFile)
}
if ($OutFile) {
    $path = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OutFile)
    [IO.File]::WriteAllText($path, $notes, $utf8)
}
if ($BodyFile) {
    # 本文を書けなければ、書いたノートも消す（片方だけを残さない）
    try { [IO.File]::WriteAllText($bodyPath, $body, $utf8) }
    catch {
        if ($OutFile) { Remove-Item -LiteralPath $path -ErrorAction SilentlyContinue }
        Fail "本文を書けない: $($_.Exception.Message)"
    }
}
if ($OutFile) {
    Write-Output "release-notes: $Tag の前のタグ = $(if ($prev) { $prev } else { '(無し: 全履歴)' })、$counted 件 → $path"
}
else {
    Write-Output $notes
}
exit 0
