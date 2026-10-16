<#
.SYNOPSIS
  .kiro/steering/roadmap.md を読み、版ごとの帯と spec のカードを並べた 1 枚の HTML を作る。

.DESCRIPTION
  areka の同名の道具（areka main 0bb51d42）を、pasta の roadmap.md の形に合わせて写したもの。
  roadmap.md は読むだけで、書き替えない。画面へ出す文は ASCII だけ。
  端末の文字コードは読みも書きもしない。今のフォルダを変えない。呼ばれた場所によらず、このスクリプトの在るリポジトリを見る。
  日本語の読み取りの目印と HTML の決まった文言は、文言のファイル（STRINGS_FILE）の 1 か所だけに置く。

  読む所（目印は文言のファイルの markers）:
    更新日      「## 棚卸（YYYY-MM-DD・…）」の見出しの日付
    帯          見出しの行の最初の 2 欄が「順｜spec」の表（「0.4.0 までの道筋」と Phase 13 の「順番」など）。
                1 表が 1 つの帯。帯の名前はすぐ上の「## 」見出しの版の番号、目標はその見出しの地の文。
                spec の欄が spec 名に見えない行（「0.4.0 のリリース」など）は読まない。順が「完了」の行は完了
    台帳        「## Specs (dependency order)」の「- [ ] 名前 -- 何をするか。Dependencies: …」の行
    テーマ      棚卸の節と台帳の節の間の「## 」の節（Phase ごとの節など）。spec 名が最初に出た節がその spec のテーマ
    ウェーブ    「### ウェーブ（日付）」の節の最初の表の 1 欄目と、その節の最初の ``` の塊（開始コマンド）
    完了        「## 完了フェーズ」の節の最初の表（Phase・主題・spec）
  台帳に在ってどの帯にも無い未完了の spec は、末尾の「版が決まっていないもの」の帯に置く（警告にしない）。

    （引数なし） [-Open] [-OutFile <path>]
        roadmap.md から HTML を作る。-Open を付けると、書けたときだけ OS の既定のアプリで開く。
        -OutFile を付けると doc/ の日付つきの名前でなく、その場所へ書く（相対なら今のフォルダから）。
          FAIL strings: {理由}                   文言のファイルが読めない・鍵が欠けるか空
          FAIL template: {理由}                  ひな形が読めない・約束を破る
          FAIL missing roadmap                   .kiro/steering/roadmap.md が読めない
          FAIL missing {鍵}                      読めない所（欠けごとに 1 行・何も書かない）
          WARN {鍵} ...                          読めたが食い違う所（警告ごとに 1 行・書く）
          WARN git: blob unknown                 中身の印（git hash-object の先頭 12 字）を求められない
          FAIL write {path}: {理由}              HTML を書けない
          OK wrote {path} ({帯} band(s), {spec} spec(s), {警告} warning(s))
        終了コード: 0 書けた（警告が在っても 0）／1 FAIL の行を出した
        欠けの鍵: updated-at・version-table・ledger・wave-table・summary-table
        警告の鍵: not-in-ledger（帯に在る未完了の spec が台帳に無い）・duplicate（同じ spec が帯に 2 回）・
                  table-columns（表の行の欄の数が見出しの行と違う）
    -SelfTest
        自分の検査。判定を 1 つずつ「ok selftest {名前}」か「FAIL selftest {名前}」で印字する。
        見本はこのスクリプトの隣の roadmap-overview-fixture/（roadmap.md＝見本の断片・expected-data.json＝期待の値）。
          1 strings  文言のファイルが読めて、markers と labels の鍵がそろい、どれも空でない
                     （鍵を 1 つ外した写しと、1 つを空にした写しが落ちることも確かめる）
          2 template ひな形が約束を守る: 差し込み口がちょうど 1 つ・CHARSET を持つ・
                     EXTERNAL に当たる書き方が無い・ASCII だけ（1 か所ずつ崩した写しがその約束だけで落ちることも確かめる）
          3 fixture data equals the expected data
                     見本から作ったデータの JSON（字下げつき・LF・中身の印は 0123456789ab）が期待の値と 1 字違わず同じ
          4 fixture has no missing and no warning
          5 fixture broken for {鍵} warns with {鍵} only
                     見本を 1 か所ずつ壊した文が、その警告の鍵だけを出す。壊した文が見本と同じなら落とす
          6 fixture without {所} of {鍵} is missing {鍵} only
                     見本から 1 か所ずつ外した文が、その欠けの鍵だけを出し、データを作らない
                     （外し方が 7 でなければ「fixture removals cover every missing key」が落ちる）
          7 fixture html is the same twice and escapes the script end tag
                     見本から 2 回作った HTML が 1 字違わず同じで、台帳の文の </script> が JSON の逃がしになっている

  終了コード: 0 通った／1 落ちた判定が在る

  較正値（変更はスクリプト冒頭の「較正値」の 1 か所だけ）:
    STRINGS_FILE   文言のファイル（このスクリプトと同じフォルダ）
    MARKER_KEYS    読み取りの目印（markers）の鍵
    LABEL_KEYS     HTML の決まった文言（labels）の鍵
    SECTION_PREFIX 節の見出しの頭（「## 」）
    TABLE_RULE     表の区切り行（|---）に当てる正規表現
    ONE_LINER_MAX  カードの一言の上限の字数（越えれば切って「…」を足す）
    SPEC_NAME      spec 名に見える語に当てる正規表現
    SPEC_IN_TEXT   文の中の spec 名に当てる正規表現（前提の文・完了の表の spec の欄・テーマの節の本文）
    TEMPLATE_FILE  HTML・CSS・JS のひな形（このスクリプトと同じフォルダ）
    PLACEHOLDER    ひな形のデータの差し込み口
    CHARSET        ひな形が持つ文字コードの宣言
    EXTERNAL       ひな形に在ってはならない、外の物を読む書き方に当てる正規表現

.PARAMETER Open
  書けたときだけ、作った HTML を OS の既定のアプリ（ブラウザ）で開く。

.PARAMETER OutFile
  書く先のファイル。省くと doc/roadmap-overview-{更新日}.html。/kiro-next は棚卸の最後に
  scratch の roadmap-overview-latest.html へ書いて、セッションで公開する。

.PARAMETER SelfTest
  自分の検査だけを回す。何も書かない。

.EXAMPLE
  pwsh -NoProfile -File .claude/skills/kiro-next/overview/roadmap-overview.ps1 -OutFile C:\some\folder\roadmap-overview-latest.html
  pwsh -NoProfile -File .claude/skills/kiro-next/overview/roadmap-overview.ps1 -SelfTest
#>
#Requires -Version 7
# 知らない引数（綴り違いを含む）は黙って捨てずに断る
[CmdletBinding(DefaultParameterSetName = 'Build')]
param(
    [Parameter(ParameterSetName = 'Build')][switch]$Open,
    [Parameter(ParameterSetName = 'Build')][string]$OutFile,
    [Parameter(ParameterSetName = 'SelfTest', Mandatory)][switch]$SelfTest
)

Set-StrictMode -Version 3.0
$ErrorActionPreference = 'Stop'

# =============================================================================
# 較正値（説明の一覧と対応。変更はここだけ）
# =============================================================================
$STRINGS_FILE = 'roadmap-overview.strings.json'
$MARKER_KEYS  = 'secInventory', 'secWave', 'secLedger', 'secSummary', 'bandHeader', 'versionRegex', 'dateRegex',
                'doneOrder', 'rowSep', 'depsMarker', 'brackets', 'sentenceEnd', 'ellipsis'
$LABEL_KEYS   = 'title', 'sourceUpdated', 'sourceBlob', 'sourceBlobUnknown', 'bandCount', 'warningCount', 'wave',
                'done', 'notInLedger', 'notInBand', 'otherBands', 'prereqs', 'notInRoadmap', 'order', 'model', 'size',
                'noTheme', 'clearFilter', 'completedHeading', 'colPhase', 'colTopic', 'colSpecs', 'completedExtra',
                'what', 'note', 'waveCommands', 'copy', 'copied'
$SECTION_PREFIX = '## '
$TABLE_RULE     = '^\|\s*:?-'
$ONE_LINER_MAX  = 60
$SPEC_NAME      = '^[a-z0-9]+(-[a-z0-9]+)+$'
$SPEC_IN_TEXT   = '(?<![A-Za-z0-9_.-])[a-z0-9]+(?:-[a-z0-9]+)+(?![A-Za-z0-9_-])'
$TEMPLATE_FILE  = 'roadmap-overview.template.html'
$PLACEHOLDER    = '@@ROADMAP_DATA@@'
$CHARSET        = '<meta charset="utf-8">'
$EXTERNAL       = '(?i)<script\b[^>]*\ssrc\s*=|<link\b|@import|url\(|https?://'

# リポジトリの根＝このスクリプトから上へ辿って最初に .kiro の在るフォルダ。
# 無ければスクリプトのフォルダ（roadmap.md が読めずに FAIL missing roadmap になる）
$root = $PSScriptRoot
for ($d = $PSScriptRoot; $d; $d = Split-Path $d -Parent) {
    if ([System.IO.Directory]::Exists((Join-Path $d '.kiro'))) { $root = $d; break }
}

# ファイルを UTF-8 で読み、改行を LF にそろえて返す。無ければ $null
function Read-Lf([string]$Path) {
    if (-not [System.IO.File]::Exists($Path)) { return $null }
    [System.IO.File]::ReadAllText($Path, [System.Text.UTF8Encoding]::new($false)).Replace("`r`n", "`n")
}

# BOM なしの UTF-8 で書く（親のフォルダが無ければ作る）
function Write-Utf8([string]$Path, [string]$Text) {
    [void][System.IO.Directory]::CreateDirectory((Split-Path $Path -Parent))
    [System.IO.File]::WriteAllText($Path, $Text, [System.Text.UTF8Encoding]::new($false))
}

# 文言（ConvertFrom-Json の結果）の欠けを「markers.secLedger」の形で全部返す（空なら通った）。
# 値は空でない文字列か、空でない文字列だけの空でない並び
function Test-Strings($Strings) {
    foreach ($group in @{ markers = $MARKER_KEYS; labels = $LABEL_KEYS }.GetEnumerator() | Sort-Object Key) {
        $obj = $Strings -is [pscustomobject] ? $Strings.PSObject.Properties[$group.Key]?.Value : $null
        foreach ($k in $group.Value) {
            $v = $obj -is [pscustomobject] ? $obj.PSObject.Properties[$k]?.Value : $null
            $items = @($v)
            # 1 要素の並びの真偽は中身で決まる（@('') は偽）ので、数で見る
            $ok = $null -ne $v -and $items.Count -gt 0 -and
                  @($items | Where-Object { $_ -isnot [string] -or $_ -eq '' }).Count -eq 0
            if (-not $ok) { "$($group.Key).$k" }
        }
    }
}

# 文言を UTF-8 で読んで返す。読めない・鍵が欠けるか空のときは throw（文は ASCII）
function Read-Strings {
    $text = Read-Lf (Join-Path $PSScriptRoot $STRINGS_FILE)
    if ($null -eq $text) { throw "cannot read $STRINGS_FILE" }
    try { $s = $text | ConvertFrom-Json } catch { throw "$STRINGS_FILE is not valid JSON" }
    $bad = @(Test-Strings $s)
    if ($bad.Count) { throw "missing or empty key(s) in ${STRINGS_FILE}: $($bad -join ', ')" }
    $s
}

# ひな形の約束の破れを鍵（placeholder・charset・external・ascii）で全部返す（空なら通った）
function Test-Template([string]$Template) {
    if (([regex]::Matches($Template, [regex]::Escape($PLACEHOLDER))).Count -ne 1) { 'placeholder' }
    if (-not $Template.Contains($CHARSET)) { 'charset' }
    if ($Template -match $EXTERNAL) { 'external' }
    if ($Template -match '[^\x00-\x7F]') { 'ascii' }
}

# 文言とデータを HTML の字を逃がす JSON にし、ひな形のただ 1 つの差し込み口を文字列の置き換え（正規表現でない）で替える。
# 改行は LF。時刻・乱数・環境の値は入れない
function Format-Html([string]$Template, [pscustomobject]$Data, [pscustomobject]$Labels) {
    if (([regex]::Matches($Template, [regex]::Escape($PLACEHOLDER))).Count -ne 1) { throw 'template must have exactly one placeholder' }
    $json = [ordered]@{ labels = $Labels; data = $Data } | ConvertTo-Json -Depth 10 -Compress -EscapeHandling EscapeHtml
    $Template.Replace("`r`n", "`n").Replace($PLACEHOLDER, $json)
}

# 表の 1 行を欄に割る。先頭と末尾の | は在っても無くてもよい。バッククォートの中の | と \| は区切りにしない
# （\| は | に直す）。欄は前後の空白を外す
function Split-Row([string]$Line) {
    $s = $Line.Trim()
    $cells = [System.Collections.Generic.List[string]]::new()
    $cur = [System.Text.StringBuilder]::new()
    $inCode = $false
    $endsWithSep = $false
    for ($i = 0; $i -lt $s.Length; $i++) {
        $ch = $s[$i]
        $endsWithSep = $false
        if ($ch -eq '\' -and $i + 1 -lt $s.Length -and $s[$i + 1] -eq '|') { [void]$cur.Append('|'); $i++; continue }
        if ($ch -eq '`') { $inCode = -not $inCode }
        if ($ch -eq '|' -and -not $inCode) {
            $cells.Add($cur.ToString().Trim()); [void]$cur.Clear(); $endsWithSep = $true; continue
        }
        [void]$cur.Append($ch)
    }
    if (-not $endsWithSep) { $cells.Add($cur.ToString().Trim()) }
    if ($s.StartsWith('|') -and $cells.Count) { $cells.RemoveAt(0) }
    [string[]]$cells
}

# $Prefix で始まる見出しの行の節を返す。終わりは、同じかより上の段の次の見出しの手前。
# 返り値 @{ Start = 見出しの行の添字; End = 終わりの添字（含まない） }。無ければ $null
function Get-Section([string[]]$Lines, [string]$Prefix) {
    $level = ([regex]::Match($Prefix, '^#+')).Length
    for ($i = 0; $i -lt $Lines.Count; $i++) {
        if (-not $Lines[$i].StartsWith($Prefix)) { continue }
        $end = $i + 1
        while ($end -lt $Lines.Count -and $Lines[$end] -notmatch "^#{1,$level} ") { $end++ }
        return [pscustomobject]@{ Start = $i; End = $end }
    }
    $null
}

# 行の添字 $From から $To（含まない）までの最初の表を返す。無ければ $null。
# 見出し行と区切り行（|---）は Rows に入れない。表は「|」で始まる行が続く所まで。Line は 1 から数えた行番号
function Get-Table([string[]]$Lines, [int]$From, [int]$To) {
    $i = $From
    while ($i -lt $To -and -not $Lines[$i].StartsWith('|')) { $i++ }
    if ($i -ge $To) { return $null }
    $head = [string[]](Split-Row $Lines[$i])
    $i++
    if ($i -lt $To -and $Lines[$i] -match $TABLE_RULE) { $i++ }
    $rows = [System.Collections.Generic.List[object]]::new()
    for (; $i -lt $To -and $Lines[$i].StartsWith('|'); $i++) {
        $rows.Add([pscustomobject]@{ Line = $i + 1; Cells = [string[]](Split-Row $Lines[$i]) })
    }
    [pscustomobject]@{ Head = $head; Width = $head.Count; Rows = [object[]]$rows; Next = $i }
}

# ** とバッククォートを外した地の文（前後の空白も外す）
function ConvertTo-Plain([string]$Text) { ($Text -replace '\*\*', '' -replace '`', '').Trim() }

# 括弧の組（markers.brackets）の中を入れ子ごと外した、深さ 0 の文。
# バッククォートの中は塊として残し、中の括弧は数えない
function Get-TopText([string]$Text, [pscustomobject]$Markers) {
    $opens = @($Markers.brackets | ForEach-Object { [string]$_[0] })
    $closes = @($Markers.brackets | ForEach-Object { [string]$_[1] })
    $sb = [System.Text.StringBuilder]::new()
    $depth = 0
    $inCode = $false
    foreach ($c in $Text.ToCharArray()) {
        $ch = [string]$c
        if ($ch -eq '`') { $inCode = -not $inCode }
        elseif (-not $inCode) {
            if ($opens -ccontains $ch) { $depth++; continue }
            if ($closes -ccontains $ch) { if ($depth -gt 0) { $depth-- }; continue }
        }
        if ($depth -eq 0) { [void]$sb.Append($ch) }
    }
    $sb.ToString()
}

# 一言: 最初の括弧の組（（））を入れ子ごと外す（「」などは残す）→ ConvertTo-Plain → 最初の「。」の前まで →
# 前後の空白を外す → ONE_LINER_MAX 字（書記素で数える）を越えれば切って「…」を足す
function Get-OneLiner([string]$Text, [pscustomobject]$Markers) {
    $open, $close = [string]$Markers.brackets[0][0], [string]$Markers.brackets[0][1]
    $sb = [System.Text.StringBuilder]::new()
    $depth = 0
    foreach ($c in $Text.ToCharArray()) {
        $ch = [string]$c
        if ($ch -ceq $open) { $depth++ }
        elseif ($ch -ceq $close -and $depth -gt 0) { $depth-- }
        elseif ($depth -eq 0) { [void]$sb.Append($ch) }
    }
    $t = ConvertTo-Plain $sb.ToString()
    $cut = $t.IndexOf($Markers.sentenceEnd)
    if ($cut -ge 0) { $t = $t.Substring(0, $cut) }
    $t = $t.Trim()
    $info = [System.Globalization.StringInfo]::new($t)
    if ($info.LengthInTextElements -le $ONE_LINER_MAX) { return $t }
    $info.SubstringByTextElements(0, $ONE_LINER_MAX) + $Markers.ellipsis
}

# 表の欄（括弧の注とバッククォートを外した地の文）が spec 名に見えれば、その名前。見えなければ $null
function Get-SpecName([string]$Cell, [pscustomobject]$Markers) {
    $w = ConvertTo-Plain (Get-TopText $Cell $Markers)
    $w -cmatch $SPEC_NAME ? $w : $null
}

# 文の深さ 0（括弧の注を外した所）に出る spec 名を、出る順に重なりなく返す（呼ぶ側が @() で包む）
function Get-TextNames([string]$Text, [pscustomobject]$Markers) {
    $seen = [System.Collections.Generic.List[string]]::new()
    foreach ($m in [regex]::Matches((Get-TopText $Text $Markers), $SPEC_IN_TEXT)) {
        if (-not $seen.Contains($m.Value)) { $seen.Add($m.Value) }
    }
    $seen
}

# 表の行の欄を、見出し行の欄の数（$Width）の長さ以上にそろえて返す。数が違えば table-columns の警告を
# $Warnings へ足す（足りない欄は空として読む・余りの欄は残す）。$Key は表の鍵（version-table など）
function Get-Cells($Row, [int]$Width, [string]$Key, $Warnings) {
    $cells = [string[]]$Row.Cells
    if ($cells.Count -ne $Width) {
        $Warnings.Add([pscustomobject]@{ Key = 'table-columns'; Detail = "$Key line $($Row.Line): $($cells.Count) cell(s), expected $Width" })
    }
    if ($cells.Count -lt $Width) { $cells = [string[]](@($cells) + @('') * ($Width - $cells.Count)) }
    $cells
}

# 帯の表（見出しの行の最初の 2 欄が markers.bandHeader）を全部、出る順に読む。1 つも無ければ $null。
# 帯の名前はすぐ上の「## 」見出しの版の番号（無ければ目標と同じ）、目標はその見出しの深さ 0 の地の文。
# 欄は 順・spec・要件定義・規模・注（5 欄目。無ければ注なし）
function Get-Bands([string[]]$Lines, [pscustomobject]$Markers) {
    $bands = [System.Collections.Generic.List[object]]::new()
    $warnings = [System.Collections.Generic.List[object]]::new()
    $heading = ''
    $i = 0
    while ($i -lt $Lines.Count) {
        if ($Lines[$i].StartsWith($SECTION_PREFIX)) { $heading = $Lines[$i].Substring($SECTION_PREFIX.Length) }
        if (-not $Lines[$i].StartsWith('|')) { $i++; continue }
        $table = Get-Table $Lines $i $Lines.Count
        $i = $table.Next
        $h = $table.Head
        if ($h.Count -lt 2 -or $h[0] -cne $Markers.bandHeader[0] -or $h[1] -cne $Markers.bandHeader[1]) { continue }
        $cards = foreach ($row in $table.Rows) {
            $c = Get-Cells $row $table.Width 'version-table' $warnings
            $name = Get-SpecName $c[1] $Markers
            if ($null -eq $name) { continue }
            $order = ConvertTo-Plain $c[0]
            $note = $table.Width -gt 4 ? (ConvertTo-Plain $c[4]) : ''
            [pscustomobject][ordered]@{
                name = $name; order = $order; done = $order -ceq $Markers.doneOrder
                model = ConvertTo-Plain $c[2]; size = ConvertTo-Plain $c[3]; note = $note -eq '' ? $null : $note
            }
        }
        $goal = ConvertTo-Plain (Get-TopText $heading $Markers)
        $ver = [regex]::Match($heading, $Markers.versionRegex)
        $bands.Add([pscustomobject]@{ Name = $ver.Success ? $ver.Value : $goal; Goal = $goal; Cards = [object[]]@($cards) })
    }
    if ($bands.Count -eq 0) { return $null }
    [pscustomobject]@{ Bands = [object[]]$bands; Warnings = [object[]]$warnings }
}

# 台帳（markers.secLedger の節の「- [{印}] 名前 -- 何をするか。Dependencies: …」の行）を読む。節が無ければ $null。
# 返り値は行の並び @{ name; done; what（Dependencies の前の地の文）; prereq（前提の全文の地の文）; prereqs（深さ 0 の名前） }
function Get-Ledger([string[]]$Lines, [pscustomobject]$Markers) {
    $sec = Get-Section $Lines $Markers.secLedger
    if ($null -eq $sec) { return $null }
    $rows = for ($i = $sec.Start + 1; $i -lt $sec.End; $i++) {
        if ($Lines[$i] -notmatch '^- \[(.)\]\s*(.*)$') { continue }
        $mark, $body = $Matches[1], $Matches[2]
        $sep = $body.IndexOf($Markers.rowSep)
        $head, $rest = $sep -ge 0 ? @($body.Substring(0, $sep), $body.Substring($sep + $Markers.rowSep.Length)) : @($body, '')
        $name = Get-SpecName $head $Markers
        if ($null -eq $name) { continue }
        $d = $rest.LastIndexOf($Markers.depsMarker)
        $what, $deps = $d -ge 0 ? @($rest.Substring(0, $d), $rest.Substring($d + $Markers.depsMarker.Length)) : @($rest, '')
        [pscustomobject][ordered]@{
            name = $name; done = $mark -ceq 'x'; what = ConvertTo-Plain $what
            prereq = ConvertTo-Plain $deps; prereqs = [string[]]@(Get-TextNames $deps $Markers)
        }
    }
    , [object[]]@($rows)
}

# テーマの節（棚卸の節の後から台帳の節の前までの「## 」の節）を、名前（見出しの深さ 0 の地の文）と本文で返す。
# 棚卸の節か台帳の節が無ければ空
function Get-Themes([string[]]$Lines, [pscustomobject]$Markers) {
    $inv = Get-Section $Lines $Markers.secInventory
    $led = Get-Section $Lines $Markers.secLedger
    if ($null -eq $inv -or $null -eq $led) { return , [object[]]@() }
    $themes = [System.Collections.Generic.List[object]]::new()
    for ($i = $inv.End; $i -lt $led.Start; $i++) {
        if (-not $Lines[$i].StartsWith($SECTION_PREFIX)) { continue }
        $end = $i + 1
        while ($end -lt $led.Start -and -not $Lines[$end].StartsWith($SECTION_PREFIX)) { $end++ }
        $themes.Add([pscustomobject]@{
            Name = ConvertTo-Plain (Get-TopText $Lines[$i].Substring($SECTION_PREFIX.Length) $Markers)
            Text = $Lines[$i..($end - 1)] -join "`n"
        })
    }
    , [object[]]$themes
}

# ウェーブの節（markers.secWave）の最初の表の 1 欄目の spec 名と、節の最初の ``` の塊（囲みを外す・無ければ $null）。
# 節か表が無ければ $null（欠け wave-table は呼ぶ側が出す）。名前は見出しの日付（無ければ見出しの地の文）
function Get-Wave([string[]]$Lines, [pscustomobject]$Markers) {
    $sec = Get-Section $Lines $Markers.secWave
    if ($null -eq $sec) { return $null }
    $table = Get-Table $Lines ($sec.Start + 1) $sec.End
    if ($null -eq $table) { return $null }
    $warnings = [System.Collections.Generic.List[object]]::new()
    $names = foreach ($row in $table.Rows) {
        $c = Get-Cells $row $table.Width 'wave-table' $warnings
        Get-SpecName $c[0] $Markers
    }
    $commands = $null
    for ($i = $sec.Start + 1; $i -lt $sec.End; $i++) {
        if (-not $Lines[$i].StartsWith('```')) { continue }
        $body = [System.Collections.Generic.List[string]]::new()
        for ($i++; $i -lt $sec.End -and -not $Lines[$i].StartsWith('```'); $i++) { $body.Add($Lines[$i]) }
        $commands = $body -join "`n"
        break
    }
    $date = [regex]::Match($Lines[$sec.Start], $Markers.dateRegex)
    [pscustomobject]@{
        Name = $date.Success ? $date.Value : (ConvertTo-Plain $Lines[$sec.Start].TrimStart('#'))
        Names = [string[]]@($names | Where-Object { $_ }); Commands = $commands; Warnings = [object[]]$warnings
    }
}

# 完了の表（markers.secSummary の節の最初の表。Phase・主題・spec）を読む。節か表が無ければ $null。
# 返り値 @{ Rows = @{ phase; topic; specs; names（spec の欄の深さ 0 の名前＝完了） }; Warnings }
function Get-Summary([string[]]$Lines, [pscustomobject]$Markers) {
    $sec = Get-Section $Lines $Markers.secSummary
    if ($null -eq $sec) { return $null }
    $table = Get-Table $Lines ($sec.Start + 1) $sec.End
    if ($null -eq $table) { return $null }
    $warnings = [System.Collections.Generic.List[object]]::new()
    $rows = foreach ($row in $table.Rows) {
        $c = Get-Cells $row $table.Width 'summary-table' $warnings
        [pscustomobject]@{ phase = ConvertTo-Plain $c[0]; topic = ConvertTo-Plain $c[1]; specs = ConvertTo-Plain $c[2]; names = [string[]]@(Get-TextNames $c[2] $Markers) }
    }
    [pscustomobject]@{ Rows = [object[]]@($rows); Warnings = [object[]]$warnings }
}

# roadmap.md の全文（LF）から、HTML に埋めるデータ・警告・欠けを作る。ファイルにも git にも触らず、時刻・乱数・環境を読まない。
# $Source からは blob（中身の印・無ければ $null）だけを使う。
# 返り値 @{ Missing = 欠けの鍵（在れば Data は $null）; Warnings = @{ Key; Detail }; Data }。
# 並びはすべて roadmap.md に出る順で、1 要素でも配列として持つ
function Read-Roadmap([string]$Text, [pscustomobject]$Markers, [pscustomobject]$Source) {
    $lines = [string[]]$Text.Split("`n")
    $missing = [System.Collections.Generic.List[string]]::new()
    $inv = Get-Section $lines $Markers.secInventory
    $date = $null -ne $inv ? [regex]::Match($lines[$inv.Start], $Markers.dateRegex) : $null
    $updatedAt = $null -ne $date -and $date.Success ? $date.Value : $null
    if ($null -eq $updatedAt) { $missing.Add('updated-at') }
    $bands = Get-Bands $lines $Markers;     if ($null -eq $bands) { $missing.Add('version-table') }
    $ledger = Get-Ledger $lines $Markers;   if ($null -eq $ledger) { $missing.Add('ledger') }
    $wave = Get-Wave $lines $Markers;       if ($null -eq $wave) { $missing.Add('wave-table') }
    $summary = Get-Summary $lines $Markers; if ($null -eq $summary) { $missing.Add('summary-table') }
    $warnings = [System.Collections.Generic.List[object]]::new()
    foreach ($r in @($bands, $wave, $summary) | Where-Object { $_ }) { foreach ($w in $r.Warnings) { $warnings.Add($w) } }
    if ($missing.Count) { return [pscustomobject]@{ Missing = [string[]]$missing; Warnings = [object[]]$warnings; Data = $null } }

    # 引き当て。台帳は最初に出た行を使う
    $ledgerBy = @{}
    foreach ($r in $ledger) { if (-not $ledgerBy.ContainsKey($r.name)) { $ledgerBy[$r.name] = $r } }
    $summaryBy = @{}
    foreach ($r in $summary.Rows) { foreach ($n in $r.names) { if (-not $summaryBy.ContainsKey($n)) { $summaryBy[$n] = $r } } }
    $waveNames = [System.Collections.Generic.HashSet[string]]::new($wave.Names)
    $themeSecs = Get-Themes $lines $Markers

    # テーマ＝その spec の居場所の節の添字（無ければ -1）。居場所は、名前で始まる箇条（「- `名前`」）か、
    # 欄が名前だけ（括弧の注は付いてよい）の表の行が最初に出た節。どこにも無ければ、名前が最初に出た節。
    # ponytail: 文字列で探すだけ。居場所の無い spec は、別の spec の話の中に名前が出た節へ割り当たる
    function Find-Theme([string]$Name) {
        $n = [regex]::Escape($Name)
        $own = '(?m)^- `' + $n + '`|\|\s*`?' + $n + '`?(?:（[^|\n]*）)?\s*\|'
        $mention = '(?<![A-Za-z0-9_.-])' + $n + '(?![A-Za-z0-9_-])'
        foreach ($re in $own, $mention) {
            for ($t = 0; $t -lt $themeSecs.Count; $t++) { if ($themeSecs[$t].Text -cmatch $re) { return $t } }
        }
        -1
    }

    function New-Card([string]$Name, $Row) {
        $lr = $ledgerBy[$Name]
        $sr = $summaryBy[$Name]
        $done = ($null -ne $Row -and $Row.done) -or $null -ne $sr -or ($null -ne $lr -and $lr.done)
        [pscustomobject][ordered]@{
            name = $Name
            # 完了の表の主題は数本をまとめた文なので、一言には使わない
            oneLiner = $null -ne $lr ? (Get-OneLiner $lr.what $Markers) : ''
            theme = Find-Theme $Name
            order = ${Row}?.order; model = ${Row}?.model; size = ${Row}?.size; note = ${Row}?.note
            wave = $waveNames.Contains($Name) ? $wave.Name : $null
            done = $done; notInLedger = $null -eq $lr -and -not $done
            otherBands = [System.Collections.Generic.List[string]]::new()
            prereqs = [object[]]@()
            ledger = $null -ne $lr ? [pscustomobject][ordered]@{ what = $lr.what; prereq = $lr.prereq } : $null
        }
    }

    # 帯に置く。2 つ目以降に出た名前はカードにせず、先のカードの otherBands に帯の名前を足す
    $placed = [ordered]@{}   # 名前 -> @{ Band = 置いた帯の名前; Card }
    $outBands = [System.Collections.Generic.List[object]]::new()
    foreach ($b in $bands.Bands) {
        $cards = foreach ($k in $b.Cards) {
            if ($placed.Contains($k.name)) {
                $p = $placed[$k.name]
                $warnings.Add([pscustomobject]@{ Key = 'duplicate'; Detail = "$($k.name): bands $($p.Band), $($b.Name)" })
                if ($p.Band -cne $b.Name -and -not $p.Card.otherBands.Contains($b.Name)) { $p.Card.otherBands.Add($b.Name) }
                continue
            }
            $card = New-Card $k.name $k
            $placed[$k.name] = @{ Band = $b.Name; Card = $card }
            if ($card.notInLedger) { $warnings.Add([pscustomobject]@{ Key = 'not-in-ledger'; Detail = $k.name }) }
            $card
        }
        $outBands.Add([pscustomobject][ordered]@{ name = $b.Name; goal = $b.Goal; notInBand = $false; cards = [object[]]@($cards) })
    }

    # 台帳に在ってどの帯にも無い未完了の spec は、末尾の帯（名前と目標は空・台帳の順）へ。
    # 完了の印の付いた行は、完了の表に無ければ「まだ畳んでいない完了」へ
    $extra = [System.Collections.Generic.List[object]]::new()
    $rest = foreach ($r in $ledger) {
        if ($placed.Contains($r.name)) { continue }
        $card = New-Card $r.name $null
        $placed[$r.name] = @{ Band = ''; Card = $card }
        if (-not $r.done) { $card; continue }
        if ($summaryBy.ContainsKey($r.name)) { continue }
        $extra.Add([pscustomobject][ordered]@{ name = $r.name; oneLiner = $card.oneLiner; theme = $card.theme })
    }
    $rest = [object[]]@($rest)
    if ($rest.Count) { $outBands.Add([pscustomobject][ordered]@{ name = ''; goal = ''; notInBand = $true; cards = $rest }) }

    # 前提の居場所は、置いた帯の名前と完了か（どちらも無い＝roadmap に無い）
    foreach ($p in $placed.Values) {
        $card = $p.Card
        $lr = $ledgerBy[$card.name]
        $card.prereqs = [object[]]@(foreach ($n in @(if ($null -ne $lr) { $lr.prereqs })) {
            $q = $placed.Contains($n) ? $placed[$n] : $null
            [pscustomobject][ordered]@{
                name = $n; band = ${q}?.Band; done = $summaryBy.ContainsKey($n) -or ($null -ne $q -and $q.Card.done)
            }
        })
        $card.otherBands = [string[]]@($card.otherBands)
    }

    # テーマは見せるもの（帯のカードと、まだ畳んでいない完了）のあるものだけを、節の順に残し、添字を詰める
    # （テーマなしは末尾の添字＝テーマの数）
    $shown = @(foreach ($b in $outBands) { $b.cards }) + @($extra)
    $used = [System.Collections.Generic.SortedSet[int]]::new()
    foreach ($o in $shown) { if ($o.theme -ge 0) { [void]$used.Add($o.theme) } }
    $map = @{}
    $themes = foreach ($t in $used) { $map[$t] = $map.Count; $themeSecs[$t].Name }
    $themes = [string[]]@($themes)
    foreach ($o in @($placed.Values | ForEach-Object { $_.Card }) + @($extra)) {
        $o.theme = $o.theme -ge 0 -and $map.ContainsKey($o.theme) ? $map[$o.theme] : $themes.Count
    }

    $rows = foreach ($r in $summary.Rows) {
        [pscustomobject][ordered]@{ phase = $r.phase; topic = $r.topic; oneLiner = Get-OneLiner $r.topic $Markers; specs = $r.specs }
    }

    $data = [pscustomobject][ordered]@{
        source = [pscustomobject][ordered]@{ updatedAt = $updatedAt; blob = $Source.blob }
        themes = $themes
        bands = [object[]]$outBands
        waveCommands = $wave.Commands
        completed = [pscustomobject][ordered]@{ rows = [object[]]@($rows); extra = [object[]]$extra }
        warningCount = $warnings.Count
    }
    [pscustomobject]@{ Missing = [string[]]@(); Warnings = [object[]]$warnings; Data = $data }
}

if ($SelfTest) {
    $script:selfFails = 0
    function Judge([string]$Name, [bool]$Ok) {
        if ($Ok) { "ok selftest $Name" } else { $script:selfFails++; "FAIL selftest $Name" }
    }
    function Try-Judge([string]$Name, [scriptblock]$Body) { $ok = $false; try { $ok = [bool](& $Body) } catch { }; Judge $Name $ok }

    # ---- 1. 文言の鍵がそろう ----
    $strings = $null
    try { $strings = Read-Strings } catch { }
    # 鍵を 1 つ外した写しと 1 つを空にした写しが、その鍵だけで落ちる（判定の空振りを防ぐ）
    $caught = $false
    if ($null -ne $strings) {
        $cut = $strings | ConvertTo-Json -Depth 10 | ConvertFrom-Json
        $cut.markers.PSObject.Properties.Remove('secLedger')
        $empty = $strings | ConvertTo-Json -Depth 10 | ConvertFrom-Json
        $empty.labels.title = ''
        $caught = ((@(Test-Strings $cut) -join ',') -ceq 'markers.secLedger') -and
                  ((@(Test-Strings $empty) -join ',') -ceq 'labels.title')
    }
    Judge 'strings' $caught

    # ---- 2. ひな形の約束（差し込み口が 1 つ・文字コードの宣言・外の物を読む書き方が無い・ASCII だけ） ----
    $template = Read-Lf (Join-Path $PSScriptRoot $TEMPLATE_FILE)
    Try-Judge 'template' {
        $null -ne $template -and @(Test-Template $template).Count -eq 0 -and
        (@(Test-Template ($template + $PLACEHOLDER)) -join ',') -ceq 'placeholder' -and
        (@(Test-Template $template.Replace($PLACEHOLDER, '')) -join ',') -ceq 'placeholder' -and
        (@(Test-Template $template.Replace($CHARSET, '')) -join ',') -ceq 'charset' -and
        (@(Test-Template ($template + '<link rel="x">')) -join ',') -ceq 'external' -and
        (@(Test-Template ($template + '<script src="x.js"></script>')) -join ',') -ceq 'external' -and
        (@(Test-Template ($template + "/* $([char]0x3042) */")) -join ',') -ceq 'ascii'
    }

    # ---- 3〜6. 見本の読み取り（中身の印は固定の値） ----
    $fxDir = Join-Path $PSScriptRoot 'roadmap-overview-fixture'
    $fixture = Read-Lf (Join-Path $fxDir 'roadmap.md')
    $expected = Read-Lf (Join-Path $fxDir 'expected-data.json')
    $m = ${strings}?.markers
    $src = [pscustomobject]@{ blob = '0123456789ab' }
    function Read-Fixture([string]$Text) { Read-Roadmap $Text $m $src }
    $base = $null
    if ($null -ne $m -and $null -ne $fixture) { try { $base = Read-Fixture $fixture } catch { } }

    # 3. 見本から作ったデータの JSON（字下げつき・LF）が期待の値と 1 字違わず同じ
    Try-Judge 'fixture data equals the expected data' {
        $null -ne $expected -and (($base.Data | ConvertTo-Json -Depth 10).Replace("`r`n", "`n") + "`n") -ceq $expected
    }
    # 4. 見本は欠け 0・警告 0
    Try-Judge 'fixture has no missing and no warning' {
        $null -ne $base.Data -and @($base.Missing).Count -eq 0 -and @($base.Warnings).Count -eq 0 -and $base.Data.warningCount -eq 0
    }

    # 5. 見本を 1 か所ずつ壊した文が、決まった警告の鍵だけを出す（壊した文が見本と同じなら空振りなので落とす）。
    # 壊す所は 0.9.0 の帯の beta-two の行
    $broken = [ordered]@{}
    if ($null -ne $fixture) {
        $row = "`n| 2 | beta-two |"
        $broken['not-in-ledger'] = $fixture.Replace($row, "`n| 3 | zz-new | Opus | 1 | - |$row")
        $broken['duplicate']     = $fixture.Replace($row, "`n| 3 | gamma-three | Opus | 1 | - |$row")
        $broken['table-columns'] = $fixture.Replace($row, "`n| 3 | delta-free | Opus |$row")
    }
    foreach ($k in 'not-in-ledger', 'duplicate', 'table-columns') {
        Try-Judge "fixture broken for $k warns with $k only" {
            $t = $broken[$k]
            $r = Read-Fixture $t
            $t -cne $fixture -and @($r.Missing).Count -eq 0 -and $r.Data.warningCount -eq @($r.Warnings).Count -and
            (@($r.Warnings | ForEach-Object Key) -join ',') -ceq $k
        }
    }

    # 6. 見本から 1 か所ずつ外した文が、決まった欠けの鍵だけを出し、Data が無い。
    # 節の見出しの行を別の名前に替える（節が無い）か、見出しの直後に別の節を挟む（節は在るが表が無い）。
    # 帯は表の見出しの「順」を替える。外し方は合わせて 7（数が違えば判定は空振りなので落とす）
    function Edit-Heading([string]$Marker, [scriptblock]$To) {
        $lines = $fixture.Split("`n")
        for ($i = 0; $i -lt $lines.Count; $i++) {
            if ($lines[$i].StartsWith($Marker)) { $lines[$i] = & $To $lines[$i]; break }
        }
        $lines -join "`n"
    }
    $cut = [System.Collections.Generic.List[object]]::new()
    if ($null -ne $m -and $null -ne $fixture) {
        $cut.Add(@('updated-at', 'the date', (Edit-Heading $m.secInventory { param($h) $h -replace $m.dateRegex, 'someday' })))
        $cut.Add(@('version-table', 'the header', $fixture.Replace("| $($m.bandHeader[0]) | $($m.bandHeader[1]) |", '| x | spec |')))
        $cut.Add(@('ledger', 'the section', (Edit-Heading $m.secLedger { "${SECTION_PREFIX}gone" })))
        foreach ($k in @(@('wave-table', $m.secWave, '### '), @('summary-table', $m.secSummary, $SECTION_PREFIX))) {
            $cut.Add(@($k[0], 'the section', (Edit-Heading $k[1] { "$($k[2])gone" })))
            $cut.Add(@($k[0], 'the table', (Edit-Heading $k[1] { param($h) "$h`n`n$($k[2])gone" })))
        }
    }
    Judge 'fixture removals cover every missing key' ($cut.Count -eq 7)
    foreach ($c in $cut) {
        Try-Judge "fixture without $($c[1]) of $($c[0]) is missing $($c[0]) only" {
            $r = Read-Fixture $c[2]
            $c[2] -cne $fixture -and (@($r.Missing) -join ',') -ceq $c[0] -and $null -eq $r.Data
        }
    }

    # 7. 見本から 2 回作った HTML（labels は文言のファイルのもの）が 1 字違わず同じで、
    # 台帳の文の </script> が \u003c/script\u003e に逃がされている（HTML の </script> の数はひな形と同じ）
    Try-Judge 'fixture html is the same twice and escapes the script end tag' {
        $h1 = Format-Html $template (Read-Fixture $fixture).Data $strings.labels
        $h2 = Format-Html $template (Read-Fixture $fixture).Data $strings.labels
        $escEnd = '\' + 'u003c/script' + '\' + 'u003e'
        $ends = { param($t) ([regex]::Matches($t, '</script>')).Count }
        $h1 -ceq $h2 -and $h1.Contains($escEnd) -and -not $h1.Contains($PLACEHOLDER) -and
        (& $ends $h1) -eq (& $ends $template) -and -not $h1.Contains("`r")
    }

    if ($script:selfFails) { "selftest: $($script:selfFails) failure(s)"; exit 1 }
    'selftest: all passed'
    exit 0
}

# ---- 作る呼び方 ----
try { $strings = Read-Strings }
catch { "FAIL strings: $($_.Exception.Message -replace '[^\x20-\x7E]', '?')"; exit 1 }
function ConvertTo-Ascii([string]$Text) { $Text -replace '[^\x20-\x7E]', '?' }

$template = Read-Lf (Join-Path $PSScriptRoot $TEMPLATE_FILE)
if ($null -eq $template) { "FAIL template: cannot read $TEMPLATE_FILE"; exit 1 }
$bad = @(Test-Template $template)
if ($bad.Count) { "FAIL template: $TEMPLATE_FILE breaks: $($bad -join ', ')"; exit 1 }

# 読むのは根の roadmap.md だけ（今のフォルダによらない）
$roadmapRel = '.kiro/steering/roadmap.md'
$text = Read-Lf (Join-Path $root $roadmapRel)
if ($null -eq $text) { 'FAIL missing roadmap'; exit 1 }

# 中身の印＝git hash-object の先頭 12 字（ファイルの中身だけで決まる。git log --find-object で辿れる）
$blob = $null
try {
    $g = (& git -C $root hash-object -- $roadmapRel 2>$null) -join ''
    if ($LASTEXITCODE -eq 0 -and $g.Trim() -cmatch '^[0-9a-f]{40,64}$') { $blob = $g.Trim().Substring(0, 12) }
} catch { }

$r = Read-Roadmap $text $strings.markers ([pscustomobject]@{ blob = $blob })
if (@($r.Missing).Count) { foreach ($k in $r.Missing) { "FAIL missing $k" }; exit 1 }

$warnLines = @(foreach ($w in $r.Warnings) { ConvertTo-Ascii "WARN $($w.Key) $($w.Detail)" })
if ($null -eq $blob) { $warnLines += 'WARN git: blob unknown'; $r.Data.warningCount++ }
$warnLines

if ($OutFile) {
    $outPath = $ExecutionContext.SessionState.Path.GetUnresolvedProviderPathFromPSPath($OutFile)
    $outRel = ConvertTo-Ascii $outPath
} else {
    $outRel = "doc/roadmap-overview-$($r.Data.source.updatedAt).html"
    $outPath = Join-Path $root $outRel
}
try { Write-Utf8 $outPath (Format-Html $template $r.Data $strings.labels) }
catch { "FAIL write ${outRel}: $(ConvertTo-Ascii $_.Exception.GetBaseException().Message)"; exit 1 }

# 帯の数は帯の表の数、spec の数は置いたカードの数（「版が決まっていないもの」の帯を含む）
$bandN = @($r.Data.bands | Where-Object { -not $_.notInBand }).Count
$specN = 0
foreach ($b in $r.Data.bands) { $specN += @($b.cards).Count }
"OK wrote $outRel ($bandN band(s), $specN spec(s), $($r.Data.warningCount) warning(s))"
if ($Open) { Invoke-Item -LiteralPath $outPath }
exit 0
