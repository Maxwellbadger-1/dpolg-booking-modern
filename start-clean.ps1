# DPolG Booking System - Clean Start Script (Windows PowerShell Version)
# Stops all processes, clears caches, and starts fresh
# Optimized for Windows with proper process management

Write-Host "🧹 Cleaning up..." -ForegroundColor Cyan

# 1. Kill all running processes
Write-Host "  → Stopping old processes..." -ForegroundColor Gray

# Kill processes by name
Get-Process | Where-Object {$_.ProcessName -like "*dpolg-booking*"} | Stop-Process -Force -ErrorAction SilentlyContinue
Get-Process | Where-Object {$_.ProcessName -like "*tauri*"} | Stop-Process -Force -ErrorAction SilentlyContinue

# 2. Kill processes on ports using npx kill-port
Write-Host "  → Freeing ports 1420, 1421..." -ForegroundColor Gray
npx kill-port 1420 1421 2>$null

# Wait for processes to terminate
Start-Sleep -Seconds 2

# 3. Clear caches (optional - only if --full-clean specified)
if ($args -contains "--full-clean") {
    Write-Host "  → Clearing ALL caches (--full-clean mode)..." -ForegroundColor Gray
    Remove-Item -Path "node_modules\.vite" -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item -Path "src-tauri\target\debug" -Recurse -Force -ErrorAction SilentlyContinue
    Write-Host "     Cache cleared! Next build will take ~1-2 minutes." -ForegroundColor Yellow
}

# 4. Start app with SEPARATE processes (Windows-native method)
Write-Host ""
Write-Host "🚀 Starting app..." -ForegroundColor Green
Write-Host ""
Write-Host "  📍 Method: Separate Vite + Tauri processes" -ForegroundColor Gray
Write-Host "  📦 Vite will start on: http://localhost:1420" -ForegroundColor Gray
Write-Host "  🦀 Tauri will compile and connect to Vite" -ForegroundColor Gray
Write-Host ""

# Start Vite in background job
Write-Host "  ⚡ Starting Vite dev server..." -ForegroundColor Yellow
$viteJob = Start-Job -ScriptBlock {
    Set-Location $using:PWD
    npm run dev
}

# Wait for Vite to be ready
Write-Host "  ⏳ Waiting for Vite to start..." -ForegroundColor Gray
Start-Sleep -Seconds 3

# Start Tauri (foreground process)
Write-Host "  🦀 Starting Tauri (compiling Rust backend)..." -ForegroundColor Yellow
Write-Host ""

try {
    Set-Location src-tauri
    cargo run
}
finally {
    # Cleanup on exit (stop Vite when Tauri exits)
    Write-Host ""
    Write-Host "🧹 Cleaning up Vite process..." -ForegroundColor Cyan
    Stop-Job -Job $viteJob -ErrorAction SilentlyContinue
    Remove-Job -Job $viteJob -ErrorAction SilentlyContinue

    # Also kill any remaining Vite processes on port
    npx kill-port 1420 1421 2>$null

    Write-Host "✅ Cleanup complete!" -ForegroundColor Green
}
