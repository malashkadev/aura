# Aura Portable Packaging Script for Windows x64
# Usage: powershell -ExecutionPolicy Bypass -File scripts/build-portable.ps1 [-SkipBuild]

param(
    [switch]$SkipBuild = $false
)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectDir = Split-Path -Parent $ScriptDir
$TauriDir = Join-Path $ProjectDir "src-tauri"
$DistDir = Join-Path $ProjectDir "dist"
$Version = "1.1.0"
$PortableDirName = "Aura-$Version-portable"
$StageDir = Join-Path $DistDir $PortableDirName
$ZipOutput = Join-Path $DistDir "Aura-$Version-windows-x64-portable.zip"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host "  Aura $Version Portable Builder        " -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan

if (-not $SkipBuild) {
    Write-Host "[1/4] Building Tauri release executable..." -ForegroundColor Yellow
    Push-Location $ProjectDir
    try {
        npm run build
    } finally {
        Pop-Location
    }
} else {
    Write-Host "[1/4] Skipping build step (-SkipBuild specified)..." -ForegroundColor Yellow
}

$ExeCandidates = @(
    (Join-Path $TauriDir "target/release/aura.exe"),
    (Join-Path $TauriDir "target/release/aura-app.exe"),
    (Join-Path $TauriDir "target/release/Aura.exe"),
    (Join-Path $TauriDir "target/release/aura-voice.exe")
)
$ExePath = $null
foreach ($candidate in $ExeCandidates) {
    if (Test-Path $candidate) {
        $ExePath = $candidate
        break
    }
}
if (-not $ExePath) {
    throw "Could not find built executable in $($TauriDir)/target/release. Run without -SkipBuild to compile."
}

Write-Host "[2/4] Staging portable layout into $StageDir..." -ForegroundColor Yellow
if (Test-Path $StageDir) {
    Remove-Item -Recurse -Force $StageDir
}
New-Item -ItemType Directory -Path $StageDir -Force | Out-Null

# Copy main binary
Copy-Item -Path $ExePath -Destination (Join-Path $StageDir "Aura.exe") -Force

# Create .portable marker file
New-Item -ItemType File -Path (Join-Path $StageDir ".portable") -Force | Out-Null

# Copy license and notices
$LicensePath = Join-Path $ProjectDir "LICENSE"
if (Test-Path $LicensePath) {
    Copy-Item -Path $LicensePath -Destination (Join-Path $StageDir "LICENSE.txt") -Force
}
$NoticesPath = Join-Path $ProjectDir "THIRD_PARTY_NOTICES.md"
if (Test-Path $NoticesPath) {
    Copy-Item -Path $NoticesPath -Destination (Join-Path $StageDir "THIRD_PARTY_NOTICES.txt") -Force
}

# Copy documentation / readme
$ReadmePath = Join-Path $ProjectDir "README.md"
if (Test-Path $ReadmePath) {
    Copy-Item -Path $ReadmePath -Destination (Join-Path $StageDir "README.txt") -Force
}

Write-Host "[3/4] Creating ZIP archive $ZipOutput..." -ForegroundColor Yellow
if (Test-Path $ZipOutput) {
    Remove-Item -Force $ZipOutput
}
Compress-Archive -Path "$StageDir\*" -DestinationPath $ZipOutput -CompressionLevel Optimal

Write-Host "[4/4] Calculating SHA-256 checksum..." -ForegroundColor Yellow
$Hash = (Get-FileHash -Path $ZipOutput -Algorithm SHA256).Hash

Write-Host ""
Write-Host "========================================" -ForegroundColor Green
Write-Host "Portable package created successfully!" -ForegroundColor Green
Write-Host "File:   $ZipOutput" -ForegroundColor White
Write-Host "SHA256: $Hash" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Green
