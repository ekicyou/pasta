#Requires -Version 7
<#
.SYNOPSIS
  定型コマンド D: 見どころのファイルの 5 つの確認（release スキルの段 1・4）。読むだけで、何も変えない。
.DESCRIPTION
  作業ディレクトリの doc/release-notes/v{V}.md と .kiro/specs/ を読む。材料は .github/scripts/release/release-notes.ps1 の出力。
  5 行とも「合格」であること。リポジトリのルートを作業ディレクトリにして実行する。
.PARAMETER Version
  リリースする版（X.Y.Z）。
.PARAMETER Head
  材料の範囲の終わり（タグを打つ前は origin/main）。省くとタグ v{V} までを材料にする。
.EXAMPLE
  & "$PWD/.claude/skills/release/scripts/highlights-check.ps1" -Version 0.3.9 -Head origin/main
#>
param([Parameter(Mandatory)][string]$Version, [string]$Head)
$V = $Version -replace '^v', ''
$root = (Get-Location).Path
$rel = "doc/release-notes/v$V.md"
$path = Join-Path $root $rel
$text = if (Test-Path -LiteralPath $path -PathType Leaf) { [IO.File]::ReadAllText($path) } else { '' }
$ok = { param($c) if ($c) { '合格' } else { '不合格' } }

if ([string]::IsNullOrWhiteSpace($text)) {
    "見どころのファイル | $rel | 不合格"
    foreach ($n in 'Claudia の出典リンク', 'PR の番号', '動きが変わる点', 'spec 名') { "$n | 見どころのファイルが無いか空 | 不合格" }
    exit 0
}
"見どころのファイル | $rel | 合格"

# 2. 出典リンクがちょうど 1 つで、その前に素の Claudia が無い
$links = [regex]::Matches($text, '\[[^\]]*Claudia[^\]]*\]\(https://github\.com/ponapalt/claudia\)')
$bare = $links.Count -gt 0 -and $text.Substring(0, $links[0].Index).Contains('Claudia')
"Claudia の出典リンク | $($links.Count)$(if ($bare) { '、リンクの前に Claudia' }) | $(& $ok ($links.Count -eq 1 -and -not $bare))"

# 3・4. 材料（詳しいリリースノートの形）と PR の番号を突き合わせる
$numPattern = '(?<![&\w#])#(\d+)\b'
$fileNums = @([regex]::Matches($text, $numPattern) | ForEach-Object { $_.Groups[1].Value } | Select-Object -Unique)
$tmp = Join-Path ([IO.Path]::GetTempPath()) "highlights-check-$([guid]::NewGuid().ToString('N')).md"
$notesArgs = @('-Tag', "v$V", '-Repo', 'ekicyou/pasta', '-WorkspaceRoot', $root, '-OutFile', $tmp)
if ($Head) { $notesArgs += @('-Head', $Head) }
$err = & ([Environment]::ProcessPath) -NoProfile -File (Join-Path $PSScriptRoot '../../../../.github/scripts/release/release-notes.ps1') @notesArgs 2>&1 | Out-String
if ($LASTEXITCODE -ne 0 -or -not (Test-Path -LiteralPath $tmp)) {
    $why = "材料を出せない（release-notes.ps1 の終了コード $LASTEXITCODE`: $($err.Trim() -replace '\s+', ' ')）"
    "PR の番号 | $why | 不合格"
    "動きが変わる点 | $why | 不合格"
}
else {
    try { $material = [IO.File]::ReadAllText($tmp) } finally { Remove-Item -LiteralPath $tmp -ErrorAction SilentlyContinue }
    $materialNums = @([regex]::Matches($material, $numPattern) | ForEach-Object { $_.Groups[1].Value })
    $unknown = @($fileNums | Where-Object { $materialNums -notcontains $_ } | ForEach-Object { "#$_" })
    "PR の番号 | $(if ($unknown) { $unknown -join ', ' } else { 'なし' }) | $(& $ok (-not $unknown))"
    # 見出し「動きが変わる変更」の下（次の見出しまで）の番号
    $breaking = if ($material -match '(?ms)^## ⚠️ 動きが変わる変更\r?$(.*?)(?=^#|^\*\*Full Changelog\*\*|\z)') { $Matches[1] } else { '' }
    $absent = @([regex]::Matches($breaking, $numPattern) | ForEach-Object { $_.Groups[1].Value } | Where-Object { $fileNums -notcontains $_ } | ForEach-Object { "#$_" })
    "動きが変わる点 | $(if ($absent) { $absent -join ', ' } else { 'なし' }) | $(& $ok (-not $absent))"
}

# 5. spec 名（.kiro/specs/ と .kiro/specs/completed/ のフォルダ名）が単語として出てこない。大文字と小文字を区別する
$specs = @(foreach ($d in '.kiro/specs', '.kiro/specs/completed') {
        Get-ChildItem -LiteralPath (Join-Path $root $d) -Directory -ErrorAction SilentlyContinue | ForEach-Object Name
    }) | Where-Object { $_ -cne 'completed' } | Select-Object -Unique
$found = @($specs | Where-Object { $text -cmatch "(?<![A-Za-z0-9_-])$([regex]::Escape($_))(?![A-Za-z0-9_-])" })
"spec 名 | $(if ($found) { $found -join ', ' } else { 'なし' }) | $(& $ok (-not $found))"
