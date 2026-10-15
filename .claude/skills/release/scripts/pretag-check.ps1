#Requires -Version 7
<#
.SYNOPSIS
  定型コマンド C: タグを付ける前の検査（release スキルの段 5・6・8）。読むだけで、何も変えない。
.DESCRIPTION
  .github/scripts/release/verify-tag.ps1 と同じ 5 つの条件を、タグを付けるコミットの中身で確かめる。
  5 行とも「合格」であること。先に git fetch origin main を済ませ、リポジトリのルートを作業ディレクトリにして実行する。
.PARAMETER Version
  リリースする版（X.Y.Z）。
.PARAMETER Target
  タグを付けるコミットの SHA。
.EXAMPLE
  & "$PWD/.claude/skills/release/scripts/pretag-check.ps1" -Version 0.3.8 -Target <SHA>
#>
param([Parameter(Mandatory)][string]$Version, [Parameter(Mandatory)][string]$Target)
$V = $Version -replace '^v', ''
$cargo = (git show "${TARGET}:Cargo.toml") -join "`n"
$ws = if ($cargo -match '(?ms)^\[workspace\.package\][ \t]*\r?$(.*?)(?=^\[|\z)' -and $Matches[1] -match '(?m)^[ \t]*version[ \t]*=[ \t]*"([^"]*)"') { $Matches[1] } else { '' }
$ext = try { [string]((git show "${TARGET}:editors/vscode/package.json") -join "`n" | ConvertFrom-Json).version } catch { '' }
$hl = (git show "${TARGET}:doc/release-notes/v$V.md" 2>$null) -join "`n"
git merge-base --is-ancestor $TARGET origin/main
$reach = $LASTEXITCODE
"タグの形 | v$V | $(if ("v$V" -cmatch '^v[0-9]+\.[0-9]+\.[0-9]+\z') { '合格' } else { '不合格' })"
"Cargo.toml の版 | $ws | $(if ($ws -ceq $V) { '合格' } else { '不合格' })"
"package.json の版 | $ext | $(if ($ext -ceq $V) { '合格' } else { '不合格' })"
"main からの到達 | $TARGET | $(if ($reach -eq 0) { '合格' } else { '不合格' })"
"見どころ | doc/release-notes/v$V.md | $(if (-not [string]::IsNullOrWhiteSpace($hl)) { '合格' } else { '不合格' })"
