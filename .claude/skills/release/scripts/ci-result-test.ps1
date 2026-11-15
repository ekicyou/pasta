#Requires -Version 7
<#
.SYNOPSIS
  定型コマンド B（ci-result.ps1）の自己テスト。偽の gh（関数）で、ネットワークを使わずに出力の形を確かめる。
.EXAMPLE
  & "$PWD/.claude/skills/release/scripts/ci-result-test.ps1"
#>
$ErrorActionPreference = 'Stop'

# 偽の gh。ci-result.ps1 は子のスコープで動くので、この関数が本物の gh より先に見つかる。
function gh {
    if ($args -contains '--log') {
        "publish-crates`tPublish pasta_core`t2026-11-15T00:00:00.0000000Z status=published"
        "publish-ghost-site`tPublish hello-pasta ghost site`t2026-11-15T00:00:00.0000000Z status=skipped reason=stale-tag"
        "publish-ghost-site`tSummarize ghost site result`t2026-11-15T00:00:00.0000000Z status=failed"
        return
    }
    $step = { param($n) @{ name = $n; conclusion = 'success'; startedAt = '2026-11-15T00:00:00Z'; completedAt = '2026-11-15T00:00:10Z' } }
    @{
        status = 'completed'; conclusion = 'success'; attempt = 1; url = 'https://example.invalid/run/1'; headSha = 'abc'
        jobs   = @(
            @{ name = 'publish-crates'; conclusion = 'success'; steps = @((& $step 'Auth crates.io (pasta_core)'), (& $step 'Publish pasta_core')) }
            @{ name = 'publish-ghost-site'; conclusion = 'success'; steps = @((& $step 'Publish hello-pasta ghost site'), (& $step 'Summarize ghost site result')) }
        )
    } | ConvertTo-Json -Depth 5
}

$out = & "$PSScriptRoot/ci-result.ps1" -Run 1
$expect = @(
    'step | publish-crates | Publish pasta_core | success | 10 秒 | 期限まで 1790 秒'
    'step | publish-ghost-site | Publish hello-pasta ghost site | success | 10 秒'
    'line | attempt 1 | publish-crates | Publish pasta_core | status=published'
    'line | attempt 1 | publish-ghost-site | Publish hello-pasta ghost site | status=skipped reason=stale-tag'
)
$missing = @($expect | Where-Object { $_ -notin $out })
$extra = @($out | Where-Object { $_ -like '*Summarize*' })
if ($missing -or $extra) {
    $out
    $missing | ForEach-Object { "不合格: 出力に無い: $_" }
    $extra | ForEach-Object { "不合格: 出してはいけない: $_" }
    exit 1
}
'合格: ci-result.ps1 が publish-ghost-site の step と status= の行を読む'
