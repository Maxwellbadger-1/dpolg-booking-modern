#!/bin/bash

# DPolG Booking System - Clean Start Script
# Stops all processes, clears caches, and starts fresh

echo "🧹 Cleaning up..."

# 1. Kill all running processes
echo "  → Stopping old processes..."
pkill -9 -f "tauri" 2>/dev/null || true
pkill -9 -f "vite" 2>/dev/null || true
pkill -9 -f "dpolg-booking" 2>/dev/null || true
lsof -ti:1420 | xargs kill -9 2>/dev/null || true

# Wait for processes to terminate
sleep 2

# 2. Clear caches
echo "  → Clearing caches..."
rm -rf node_modules/.vite 2>/dev/null || true
rm -rf src-tauri/target/debug 2>/dev/null || true

# 3. Start app
echo ""
echo "🚀 Starting app with fresh build..."
echo "   (First build takes ~1-2 minutes)"
echo ""

npm run tauri dev
