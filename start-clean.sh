#!/bin/bash

# DPolG Booking System - Clean Start Script
# Dieses Script nur verwenden wenn Cache-Probleme auftreten oder explizit gewünscht

echo "🧹 Räume auf..."

# 1. Alle laufenden Prozesse beenden
echo "  → Beende alte Prozesse..."
pkill -9 -f "tauri" 2>/dev/null || true
pkill -9 -f "vite" 2>/dev/null || true
pkill -9 -f "dpolg-booking" 2>/dev/null || true
lsof -ti:1420 | xargs kill -9 2>/dev/null || true

# Kurz warten bis Prozesse beendet sind
sleep 2

# 2. Cache löschen
echo "  → Lösche Caches..."
rm -rf node_modules/.vite 2>/dev/null || true
rm -rf src-tauri/target/debug 2>/dev/null || true

# 3. App starten
echo ""
echo "🚀 Starte App mit frischem Build..."
echo "   (Dies dauert ca. 1-2 Minuten beim ersten Mal)"
echo ""

npm run tauri dev
