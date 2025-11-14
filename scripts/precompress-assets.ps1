param(
    [Parameter(Position=0)]
    [string]$SourceDir = (Join-Path $PSScriptRoot "..\p-project-web\pkg"),

    [string[]]$Extensions = @(
        ".js", ".mjs", ".css", ".html", ".svg",
        ".json", ".wasm", ".xml", ".txt", ".webmanifest"
    )
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Ensure-Directory([string]$Path) {
    if (-not (Test-Path -LiteralPath $Path -PathType Container)) {
        throw "Source directory not found: $Path"
    }
}

function New-GzipFile {
    param(
        [Parameter(Mandatory=$true)][string]$InputPath,
        [Parameter(Mandatory=$true)][string]$OutputPath
    )
    $inStream  = [System.IO.File]::OpenRead($InputPath)
    try {
        $outStream = [System.IO.File]::Create($OutputPath)
        try {
            $level = [System.IO.Compression.CompressionLevel]::Optimal
            $gzip = New-Object System.IO.Compression.GZipStream($outStream, $level, $true)
            try {
                $inStream.CopyTo($gzip)
            } finally { $gzip.Dispose() }
        } finally { $outStream.Dispose() }
    } finally { $inStream.Dispose() }
}

function New-BrotliFile {
    param(
        [Parameter(Mandatory=$true)][string]$InputPath,
        [Parameter(Mandatory=$true)][string]$OutputPath
    )
    $inStream  = [System.IO.File]::OpenRead($InputPath)
    try {
        $outStream = [System.IO.File]::Create($OutputPath)
        try {
            $levelNames = [Enum]::GetNames([System.IO.Compression.CompressionLevel])
            if ($levelNames -contains 'SmallestSize') {
                $level = [System.IO.Compression.CompressionLevel]::SmallestSize
            } else {
                $level = [System.IO.Compression.CompressionLevel]::Optimal
            }
            $br = New-Object System.IO.Compression.BrotliStream($outStream, $level, $true)
            try {
                $inStream.CopyTo($br)
            } finally { $br.Dispose() }
        } finally { $outStream.Dispose() }
    } finally { $inStream.Dispose() }
}

function Should-Compress([System.IO.FileInfo]$Source, [string]$CompressedPath) {
    if (-not (Test-Path -LiteralPath $CompressedPath -PathType Leaf)) {
        return $true
    }
    $compressed = Get-Item -LiteralPath $CompressedPath
    return ($Source.LastWriteTimeUtc -gt $compressed.LastWriteTimeUtc)
}

Ensure-Directory -Path $SourceDir

Write-Host "Precompressing assets in: $SourceDir" -ForegroundColor Cyan
$files = Get-ChildItem -LiteralPath $SourceDir -Recurse -File |
    Where-Object { $Extensions -contains $_.Extension.ToLowerInvariant() }

if ($files.Count -eq 0) {
    Write-Host "No matching files found." -ForegroundColor Yellow
    exit 0
}

$total = 0
$skipped = 0
$doneGz = 0
$doneBr = 0

foreach ($f in $files) {
    $total += 1
    $gz = "$($f.FullName).gz"
    $br = "$($f.FullName).br"

    $didAny = $false

    if (Should-Compress -Source $f -CompressedPath $gz) {
        New-GzipFile -InputPath $f.FullName -OutputPath $gz
        (Get-Item -LiteralPath $gz).LastWriteTimeUtc = $f.LastWriteTimeUtc
        $doneGz += 1
        $didAny = $true
    }
    if (Should-Compress -Source $f -CompressedPath $br) {
        New-BrotliFile -InputPath $f.FullName -OutputPath $br
        (Get-Item -LiteralPath $br).LastWriteTimeUtc = $f.LastWriteTimeUtc
        $doneBr += 1
        $didAny = $true
    }
    if (-not $didAny) { $skipped += 1 }
}

Write-Host ("Processed {0} files | updated: {1} .gz, {2} .br | skipped up-to-date: {3}" -f $total, $doneGz, $doneBr, $skipped) -ForegroundColor Green

