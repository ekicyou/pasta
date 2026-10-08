#Requires -Version 7
<#
.SYNOPSIS
  1 クレートについて、crates.io での公開済みの判定・依存先の索引への反映待ち・cargo publish・結果の出力を行う。
.DESCRIPTION
  トークンは環境変数 CARGO_REGISTRY_TOKEN（crates-io-auth-action の出力）で受け取り、cargo がそのまま読む。
  -DryRun では判定だけを行い、公開せずに行うはずの操作を表示する（トークン不要）。
  yank・上書き・--no-verify は行わない。
.EXAMPLE
  pwsh -NoProfile -File .github/scripts/release/publish-crate.ps1 -Crate pasta_lua -Version 0.3.7 -DependsOn pasta_core,pasta_dsl -DryRun
#>
param(
    [Parameter(Mandatory)][string]$Crate,
    [Parameter(Mandatory)][string]$Version,
    [string[]]$DependsOn = @(),
    [switch]$DryRun,
    # 検証用に問い合わせ先の API の基点を差し替える。
    [string]$ApiBase = 'https://crates.io'
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$UserAgent = 'pasta-release-ci (https://github.com/ekicyou/pasta)'
$IndexBase = 'https://index.crates.io'
$PollSeconds = 10
$WaitLimit = [TimeSpan]::FromMinutes(5)   # 依存先の索引への反映待ち・公開後の版の確認、それぞれの上限
$RetryMax = 6                             # 依存先の未解決で失敗したときの再試行の回数
$RetrySeconds = 30
$started = Get-Date

# $GITHUB_OUTPUT / $GITHUB_STEP_SUMMARY が未設定なら標準出力へ（手元での実行）。
# 値を返す関数の中からも呼ぶので、Write-Output でなく Write-Host（戻り値に混ざらない）。
function Write-To([string]$EnvName, [string]$Text) {
    $path = [Environment]::GetEnvironmentVariable($EnvName)
    if ($path) { Add-Content -LiteralPath $path -Value $Text -Encoding utf8 }
    else { Write-Host $Text }
}

# status（と failed のときの reason）を書いて終える。exit 0 = published / skipped、1 = failed。
function Complete([string]$Status, [string]$Reason, [string]$Message) {
    Write-To GITHUB_OUTPUT "status=$Status"
    if ($Reason) { Write-To GITHUB_OUTPUT "reason=$Reason" }
    if ($env:GITHUB_OUTPUT) { Write-Host "status=$Status$(if ($Reason) { " reason=$Reason" })" }
    $secs = [int]((Get-Date) - $started).TotalSeconds
    Write-To GITHUB_STEP_SUMMARY "- ``$Crate`` $Version`: **$Status**$(if ($Reason) { "（reason=$Reason）" }) — $Message（所要 $secs 秒）"
    exit $(if ($Status -eq 'failed') { 1 } else { 0 })
}

# GET して HTTP の状態コードと本文を返す。ネットワーク・タイムアウトは状態コード 0。
function Get-Url([string]$Url) {
    try {
        $r = Invoke-WebRequest -Uri $Url -UserAgent $UserAgent -SkipHttpErrorCheck -TimeoutSec 30 -MaximumRetryCount 0
        [pscustomobject]@{ Code = [int]$r.StatusCode; Body = [string]$r.Content; Error = '' }
    }
    catch { [pscustomobject]@{ Code = 0; Body = ''; Error = $_.Exception.Message } }
}

# 200 と 404 だけを判定に使い、それ以外は公開済みと見なさず transient で失敗する（7.4）。
function Test-Exists([string]$Url) {
    $r = Get-Url $Url
    if ($r.Code -eq 200) { return $true }
    if ($r.Code -eq 404) { return $false }
    $what = if ($r.Code) { "HTTP $($r.Code)" } else { "ネットワークの失敗: $($r.Error)" }
    Complete failed transient "``$Url`` の問い合わせが $what。公開済みと見なさない。再実行で続く"
}

# スパース索引のパス（https://doc.rust-lang.org/cargo/reference/registry-index.html#index-files）
function Get-IndexPath([string]$Name) {
    $n = $Name.ToLowerInvariant()
    switch ($n.Length) {
        1 { "1/$n" }
        2 { "2/$n" }
        3 { "3/$($n[0])/$n" }
        default { "$($n.Substring(0, 2))/$($n.Substring(2, 2))/$n" }
    }
}

function Test-InIndex([string]$Name) {
    $r = Get-Url "$IndexBase/$(Get-IndexPath $Name)"
    if ($r.Code -ne 200) { return $false }
    foreach ($line in $r.Body -split "`n") {
        if ($line.Trim() -and ($line | ConvertFrom-Json).vers -eq $Version) { return $true }
    }
    $false
}

try {
    $crateUrl = "$($ApiBase.TrimEnd('/'))/api/v1/crates/$Crate"
    $versionUrl = "$crateUrl/$Version"

    # (1) クレート自体が無い → 初回の公開は Trusted Publishing ではできない（4.6）
    if (-not (Test-Exists $crateUrl)) {
        Complete failed not-registered "crates.io にクレートが無い。初回の公開は手で行う（手順書 ``.github/release-ci-setup.md`` §初回公開）"
    }
    # (2) 版が公開済み → 飛ばす（4.2・7.3）
    if (Test-Exists $versionUrl) { Complete skipped '' '公開済みのため飛ばした' }

    # pwsh -File 経由では `a,b` が 1 つの文字列で届くので、カンマでも分ける。
    $deps = @($DependsOn -split ',' | ForEach-Object Trim | Where-Object { $_ })
    if ($DryRun) {
        Write-Output "dry-run: ``$Crate`` $Version は未公開。行うはずの操作:"
        if ($deps) { Write-Output "  索引で依存先の $Version を待つ: $($deps -join ', ')（上限 5 分）" }
        Write-Output "  cargo publish -p $Crate --locked"
        exit 0
    }

    # (3) 依存先の同じ版がスパース索引に現れるまで待つ。時間切れでも公開へ進み、未解決なら下の再試行に任せる（4.5）
    foreach ($dep in $deps) {
        $deadline = (Get-Date) + $WaitLimit
        while (-not (Test-InIndex $dep)) {
            if ((Get-Date) -ge $deadline) { Write-Warning "索引に ``$dep`` $Version が 5 分で現れなかった。公開へ進む"; break }
            Start-Sleep -Seconds $PollSeconds
        }
    }

    # (4) 公開。索引の反映待ちの時間切れは cargo が警告・exit 0 で終えるので成功とみなす。依存先の未解決は再試行（4.5）
    for ($attempt = 0; ; $attempt++) {
        $lines = @()   # 出力が無いと Tee-Object は変数を作らない（StrictMode で参照エラーになる）
        & cargo publish -p $Crate --locked 2>&1 | ForEach-Object { "$_" } | Tee-Object -Variable lines | Write-Host
        if ($LASTEXITCODE -eq 0) { break }
        $text = $lines -join "`n"
        $unresolved = $text -match 'failed to select a version for the requirement|no matching package named'
        if (-not $unresolved) { Complete failed publish "``cargo publish`` が失敗した（終了コード $LASTEXITCODE）。ログを読む" }
        if ($attempt -ge $RetryMax) { Complete failed publish "依存先が索引で解決できず、$RetryMax 回の再試行でも ``cargo publish`` が失敗した" }
        Write-Warning "依存先が未解決。$RetrySeconds 秒後に再試行（$($attempt + 1)/$RetryMax）"
        Start-Sleep -Seconds $RetrySeconds
    }

    # (5) 公開した版が API に現れるまで待つ
    $deadline = (Get-Date) + $WaitLimit
    while (-not (Test-Exists $versionUrl)) {
        if ((Get-Date) -ge $deadline) {
            Complete failed transient '`cargo publish` は成功したが、5 分以内に API に版が現れなかった。再実行で状態を確かめて続く'
        }
        Start-Sleep -Seconds $PollSeconds
    }
    Complete published '' '今回公開した'
}
catch {
    Complete failed publish "予期しないエラー: $($_.Exception.Message)"
}
