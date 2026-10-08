#Requires -Version 7
<#
.SYNOPSIS
  VSCode 拡張の 1 版について、Marketplace での公開済みの判定・vsce publish・公開後の確認・結果の出力を行う。
.DESCRIPTION
  認証は Microsoft Entra ID（Azure/login 済みの Azure CLI のセッション）を --azure-credential で使う。PAT は使わない。
  vsce は editors/vscode で npm ci 済みの lock の版を、そのディレクトリで呼ぶ。
  -DryRun では判定だけを行い、公開せずに行うはずの操作を表示する（認証不要）。
  unpublish・上書きは行わない。
.EXAMPLE
  pwsh -NoProfile -File .github/scripts/release/publish-vsix.ps1 -VsixPath release-assets/*.vsix -Version 0.3.7 -Extension ekicyou.pasta-vscode -DryRun
#>
param(
    # ワイルドカード可（1 つに定まること）。呼び出し元のカレントディレクトリからの相対パス。
    [Parameter(Mandatory)][string]$VsixPath,
    [Parameter(Mandatory)][string]$Version,
    [Parameter(Mandatory)][string]$Extension,
    # 空白で区切ってコマンドと引数に分ける（引用符は扱わない）。
    [string]$VsceCommand = 'npx --no-install vsce',
    [switch]$DryRun
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$PollSeconds = 15
$WaitLimit = [TimeSpan]::FromMinutes(10)   # 公開後に show に版が現れるまでの上限（Marketplace の検査を待つ。job は 20 分）
$AuthPattern = 'Entra ID access token|Unauthorized|\b40[13]\b|Forbidden|Access Denied|TF400813|not authorized|Personal Access Token'
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
    Write-To GITHUB_STEP_SUMMARY "- Marketplace ``$Extension`` $Version`: **$Status**$(if ($Reason) { "（reason=$Reason）" }) — $Message（所要 $secs 秒）"
    exit $(if ($Status -eq 'failed') { 1 } else { 0 })
}

# vsce を呼び、終了コード・標準出力の行・全出力（標準エラーを含む）を返す。-Echo で全出力を表示する。
function Invoke-Vsce([string[]]$VsceArgs, [switch]$Echo) {
    $out = [System.Collections.Generic.List[string]]::new()
    $all = [System.Collections.Generic.List[string]]::new()
    & $vsceExe @vscePre @VsceArgs 2>&1 | ForEach-Object {
        $line = "$_"
        if ($_ -isnot [System.Management.Automation.ErrorRecord]) { $out.Add($line) }
        $all.Add($line)
        if ($Echo -or $_ -is [System.Management.Automation.ErrorRecord]) { Write-Host $line }
    }
    [pscustomobject]@{ Code = $LASTEXITCODE; Out = $out; Text = $all -join "`n" }
}

# Marketplace の版の一覧（vsce show --json の versions[].version）。
# 失敗・読めない出力（拡張が無いと vsce は "undefined" を出して 0 で終える）は公開済みと見なさず transient で失敗する（7.4）。
function Get-Versions {
    $r = Invoke-Vsce @('show', $Extension, '--json')
    try {
        if ($r.Code -ne 0) { throw "終了コード $($r.Code)" }
        @(($r.Out -join "`n" | ConvertFrom-Json).versions | ForEach-Object { [string]$_.version })
    }
    catch {
        Complete failed transient "``vsce show $Extension`` の結果を得られない（$($_.Exception.Message)）。公開済みと見なさない。再実行で続く"
    }
}

try {
    # vsce の --pat は既定で VSCE_PAT を読み、--azure-credential より優先する（vsce の publish.js の getPAT）。
    # PAT を使わないこと（5.5・9.6）を確かにするため、vsce に渡さない。値は読まない。
    Remove-Item Env:VSCE_PAT -ErrorAction Ignore

    $vsix = @(Resolve-Path -Path $VsixPath -ErrorAction Ignore | ForEach-Object Path)
    $vsceExe, $vscePre = -split $VsceCommand
    $vscePre = @($vscePre | Where-Object { $_ })
    # npx --no-install が lock の版の vsce を見つけるよう、拡張のディレクトリで呼ぶ。
    Push-Location (Join-Path $PSScriptRoot '../../../editors/vscode')

    # (1) 版が公開済み → 飛ばす（5.2・5.3・7.3）
    if ((Get-Versions) -contains $Version) { Complete skipped '' '公開済みのため飛ばした' }

    if ($vsix.Count -ne 1) {
        Complete failed publish "VSIX が 1 つに定まらない（``$VsixPath`` に一致: $($vsix.Count) 件）"
    }
    $publishArgs = @('publish', '--azure-credential', '--packagePath', $vsix[0], '--skip-duplicate')
    if ($DryRun) {
        Write-Output "dry-run: ``$Extension`` $Version は未公開。行うはずの操作:"
        Write-Output "  $VsceCommand $($publishArgs -join ' ')"
        exit 0
    }

    # (2) 公開。--skip-duplicate は (1) との競合の保険で、その場合は「already published」で 0 終了する
    $r = Invoke-Vsce $publishArgs -Echo
    if ($r.Code -ne 0) {
        if ($r.Text -match $AuthPattern) {
            Complete failed auth "``vsce publish`` が認証で失敗した（終了コード $($r.Code)）。Azure のフェデレーション資格情報の subject・publisher の Members を手順書 ``.github/release-ci-setup.md`` の名前の表と照合する"
        }
        Complete failed publish "``vsce publish`` が失敗した（終了コード $($r.Code)）。ログを読む"
    }
    if ($r.Text -match 'is already published\. Skipping publish') { Complete skipped '' '公開済みのため飛ばした（vsce publish が重複を検出）' }

    # (3) 公開した版が show に現れるまで待つ
    $deadline = (Get-Date) + $WaitLimit
    while ((Get-Versions) -notcontains $Version) {
        if ((Get-Date) -ge $deadline) {
            Complete failed transient "``vsce publish`` は成功したが、$($WaitLimit.TotalMinutes) 分以内に ``vsce show`` に版が現れなかった。再実行で状態を確かめて続く"
        }
        Start-Sleep -Seconds $PollSeconds
    }
    Complete published '' '今回公開した'
}
catch {
    Complete failed publish "予期しないエラー: $($_.Exception.Message)"
}
