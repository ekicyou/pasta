#Requires -Version 7
<#
.SYNOPSIS
  リリースタグが最新の Release かを判定し、最新なら hello-pasta の置き場所（manual.yml の公開）の更新に進む。結果を status/reason で出す。
.DESCRIPTION
  最新のタグは gh api repos/$GITHUB_REPOSITORY/releases/latest（-LatestTag で上書き）。認証は環境変数 GH_TOKEN。
  -Tag が最新より古ければ何も起こさず skipped（reason stale-tag）。最新より新しければ 10 秒おきに取り直し（-LatestWaitSeconds まで）、
  それでも新しいままなら failed（reason transient）。最新のタグが vX.Y.Z の形でなければ failed（reason publish。再実行では直らない）。
  最新なら、Release の hello-pasta.nar を取り（見比べの正本。失敗は transient）、manual.yml を -Ref で起動する（REST の dispatch・return_run_details）。
  応答に workflow_run_id が無い・API が拒否したときは、起動前の時刻より後に作られた最新の run を gh run list（--branch は -Ref）から探す。無ければ dispatch。
  run の完了を -RunPollSeconds おきに待ち（上限 -RunTimeoutMinutes。過ぎたら transient）、結論が success・cancelled なら見比べへ、それ以外は build。
  -DescriptPath の homeurl は、古いタグの判定と run の完了を終えた後・見比べの直前にだけ読む（無いときは publish）。
  見比べは ghost-site.mjs verify-live に --tree "<タグ>:crates/pasta_sample_ghost/ghosts/hello-pasta" を渡し、公開した updates.txt が Release の nar の側の行と一致するかを見る（一致で published、合わなければ verify）。Release には触れない（読むだけ）。再実行は最初からやり直す。
  -Ref が main 以外（リハーサル）では、deploy が main 限定なので見比べず、結論の判定までで終える（status を書かない）。
  -DryRun では判定だけを行い、起動の手前で行うはずの操作を表示して終える。
  skipped のときも reason を出す（置き場所の行に理由を表に出すため）。summary にはタグ・最新タグ・run の URL・見比べの回数を出す。
.EXAMPLE
  pwsh -NoProfile -File .github/scripts/release/publish-ghost-site.ps1 -Tag v0.6.2 -DescriptPath crates/pasta_sample_ghost/ghosts/hello-pasta/ghost/master/descript.txt -DryRun
#>
param(
    [Parameter(Mandatory)][string]$Tag,
    [Parameter(Mandatory)][string]$DescriptPath,
    # テスト用: 最新のタグ（releases/latest）の代わり。取り直しも同じ値を返す。
    [string]$LatestTag,
    # 起動する manual.yml の ref（リハーサルではブランチを渡す。deploy は main 限定なので組み立てまで走る）
    [string]$Ref = 'main',
    [int]$LatestWaitSeconds = 60,
    [int]$RunTimeoutMinutes = 30,
    [int]$RunPollSeconds = 20,
    [int]$VerifyAttempts = 30,
    [int]$VerifyIntervalSeconds = 30,
    # テスト用: gh と node の代わり（自己テストは偽の .ps1 を渡す。`& $GhCommand <引数>` で呼ぶ）
    [string]$GhCommand = 'gh',
    [string]$NodeCommand = 'node',
    [switch]$DryRun
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$LatestPollSeconds = 10
$TagPattern = '^v[0-9]+\.[0-9]+\.[0-9]+\z'
# run list で起動後の run を探す回数（間は -RunPollSeconds。run は起動から少し遅れて一覧に出る）
$RunListAttempts = 6
# 起動前の時刻と run の作成時刻を比べるときの、こちらの時計と GitHub の時計のずれの許し
$ClockSkewSeconds = 10
$started = Get-Date
$latest = ''
$runUrl = ''
$verifyCount = ''

# $GITHUB_OUTPUT / $GITHUB_STEP_SUMMARY が未設定なら標準出力へ（手元での実行）。
# 値を返す関数の中からも呼ぶので、Write-Output でなく Write-Host（戻り値に混ざらない）。
function Write-To([string]$EnvName, [string]$Text) {
    $path = [Environment]::GetEnvironmentVariable($EnvName)
    if ($path) { Add-Content -LiteralPath $path -Value $Text -Encoding utf8 }
    else { Write-Host $Text }
}

# status と reason を書いて終える（skipped でも reason を書く）。exit 0 = published / skipped、1 = failed。
function Complete([string]$Status, [string]$Reason, [string]$Message) {
    Write-To GITHUB_OUTPUT "status=$Status"
    if ($Reason) { Write-To GITHUB_OUTPUT "reason=$Reason" }
    if ($env:GITHUB_OUTPUT) { Write-Host "status=$Status$(if ($Reason) { " reason=$Reason" })" }
    $secs = [int]((Get-Date) - $started).TotalSeconds
    $detail = @(if ($runUrl) { "run $runUrl" }) + @(if ($verifyCount) { "見比べ $verifyCount 回目で一致" }) + "所要 $secs 秒"
    Write-To GITHUB_STEP_SUMMARY "- 置き場所（hello-pasta）``$Tag``（最新 ``$(if ($latest) { $latest } else { '(未取得)' })``）: **$Status**$(if ($Reason) { "（reason=$Reason）" }) — $Message（$($detail -join '・')）"
    exit $(if ($Status -eq 'failed') { 1 } else { 0 })
}

# gh を呼び、終了コード・標準出力・全出力（標準エラーを含む）を返す。
function Invoke-Gh([string[]]$GhArgs) {
    $out = [System.Collections.Generic.List[string]]::new()
    $all = [System.Collections.Generic.List[string]]::new()
    & $GhCommand @GhArgs 2>&1 | ForEach-Object {
        $line = "$_"
        if ($_ -isnot [System.Management.Automation.ErrorRecord]) { $out.Add($line) }
        $all.Add($line)
    }
    [pscustomobject]@{ Code = $LASTEXITCODE; Out = $out -join "`n"; Text = $all -join "`n" }
}

# 最新の Release のタグ。取れないときは transient、形が違うときは再実行で直らないので publish で失敗する。
function Get-LatestTag {
    if ($LatestTag) { $t = $LatestTag }
    else {
        $r = Invoke-Gh @('api', "repos/$env:GITHUB_REPOSITORY/releases/latest", '--jq', '.tag_name')
        if ($r.Code -ne 0) { Complete failed transient "最新の Release を取れない（終了コード $($r.Code): $($r.Text)）。再実行で続く" }
        $t = $r.Out.Trim()
    }
    if ($t -cnotmatch $TagPattern) { Complete failed publish "最新の Release のタグ ``$t`` が vX.Y.Z の形でない。判定できない" }
    $t
}

# タグの数字 3 つ（[version] は数字ごとに比べる。0.10.0 > 0.9.0）
function ConvertTo-Version([string]$T) { [version]$T.Substring(1) }

try {
    if ($Tag -cnotmatch $TagPattern) { Complete failed publish "タグ ``$Tag`` が vX.Y.Z の形でない" }
    if (-not $LatestTag -and -not $env:GITHUB_REPOSITORY) { Complete failed publish '環境変数 GITHUB_REPOSITORY が無い（-LatestTag も無い）' }

    # (1) 最新のタグとの比較（起動の前に判定し、古いタグでは何も変えない。6.3）
    $latest = Get-LatestTag
    $retries = [int][Math]::Ceiling($LatestWaitSeconds / $LatestPollSeconds)
    for ($i = 0; $i -lt $retries -and (ConvertTo-Version $Tag) -gt (ConvertTo-Version $latest); $i++) {
        Start-Sleep -Seconds $LatestPollSeconds
        $latest = Get-LatestTag
    }
    $cmp = (ConvertTo-Version $Tag).CompareTo((ConvertTo-Version $latest))
    if ($cmp -lt 0) { Complete skipped stale-tag "最新のタグ ``$latest`` より古いタグの再実行。起動せず、置き場所を変えていない" }
    if ($cmp -gt 0) { Complete failed transient "$LatestWaitSeconds 秒待っても最新のタグが ``$latest`` のまま（``$Tag`` が最新にならない）。再実行で続く" }

    if ($DryRun) {
        Write-Output "dry-run: ``$Tag`` は最新の Release。行うはずの操作:"
        Write-Output "  gh release download $Tag --pattern hello-pasta.nar（見比べの正本）"
        Write-Output "  POST repos/$env:GITHUB_REPOSITORY/actions/workflows/manual.yml/dispatches ref=$Ref return_run_details=true"
        Write-Output "  run の完了を待つ（$RunPollSeconds 秒おき・最大 $RunTimeoutMinutes 分）"
        if ($Ref -ne 'main') { Write-Output "  リハーサル（ref $Ref）なので結論の判定までで終え、見比べない" }
        else { Write-Output "  ``$DescriptPath`` の homeurl を読み、verify-live で updates.txt を Release の nar の側の行と見比べる（--tree）（$VerifyIntervalSeconds 秒 × $VerifyAttempts 回）" }
        exit 0
    }

    $repo = $env:GITHUB_REPOSITORY
    if (-not $repo) { Complete failed publish '環境変数 GITHUB_REPOSITORY が無い' }

    # (2) Release の nar（見比べの正本）。毎回新しいフォルダに取る（再実行でも前の nar を使わない）。
    $tempRoot = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { [IO.Path]::GetTempPath() }
    $narDir = Join-Path $tempRoot "ghost-site-$([guid]::NewGuid().ToString('N'))"
    $nar = Join-Path $narDir 'hello-pasta.nar'
    $r = Invoke-Gh @('release', 'download', $Tag, '-R', $repo, '--pattern', 'hello-pasta.nar', '--dir', $narDir)
    if ($r.Code -ne 0 -or -not (Test-Path -LiteralPath $nar -PathType Leaf)) { Complete failed transient "Release ``$Tag`` の hello-pasta.nar を取れない（終了コード $($r.Code): $($r.Text)）。再実行で続く" }

    # (3) 起動。応答に run の id が無い・拒否されたときは、起動前の時刻より後に作られた最新の run を探す。
    $since = [DateTimeOffset]::UtcNow
    $r = Invoke-Gh @('api', '-X', 'POST', "repos/$repo/actions/workflows/manual.yml/dispatches", '-f', "ref=$Ref", '-F', 'return_run_details=true')
    $runId = $null
    if ($r.Code -eq 0 -and $r.Out.Trim()) {
        try { $runId = ($r.Out | ConvertFrom-Json).PSObject.Properties['workflow_run_id']?.Value } catch { $runId = $null }
    }
    if ($r.Code -ne 0) { Write-Host "manual.yml の起動の API が拒否した（終了コード $($r.Code): $($r.Text)）。run の一覧から探す" }
    elseif (-not $runId) { Write-Host '起動の応答に workflow_run_id が無い。run の一覧から探す' }
    for ($i = 0; -not $runId -and $i -lt $RunListAttempts; $i++) {
        if ($i) { Start-Sleep -Seconds $RunPollSeconds }
        $l = Invoke-Gh @('run', 'list', '-R', $repo, '--workflow', 'manual.yml', '--branch', $Ref, '--event', 'workflow_dispatch', '--json', 'databaseId,createdAt,url')
        if ($l.Code -ne 0) { continue }
        $pick = @($l.Out | ConvertFrom-Json) | Where-Object { [DateTimeOffset]$_.createdAt -ge $since.AddSeconds(-$ClockSkewSeconds) } |
            Sort-Object { [DateTimeOffset]$_.createdAt } -Descending | Select-Object -First 1
        if ($pick) { $runId = $pick.databaseId; $runUrl = $pick.url }
    }
    if (-not $runId) { Complete failed dispatch "manual.yml（ref $Ref）の起動後の run を特定できない。権限（actions: write）と API の応答をログで確かめる" }
    Write-Host "起動した run: $runId"

    # (4) 完了を待つ。gh run view の一時的な失敗は待ちの中で取り直す。
    $deadline = (Get-Date).AddMinutes($RunTimeoutMinutes)
    while ($true) {
        $v = Invoke-Gh @('run', 'view', "$runId", '-R', $repo, '--json', 'status,conclusion,url')
        if ($v.Code -eq 0) {
            $run = $v.Out | ConvertFrom-Json
            if ($run.url) { $runUrl = $run.url }
            if ($run.status -eq 'completed') { break }
        }
        if ((Get-Date) -ge $deadline) { Complete failed transient "$RunTimeoutMinutes 分待っても run $runId が完了しない。再実行で続く" }
        Start-Sleep -Seconds $RunPollSeconds
    }
    # success と cancelled だけ見比べへ（cancelled は concurrency の取り消しで、後続の run が同じ最新 Release から作る。見比べが最終の判定者）。
    # それ以外（failure・timed_out・startup_failure・action_required・neutral・skipped・stale など）は組み立てが通っていないので build。
    if ($run.conclusion -notin 'success', 'cancelled') { Complete failed build "manual.yml の run の結論が $($run.conclusion)。run のログを読む" }
    if ($Ref -ne 'main') {
        Write-Output "rehearsal: ref $Ref の run $runUrl の結論は $($run.conclusion)。deploy は main 限定なので見比べない"
        exit 0
    }

    # (5) homeurl はここで初めて読む（古いタグ・失敗した run では要求しない）。
    $homeLine = if (Test-Path -LiteralPath $DescriptPath -PathType Leaf) { Get-Content -LiteralPath $DescriptPath -Encoding utf8 | Where-Object { $_ -match '^homeurl,' } | Select-Object -First 1 }
    $siteBase = if ($homeLine) { $homeLine.Substring('homeurl,'.Length).Trim() }
    if (-not $siteBase) { Complete failed publish "``$DescriptPath`` に homeurl が無い（読めない）" }
    $mjs = Join-Path $PSScriptRoot '../../../book/tools/ghost-site/ghost-site.mjs'
    $out = @(& $NodeCommand $mjs verify-live --nar $nar --base $siteBase --attempts $VerifyAttempts --interval $VerifyIntervalSeconds --tree "${Tag}:crates/pasta_sample_ghost/ghosts/hello-pasta" 2>&1 | ForEach-Object { "$_" })
    $code = $LASTEXITCODE
    $out | ForEach-Object { Write-Host $_ }
    if ($code -ne 0) { Complete failed verify "$siteBase の updates.txt が Release ``$Tag`` の nar と合わない・取れない（$VerifyIntervalSeconds 秒 × $VerifyAttempts 回）" }
    $m = [regex]::Match($out -join "`n", 'OK updates\.txt: (\d+) 回目で')
    if ($m.Success) { $verifyCount = $m.Groups[1].Value }
    Complete published '' "$siteBase の updates.txt が Release の nar の側の行と一致"
}
catch {
    Complete failed publish "予期しないエラー: $($_.Exception.Message)"
}
