# DPolG Booking System - Local Cleanup Script
# Removes build artifacts to free up disk space

Write-Host "==================================================" -ForegroundColor Cyan
Write-Host "DPolG Booking System - Local Cleanup" -ForegroundColor Cyan
Write-Host "==================================================" -ForegroundColor Cyan
Write-Host ""

# Clean Rust build cache
Write-Host "[1/4] Cleaning Rust build artifacts (src-tauri/target/)..." -ForegroundColor Yellow
Push-Location src-tauri
try {
    cargo clean --release
    Write-Host "✓ Rust release builds cleaned (~5-6 GB freed)" -ForegroundColor Green
} catch {
    Write-Host "✗ Error cleaning Rust builds: $_" -ForegroundColor Red
} finally {
    Pop-Location
}
Write-Host ""

# Clean Vite cache
Write-Host "[2/4] Cleaning Vite cache..." -ForegroundColor Yellow
if (Test-Path ".vite") {
    Remove-Item -Recurse -Force .vite -ErrorAction SilentlyContinue
    Write-Host "✓ Vite cache cleaned" -ForegroundColor Green
} else {
    Write-Host "✓ Vite cache already clean" -ForegroundColor Green
}
Write-Host ""

# Clean dist folder
Write-Host "[3/4] Cleaning dist folder..." -ForegroundColor Yellow
if (Test-Path "dist") {
    Remove-Item -Recurse -Force dist -ErrorAction SilentlyContinue
    Write-Host "✓ Dist folder cleaned" -ForegroundColor Green
} else {
    Write-Host "✓ Dist folder already clean" -ForegroundColor Green
}
Write-Host ""

# Optional: Clean npm cache (commented out by default)
Write-Host "[4/4] NPM cache (skipped - uncomment if needed)" -ForegroundColor Gray
# npm cache clean --force

Write-Host ""
Write-Host "==================================================" -ForegroundColor Cyan
Write-Host "Cleanup complete!" -ForegroundColor Green
Write-Host "==================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "To rebuild the project, run:" -ForegroundColor Yellow
Write-Host "  npm run tauri:dev" -ForegroundColor White
Write-Host ""
