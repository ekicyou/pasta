#Requires -Version 7
<#
.SYNOPSIS
  リリースタグが最新の Release かを判定し、最新なら hello-pasta の置き場所（manual.yml の公開）の更新に進む。結果を status/reason で出す。
.DESCRIPTION
  最新のタグは gh api repos/$GITHUB_REPOSITORY/releases/latest（-LatestTag で上書き）。認証は環境変数 GH_TOKEN。
  -Tag が最新より古ければ何も起こさず skipped（reason stale-tag）。最新より新しければ 10 秒おきに取り直し（-LatestWaitSeconds まで）、
  それでも新しいままなら failed（reason transient）。
  -DescriptPath の homeurl は、古いタグの判定を終えた後・見比べの直前にだけ読む。
  -DryRun では判定だけを行い、起動の手前で行うはずの操作を表示して終える。
  skipped のときも reason を出す（置き場所の行に理由を表に出すため）。
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
    [switch]$DryRun
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$LatestPollSeconds = 10
$TagPattern = '^v[0-9]+\.[0-9]+\.[0-9]+\z'
$started = Get-Date
$latest = ''

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
    Write-To GITHUB_STEP_SUMMARY "- 置き場所（hello-pasta）``$Tag``（最新 ``$(if ($latest) { $latest } else { '(未取得)' })``）: **$Status**$(if ($Reason) { "（reason=$Reason）" }) — $Message（所要 $secs 秒）"
    exit $(if ($Status -eq 'failed') { 1 } else { 0 })
}

# gh を呼び、終了コード・標準出力・全出力（標準エラーを含む）を返す。
function Invoke-Gh([string[]]$GhArgs) {
    $out = [System.Collections.Generic.List[string]]::new()
    $all = [System.Collections.Generic.List[string]]::new()
    & gh @GhArgs 2>&1 | ForEach-Object {
        $line = "$_"
        if ($_ -isnot [System.Management.Automation.ErrorRecord]) { $out.Add($line) }
        $all.Add($line)
    }
    [pscustomobject]@{ Code = $LASTEXITCODE; Out = $out -join "`n"; Text = $all -join "`n" }
}

# 最新の Release のタグ。取れない・形が違うときは判定できないので transient で失敗する。
function Get-LatestTag {
    if ($LatestTag) { $t = $LatestTag }
    else {
        $r = Invoke-Gh @('api', "repos/$env:GITHUB_REPOSITORY/releases/latest", '--jq', '.tag_name')
        if ($r.Code -ne 0) { Complete failed transient "最新の Release を取れない（終了コード $($r.Code): $($r.Text)）。再実行で続く" }
        $t = $r.Out.Trim()
    }
    if ($t -cnotmatch $TagPattern) { Complete failed transient "最新の Release のタグ ``$t`` が vX.Y.Z の形でない。判定できない" }
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
        Write-Output "  ``$DescriptPath`` の homeurl を読み、verify-live で updates.txt を見比べる（$VerifyIntervalSeconds 秒 × $VerifyAttempts 回）"
        exit 0
    }

    Complete failed publish '起動の段は未実装（判定だけを行う版）。-DryRun で判定を確かめる'
}
catch {
    Complete failed publish "予期しないエラー: $($_.Exception.Message)"
}
