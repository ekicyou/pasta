# Static check script to verify no global state in PastaEngine
# Ensures that the pasta-engine-independence specification is maintained

Write-Host "Checking for global state in pasta crate..." -ForegroundColor Cyan

$hasErrors = $false

# Check for static variables (excluding allowed constants)
Write-Host "`nChecking for static variables..." -ForegroundColor Yellow
$staticMut = git grep -n "static\s\+mut" -- "src/**/*.rs" 2>$null
if ($staticMut) {
    Write-Host "ERROR: Found 'static mut' declarations:" -ForegroundColor Red
    Write-Host $staticMut
    $hasErrors = $true
}

$onceLock = git grep -n "OnceLock" -- "src/**/*.rs" 2>$null
if ($onceLock) {
    Write-Host "ERROR: Found 'OnceLock' usage:" -ForegroundColor Red
    Write-Host $onceLock
    $hasErrors = $true
}

$lazyLock = git grep -n "LazyLock" -- "src/**/*.rs" 2>$null
if ($lazyLock) {
    Write-Host "ERROR: Found 'LazyLock' usage:" -ForegroundColor Red
    Write-Host $lazyLock
    $hasErrors = $true
}

# Check for PARSE_CACHE global
$parseCache = git grep -n "PARSE_CACHE" -- "src/**/*.rs" 2>$null
if ($parseCache) {
    Write-Host "ERROR: Found 'PARSE_CACHE' global:" -ForegroundColor Red
    Write-Host $parseCache
    $hasErrors = $true
}

# Check for global_cache function
$globalCache = git grep -n "global_cache" -- "src/**/*.rs" 2>$null
if ($globalCache) {
    Write-Host "ERROR: Found 'global_cache' function:" -ForegroundColor Red
    Write-Host $globalCache
    $hasErrors = $true
}

if (-not $hasErrors) {
    Write-Host "`n✓ No global state found - pasta-engine-independence maintained!" -ForegroundColor Green
    exit 0
} else {
    Write-Host "`n✗ Global state violations detected - please fix before merging" -ForegroundColor Red
    exit 1
}
