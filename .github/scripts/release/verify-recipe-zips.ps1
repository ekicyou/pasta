#Requires -Version 7
<#
.SYNOPSIS
  pasta-recipe の 2 つの zip（x64・arm64）を同じ項目で検査し、問題を全部示してから失敗する。
.DESCRIPTION
  zip ごとに、在る → 中身 2 つ → exe の版 → PE の Machine → 表示の tauri・monaco-editor → 30 MB 以下 →
  VC++ の再頒布可能パッケージの DLL を読まない、を見る。無い・中身違いはその zip の残りの項目を見ない。
  UCRT（ucrtbase.dll・api-ms-win-crt-*）は OS の部品なので数えない。読むだけで zip は変えない。
  リリース CI と main の CI が呼ぶ。手元でも呼べる。
.EXAMPLE
  pwsh -NoProfile -File .github/scripts/release/verify-recipe-zips.ps1 -Version 0.5.2
#>
param(
    [Parameter(Mandatory)][ValidatePattern('^\d+\.\d+\.\d+$')][string]$Version,
    [string]$Dir = (Join-Path $PSScriptRoot '../../../release'),
    # .ps1 でも .exe でも `& $DumpbinCommand /nologo /dependents <exe>` で呼ぶ（自己テストは偽の .ps1 を渡す）。
    [string]$DumpbinCommand
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$Machines = [ordered]@{ x64 = 0x8664; arm64 = 0xAA64 }
$Expected = 'pasta-recipe.exe|THIRD_PARTY_LICENSES.txt'
$SizeLimit = 31457280
$VcRuntime = '^(vcruntime|msvcp|vccorlib|concrt|msvcr\d)'
$errors = [System.Collections.Generic.List[string]]::new()

# 0x3C の e_lfanew → その位置に PE\0\0 → 次の 2 バイト（little endian）。PE でなければ $null。
function Get-PeMachine([string]$Path) {
    $b = [IO.File]::ReadAllBytes($Path)
    if ($b.Length -lt 0x40 -or $b[0] -ne 0x4D -or $b[1] -ne 0x5A) { return $null }
    $pe = [BitConverter]::ToInt32($b, 0x3C)
    if ($pe -lt 0 -or $pe + 6 -gt $b.Length) { return $null }
    if ($b[$pe] -ne 0x50 -or $b[$pe + 1] -ne 0x45 -or $b[$pe + 2] -ne 0 -or $b[$pe + 3] -ne 0) { return $null }
    [BitConverter]::ToUInt16($b, $pe + 4)
}

if (-not $DumpbinCommand) {
    $vswhere = if (${env:ProgramFiles(x86)}) { Join-Path ${env:ProgramFiles(x86)} 'Microsoft Visual Studio/Installer/vswhere.exe' }
    if ($vswhere -and (Test-Path -LiteralPath $vswhere)) {
        $DumpbinCommand = @(& $vswhere -latest -products * -find 'VC/Tools/MSVC/**/bin/Hostx64/x64/dumpbin.exe') | Select-Object -First 1
    }
    if (-not $DumpbinCommand) { $errors.Add('dumpbin が見つからない（vswhere で VC/Tools/MSVC の dumpbin.exe を探した）') }
}

$work = Join-Path ([IO.Path]::GetTempPath()) "verify-recipe-zips-$([Guid]::NewGuid().ToString('N'))"
try {
    foreach ($arch in $Machines.Keys) {
        $name = "pasta-recipe.$arch.zip"
        $zipPath = Join-Path $Dir $name
        if (-not (Test-Path -LiteralPath $zipPath -PathType Leaf)) { $errors.Add("配布物が無い: $name"); continue }

        $zip = [IO.Compression.ZipFile]::OpenRead((Resolve-Path -LiteralPath $zipPath).Path)
        try { $entries = @($zip.Entries | ForEach-Object FullName | Sort-Object) } finally { $zip.Dispose() }
        if (($entries -join '|') -ne $Expected) {
            $errors.Add("$name の中身が pasta-recipe.exe・THIRD_PARTY_LICENSES.txt の 2 つでない: $($entries -join ', ')")
            continue
        }

        $out = Join-Path $work $arch
        Expand-Archive -LiteralPath $zipPath -DestinationPath $out -Force
        $exe = Join-Path $out 'pasta-recipe.exe'

        $vi = (Get-Item -LiteralPath $exe).VersionInfo
        $exeVersion = "$($vi.FileMajorPart).$($vi.FileMinorPart).$($vi.FileBuildPart)"
        if ($exeVersion -cne $Version) { $errors.Add("$name の pasta-recipe.exe の版が $Version でない: $exeVersion") }

        $machine = Get-PeMachine $exe
        $want = $Machines[$arch]
        if ($null -eq $machine) { $errors.Add("$name の pasta-recipe.exe が PE でない") }
        elseif ($machine -ne $want) { $errors.Add(('{0} の pasta-recipe.exe の機械の種類が 0x{1:X4}（{2}）でない: 0x{3:X4}' -f $name, $want, $arch, $machine)) }

        $notice = Get-Content -LiteralPath (Join-Path $out 'THIRD_PARTY_LICENSES.txt') -Raw
        foreach ($n in 'tauri', 'monaco-editor') {
            if (-not $notice.Contains($n)) { $errors.Add("$name の THIRD_PARTY_LICENSES.txt に $n の名前が無い") }
        }

        $size = (Get-Item -LiteralPath $zipPath).Length
        if ($size -gt $SizeLimit) { $errors.Add("$name が 30 MB（31,457,280 バイト）を超える: $size バイト") }

        if ($DumpbinCommand) {
            try {
                # 偽の .ps1 は exit を書かないことがあるので、先に 0 にしておく（StrictMode で未設定を読まない）。
                $global:LASTEXITCODE = 0
                $lines = @(& $DumpbinCommand /nologo /dependents $exe 2>&1 | ForEach-Object { "$_".Trim() })
                if ($global:LASTEXITCODE) { throw "終了コード $global:LASTEXITCODE" }
                foreach ($dll in @($lines | Where-Object { $_ -match '\.dll$' -and $_ -match $VcRuntime })) {
                    $errors.Add("$name の pasta-recipe.exe が VC++ ランタイムの DLL を読む: $dll")
                }
            }
            catch { $errors.Add("$name の pasta-recipe.exe の DLL を dumpbin で読めない（$DumpbinCommand）: $_") }
        }
    }
}
finally {
    if (Test-Path -LiteralPath $work) { Remove-Item -LiteralPath $work -Recurse -Force }
}

if ($errors.Count -gt 0) {
    $errors | ForEach-Object { Write-Output "::error::$_" }
    exit 1
}
Write-Output "pasta-recipe の 2 つの zip の検査: 成功（版 $Version）"
