#Requires -Version 7
<#
.SYNOPSIS
  リリースタグの GitHub Release を、下書き → 添付 → 公開の順で作る。既存の Release は足りない添付だけを補う。
.DESCRIPTION
  認証は環境変数 GH_TOKEN（gh がそのまま読む）。対象のリポジトリは gh の既定（checkout の remote か GH_REPO）。
  状態は GitHub 側だけにあり、再実行では gh release view で無し / 下書き / 公開済みを見て続ける。
  下書きの添付のうち state が uploaded でないもの（前の試行の中断で残ったもの）だけを削除して付け直す。
  公開済みの Release の添付は削除・上書き（--clobber）しない。タグは作らない（--verify-tag）。
  -DryRun では判定だけを行い、作成・添付・公開をせずに行うはずの操作を表示する。
.EXAMPLE
  pwsh -NoProfile -File .github/scripts/release/github-release.ps1 -Tag v0.3.7 -NotesFile notes.md -AssetDir release-assets -DryRun
#>
param(
    [Parameter(Mandatory)][string]$Tag,
    [string]$Title = "pasta $Tag",
    [Parameter(Mandatory)][string]$NotesFile,
    # pasta.dll.zip・hello-pasta.nar・pasta-vscode-X.Y.Z.vsix を置いたディレクトリ。呼び出し元のカレントディレクトリからの相対パス可。
    [Parameter(Mandatory)][string]$AssetDir,
    [switch]$DryRun
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$started = Get-Date
$url = ''

# $GITHUB_OUTPUT / $GITHUB_STEP_SUMMARY が未設定なら標準出力へ（手元での実行）。
# 値を返す関数の中からも呼ぶので、Write-Output でなく Write-Host（戻り値に混ざらない）。
function Write-To([string]$EnvName, [string]$Text) {
    $path = [Environment]::GetEnvironmentVariable($EnvName)
    if ($path) { Add-Content -LiteralPath $path -Value $Text -Encoding utf8 }
    else { Write-Host $Text }
}

# status（と failed のときの reason）と、分かっていれば url を書いて終える。exit 0 = published / skipped、1 = failed。
function Complete([string]$Status, [string]$Reason, [string]$Message) {
    Write-To GITHUB_OUTPUT "status=$Status"
    if ($Reason) { Write-To GITHUB_OUTPUT "reason=$Reason" }
    if ($url) { Write-To GITHUB_OUTPUT "url=$url" }
    if ($env:GITHUB_OUTPUT) { Write-Host "status=$Status$(if ($Reason) { " reason=$Reason" })$(if ($url) { " url=$url" })" }
    $secs = [int]((Get-Date) - $started).TotalSeconds
    Write-To GITHUB_STEP_SUMMARY "- GitHub Release ``$Tag``: **$Status**$(if ($Reason) { "（reason=$Reason）" }) — $Message$(if ($url) { "（$url）" })（所要 $secs 秒）"
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

# タグの Release（下書きも tag 名で見つかる）。無ければ $null。
# 「無い」以外の失敗は状態を判定できないので transient で失敗する（7.4）。
function Get-Release {
    $r = Invoke-Gh @('release', 'view', $Tag, '--json', 'isDraft,assets,url,name')
    if ($r.Code -eq 0) { return ($r.Out | ConvertFrom-Json) }
    if ($r.Text -match 'release not found') { return $null }
    Complete failed transient "``gh release view $Tag`` が失敗した（終了コード $($r.Code): $($r.Text)）。状態を判定できない。再実行で続く"
}

try {
    if ($Tag -cnotmatch '^v[0-9]+\.[0-9]+\.[0-9]+\z') { Complete failed publish "タグ ``$Tag`` が vX.Y.Z の形でない" }

    # (1) 3 つの配布物がそろっていることを先に検査する（6.2）
    $names = @('pasta.dll.zip', 'hello-pasta.nar', "pasta-vscode-$($Tag.Substring(1)).vsix")
    $files = @{}
    foreach ($n in $names) {
        $p = Join-Path $AssetDir $n
        if (Test-Path -LiteralPath $p -PathType Leaf) { $files[$n] = (Resolve-Path -LiteralPath $p).Path }
    }
    $absent = @($names | Where-Object { -not $files.ContainsKey($_) })
    if ($absent) { Complete failed publish "配布物がそろっていない（``$AssetDir`` に無い: $($absent -join ', ')）" }

    # (2) Release の状態を見る
    $rel = Get-Release
    if ($rel) { $url = $rel.url }
    $assets = @(if ($rel) { $rel.assets })
    $uploaded = @($assets | Where-Object state -eq 'uploaded' | ForEach-Object name)
    $missing = @($names | Where-Object { $uploaded -notcontains $_ })

    # 公開済み・3 つ添付済み → 飛ばす。題名・ノートは触らない（6.4・7.3）
    if ($rel -and -not $rel.isDraft -and -not $missing) { Complete skipped '' '公開済みで 3 つの配布物が添付済みのため飛ばした' }

    # (3) 行う操作を組み立てる
    $ops = [System.Collections.Generic.List[object]]::new()
    if (-not $rel) {
        # 無い → 下書きで作る。--verify-tag で既存のタグだけを使い、タグを作らない
        $ops.Add(@('release', 'create', $Tag, '--draft', '--verify-tag', '--title', $Title, '--notes-file', $NotesFile))
    }
    elseif ($rel.isDraft) {
        # 下書き → 未完了（state が uploaded でない）の添付を消して付け直す。公開済みの添付には行わない（7.6）
        foreach ($a in $assets | Where-Object state -ne 'uploaded') { $ops.Add(@('release', 'delete-asset', $Tag, $a.name, '--yes')) }
    }
    # 足りないものだけを添付する。--clobber は使わない（7.6）
    if ($missing) { $ops.Add(@('release', 'upload', $Tag) + @($missing | ForEach-Object { $files[$_] })) }
    # 3 つがそろってから公開する（6.5）。公開済みの Release には行わない
    if (-not $rel -or $rel.isDraft) { $ops.Add(@('release', 'edit', $Tag, '--draft=false')) }

    if ($DryRun) {
        $state = if (-not $rel) { '無い' } elseif ($rel.isDraft) { '下書き' } else { "公開済みで添付が足りない（$($missing -join ', ')）" }
        Write-Output "dry-run: ``$Tag`` の Release は$state。行うはずの操作:"
        foreach ($op in $ops) { Write-Output "  gh $($op -join ' ')" }
        if ($url) { Write-To GITHUB_OUTPUT "url=$url" }
        exit 0
    }

    # (4) 実行する
    foreach ($op in $ops) {
        $r = Invoke-Gh $op
        if ($r.Code -eq 0) { continue }
        if ($op[1] -eq 'upload' -and $rel -and -not $rel.isDraft -and $r.Text -match 'immutable') {
            Complete failed immutable "公開済みの Release に添付できない（Immutable Releases）。足りない配布物: $($missing -join ', ')。手で対応する"
        }
        Complete failed publish "``gh $($op[0..2] -join ' ')`` が失敗した（終了コード $($r.Code): $($r.Text)）。ログを読む"
    }

    # 公開後の URL（下書きの間の URL は untagged-… なので取り直す）
    $rel = Get-Release
    if (-not $rel -or $rel.isDraft) { Complete failed transient '操作は成功したが、Release が公開済みとして見つからない。再実行で状態を確かめて続く' }
    $url = $rel.url
    Complete published '' '今回作成・添付した'
}
catch {
    Complete failed publish "予期しないエラー: $($_.Exception.Message)"
}
