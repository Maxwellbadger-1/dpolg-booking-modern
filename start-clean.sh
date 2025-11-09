#!/bin/bash

# DPolG Booking System - Clean Start Script (2025 Optimized)
# Stops all processes, clears caches, and starts fresh
# Uses separate Vite + Tauri start method (avoids Windows hanging issues)

echo "🧹 Cleaning up..."

# 1. Kill all running processes
echo "  → Stopping old processes..."
pkill -9 -f "tauri" 2>/dev/null || true
pkill -9 -f "vite" 2>/dev/null || true
pkill -9 -f "dpolg-booking" 2>/dev/null || true

# 2. Kill processes on ports (cross-platform with npx kill-port)
echo "  → Freeing ports 1420, 1421..."
npx kill-port 1420 1421 2>/dev/null || true

# Wait for processes to terminate
sleep 2

# 3. Clear caches (optional - only if specified)
if [ "$1" == "--full-clean" ]; then
  echo "  → Clearing ALL caches (--full-clean mode)..."
  rm -rf node_modules/.vite 2>/dev/null || true
  rm -rf src-tauri/target/debug 2>/dev/null || true
  echo "     Cache cleared! Next build will take ~1-2 minutes."
fi

# 4. Start app with SEPARATE processes (Windows-compatible method)
echo ""
echo "🚀 Starting app..."
echo ""
echo "  📍 Method: Separate Vite + Tauri processes"
echo "  📦 Vite will start on: http://localhost:1420"
echo "  🦀 Tauri will compile and connect to Vite"
echo ""

# Start Vite in background
echo "  ⚡ Starting Vite dev server..."
npm run dev &
VITE_PID=$!

# Wait for Vite to be ready
echo "  ⏳ Waiting for Vite to start..."
sleep 3

# Start Tauri (this will be foreground process)
echo "  🦀 Starting Tauri (compiling Rust backend)..."
echo ""
cd src-tauri && cargo run

# Cleanup on exit (kill Vite when Tauri exits)
kill $VITE_PID 2>/dev/null || true
