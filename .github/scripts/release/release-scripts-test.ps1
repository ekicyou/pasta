#Requires -Version 7
<#
.SYNOPSIS
  リリースの補助スクリプト（release-notes.ps1・verify-tag.ps1・定型コマンド D の highlights-check.ps1・添付の欠けを見る github-release.ps1・公開済みなら飛ばす publish-vsix.ps1・pasta-recipe の 2 つの zip を検査する verify-recipe-zips.ps1・最新のタグを判定する publish-ghost-site.ps1）を、一時の git リポジトリで実際に動かして確かめる。
.DESCRIPTION
  一時のフォルダに git リポジトリを作り、決まった件名のコミットとタグ（v0.0.1・v0.0.2）を積んで、スクリプトを子の pwsh で呼ぶ。
  ネットワークと gh を使わない（publish-ghost-site.ps1 の起動・待ち・見比べは偽の gh と node の .ps1 で通す）。一時のフォルダは終わりに消し、リポジトリの作業ツリーを変えない。
  項目ごとに ok か NG を 1 行で出す。1 つでも NG なら NG の項目を並べて終了コード 1、すべて ok なら 0。
  項目を足すときは、末尾の Test-Item の並びに 1 つ足す。
.EXAMPLE
  pwsh -NoProfile -File .github/scripts/release/release-scripts-test.ps1
#>
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$Repo = 'test/repo'
$Pwsh = [Environment]::ProcessPath
# 詳しいリリースノートの見出し（この順で出る）
$Headings = @(
    '## ⚠️ 動きが変わる変更'
    '## ✨ 新機能'
    '## 🐛 不具合の修正'
    '## ♻️ リファクタリング'
    '## 📝 ドキュメント'
    '## 🧪 テスト'
    '## 🔧 そのほか'
)
# v0.0.1..v0.0.2 で載る件数（別名・戻り先・失敗表記・落ちる不具合・手引き・接頭辞の無い件名）
$ListedCount = 6
$Highlights = "## 今回の見どころ`n`nおほほ、ごきげんよう。[Claudia](https://github.com/ponapalt/claudia) ですわ。`n`n### 別名が書けますの (#1)`n`nこれは見本ですわ。`n`n`n"

# 子のスクリプトが GitHub Actions の出力先へ書かないようにする（この処理の中だけで効く）
Remove-Item Env:GITHUB_OUTPUT, Env:GITHUB_STEP_SUMMARY -ErrorAction SilentlyContinue

$tmp = Join-Path ([IO.Path]::GetTempPath()) "pasta-release-scripts-test-$([guid]::NewGuid().ToString('N'))"
$repoDir = Join-Path $tmp 'repo'
$ng = [System.Collections.Generic.List[string]]::new()
$savedEncoding = [Console]::OutputEncoding
[Console]::OutputEncoding = [Text.UTF8Encoding]::new($false)

function Write-Utf8([string]$Path, [string]$Text) {
    New-Item -ItemType Directory -Force -Path (Split-Path $Path) | Out-Null
    [IO.File]::WriteAllText($Path, $Text, [Text.UTF8Encoding]::new($false))
}

# 一時のリポジトリで git を動かす。失敗したら止める（検査の前提が崩れている）。
function Invoke-Git {
    $out = & git -C $repoDir -c user.name=release-test -c user.email=release-test@example.invalid `
        -c commit.gpgsign=false -c tag.gpgsign=false -c core.autocrlf=false @args 2>&1
    if ($LASTEXITCODE -ne 0) { throw "git $args が失敗した: $($out | Out-String)" }
    $out
}

function Add-Commit([string]$Subject) { Invoke-Git commit -q --allow-empty --no-verify -m $Subject | Out-Null }

# 補助スクリプトを子の pwsh で呼ぶ（スクリプトの exit で検査が終わらないように）。
function Invoke-Script([string]$Name, [string[]]$ArgList) {
    $out = & $Pwsh -NoProfile -File (Join-Path $PSScriptRoot $Name) @ArgList 2>&1 | Out-String
    [pscustomobject]@{ Exit = $LASTEXITCODE; Output = $out.Trim() }
}

function Read-Lines([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path)) { return @() }
    @([IO.File]::ReadAllText($Path) -split "`r?`n")
}

# 見出しの下の箇条書き（次の見出しか Full Changelog の行まで）
function Get-Section([string[]]$Lines, [string]$Heading) {
    $items = [System.Collections.Generic.List[string]]::new()
    $in = $false
    foreach ($l in $Lines) {
        if ($l -eq $Heading) { $in = $true; continue }
        if ($in -and ($l.StartsWith('#') -or $l.StartsWith('**Full Changelog**'))) { break }
        if ($in -and $l.StartsWith('- ')) { $items.Add($l) }
    }
    , $items.ToArray()
}

function Get-LastLine([string[]]$Lines) { @($Lines | Where-Object { $_.Trim() })[-1] }

# 項目を 1 つ確かめる。$Body は問題が無ければ何も返さず、あれば理由の文字列を返す。
function Test-Item([int]$Number, [string]$Desc, [scriptblock]$Body) {
    try { $reason = (& $Body) -join ' / ' }
    catch { $reason = "例外: $($_.Exception.Message)" }
    if ($reason) {
        Write-Output "NG $Number $Desc — $reason"
        $ng.Add("$Number $Desc")
    }
    else { Write-Output "ok $Number $Desc" }
}

# pasta-recipe.<arch>.zip を $Dir に作る。exe は MZ・e_lfanew（0x40）・PE\0\0・Machine の 2 バイトだけの偽物。
function New-RecipeZip([string]$Dir, [string]$Arch, [int]$Machine, [string]$Notice = "tauri`nmonaco-editor`n") {
    $src = Join-Path $Dir "src-$Arch"
    $b = [byte[]]::new(0x46)
    $b[0] = 0x4D; $b[1] = 0x5A; $b[0x3C] = 0x40
    $b[0x40] = 0x50; $b[0x41] = 0x45
    [BitConverter]::GetBytes([uint16]$Machine).CopyTo($b, 0x44)
    New-Item -ItemType Directory -Force -Path $src | Out-Null
    [IO.File]::WriteAllBytes((Join-Path $src 'pasta-recipe.exe'), $b)
    Write-Utf8 (Join-Path $src 'THIRD_PARTY_LICENSES.txt') $Notice
    Compress-Archive -Path (Join-Path $src '*') -DestinationPath (Join-Path $Dir "pasta-recipe.$Arch.zip") -Force
}

# release-notes.ps1 の実行が失敗していれば、その理由を返す。
function Get-RunFailure($Run) {
    if ($Run.Exit -ne 0) { "release-notes.ps1 が終了コード $($Run.Exit) で終わった: $($Run.Output)" }
}

try {
    # --- 一時のリポジトリ ---
    New-Item -ItemType Directory -Force -Path $repoDir | Out-Null
    & git init -q -b main $repoDir 2>&1 | Out-Null
    if ($LASTEXITCODE -ne 0) { throw 'git init が失敗した' }
    Write-Utf8 (Join-Path $repoDir 'Cargo.toml') "[workspace]`nmembers = []`n`n[workspace.package]`nversion = `"0.0.2`"`n"
    Write-Utf8 (Join-Path $repoDir 'editors/vscode/package.json') "{ `"name`": `"test`", `"version`": `"0.0.2`" }`n"
    Invoke-Git add Cargo.toml editors/vscode/package.json | Out-Null
    Add-Commit 'chore: 最初のコミット'
    Invoke-Git tag v0.0.1 | Out-Null

    # v0.0.1..v0.0.2（古い順）
    Add-Commit 'feat(scene-name-alias): 別名 (#1)'
    Add-Commit 'fix(pasta_lua)!: 戻り先の文脈を変える (#2)'
    Add-Commit 'feat!(failure-output): 失敗表記を出す (#5)'
    Add-Commit 'fix: 落ちる不具合を直す (#3)'
    Add-Commit 'docs(manual): 手引きを足す (#4)'
    Add-Commit 'spec(discovery): 起票する'
    Add-Commit 'docs(roadmap): 道筋を直す'
    Add-Commit 'chore(kiro-skills): スキルを更新する'
    Add-Commit '接頭辞の無い件名'
    Add-Commit 'chore(release): v0.0.2'
    Invoke-Git tag v0.0.2 | Out-Null
    $sha = (Invoke-Git rev-parse 'v0.0.2^{commit}' | Select-Object -First 1).ToString().Trim()

    # タグの無い版（v0.0.3）の分
    Add-Commit 'feat(after): 後の機能 (#9)'

    $hlPath = Join-Path $repoDir 'doc/release-notes/v0.0.2.md'
    Write-Utf8 $hlPath $Highlights

    # --- 詳しいリリースノート（v0.0.2）---
    $notesPath = Join-Path $tmp 'notes.md'
    $run = Invoke-Script release-notes.ps1 @('-Tag', 'v0.0.2', '-Repo', $Repo, '-OutFile', $notesPath, '-WorkspaceRoot', $repoDir)
    $notes = Read-Lines $notesPath
    $text = $notes -join "`n"

    Test-Item 1 'feat(scene-name-alias): 別名 (#1) が「- 別名 (#1)」で新機能に載り、スコープの文字列が出力に無い' {
        Get-RunFailure $run
        if ((Get-Section $notes '## ✨ 新機能') -notcontains '- 別名 (#1)') { '「## ✨ 新機能」の下に「- 別名 (#1)」が無い' }
        if ($text.Contains('scene-name-alias')) { '出力に scene-name-alias が残っている' }
    }

    Test-Item 2 'fix(pasta_lua)!: … と feat!(failure-output): …（`!` がスコープの前）が「動きが変わる変更」の見出しの下に載る' {
        Get-RunFailure $run
        if ((Get-Section $notes '## ⚠️ 動きが変わる変更') -notcontains '- 戻り先の文脈を変える (#2)') { '「## ⚠️ 動きが変わる変更」の下に「- 戻り先の文脈を変える (#2)」が無い' }
        if ((Get-Section $notes '## 🐛 不具合の修正') -contains '- 戻り先の文脈を変える (#2)') { '「## 🐛 不具合の修正」にも載っている' }
        if ($text.Contains('pasta_lua')) { '出力に pasta_lua が残っている' }
        if ((Get-Section $notes '## ⚠️ 動きが変わる変更') -notcontains '- 失敗表記を出す (#5)') { '「## ⚠️ 動きが変わる変更」の下に「- 失敗表記を出す (#5)」が無い' }
        if ($text.Contains('failure-output')) { '出力に failure-output が残っている' }
    }

    Test-Item 3 '内部の作業（spec(discovery)・docs(roadmap)・chore(kiro-skills)・chore(release)）が載らない' {
        Get-RunFailure $run
        foreach ($s in '起票する', '道筋を直す', 'スキルを更新する', 'discovery', 'roadmap', 'kiro-skills', 'chore(release)', 'spec(') {
            if ($text.Contains($s)) { "出力に「$s」がある" }
        }
        if (@($notes | Where-Object { $_.StartsWith('- ') -and $_.Contains('v0.0.2') })) { '「v0.0.2」の項目が載っている' }
    }

    Test-Item 4 '接頭辞の無い件名が「そのほか」に、そのまま載る' {
        Get-RunFailure $run
        if ((Get-Section $notes '## 🔧 そのほか') -notcontains '- 接頭辞の無い件名') { '「## 🔧 そのほか」の下に「- 接頭辞の無い件名」が無い' }
    }

    Test-Item 5 '変更の無い見出しが出ず、末尾が compare/v0.0.1...v0.0.2 のリンクである' {
        Get-RunFailure $run
        foreach ($h in @($notes | Where-Object { $_.StartsWith('## ') })) {
            if ((Get-Section $notes $h).Count -eq 0) { "変更の無い見出し「$h」が出ている" }
        }
        $known = @($notes | Where-Object { $Headings -contains $_ } | ForEach-Object { [array]::IndexOf($Headings, $_) })
        if (($known -join ',') -ne (($known | Sort-Object) -join ',')) { '見出しの順が決まりと違う' }
        $last = Get-LastLine $notes
        if ($last -ne "**Full Changelog**: https://github.com/$Repo/compare/v0.0.1...v0.0.2") { "末尾の行が違う: $last" }
    }

    Test-Item 6 'タグの無い版（-Tag v0.0.3 -Head HEAD）で、終了コード 0 で同じ形の一覧が出る' {
        $p = Join-Path $tmp 'notes-head.md'
        $r = Invoke-Script release-notes.ps1 @('-Tag', 'v0.0.3', '-Head', 'HEAD', '-Repo', $Repo, '-OutFile', $p, '-WorkspaceRoot', $repoDir)
        if ($r.Exit -ne 0) { "終了コード $($r.Exit): $($r.Output)"; return }
        $l = Read-Lines $p
        if ($l[0] -ne '# pasta v0.0.3 の詳しいリリースノート') { "1 行目が違う: $($l[0])" }
        $feat = Get-Section $l '## ✨ 新機能'
        if ($feat -notcontains '- 後の機能 (#9)') { '「## ✨ 新機能」の下に「- 後の機能 (#9)」が無い' }
        if ($feat -contains '- 別名 (#1)') { 'v0.0.2 より前の変更が載っている' }
        $last = Get-LastLine $l
        if ($last -ne "**Full Changelog**: https://github.com/$Repo/compare/v0.0.2...v0.0.3") { "末尾の行が違う: $last" }
    }

    Test-Item 7 '-BodyFile の本文が、見どころの内容で始まり、release-notes-v0.0.2.md へのリンクと件数の行で終わる' {
        $p = Join-Path $tmp 'notes-body.md'
        $b = Join-Path $tmp 'body.md'
        $r = Invoke-Script release-notes.ps1 @('-Tag', 'v0.0.2', '-Repo', $Repo, '-OutFile', $p, '-BodyFile', $b, '-WorkspaceRoot', $repoDir)
        if ($r.Exit -ne 0) { "終了コード $($r.Exit): $($r.Output)"; return }
        if (-not (Test-Path -LiteralPath $b)) { '本文のファイルが無い'; return }
        $line = "すべての変更（$ListedCount 件）は、Assets の [release-notes-v0.0.2.md](https://github.com/$Repo/releases/download/v0.0.2/release-notes-v0.0.2.md) にまとめてありますわ。"
        $expected = $Highlights.TrimEnd() + "`n`n---`n`n" + $line
        $actual = [IO.File]::ReadAllText($b).TrimEnd()
        if ($actual -cne $expected) { "本文が違う:`n$actual" }
    }

    Test-Item 8 'verify-tag.ps1 が、見どころのファイルが在る → 0、無い → 1、空白だけ → 1' {
        $vt = @('-Tag', 'v0.0.2', '-Sha', $sha, '-MainRef', 'main', '-WorkspaceRoot', $repoDir)
        Write-Utf8 $hlPath $Highlights
        $r = Invoke-Script verify-tag.ps1 $vt
        if ($r.Exit -ne 0) { "在る: 終了コード $($r.Exit): $($r.Output)" }
        Remove-Item -LiteralPath $hlPath
        $r = Invoke-Script verify-tag.ps1 $vt
        if ($r.Exit -ne 1) { "無い: 終了コード $($r.Exit)（1 のはず）" }
        Write-Utf8 $hlPath " `n`t`n  `n"
        $r = Invoke-Script verify-tag.ps1 $vt
        if ($r.Exit -ne 1) { "空白だけ: 終了コード $($r.Exit)（1 のはず）" }
        Write-Utf8 $hlPath $Highlights
    }

    Test-Item 9 '定型コマンド D が、決まりどおりで 5 行とも合格、材料に無い番号で 3 行目が不合格、リンクの前の素の Claudia で 2 行目が不合格' {
        $d = Join-Path $PSScriptRoot '../../../.claude/skills/release/scripts/highlights-check.ps1'
        New-Item -ItemType Directory -Force -Path (Join-Path $repoDir '.kiro/specs/demo-spec') | Out-Null
        $link = '[Claudia](https://github.com/ponapalt/claudia)'
        $tail = "### 別名が書けますの (#1)`n`nこれは見本ですわ。`n`n### ご注意あそばせ`n`n- 戻り先の文脈が変わります。 (#2)`n- 失敗表記が出ます。 (#5)`n"
        $cases = @(
            @{ Name = '決まりどおり'; Text = "おほほ。$link ですわ。`n`n$tail"; Expect = @('合格', '合格', '合格', '合格', '合格') }
            @{ Name = '材料に無い番号'; Text = "おほほ。$link ですわ。`n`n$tail`n### 無い変更 (#99)`n"; Expect = @('合格', '合格', '不合格', '合格', '合格') }
            @{ Name = 'リンクの前の Claudia'; Text = "## Claudia の見どころ`n`nおほほ。$link ですわ。`n`n$tail"; Expect = @('合格', '不合格', '合格', '合格', '合格') }
            @{ Name = '動きが変わる点の抜け'; Text = "おほほ。$link ですわ。`n`n### 別名が書けますの (#1)`n"; Expect = @('合格', '合格', '合格', '不合格', '合格') }
            @{ Name = 'spec 名'; Text = "おほほ。$link ですわ。`n`n$tail`ndemo-spec で作りましたの。`n"; Expect = @('合格', '合格', '合格', '合格', '不合格') }
            @{ Name = '空白だけ'; Text = " `n`n"; Expect = @('不合格', '不合格', '不合格', '不合格', '不合格') }
        )
        foreach ($c in $cases) {
            Write-Utf8 $hlPath $c.Text
            Push-Location $repoDir
            try { $out = @(& $Pwsh -NoProfile -File $d -Version 0.0.2 2>&1 | ForEach-Object { "$_" }) }
            finally { Pop-Location }
            $got = @($out | ForEach-Object { ($_ -split ' \| ')[-1] })
            if (($got -join ',') -cne ($c.Expect -join ',')) { "$($c.Name): $($out -join ' ／ ')" }
        }
        Write-Utf8 $hlPath $Highlights
    }

    Test-Item 10 'github-release.ps1 が、添付 8 つのうち pasta_check.zip だけが無いとき、終了コード 1 で、出力にその名前を示す' {
        # 添付の有無は gh の呼び出しより前に検査されるので、ネットワークと gh を使わない
        $dir = Join-Path $tmp 'assets'
        foreach ($n in 'pasta.dll.zip', 'hello-pasta.nar', 'pasta-vscode-0.0.2.vsix', 'pasta-skills.zip', 'pasta-recipe.x64.zip', 'pasta-recipe.arm64.zip', 'release-notes-v0.0.2.md') { Write-Utf8 (Join-Path $dir $n) '' }
        $r = Invoke-Script github-release.ps1 @('-Tag', 'v0.0.2', '-NotesFile', $notesPath, '-AssetDir', $dir)
        if ($r.Exit -ne 1) { "終了コード $($r.Exit)（1 のはず）: $($r.Output)" }
        if (-not $r.Output.Contains('pasta_check.zip')) { "出力に pasta_check.zip が無い: $($r.Output)" }
        if ($r.Output.Contains('pasta-skills.zip')) { "出力に pasta-skills.zip が出る（無いのは pasta_check.zip だけのはず）: $($r.Output)" }
    }

    Test-Item 11 'github-release.ps1 が、添付 8 つのうち pasta-skills.zip だけが無いとき、終了コード 1 で、出力にその名前を示す' {
        $dir = Join-Path $tmp 'assets-skills'
        foreach ($n in 'pasta.dll.zip', 'hello-pasta.nar', 'pasta-vscode-0.0.2.vsix', 'pasta_check.zip', 'pasta-recipe.x64.zip', 'pasta-recipe.arm64.zip', 'release-notes-v0.0.2.md') { Write-Utf8 (Join-Path $dir $n) '' }
        $r = Invoke-Script github-release.ps1 @('-Tag', 'v0.0.2', '-NotesFile', $notesPath, '-AssetDir', $dir)
        if ($r.Exit -ne 1) { "終了コード $($r.Exit)（1 のはず）: $($r.Output)" }
        if (-not $r.Output.Contains('pasta-skills.zip')) { "出力に pasta-skills.zip が無い: $($r.Output)" }
        if ($r.Output.Contains('pasta_check.zip')) { "出力に pasta_check.zip が出る（無いのは pasta-skills.zip だけのはず）: $($r.Output)" }
    }

    Test-Item 12 'publish-vsix.ps1 が、偽の vsce が公開済みと答えるとき、status=skipped・終了コード 0 で終わる' {
        # 偽の vsce は show にだけ答える。publish を呼べば 1 で終わるので、公開に進めば分かる
        # shortcut: -VsceCommand は空白で区切るので、一時のフォルダのパスに空白があると NG になる。そうなったら -VsceCommand を配列の引数に変える
        $fake = Join-Path $tmp 'fake-vsce.ps1'
        Write-Utf8 $fake "if (`$args[0] -eq 'show') { '{`"versions`":[{`"version`":`"0.0.2`"}]}'; exit 0 }`nWrite-Error `"想定外の呼び出し: `$args`"; exit 1`n"
        $r = Invoke-Script publish-vsix.ps1 @('-VsixPath', 'no-such/*.vsix', '-Version', '0.0.2', '-Extension', 'test.ext', '-VsceCommand', $fake)
        if ($r.Exit -ne 0) { "終了コード $($r.Exit)（0 のはず）: $($r.Output)" }
        if (-not $r.Output.Contains('status=skipped')) { "出力に status=skipped が無い: $($r.Output)" }
    }

    Test-Item 13 'github-release.ps1 が、添付 8 つのうち pasta-recipe.x64.zip だけが無いとき、終了コード 1 で、出力にその名前を示す' {
        $dir = Join-Path $tmp 'assets-recipe-x64'
        foreach ($n in 'pasta.dll.zip', 'hello-pasta.nar', 'pasta-vscode-0.0.2.vsix', 'pasta_check.zip', 'pasta-skills.zip', 'pasta-recipe.arm64.zip', 'release-notes-v0.0.2.md') { Write-Utf8 (Join-Path $dir $n) '' }
        $r = Invoke-Script github-release.ps1 @('-Tag', 'v0.0.2', '-NotesFile', $notesPath, '-AssetDir', $dir)
        if ($r.Exit -ne 1) { "終了コード $($r.Exit)（1 のはず）: $($r.Output)" }
        if (-not $r.Output.Contains('pasta-recipe.x64.zip')) { "出力に pasta-recipe.x64.zip が無い: $($r.Output)" }
        if ($r.Output.Contains('pasta-recipe.arm64.zip')) { "出力に pasta-recipe.arm64.zip が出る（無いのは pasta-recipe.x64.zip だけのはず）: $($r.Output)" }
    }

    Test-Item 14 'github-release.ps1 が、添付 8 つのうち pasta-recipe.arm64.zip だけが無いとき、終了コード 1 で、出力にその名前を示す' {
        $dir = Join-Path $tmp 'assets-recipe-arm64'
        foreach ($n in 'pasta.dll.zip', 'hello-pasta.nar', 'pasta-vscode-0.0.2.vsix', 'pasta_check.zip', 'pasta-skills.zip', 'pasta-recipe.x64.zip', 'release-notes-v0.0.2.md') { Write-Utf8 (Join-Path $dir $n) '' }
        $r = Invoke-Script github-release.ps1 @('-Tag', 'v0.0.2', '-NotesFile', $notesPath, '-AssetDir', $dir)
        if ($r.Exit -ne 1) { "終了コード $($r.Exit)（1 のはず）: $($r.Output)" }
        if (-not $r.Output.Contains('pasta-recipe.arm64.zip')) { "出力に pasta-recipe.arm64.zip が無い: $($r.Output)" }
        if ($r.Output.Contains('pasta-recipe.x64.zip')) { "出力に pasta-recipe.x64.zip が出る（無いのは pasta-recipe.arm64.zip だけのはず）: $($r.Output)" }
    }

    # --- verify-recipe-zips.ps1（偽の exe・表示・dumpbin で。Windows の道具を使わない）---
    # 版（2.3）と 30 MB（2.6）の失敗は見ない（設計議題 4）。偽の exe の版は 0.0.0 と読めて版の問題も出るが、見たい文が含まれるかだけを見る。
    $noVc = Join-Path $tmp 'fake-dumpbin-ok.ps1'
    Write-Utf8 $noVc "'  KERNEL32.dll'`n'  ucrtbase.dll'`n"

    Test-Item 15 'verify-recipe-zips.ps1 が、arm64 の zip の exe の機械の種類が 0x8664 のとき、終了コード 1 で、zip の名前と 0xAA64・0x8664 を示す' {
        $dir = Join-Path $tmp 'recipe-15'
        New-RecipeZip $dir x64 0x8664
        New-RecipeZip $dir arm64 0x8664
        $r = Invoke-Script verify-recipe-zips.ps1 @('-Version', '0.0.2', '-Dir', $dir, '-DumpbinCommand', $noVc)
        if ($r.Exit -ne 1) { "終了コード $($r.Exit)（1 のはず）: $($r.Output)" }
        if (-not $r.Output.Contains('pasta-recipe.arm64.zip の pasta-recipe.exe の機械の種類が 0xAA64（arm64）でない: 0x8664')) { "出力に arm64 の機械の種類の文が無い: $($r.Output)" }
        if ($r.Output.Contains('pasta-recipe.x64.zip の pasta-recipe.exe の機械の種類')) { "x64 の zip の機械の種類まで問題になる: $($r.Output)" }
    }

    Test-Item 16 'verify-recipe-zips.ps1 が、x64 の zip が無く arm64 の表示に monaco-editor が無いとき、終了コード 1 で、両方の問題を示す' {
        $dir = Join-Path $tmp 'recipe-16'
        New-RecipeZip $dir arm64 0xAA64 'tauri'
        $r = Invoke-Script verify-recipe-zips.ps1 @('-Version', '0.0.2', '-Dir', $dir, '-DumpbinCommand', $noVc)
        if ($r.Exit -ne 1) { "終了コード $($r.Exit)（1 のはず）: $($r.Output)" }
        if (-not $r.Output.Contains('配布物が無い: pasta-recipe.x64.zip')) { "出力に x64 の zip が無い文が無い: $($r.Output)" }
        if (-not $r.Output.Contains('pasta-recipe.arm64.zip の THIRD_PARTY_LICENSES.txt に monaco-editor の名前が無い')) { "出力に monaco-editor の欠けの文が無い: $($r.Output)" }
    }

    Test-Item 17 'verify-recipe-zips.ps1 が、偽の dumpbin が VCRUNTIME140.dll を返すとき、終了コード 1 で、zip の名前と DLL の名前を示す' {
        $dir = Join-Path $tmp 'recipe-17'
        New-RecipeZip $dir x64 0x8664
        New-RecipeZip $dir arm64 0xAA64
        $vc = Join-Path $tmp 'fake-dumpbin-vc.ps1'
        Write-Utf8 $vc "'Dump of file pasta-recipe.exe'`n'  Image has the following dependencies:'`n'    KERNEL32.dll'`n'    VCRUNTIME140.dll'`n'    api-ms-win-crt-runtime-l1-1-0.dll'`n"
        $r = Invoke-Script verify-recipe-zips.ps1 @('-Version', '0.0.2', '-Dir', $dir, '-DumpbinCommand', $vc)
        if ($r.Exit -ne 1) { "終了コード $($r.Exit)（1 のはず）: $($r.Output)" }
        foreach ($n in 'pasta-recipe.x64.zip', 'pasta-recipe.arm64.zip') {
            if (-not $r.Output.Contains("$n の pasta-recipe.exe が VC++ ランタイムの DLL を読む: VCRUNTIME140.dll")) { "出力に $n の VCRUNTIME140.dll の文が無い: $($r.Output)" }
        }
        if ($r.Output -match 'DLL を読む: (KERNEL32|api-ms-win-crt)') { "OS の部品まで VC++ ランタイムに数える: $($r.Output)" }
    }

    # --- publish-ghost-site.ps1（-LatestTag で最新タグを渡し、待ちを 0 にする。ネットワークと gh を使わない）---
    $descript = Join-Path $tmp 'ghost-descript.txt'
    Write-Utf8 $descript "charset,UTF-8`nhomeurl,https://example.invalid/hello-pasta/`n"

    Test-Item 18 'publish-ghost-site.ps1 が、最新より古いタグ（v0.0.9 < v0.0.10）で、status=skipped・reason=stale-tag・終了コード 0 で終わる' {
        $r = Invoke-Script publish-ghost-site.ps1 @('-Tag', 'v0.0.9', '-LatestTag', 'v0.0.10', '-DescriptPath', $descript, '-LatestWaitSeconds', '0', '-DryRun')
        if ($r.Exit -ne 0) { "終了コード $($r.Exit)（0 のはず）: $($r.Output)" }
        if (-not $r.Output.Contains('status=skipped')) { "出力に status=skipped が無い: $($r.Output)" }
        if (-not $r.Output.Contains('reason=stale-tag')) { "出力に reason=stale-tag が無い: $($r.Output)" }
        if ($r.Output.Contains('dry-run:')) { "古いタグなのに計画まで進んだ: $($r.Output)" }
    }

    Test-Item 19 'publish-ghost-site.ps1 が、homeurl の無い（無いファイルを指す）古いタグでも、status=skipped・reason=stale-tag で終わる（homeurl を判定の後に読む）' {
        $r = Invoke-Script publish-ghost-site.ps1 @('-Tag', 'v0.0.1', '-LatestTag', 'v0.0.2', '-DescriptPath', (Join-Path $tmp 'no-such-descript.txt'), '-LatestWaitSeconds', '0', '-DryRun')
        if ($r.Exit -ne 0) { "終了コード $($r.Exit)（0 のはず）: $($r.Output)" }
        if (-not $r.Output.Contains('status=skipped')) { "出力に status=skipped が無い: $($r.Output)" }
        if (-not $r.Output.Contains('reason=stale-tag')) { "出力に reason=stale-tag が無い: $($r.Output)" }
    }

    Test-Item 20 'publish-ghost-site.ps1 が、最新より新しすぎるタグ（待ち 0）で、status=failed・reason=transient・終了コード 1 で終わる' {
        $r = Invoke-Script publish-ghost-site.ps1 @('-Tag', 'v0.1.0', '-LatestTag', 'v0.0.2', '-DescriptPath', $descript, '-LatestWaitSeconds', '0', '-DryRun')
        if ($r.Exit -ne 1) { "終了コード $($r.Exit)（1 のはず）: $($r.Output)" }
        if (-not $r.Output.Contains('status=failed')) { "出力に status=failed が無い: $($r.Output)" }
        if (-not $r.Output.Contains('reason=transient')) { "出力に reason=transient が無い: $($r.Output)" }
    }

    Test-Item 21 'publish-ghost-site.ps1 -DryRun が、最新と同じタグで、起動の手前の計画（manual.yml・ref main）を出して終了コード 0 で終わる' {
        $r = Invoke-Script publish-ghost-site.ps1 @('-Tag', 'v0.0.2', '-LatestTag', 'v0.0.2', '-DescriptPath', $descript, '-LatestWaitSeconds', '0', '-DryRun')
        if ($r.Exit -ne 0) { "終了コード $($r.Exit)（0 のはず）: $($r.Output)" }
        if (-not $r.Output.Contains('dry-run:')) { "出力に計画（dry-run:）が無い: $($r.Output)" }
        if (-not ($r.Output.Contains('manual.yml') -and $r.Output.Contains('ref=main'))) { "計画に manual.yml・ref=main が無い: $($r.Output)" }
        if ($r.Output.Contains('status=')) { "計画で status まで書いた（起動の手前で終わるはず）: $($r.Output)" }
    }

    # --- publish-ghost-site.ps1 の起動・待ち・見比べ（偽の gh と node で。答えは環境変数 FAKE_* で変える）---
    $ghLog = Join-Path $tmp 'fake-gh.log'
    $nodeLog = Join-Path $tmp 'fake-node.log'
    $fakeGh = Join-Path $tmp 'fake-gh.ps1'
    Write-Utf8 $fakeGh @'
Add-Content -LiteralPath $env:FAKE_GH_LOG -Value ($args -join ' ')
$a = @($args)
if ($a[0] -eq 'release' -and $a[1] -eq 'download') {
    if ($env:FAKE_GH_DOWNLOAD -eq 'fail') { 'release not found'; exit 1 }
    $dir = $a[[array]::IndexOf($a, '--dir') + 1]
    New-Item -ItemType Directory -Force -Path $dir | Out-Null
    Set-Content -LiteralPath (Join-Path $dir 'hello-pasta.nar') -Value 'nar'
    exit 0
}
if ($a[0] -eq 'api' -and $a -contains 'POST') { $env:FAKE_GH_DISPATCH; exit [int]$env:FAKE_GH_DISPATCH_EXIT }
if ($a[0] -eq 'run' -and $a[1] -eq 'list') { if ($env:FAKE_GH_RUNS) { $env:FAKE_GH_RUNS } else { '[]' }; exit 0 }
if ($a[0] -eq 'run' -and $a[1] -eq 'view') {
    "{`"status`":`"$env:FAKE_GH_STATUS`",`"conclusion`":`"$env:FAKE_GH_CONCLUSION`",`"url`":`"https://example.invalid/runs/$($a[2])`"}"
    exit 0
}
"想定外の呼び出し: $args"; exit 1
'@
    $fakeNode = Join-Path $tmp 'fake-node.ps1'
    Write-Utf8 $fakeNode "Add-Content -LiteralPath `$env:FAKE_NODE_LOG -Value (`$args -join ' ')`n`$env:FAKE_NODE_OUT`nexit [int]`$env:FAKE_NODE_EXIT`n"
    $noHome = Join-Path $tmp 'ghost-descript-nohome.txt'
    Write-Utf8 $noHome "charset,UTF-8`nname,hello-pasta`n"

    # 起動から見比べまでを偽の gh・node で通す。$Fake は FAKE_* の値（既定は「id を返す起動・success・見比べ 3 回目で一致」）。
    function Invoke-Ghost([hashtable]$Fake = @{}, [string[]]$Extra = @(), [string]$Descript = $descript) {
        $vals = @{ DISPATCH = '{"workflow_run_id":4242}'; DISPATCH_EXIT = '0'; RUNS = ''; STATUS = 'completed'; CONCLUSION = 'success'; DOWNLOAD = ''; NODE_OUT = 'OK updates.txt: 3 回目で一致'; NODE_EXIT = '0' }
        foreach ($k in $Fake.Keys) { $vals[$k] = $Fake[$k] }
        foreach ($p in $ghLog, $nodeLog) { Set-Content -LiteralPath $p -Value '' }
        $env:FAKE_GH_LOG = $ghLog; $env:FAKE_NODE_LOG = $nodeLog
        foreach ($k in 'DISPATCH', 'DISPATCH_EXIT', 'RUNS', 'STATUS', 'CONCLUSION', 'DOWNLOAD') { Set-Item "Env:FAKE_GH_$k" $vals[$k] }
        $env:FAKE_NODE_OUT = $vals.NODE_OUT; $env:FAKE_NODE_EXIT = $vals.NODE_EXIT
        $env:GITHUB_REPOSITORY = $Repo
        $env:RUNNER_TEMP = $tmp  # 取った nar を一時のフォルダに置き、終わりに一緒に消す
        try {
            $r = Invoke-Script publish-ghost-site.ps1 (@('-Tag', 'v0.0.2', '-LatestTag', 'v0.0.2', '-DescriptPath', $Descript, '-LatestWaitSeconds', '0',
                    '-RunPollSeconds', '0', '-VerifyIntervalSeconds', '0', '-GhCommand', $fakeGh, '-NodeCommand', $fakeNode) + $Extra)
        }
        finally { Remove-Item Env:FAKE_*, Env:GITHUB_REPOSITORY, Env:RUNNER_TEMP -ErrorAction SilentlyContinue }
        $r | Add-Member Gh ([IO.File]::ReadAllText($ghLog)) -PassThru | Add-Member Node ([IO.File]::ReadAllText($nodeLog).Trim()) -PassThru
    }

    # 結果が status・reason・終了コードの組かを見て、違えば理由を返す。
    function Get-OutcomeFailure($Run, [string]$Status, [string]$Reason, [string]$Case) {
        $exit = if ($Status -eq 'failed') { 1 } else { 0 }
        if ($Run.Exit -ne $exit) { "${Case}: 終了コード $($Run.Exit)（$exit のはず）: $($Run.Output)" }
        if (-not $Run.Output.Contains("status=$Status")) { "${Case}: 出力に status=$Status が無い: $($Run.Output)" }
        if ($Reason -and -not $Run.Output.Contains("reason=$Reason")) { "${Case}: 出力に reason=$Reason が無い: $($Run.Output)" }
    }

    Test-Item 22 'publish-ghost-site.ps1 が、起動（ref=main・return_run_details）→ success → 見比べ 3 回目で一致で、status=published で終わり、summary に run の URL と見比べの回数を出す' {
        $r = Invoke-Ghost
        Get-OutcomeFailure $r published '' '一致'
        if ($r.Output.Contains('未実装')) { "未実装の行が残っている: $($r.Output)" }
        if ($r.Gh -notmatch 'release download v0\.0\.2 .*--pattern hello-pasta\.nar') { "Release の nar を取っていない: $($r.Gh)" }
        if ($r.Gh -notmatch 'actions/workflows/manual\.yml/dispatches .*ref=main.*return_run_details=true') { "起動の引数が違う: $($r.Gh)" }
        if ($r.Gh -match 'run list') { "workflow_run_id があるのに run list を呼んだ: $($r.Gh)" }
        if ($r.Gh -notmatch 'run view 4242') { "応答の run（4242）を待っていない: $($r.Gh)" }
        if ($r.Node -notmatch 'verify-live --nar .*hello-pasta\.nar --base https://example\.invalid/hello-pasta/ --attempts 30 --interval 0') { "verify-live の引数が違う: $($r.Node)" }
        if (-not $r.Output.Contains('https://example.invalid/runs/4242')) { "summary に run の URL が無い: $($r.Output)" }
        if (-not $r.Output.Contains('見比べ 3 回')) { "summary に見比べの回数が無い: $($r.Output)" }
    }

    Test-Item 23 'publish-ghost-site.ps1 の結論の判定表: success・cancelled → 見比べへ（published）、failure・timed_out・startup_failure・action_required → failed・build（見比べない）' {
        foreach ($c in 'success', 'cancelled') {
            $r = Invoke-Ghost @{ CONCLUSION = $c }
            Get-OutcomeFailure $r published '' $c
            if (-not $r.Node) { "${c}: 見比べに進んでいない" }
        }
        foreach ($c in 'failure', 'timed_out', 'startup_failure', 'action_required') {
            $r = Invoke-Ghost @{ CONCLUSION = $c }
            Get-OutcomeFailure $r failed build $c
            if ($r.Node) { "${c}: 失敗した run なのに見比べた: $($r.Node)" }
        }
    }

    Test-Item 24 'publish-ghost-site.ps1 が、run が in_progress のまま待ちの上限（0 分）を過ぎると、failed・transient で終わる' {
        $r = Invoke-Ghost @{ STATUS = 'in_progress'; CONCLUSION = '' } @('-RunTimeoutMinutes', '0')
        Get-OutcomeFailure $r failed transient 'in_progress'
        if ($r.Node) { "完了していない run なのに見比べた: $($r.Node)" }
    }

    Test-Item 25 'publish-ghost-site.ps1 が、応答に workflow_run_id が無い・起動が拒否されたとき、run list から起動後の最新の run を特定し、無ければ failed・dispatch で終わる' {
        $now = (Get-Date).ToUniversalTime()
        $old = @{ databaseId = 11; createdAt = $now.AddHours(-1).ToString('yyyy-MM-ddTHH:mm:ssZ'); url = 'https://example.invalid/runs/11' }
        $new1 = @{ databaseId = 21; createdAt = $now.AddMinutes(1).ToString('yyyy-MM-ddTHH:mm:ssZ'); url = 'https://example.invalid/runs/21' }
        $new2 = @{ databaseId = 22; createdAt = $now.AddMinutes(2).ToString('yyyy-MM-ddTHH:mm:ssZ'); url = 'https://example.invalid/runs/22' }
        $r = Invoke-Ghost @{ DISPATCH = ''; RUNS = (ConvertTo-Json -Compress @($new1, $old, $new2)) }
        Get-OutcomeFailure $r published '' '応答が空'
        if ($r.Gh -notmatch 'run list .*--workflow manual\.yml.*--branch main.*--event workflow_dispatch') { "run list の絞り込みが違う: $($r.Gh)" }
        if ($r.Gh -notmatch 'run view 22') { "起動後の最新の run（22）を待っていない: $($r.Gh)" }
        $r = Invoke-Ghost @{ DISPATCH = 'HTTP 403'; DISPATCH_EXIT = '1'; RUNS = (ConvertTo-Json -Compress @($old, $new1)) }
        Get-OutcomeFailure $r published '' '拒否・run は在る'
        if ($r.Gh -notmatch 'run view 21') { "拒否のあと起動後の run（21）を待っていない: $($r.Gh)" }
        $r = Invoke-Ghost @{ DISPATCH = ''; RUNS = (ConvertTo-Json -Compress @($old)) }
        Get-OutcomeFailure $r failed dispatch '起動前の run だけ'
        if ($r.Gh -match 'run view') { "特定できないのに run を待った: $($r.Gh)" }
        $r = Invoke-Ghost @{ DISPATCH = 'HTTP 403'; DISPATCH_EXIT = '1' }
        Get-OutcomeFailure $r failed dispatch '拒否・run も無い'
    }

    Test-Item 26 'publish-ghost-site.ps1 が、見比べの不一致で failed・verify、homeurl の無い descript.txt で failed・publish、nar の取得の失敗で failed・transient で終わる' {
        $r = Invoke-Ghost @{ NODE_OUT = 'updates.txt が違う'; NODE_EXIT = '1' }
        Get-OutcomeFailure $r failed verify '不一致'
        $r = Invoke-Ghost -Descript $noHome
        Get-OutcomeFailure $r failed publish 'homeurl 無し'
        if ($r.Node) { "homeurl が無いのに見比べた: $($r.Node)" }
        $r = Invoke-Ghost @{ DOWNLOAD = 'fail' }
        Get-OutcomeFailure $r failed transient 'nar の取得の失敗'
        if ($r.Gh -match 'dispatches') { "nar を取れないのに起動した: $($r.Gh)" }
    }

    Test-Item 27 'publish-ghost-site.ps1 -Ref feature（リハーサル）が、ref=feature で起動し run list を --branch feature で絞り、結論の判定までで見比べずに終了コード 0 で終わる' {
        $now = (Get-Date).ToUniversalTime()
        $run = @{ databaseId = 31; createdAt = $now.AddMinutes(1).ToString('yyyy-MM-ddTHH:mm:ssZ'); url = 'https://example.invalid/runs/31' }
        $r = Invoke-Ghost @{ DISPATCH = ''; RUNS = (ConvertTo-Json -Compress @($run)) } @('-Ref', 'feature')
        if ($r.Exit -ne 0) { "終了コード $($r.Exit)（0 のはず）: $($r.Output)" }
        if ($r.Gh -notmatch 'dispatches .*ref=feature') { "ref=feature で起動していない: $($r.Gh)" }
        if ($r.Gh -notmatch 'run list .*--branch feature') { "run list を --branch feature で絞っていない: $($r.Gh)" }
        if ($r.Node) { "リハーサルなのに見比べた: $($r.Node)" }
        if (-not $r.Output.Contains('rehearsal:')) { "出力にリハーサルの結果（rehearsal:）が無い: $($r.Output)" }
        if ($r.Output.Contains('status=published')) { "リハーサルで published と書いた: $($r.Output)" }
        $r = Invoke-Ghost @{ CONCLUSION = 'failure' } @('-Ref', 'feature')
        Get-OutcomeFailure $r failed build 'リハーサルの failure'
    }

    Test-Item 28 'publish-ghost-site.ps1 が、最新の Release のタグが vX.Y.Z の形でないとき、failed・publish で終わる（再実行では直らない）' {
        $r = Invoke-Script publish-ghost-site.ps1 @('-Tag', 'v0.0.2', '-LatestTag', 'nightly', '-DescriptPath', $descript, '-LatestWaitSeconds', '0', '-DryRun')
        Get-OutcomeFailure $r failed publish '形の違う最新のタグ'
    }
}
finally {
    [Console]::OutputEncoding = $savedEncoding
    if (Test-Path -LiteralPath $tmp) { Remove-Item -LiteralPath $tmp -Recurse -Force }
}

if ($ng.Count -gt 0) {
    Write-Output ''
    Write-Output "NG の項目（$($ng.Count) 件）:"
    $ng | ForEach-Object { Write-Output "- $_" }
    exit 1
}
exit 0
