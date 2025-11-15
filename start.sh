#!/bin/bash

# DPolG Booking System - Normal Start
# Nutzt HMR (Hot Module Replacement) für schnelle Änderungen

echo "🚀 Starte DPolG Buchungssystem..."
echo ""
echo "📝 Tipp: Änderungen an React-Komponenten werden automatisch live angezeigt (HMR)"
echo "🧹 Bei Cache-Problemen: ./start-clean.sh verwenden"
echo ""

cd "/Users/maximilianfegg/Desktop/Sicherungskopie DPolG Buchungssystem/Claude Code/dpolg-booking-modern"
npm run tauri dev
