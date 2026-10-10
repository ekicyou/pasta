#Requires -Version 7
<#
.SYNOPSIS
  定型コマンド B: リリース CI の結果の読み取り（release-workflow の段 7・10）。読むだけで、何も変えない。
.DESCRIPTION
  実行・job・公開の step の結論と、公開の step がログに書いた status= の行を出す。実行が completed になってから使う。
  出力の形は design.md の Data Models。リポジトリのルートを作業ディレクトリにして実行する。
.PARAMETER Run
  リリース CI の実行の ID。
.EXAMPLE
  & "$PWD/.kiro/specs/release-workflow/scripts/ci-result.ps1" -Run 1234567890
#>
param([Parameter(Mandatory)][string]$Run)
$r = (gh run view $RUN --json status,conclusion,attempt,url,headSha,jobs) -join "`n" | ConvertFrom-Json
"run | $RUN | $($r.status) | $(if ($r.conclusion) { $r.conclusion } else { '-' }) | attempt $($r.attempt) | $($r.url)"
$r.jobs | ForEach-Object { "job | $($_.name) | $($_.conclusion)" }
$r.jobs | Where-Object { $_.name -in 'publish-crates', 'publish-vsce', 'github-release' } | ForEach-Object {
    $job = $_.name
    $_.steps | Where-Object { $_.name -match '^(Auth crates\.io|Publish |Azure login|Create GitHub Release)' } | ForEach-Object {
        $sec = if ($_.startedAt -and $_.completedAt) { [int](([datetime]$_.completedAt) - ([datetime]$_.startedAt)).TotalSeconds } else { 0 }
        "step | $job | $($_.name) | $($_.conclusion) | $sec 秒$(if ($_.name -like 'Publish pasta_*') { " | 期限まで $(1800 - $sec) 秒" })"
    }
}
$pattern = '^(publish-crates|publish-vsce|github-release)\t(Publish [^\t]*|Create GitHub Release)\t\S+ (status=(published|skipped|failed)\b.*)$'
1..$r.attempt | ForEach-Object {
    $a = $_
    gh run view $RUN --attempt $a --log 2>$null | Select-String -Pattern $pattern | ForEach-Object {
        $g = $_.Matches[0].Groups
        "line | attempt $a | $($g[1].Value) | $($g[2].Value) | $($g[3].Value.Trim())"
    }
}
