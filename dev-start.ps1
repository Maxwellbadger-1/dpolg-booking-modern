# =============================================================================
# DPolG Booking System - Development Start Script (Windows PowerShell)
# =============================================================================

$ErrorActionPreference = "SilentlyContinue"

Write-Host ""
Write-Host "=======================================================" -ForegroundColor Cyan
Write-Host "  DPolG Booking System - START" -ForegroundColor Cyan
Write-Host "=======================================================" -ForegroundColor Cyan

# Get script directory
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $ScriptDir

Write-Host "`n[1/4] Stopping existing processes..." -ForegroundColor Yellow

# Kill relevant processes (NOT node - would kill Claude Code and other tools)
$processes = @("dpolg-booking-modern", "cargo", "rustc")
foreach ($proc in $processes) {
    $running = Get-Process -Name $proc -ErrorAction SilentlyContinue
    if ($running) {
        Stop-Process -Name $proc -Force -ErrorAction SilentlyContinue
        Write-Host "      Killed: $proc" -ForegroundColor Gray
    }
}

# Kill Vite dev server specifically by port, not by killing all node processes
@(1420, 1421) | ForEach-Object {
    $conn = Get-NetTCPConnection -LocalPort $_ -ErrorAction SilentlyContinue
    if ($conn) {
        Stop-Process -Id $conn.OwningProcess -Force -ErrorAction SilentlyContinue
        Write-Host "      Port $_ freed" -ForegroundColor Gray
    }
}

Write-Host "      Done" -ForegroundColor Green

Write-Host "`n[2/4] Waiting for cleanup..." -ForegroundColor Yellow
Start-Sleep -Seconds 2
Write-Host "      Done" -ForegroundColor Green

Write-Host "`n[3/4] Checking dependencies..." -ForegroundColor Yellow

# Check node_modules
if (-not (Test-Path "node_modules")) {
    Write-Host "      Installing npm packages..." -ForegroundColor Yellow
    npm install
}

# Check cargo
$cargo = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $cargo) {
    Write-Host "      ERROR: Cargo not found! Install Rust first." -ForegroundColor Red
    exit 1
}

Write-Host "      Done" -ForegroundColor Green

Write-Host "`n[4/4] Starting Tauri app (Dev Mode)..." -ForegroundColor Yellow
Write-Host ""
Write-Host "      Desktop app opens automatically" -ForegroundColor Blue
Write-Host "      Hot Reload enabled" -ForegroundColor Blue
Write-Host ""
Write-Host "      Press Ctrl+C to stop" -ForegroundColor Gray
Write-Host ""

# Start Tauri dev mode - opens desktop app with hot reload
# Using npx tauri dev directly for better output capture
npx tauri dev
