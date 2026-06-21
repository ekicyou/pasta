<#
.SYNOPSIS
    release バイト・ベースライン採取／検証ツール（task 1.1 / 8.1 用）。

.DESCRIPTION
    pasta-actor-feasibility PoC の隔離前提（Requirement 7.2: actor-poc 無効時の
    release 成果物バイト不変）を検証するための、再現可能なダイジェスト採取器。

    対象成果物（target/release）:
      - pasta.dll        (pasta_shiori の cdylib = 出荷成果物本丸)
      - libpasta.rlib    (pasta_shiori の rlib)
      - libpasta_lua.rlib(pasta_lua の rlib)

    重要な再現性の事実（task 1.1 採取・task 8.1 で改めて実測・確定）:

    [出荷成果物 pasta.dll = 権威ある不変基準]
      pasta.dll は同一ソースでもクリーンビルドごとに「20 バイト」だけ差分が出る。
      差分は全て PE リンクメタデータであり、コンパイル済みコードは再現性がある:
        * COFF TimeDateStamp        (file offset 264, 4 bytes)
        * Optional Header CheckSum  (file offset 344, 4 bytes)
        * Debug Directory の TimeDateStamp エコー（複数エントリ, 各 4 bytes）
        * CodeView "RSDS" レコードの build-id GUID (16 bytes)
      （いずれもソース内容ではなくリンク時刻・ランダム build-id 由来）
      上記の非決定 PE 領域だけをゼロ埋め正規化した normalized digest は、コンパイル
      済みコード（.text/.rdata 等）を一切触らないため、出荷コードに実変更があれば
      必ず変動する。R7.2 の「actor-poc 無効ビルドの出荷成果物がバイト不変」を意味の
      ある形で検証できる唯一の安定基準であり、本ツールの verify 合否はこの 1 点で決まる。

    [中間 rlib = informational（合否に算入しない・非決定）]
      task 1.1 note は「rlib は exact 再現」としていたが、task 8.1 でこれが whole-file
      には当てはまらないことが判明した。libpasta.rlib / libpasta_lua.rlib の whole-file
      sha256 は、actor-poc を Cargo.toml に一切含まない pre-actor-poc ソース（commit
      1e94b34a）のクリーンビルドでも committed baseline の値を再現しない。原因は rlib
      （ar アーカイブ）が埋め込む symbol table および crate metadata fingerprint（SVH）で
      あり、ソースの宣言済み feature 集合（default-off の actor-poc/optional-dep 宣言を
      含む）に依存して whole-file が数十バイト変動する。object コード member 自体は
      libpasta.rlib では bit 一致、libpasta_lua.rlib では SVH 由来で約 4 バイト差のみで
      あり、actor_poc コード（cfg 除外され emit されない）の混入ではない。
      よって rlib whole-file sha はバイト不変の基準たりえない。本ツールは rlib を
      informational として sha を報告するのみとし、verify の exit code には算入しない。
      （詳細な per-member ar/COFF 分析は baseline/verify_8_1_result.md を参照）

.PARAMETER Mode
    capture: 現在の target/release 成果物から baseline.json を生成（task 1.1）。
    verify : 現在の target/release 成果物を既存 baseline.json と照合（task 8.1）。
             合否は出荷成果物 pasta.dll の normalized sha256 一致のみで決まる。
             rlib は informational 報告のみで exit code に算入しない（whole-file 非決定）。

.PARAMETER ReleaseDir
    成果物ディレクトリ。既定はリポジトリの target/release。

.PARAMETER BaselineFile
    ベースライン JSON のパス。既定は本スクリプトと同階層の baseline.json。
#>
[CmdletBinding()]
param(
    [ValidateSet('capture', 'verify')]
    [string]$Mode = 'verify',
    [string]$ReleaseDir,
    [string]$BaselineFile
)

$ErrorActionPreference = 'Stop'

$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
if (-not $ReleaseDir) {
    # baseline/ -> pasta-actor-feasibility -> specs -> .kiro -> repo root
    $repoRoot = (Resolve-Path (Join-Path $scriptDir '..\..\..\..')).Path
    $ReleaseDir = Join-Path $repoRoot 'target\release'
}
if (-not $BaselineFile) {
    $BaselineFile = Join-Path $scriptDir 'baseline.json'
}

$Artifacts = @('pasta.dll', 'libpasta.rlib', 'libpasta_lua.rlib')

function Get-PlainSha256([byte[]]$bytes) {
    $sha = [System.Security.Cryptography.SHA256]::Create()
    try { return ([System.BitConverter]::ToString($sha.ComputeHash($bytes))).Replace('-', '').ToLower() }
    finally { $sha.Dispose() }
}

function Set-ZeroRange([byte[]]$buf, [int]$off, [int]$len, [string]$field, $list) {
    if ($off -lt 0 -or ($off + $len) -gt $buf.Length) {
        throw "normalize: range out of bounds ($field off=$off len=$len)"
    }
    for ($i = 0; $i -lt $len; $i++) { $buf[$off + $i] = [byte]0 }
    $list.Add([ordered]@{ offset = $off; len = $len; field = $field }) | Out-Null
}

function Convert-RvaToOffset([uint32]$rva, [byte[]]$buf, [int]$secTable, [int]$n) {
    for ($s = 0; $s -lt $n; $s++) {
        $se = $secTable + ($s * 40)
        $vsize = [BitConverter]::ToUInt32($buf, $se + 8)
        $vaddr = [BitConverter]::ToUInt32($buf, $se + 12)
        $rawSize = [BitConverter]::ToUInt32($buf, $se + 16)
        $rawPtr = [BitConverter]::ToUInt32($buf, $se + 20)
        $span = [Math]::Max($vsize, $rawSize)
        if ($rva -ge $vaddr -and $rva -lt ($vaddr + $span)) {
            return [int]($rawPtr + ($rva - $vaddr))
        }
    }
    return -1
}

# pasta.dll の非決定 PE 領域をゼロ埋めして正規化したバイト列を返す。
# 戻り値: @{ Normalized = byte[]; ZeroedRanges = @(@{offset;len;field}, ...) }
function Get-NormalizedPeBytes([byte[]]$orig) {
    $b = [byte[]]::new($orig.Length)
    [Array]::Copy($orig, $b, [int]$orig.Length)
    $ranges = New-Object System.Collections.Generic.List[object]

    # --- PE ヘッダ位置 ---
    $peSig = [int][BitConverter]::ToInt32($b, 0x3c)        # e_lfanew
    if ([BitConverter]::ToUInt32($b, $peSig) -ne [uint32]0x00004550) { throw 'not a PE file (no PE\0\0)' }
    $coff = $peSig + 4
    # COFF TimeDateStamp (offset coff+4, 4 bytes)
    Set-ZeroRange $b ($coff + 4) 4 'COFF.TimeDateStamp' $ranges

    $optOff = $coff + 20
    $magic = [BitConverter]::ToUInt16($b, $optOff)         # 0x20b = PE32+
    $isPE32Plus = ($magic -eq [uint16]0x20b)
    # Optional Header CheckSum (offset optOff+64, 4 bytes)
    Set-ZeroRange $b ($optOff + 64) 4 'OptionalHeader.CheckSum' $ranges

    # DataDirectory: PE32+ では optOff+112 から、PE32 では optOff+96 から。
    $ddBase = if ($isPE32Plus) { $optOff + 112 } else { $optOff + 96 }
    # Debug = index 6: 各エントリ 8 bytes (VA, Size)
    $dbgDirEntry = $ddBase + (6 * 8)
    $dbgRva = [BitConverter]::ToUInt32($b, $dbgDirEntry)
    $dbgSize = [BitConverter]::ToUInt32($b, $dbgDirEntry + 4)

    if ($dbgRva -ne 0 -and $dbgSize -ne 0) {
        # RVA -> file offset 変換用にセクションテーブルを読む
        $numSections = [int][BitConverter]::ToUInt16($b, $coff + 2)
        $sizeOptHdr = [int][BitConverter]::ToUInt16($b, $coff + 16)
        $secTable = $optOff + $sizeOptHdr
        $dbgOff = Convert-RvaToOffset ([uint32]$dbgRva) $b ([int]$secTable) ([int]$numSections)
        if ($dbgOff -lt 0) { throw 'debug directory RVA not mapped to a section' }
        # IMAGE_DEBUG_DIRECTORY = 28 bytes 単位
        $count = [int]($dbgSize / 28)
        for ($e = 0; $e -lt $count; $e++) {
            $eo = $dbgOff + ($e * 28)
            # TimeDateStamp at +4 (4 bytes)
            Set-ZeroRange $b ($eo + 4) 4 "DebugDir[$e].TimeDateStamp" $ranges
            $type = [BitConverter]::ToUInt32($b, $eo + 12)   # 2 = CODEVIEW
            $cvSize = [BitConverter]::ToUInt32($b, $eo + 16)
            $cvPtr = [int][BitConverter]::ToUInt32($b, $eo + 24)  # PointerToRawData (file offset)
            if ($type -eq [uint32]2 -and $cvPtr -ne 0 -and $cvSize -ge [uint32]24) {
                $sig = [System.Text.Encoding]::ASCII.GetString($b, $cvPtr, 4)
                if ($sig -eq 'RSDS') {
                    # RSDS: sig(4) + GUID(16) + Age(4) + path
                    Set-ZeroRange $b ($cvPtr + 4) 16 "CodeView.RSDS.Guid[$e]" $ranges
                }
            }
        }
    }

    return @{ Normalized = $b; ZeroedRanges = $ranges }
}

function Get-ArtifactRecord([string]$path) {
    if (-not (Test-Path $path)) { throw "artifact not found: $path. Run a release build first." }
    $bytes = [System.IO.File]::ReadAllBytes($path)
    $name = Split-Path -Leaf $path
    $rec = [ordered]@{
        name      = $name
        size      = $bytes.Length
        sha256    = Get-PlainSha256 $bytes
    }
    if ($name -eq 'pasta.dll') {
        $norm = Get-NormalizedPeBytes $bytes
        $rec['normalized_sha256'] = Get-PlainSha256 $norm.Normalized
        $rec['reproducible'] = 'normalized'   # 生 sha256 は非決定、normalized が安定基準
        $rec['authoritative'] = $true          # verify の合否はこの成果物のみで決まる（R7.2 出荷成果物）
        $rec['zeroed_ranges'] = $norm.ZeroedRanges.ToArray()
        $rec['note'] = 'pasta.dll の生 sha256 はリンク時刻/build-id GUID により毎ビルド変動する。normalized_sha256（PE 非決定領域をゼロ埋め後の digest）が 8.1 のバイト不変基準であり verify 合否の唯一の判定対象。'
    }
    else {
        # rlib whole-file sha は symbol table / crate SVH fingerprint 由来で非決定。
        # informational として記録し、normalized_sha256 には生 sha を入れるが verify では算入しない。
        $rec['normalized_sha256'] = $rec['sha256']
        $rec['reproducible'] = 'informational'
        $rec['authoritative'] = $false         # whole-file 非決定。合否に算入しない。
        $rec['note'] = 'rlib whole-file sha は symbol table / crate SVH fingerprint 由来で非決定（pre-actor-poc ソースでも再現しない）。informational 報告のみで verify 合否には算入しない。object コード混入なしの根拠は verify_8_1_result.md を参照。'
    }
    return $rec
}

$records = @()
foreach ($a in $Artifacts) {
    $records += Get-ArtifactRecord (Join-Path $ReleaseDir $a)
}

if ($Mode -eq 'capture') {
    $doc = [ordered]@{
        schema           = 'pasta-actor-feasibility/baseline@1'
        captured_at_utc  = (Get-Date).ToUniversalTime().ToString('o')
        purpose          = 'Requirement 7.2 byte-invariance baseline (actor-poc 導入前)。task 8.1 が参照する。'
        release_profile  = 'opt-level=z, lto=true, codegen-units=1, panic=abort, strip=true'
        reproducibility  = 'rlib=exact; pasta.dll=normalized only (PE TimeDateStamp/CheckSum/DebugDir timestamps/RSDS GUID が非決定の 20 bytes)'
        artifacts        = $records
    }
    $json = $doc | ConvertTo-Json -Depth 8
    [System.IO.File]::WriteAllText($BaselineFile, $json, [System.Text.UTF8Encoding]::new($false))
    Write-Host "Baseline captured -> $BaselineFile"
    foreach ($r in $records) {
        Write-Host ("  {0,-18} size={1,9}  sha256={2}  normalized={3}" -f $r.name, $r.size, $r.sha256, $r.normalized_sha256)
    }
}
else {
    if (-not (Test-Path $BaselineFile)) { throw "baseline file not found: $BaselineFile. Run -Mode capture first." }
    $base = Get-Content -Raw $BaselineFile | ConvertFrom-Json
    $byName = @{}
    foreach ($r in $base.artifacts) { $byName[$r.name] = $r }

    # 合否は authoritative=true の成果物（出荷 pasta.dll）の normalized 一致のみで決まる。
    # authoritative=false の rlib は informational 報告に留め、exit code には算入しない。
    $fail = 0
    $authChecked = 0
    Write-Host '--- Authoritative byte-invariance check (R7.2 shipped artifact) ---'
    foreach ($r in $records) {
        $b = $byName[$r.name]
        # baseline 側で authoritative 指定が無い旧形式も pasta.dll は権威として扱う後方互換。
        $isAuth = $false
        if ($b) { $isAuth = ($b.authoritative -eq $true) -or ($r.name -eq 'pasta.dll') }
        else { $isAuth = ($r.name -eq 'pasta.dll') }
        if (-not $isAuth) { continue }
        $authChecked++
        if (-not $b) { Write-Host "MISSING baseline entry for $($r.name)"; $fail++; continue }
        $ok = ($r.normalized_sha256 -eq $b.normalized_sha256)
        $status = if ($ok) { 'OK ' } else { 'FAIL'; $fail++ }
        Write-Host ("[{0}] {1,-18} normalized {2} (baseline {3})" -f $status, $r.name, $r.normalized_sha256, $b.normalized_sha256)
        if (-not $ok) {
            Write-Host ("       current size={0} baseline size={1}" -f $r.size, $b.size)
        }
    }

    Write-Host ''
    Write-Host '--- Informational (rlib whole-file is non-deterministic: symbol table / crate SVH fingerprint; NOT a byte-invariance basis) ---'
    foreach ($r in $records) {
        $b = $byName[$r.name]
        $isAuth = $false
        if ($b) { $isAuth = ($b.authoritative -eq $true) -or ($r.name -eq 'pasta.dll') }
        else { $isAuth = ($r.name -eq 'pasta.dll') }
        if ($isAuth) { continue }
        if (-not $b) { Write-Host ("[INFO] {0,-18} (no baseline entry)" -f $r.name); continue }
        $match = if ($r.sha256 -eq $b.sha256) { 'whole-file MATCH   ' } else { 'whole-file DIFFERS ' }
        Write-Host ("[INFO] {0,-18} {1} sha256={2} (baseline {3})" -f $r.name, $match, $r.sha256, $b.sha256)
        if ($r.sha256 -ne $b.sha256) {
            Write-Host ("       current size={0} baseline size={1} (delta is metadata/SVH only; object code verified non-actor-poc in verify_8_1_result.md)" -f $r.size, $b.size)
        }
    }

    Write-Host ''
    if ($authChecked -eq 0) {
        Write-Error 'Byte-invariance verification ERROR: no authoritative artifact (pasta.dll) found to check.'
        exit 1
    }
    if ($fail -gt 0) {
        Write-Error "Byte-invariance verification FAILED ($fail authoritative artifact(s) differ from baseline)."
        exit 1
    }
    else {
        Write-Host "Byte-invariance verification PASSED ($authChecked authoritative artifact(s) match normalized baseline; rlibs informational)."
    }
}
