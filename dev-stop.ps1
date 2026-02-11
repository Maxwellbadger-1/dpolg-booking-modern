# =============================================================================
# DPolG Booking System - Development Stop Script (Windows PowerShell)
# Kills ALL related processes including background jobs
# =============================================================================

$ErrorActionPreference = "SilentlyContinue"

Write-Host ""
Write-Host "=======================================================" -ForegroundColor Red
Write-Host "  DPolG Booking System - STOP" -ForegroundColor Red
Write-Host "=======================================================" -ForegroundColor Red

# 1. Kill Tauri app window
Write-Host "`n[1/4] Closing application..." -ForegroundColor Yellow
$tauriApp = Get-Process -Name "dpolg-booking-modern" -ErrorAction SilentlyContinue
if ($tauriApp) {
    Stop-Process -Name "dpolg-booking-modern" -Force -ErrorAction SilentlyContinue
    Write-Host "      App window closed" -ForegroundColor Green
} else {
    Write-Host "      No app window found" -ForegroundColor Gray
}

# 2. Kill all related processes
Write-Host "`n[2/4] Killing all related processes..." -ForegroundColor Yellow
$processes = @("node", "cargo", "rustc", "rust-analyzer", "tauri", "vite")
$killed = 0

foreach ($proc in $processes) {
    $running = Get-Process -Name $proc -ErrorAction SilentlyContinue
    if ($running) {
        Stop-Process -Name $proc -Force -ErrorAction SilentlyContinue
        Write-Host "      Killed: $proc" -ForegroundColor Gray
        $killed++
    }
}

if ($killed -eq 0) {
    Write-Host "      No processes were running" -ForegroundColor Gray
}

# 3. Free up ports (native PowerShell, no npx overhead)
Write-Host "`n[3/4] Freeing ports..." -ForegroundColor Yellow
@(1420, 1421) | ForEach-Object {
    $conn = Get-NetTCPConnection -LocalPort $_ -ErrorAction SilentlyContinue
    if ($conn) {
        Stop-Process -Id $conn.OwningProcess -Force -ErrorAction SilentlyContinue
        Write-Host "      Port $_ freed" -ForegroundColor Green
    }
}

# 4. Verify cleanup
Write-Host "`n[4/4] Verification..." -ForegroundColor Yellow
Start-Sleep -Seconds 1
$remaining = Get-Process -Name "node","cargo","dpolg-booking-modern" -ErrorAction SilentlyContinue
if ($remaining) {
    Write-Host "      Warning: Some processes still running:" -ForegroundColor Red
    $remaining | Format-Table Name, Id -AutoSize
} else {
    Write-Host "      All processes cleaned up" -ForegroundColor Green
}

Write-Host ""
Write-Host "Done!" -ForegroundColor Green
Write-Host ""
