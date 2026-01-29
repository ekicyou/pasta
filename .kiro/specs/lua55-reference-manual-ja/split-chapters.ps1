# Lua 5.5 Reference Manual Chapter Splitter
# Usage: .\split-chapters.ps1

$ErrorActionPreference = "Stop"
$BaseDir = $PSScriptRoot

# Configuration
$EnglishSource = Join-Path $BaseDir "reference-lua55-en.html"
$JapaneseSource = Join-Path $BaseDir "reference-lua54-ja.html"
$ChaptersDir = Join-Path $BaseDir "chapters"
$EnDir = Join-Path $ChaptersDir "en"
$JaDir = Join-Path $ChaptersDir "ja"

# Chapter definitions based on chapter-structure-map.md
$ChapterDefs = @(
    @{ Number = "01"; Title = "introduction"; H1Pattern = '<h1>1\s'; H1PatternJa = '<h1>1\s' },
    @{ Number = "02"; Title = "basic-concepts"; H1Pattern = '<h1>2\s'; H1PatternJa = '<h1>2\s' },
    @{ Number = "03"; Title = "language"; H1Pattern = '<h1>3\s'; H1PatternJa = '<h1>3\s'; SubSplit = $true },
    @{ Number = "04"; Title = "c-api"; H1Pattern = '<h1>4\s'; H1PatternJa = '<h1>4\s'; SubSplit = $true },
    @{ Number = "05"; Title = "auxiliary-library"; H1Pattern = '<h1>5\s'; H1PatternJa = '<h1>5\s' },
    @{ Number = "06"; Title = "standard-libraries"; H1Pattern = '<h1>6\s'; H1PatternJa = '<h1>6\s'; SubSplit = $true },
    @{ Number = "07"; Title = "standalone"; H1Pattern = '<h1>7\s'; H1PatternJa = '<h1>7\s' },
    @{ Number = "08"; Title = "incompatibilities"; H1Pattern = '<h1>8\s'; H1PatternJa = '<h1>8\s' },
    @{ Number = "09"; Title = "complete-syntax"; H1Pattern = '<h1>9\s'; H1PatternJa = '<h1>9\s' },
    @{ Number = "index"; Title = "index"; H1Pattern = '(?i)<h1[^>]*>\s*Index'; H1PatternJa = '(?i)<h1[^>]*>\s*Index' }
)

function Split-HtmlByH1 {
    param(
        [string]$HtmlContent,
        [string]$OutputDir,
        [array]$ChapterDefs,
        [string]$Lang
    )
    
    # Split by h1 tags (case insensitive)
    $h1Pattern = '(?i)(<h1[^>]*>)'
    $parts = [regex]::Split($HtmlContent, $h1Pattern)
    
    Write-Host "Found $([int](($parts.Count - 1) / 2)) h1 sections in $Lang"
    
    # Combine h1 tag with its content
    $chapters = @()
    $header = $parts[0]  # Content before first h1
    
    for ($i = 1; $i -lt $parts.Count; $i += 2) {
        if ($i + 1 -lt $parts.Count) {
            $chapters += @{
                Tag     = $parts[$i]
                Content = $parts[$i] + $parts[$i + 1]
            }
        }
    }
    
    Write-Host "Processing $($chapters.Count) chapters for $Lang..."
    
    # Match and save each chapter
    foreach ($def in $ChapterDefs) {
        $pattern = if ($Lang -eq "en") { $def.H1Pattern } else { $def.H1PatternJa }
        
        foreach ($ch in $chapters) {
            if ($ch.Content -match $pattern) {
                $filename = "$($def.Number)-$($def.Title).html"
                $filepath = Join-Path $OutputDir $filename
                
                # Wrap with basic HTML structure
                $wrappedContent = @"
<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>Lua 5.5 Reference Manual - Chapter $($def.Number)</title>
</head>
<body>
$($ch.Content)
</body>
</html>
"@
                Set-Content -Path $filepath -Value $wrappedContent -Encoding UTF8
                $sizeKB = [math]::Round((Get-Item $filepath).Length / 1KB, 1)
                Write-Host "  [$Lang] Saved: $filename ($sizeKB KB)"
                break
            }
        }
    }
}

function Split-HtmlByH2 {
    param(
        [string]$ChapterFile,
        [string]$OutputDir,
        [string]$Lang
    )
    
    $content = Get-Content $ChapterFile -Raw -Encoding UTF8
    
    # Extract body content
    if ($content -match '(?s)<body[^>]*>(.*)</body>') {
        $bodyContent = $Matches[1]
    }
    else {
        $bodyContent = $content
    }
    
    # Split by h2 tags
    $h2Pattern = '(?i)(<h2[^>]*>)'
    $parts = [regex]::Split($bodyContent, $h2Pattern)
    
    # First part is chapter header (before any h2)
    $chapterHeader = $parts[0]
    
    $sections = @()
    for ($i = 1; $i -lt $parts.Count; $i += 2) {
        if ($i + 1 -lt $parts.Count) {
            $sectionContent = $parts[$i] + $parts[$i + 1]
            
            # Extract section number and title from h2 content
            # Format: <h2>3.1 &ndash; <a name="3.1">Lexical Conventions</a></h2>
            if ($sectionContent -match '(?i)<h2[^>]*>(\d+\.\d+)\s*[^<]*<a[^>]*>([^<]+)</a></h2>') {
                $sectionNum = $Matches[1]
                $sectionTitle = $Matches[2].Trim()
                $sections += @{
                    Number  = $sectionNum
                    Title   = $sectionTitle
                    Content = $sectionContent
                }
            }
        }
    }
    
    Write-Host "  Found $($sections.Count) sections in $(Split-Path $ChapterFile -Leaf)"
    
    # Create output directory
    if (-not (Test-Path $OutputDir)) {
        New-Item -ItemType Directory -Path $OutputDir -Force | Out-Null
    }
    
    # Save each section
    $counter = 1
    foreach ($sec in $sections) {
        # Generate filename from section number
        # For Japanese, use section number as slug since Japanese chars don't work in filenames
        $secNumFormatted = $sec.Number -replace '\.', '-'
        
        if ($Lang -eq "ja") {
            # Use section number for Japanese
            $filename = "{0:D2}-section-{1}.html" -f $counter, $secNumFormatted
        }
        else {
            # Create slug from title for English
            $titleSlug = $sec.Title.ToLower() -replace '\s+', '-' -replace '[^a-z0-9\-]', ''
            if ($titleSlug.Length -gt 40) { $titleSlug = $titleSlug.Substring(0, 40) }
            if ($titleSlug -eq '') { $titleSlug = "section-$secNumFormatted" }
            $filename = "{0:D2}-{1}.html" -f $counter, $titleSlug
        }
        
        $filepath = Join-Path $OutputDir $filename
        
        $wrappedContent = @"
<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>Lua 5.5 - Section $($sec.Number)</title>
</head>
<body>
$($sec.Content)
</body>
</html>
"@
        Set-Content -Path $filepath -Value $wrappedContent -Encoding UTF8
        $sizeKB = [math]::Round((Get-Item $filepath).Length / 1KB, 1)
        Write-Host "    Saved: $filename ($sizeKB KB)"
        $counter++
    }
    
    return $sections.Count
}

# Main execution
Write-Host "=== Lua 5.5 Reference Manual Chapter Splitter ==="
Write-Host ""

# Create directories
Write-Host "Creating output directories..."
New-Item -ItemType Directory -Path $EnDir -Force | Out-Null
New-Item -ItemType Directory -Path $JaDir -Force | Out-Null

# Load source files
Write-Host "Loading source files..."
$enContent = Get-Content $EnglishSource -Raw -Encoding UTF8
$jaContent = Get-Content $JapaneseSource -Raw -Encoding UTF8
Write-Host "  English: $([math]::Round($enContent.Length / 1KB, 1)) KB"
Write-Host "  Japanese: $([math]::Round($jaContent.Length / 1KB, 1)) KB"
Write-Host ""

# Split main chapters
Write-Host "=== Phase 1: Main Chapter Split ==="
Split-HtmlByH1 -HtmlContent $enContent -OutputDir $EnDir -ChapterDefs $ChapterDefs -Lang "en"
Write-Host ""
Split-HtmlByH1 -HtmlContent $jaContent -OutputDir $JaDir -ChapterDefs $ChapterDefs -Lang "ja"
Write-Host ""

# Sub-split large chapters
Write-Host "=== Phase 2: Sub-chapter Split ==="
$subSplitChapters = $ChapterDefs | Where-Object { $_.SubSplit -eq $true }

foreach ($ch in $subSplitChapters) {
    $enChapterFile = Join-Path $EnDir "$($ch.Number)-$($ch.Title).html"
    $jaChapterFile = Join-Path $JaDir "$($ch.Number)-$($ch.Title).html"
    $enSubDir = Join-Path $EnDir $ch.Title
    $jaSubDir = Join-Path $JaDir $ch.Title
    
    Write-Host "Processing chapter $($ch.Number) ($($ch.Title))..."
    
    if (Test-Path $enChapterFile) {
        Split-HtmlByH2 -ChapterFile $enChapterFile -OutputDir $enSubDir -Lang "en"
    }
    if (Test-Path $jaChapterFile) {
        Split-HtmlByH2 -ChapterFile $jaChapterFile -OutputDir $jaSubDir -Lang "ja"
    }
    Write-Host ""
}

# Summary
Write-Host "=== Summary ==="
$enFiles = Get-ChildItem -Path $EnDir -Recurse -File
$jaFiles = Get-ChildItem -Path $JaDir -Recurse -File
Write-Host "English chapters: $($enFiles.Count) files"
Write-Host "Japanese chapters: $($jaFiles.Count) files"
Write-Host ""
Write-Host "Output directory: $ChaptersDir"
Write-Host "Done!"
