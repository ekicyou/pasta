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

    重要な再現性の事実（task 1.1 で実測・確定）:
      rlib 2 種は同一ソースのクリーンビルド間で完全にバイト再現する。
      一方 pasta.dll は同一ソースでもクリーンビルドごとに「20 バイト」だけ差分が出る。
      差分は全て PE リンクメタデータであり、コンパイル済みコードは再現性がある:
        * COFF TimeDateStamp        (file offset 264, 4 bytes)
        * Optional Header CheckSum  (file offset 344, 4 bytes)
        * Debug Directory の TimeDateStamp エコー（複数エントリ, 各 4 bytes）
        * CodeView "RSDS" レコードの build-id GUID (16 bytes)
      （いずれもソース内容ではなくリンク時刻・ランダム build-id 由来）

    したがって生 sha256 を 8.1 の比較基準にすると、actor-poc コードと無関係な
    リンクメタ揺らぎで「偽の差分」が出てしまう。本ツールは上記の非決定 PE 領域を
    ゼロ埋め正規化した上で sha256 を採る normalized digest を基準とし、それにより
    8.1 の「feature off ビルドがベースラインとバイト一致」を意味のある形で検証する。

.PARAMETER Mode
    capture: 現在の target/release 成果物から baseline.json を生成（task 1.1）。
    verify : 現在の target/release 成果物を既存 baseline.json と照合（task 8.1）。

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
        $rec['zeroed_ranges'] = $norm.ZeroedRanges.ToArray()
        $rec['note'] = 'pasta.dll の生 sha256 はリンク時刻/build-id GUID により毎ビルド変動する。normalized_sha256（PE 非決定領域をゼロ埋め後の digest）が 8.1 のバイト不変基準。'
    }
    else {
        $rec['normalized_sha256'] = $rec['sha256']
        $rec['reproducible'] = 'exact'
        $rec['note'] = 'rlib はクリーンビルド間で完全にバイト再現する。生 sha256 が基準。'
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

    $fail = 0
    foreach ($r in $records) {
        $b = $byName[$r.name]
        if (-not $b) { Write-Host "MISSING baseline entry for $($r.name)"; $fail++; continue }
        $ok = ($r.normalized_sha256 -eq $b.normalized_sha256)
        $status = if ($ok) { 'OK ' } else { 'FAIL'; $fail++ }
        Write-Host ("[{0}] {1,-18} normalized {2} (baseline {3})" -f $status, $r.name, $r.normalized_sha256, $b.normalized_sha256)
        if (-not $ok) {
            Write-Host ("       current size={0} baseline size={1}" -f $r.size, $b.size)
        }
    }
    if ($fail -gt 0) {
        Write-Error "Byte-invariance verification FAILED ($fail artifact(s) differ from baseline)."
        exit 1
    }
    else {
        Write-Host 'Byte-invariance verification PASSED (all artifacts match normalized baseline).'
    }
}
