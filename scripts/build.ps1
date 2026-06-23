# RustPad build script for Windows
# Requires: Rust toolchain, cargo-bundle (optional)

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$ProjectDir = Split-Path -Parent $ScriptDir

Set-Location $ProjectDir

Write-Host "=== RustPad Windows Build Script ===" -ForegroundColor Cyan
Write-Host ""

Write-Host "[1/5] Cleaning previous builds..." -ForegroundColor Yellow
cargo clean --release 2>$null

Write-Host "[2/5] Running tests..." -ForegroundColor Yellow
cargo test --release
if ($LASTEXITCODE -ne 0) { throw "Tests failed" }

Write-Host "[3/5] Running clippy..." -ForegroundColor Yellow
cargo clippy --release -- -D warnings
if ($LASTEXITCODE -ne 0) { throw "Clippy failed" }

Write-Host "[4/5] Building release..." -ForegroundColor Yellow
cargo build --release
if ($LASTEXITCODE -ne 0) { throw "Build failed" }

Write-Host "[5/5] Packaging..." -ForegroundColor Yellow
if (Get-Command cargo-bundle -ErrorAction SilentlyContinue) {
    cargo bundle --release
    $BundleDir = "target\release\bundle\msi"
    if (Test-Path $BundleDir) {
        Write-Host "  MSI bundle created in: $BundleDir" -ForegroundColor Green
    }
} else {
    Write-Host "  Skipping bundle (install: cargo install cargo-bundle)" -ForegroundColor Yellow
    Write-Host "  Release binary: target\release\rustpad.exe" -ForegroundColor Green
}

Write-Host ""
Write-Host "=== Build complete ===" -ForegroundColor Cyan
Write-Host "Binary: target\release\rustpad.exe" -ForegroundColor Green
