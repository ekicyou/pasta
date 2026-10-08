#Requires -Version 7
<#
.SYNOPSIS
  リリースタグの形式・版の一致・main からの到達性を検査し、版を出力する。
.EXAMPLE
  pwsh -NoProfile -File .github/scripts/release/verify-tag.ps1 -Tag v0.3.7 -Sha (git rev-parse 'v0.3.7^{commit}') -MainRef origin/main
#>
param(
    [Parameter(Mandatory)][string]$Tag,
    [Parameter(Mandatory)][string]$Sha,
    [Parameter(Mandatory)][string]$MainRef,
    [string]$WorkspaceRoot = (Join-Path $PSScriptRoot '../../..')
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# $GITHUB_OUTPUT / $GITHUB_STEP_SUMMARY が未設定なら標準出力へ（手元での実行）。
function Write-To([string]$EnvName, [string]$Text) {
    $path = [Environment]::GetEnvironmentVariable($EnvName)
    if ($path) { Add-Content -LiteralPath $path -Value $Text -Encoding utf8 }
    else { Write-Output $Text }
}

$failures = [System.Collections.Generic.List[string]]::new()

# 1. タグの形式（v + 数字 3 つ。プレリリースは扱わない）
$version = $null
# 大文字小文字を区別し、ASCII 数字だけ（\d は全角数字にも合う）。末尾は \z（$ は末尾の改行も許す）。
if ($Tag -cmatch '^v([0-9]+\.[0-9]+\.[0-9]+)\z') { $version = $Matches[1] }
else { $failures.Add("タグの形式: ``$Tag`` は ``vX.Y.Z``（数字 3 つ）の形ではない") }

# 2. ワークスペースの版（Cargo.toml の [workspace.package] 節の version）
# 読めないときは理由を版の値として扱い、下の比較で失敗として報告する（途中で落とさない）。
$wsVersion = $null
try {
    $cargo = Get-Content -Raw -LiteralPath (Join-Path $WorkspaceRoot 'Cargo.toml')
    if ($cargo -match '(?ms)^\[workspace\.package\][ \t]*\r?$(.*?)(?=^\[|\z)' -and
        $Matches[1] -match '(?m)^[ \t]*version[ \t]*=[ \t]*"([^"]*)"') { $wsVersion = $Matches[1] }
}
catch { $wsVersion = "(読めない: $($_.Exception.Message))" }

# 3. 拡張の版（editors/vscode/package.json の version）
$extVersion = $null
try {
    $pkg = Get-Content -Raw -LiteralPath (Join-Path $WorkspaceRoot 'editors/vscode/package.json') | ConvertFrom-Json
    if ($pkg.PSObject.Properties['version']) { $extVersion = [string]$pkg.version }
}
catch { $extVersion = "(読めない: $($_.Exception.Message))" }

if ($version) {
    if ($wsVersion -ne $version) {
        $failures.Add("ワークスペースの版の一致（Cargo.toml [workspace.package] version）: タグ ``$version`` / ワークスペース ``$(if ($wsVersion) { $wsVersion } else { '(見つからない)' })``")
    }
    if ($extVersion -ne $version) {
        $failures.Add("拡張の版の一致（editors/vscode/package.json version）: タグ ``$version`` / 拡張 ``$(if ($extVersion) { $extVersion } else { '(見つからない)' })``")
    }
}

# 4. main からの到達性
$gitErr = git -C $WorkspaceRoot merge-base --is-ancestor $Sha $MainRef 2>&1
switch ($LASTEXITCODE) {
    0 { }
    1 { $failures.Add("main からの到達性: コミット ``$Sha`` は ``$MainRef`` から到達できない") }
    default { $failures.Add("main からの到達性: コミット ``$Sha`` と ``$MainRef`` を git で比べられない（終了コード $LASTEXITCODE`: $(($gitErr | Out-String).Trim())）") }
}

if ($failures.Count -gt 0) {
    Write-To GITHUB_STEP_SUMMARY (@(
            "## タグの検査: 失敗（``$Tag``）"
            ''
            $failures | ForEach-Object { "- $_" }
            ''
            '**どの公開先にも公開していない。**'
        ) -join "`n")
    exit 1
}

Write-To GITHUB_OUTPUT "version=$version"
if ($env:GITHUB_OUTPUT) { Write-Output "version=$version" }
Write-To GITHUB_STEP_SUMMARY "## タグの検査: 成功（``$Tag`` → 版 ``$version``、``$MainRef`` から到達可能）"
exit 0
