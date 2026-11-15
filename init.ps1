# Aura 2.0 - standard startup and verification path (Windows PowerShell).
# Run from the repository root: .\init.ps1
$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

function Invoke-AuraStep {
    param(
        [string]$Name,
        [scriptblock]$Command
    )

    Write-Host "==> $Name" -ForegroundColor Cyan
    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "$Name failed with exit code $LASTEXITCODE"
    }
}

Write-Host "==> Aura 2.0 verification" -ForegroundColor Cyan
Write-Host "==> Node: $(node --version)"
Write-Host "==> Rust: $(rustc --version)"

Invoke-AuraStep "Frontend dependencies (npm ci)" { npm.cmd ci }
Invoke-AuraStep "Frontend lint and typecheck" { npm.cmd run check }
Invoke-AuraStep "Frontend tests" { npm.cmd test }
Invoke-AuraStep "Static audit gates" { npm.cmd run audit }
Invoke-AuraStep "Release manifest consistency" { npm.cmd run verify:manifest }
Invoke-AuraStep "Rust format" { cargo fmt --manifest-path src-tauri/Cargo.toml -- --check }
Invoke-AuraStep "Rust clippy" { cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings }
Invoke-AuraStep "Rust tests" { cargo test --manifest-path src-tauri/Cargo.toml --all-targets }

Write-Host ""
Write-Host "All Aura checks passed." -ForegroundColor Green
