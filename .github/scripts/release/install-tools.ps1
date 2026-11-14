#Requires -Version 7
<#
.SYNOPSIS
  版と SHA-256 を固定したビルド済みの道具（wasm-pack・cargo-about）を取り、版を確かめて GITHUB_PATH に足す（ランナーにはプリインストールされていない）。
.EXAMPLE
  pwsh -NoProfile -File .github/scripts/release/install-tools.ps1 -Name wasm-pack,cargo-about
#>
param(
    [Parameter(Mandatory)][string[]]$Name
)
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true

# 版を上げるときはここだけを直す（release.yml・build.yml の両方がこのスクリプトを呼ぶ）。
$WASM_PACK_VERSION = '0.15.0'
$CARGO_ABOUT_VERSION = '0.9.2'
$WASM_PACK_SHA256 = '518dc51180c7bc864699c9279b3bc99025bc109123e0249a34ba34d130a509bf'
$CARGO_ABOUT_SHA256 = '1c03e5890238562497c2d89a3b75b02560af349c1fc3e713d3284f532a5cd748'

$tools = [ordered]@{
    'wasm-pack'   = @{ Name = 'wasm-pack'; Version = $WASM_PACK_VERSION; Sha256 = $WASM_PACK_SHA256
        Url = "https://github.com/rustwasm/wasm-pack/releases/download/v$WASM_PACK_VERSION/wasm-pack-v$WASM_PACK_VERSION-x86_64-pc-windows-msvc.tar.gz" }
    'cargo-about' = @{ Name = 'cargo-about'; Version = $CARGO_ABOUT_VERSION; Sha256 = $CARGO_ABOUT_SHA256
        Url = "https://github.com/EmbarkStudios/cargo-about/releases/download/$CARGO_ABOUT_VERSION/cargo-about-$CARGO_ABOUT_VERSION-x86_64-pc-windows-msvc.tar.gz" }
}

try {
    # pwsh -File は `-Name a,b` を 1 つの文字列で渡すため、param の ValidateSet は使えない。分けてから受け付ける値と照合する。
    $names = $Name -split ','
    $bad = @($names | Where-Object { $tools.Keys -notcontains $_ })
    # @('') は偽になるので要素の数で見る（空の要素も弾く）
    if ($bad.Count) { throw "-Name に受け付けない値: $(($bad | ForEach-Object { "'$_'" }) -join ', ')（受け付けるのは $($tools.Keys -join ', ')）" }

    # 手元では RUNNER_TEMP が無いので OS の一時フォルダを使う。
    $base = if ($env:RUNNER_TEMP) { $env:RUNNER_TEMP } else { [IO.Path]::GetTempPath() }
    foreach ($t in $names | ForEach-Object { $tools[$_] }) {
        $dir = Join-Path $base "tools/$($t.Name)"
        New-Item -ItemType Directory -Force -Path $dir | Out-Null
        $archive = "$dir.tar.gz"
        Invoke-WebRequest -Uri $t.Url -OutFile $archive
        $hash = (Get-FileHash -Algorithm SHA256 -LiteralPath $archive).Hash
        if ($hash -ne $t.Sha256) { throw "$($t.Name) の SHA-256 が決めた値と違う: $hash" }
        & "$env:SystemRoot\System32\tar.exe" -xzf $archive -C $dir
        $exe = Get-ChildItem -LiteralPath $dir -Recurse -Filter "$($t.Name).exe" | Select-Object -First 1
        if (-not $exe) { throw "$($t.Name).exe が $($t.Url) の中に無い" }
        $reported = & $exe.FullName --version
        if ("$reported" -notmatch "\b$([regex]::Escape($t.Version))\b") { throw "$($t.Name) の版が $($t.Version) でない: $reported" }
        Write-Output "$($t.Name): $reported"
        # GITHUB_PATH が未設定なら足さずに場所を出す（手元での実行）。
        if ($env:GITHUB_PATH) { Add-Content -LiteralPath $env:GITHUB_PATH -Value $exe.DirectoryName }
        else { Write-Output "GITHUB_PATH が未設定のため PATH に足さない: $($exe.DirectoryName)" }
    }
}
catch {
    Write-Output "ERROR: $_"
    exit 1
}
