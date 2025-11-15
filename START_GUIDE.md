# 🚀 DPolG Booking System - Start Guide

## Schnellstart (Empfohlen)

### Windows (PowerShell)
```powershell
.\start-clean.ps1
```

### Linux/Mac (Bash)
```bash
./start-clean.sh
```

### Alternative: NPM Script (EMPFOHLEN für Windows!)
```bash
npm run tauri:dev
```

---

## 📋 Was machen die Scripts?

Alle Start-Scripts führen folgende Schritte aus:

1. **🧹 Cleanup**: Stoppt alte Prozesse und gibt Ports frei (1420, 1421)
2. **⚡ Vite Start**: Startet Frontend Dev Server auf Port 1420
3. **🦀 Tauri Start**: Kompiliert Rust Backend und verbindet sich mit Vite
4. **🎯 App öffnet sich**: Desktop App wird automatisch gestartet

---

## 🔧 Optionen

### Full Clean (Cache löschen)
Löscht Vite Cache und Rust Debug Build für komplett frischen Build:

```bash
# Bash
./start-clean.sh --full-clean

# PowerShell
.\start-clean.ps1 --full-clean
```

**⚠️ Achtung**: Full Clean dauert ~2-3 Minuten beim nächsten Start!

---

## 🐛 Troubleshooting

### Problem: App startet nicht / Port 1420 belegt

**Lösung 1**: Ports manuell freigeben
```bash
npx kill-port 1420 1421
```

**Lösung 2**: Prozesse komplett beenden
```powershell
# Windows
taskkill /F /IM dpolg-booking-modern.exe
npx kill-port 1420 1421

# Linux/Mac
pkill -9 -f "dpolg-booking"
npx kill-port 1420 1421
```

### Problem: "Running BeforeDevCommand" hängt

Das ist ein bekanntes Tauri 2 + Windows Problem mit `npm run tauri dev`.

**Lösung**: Verwende IMMER die separaten Scripts oder `npm run tauri:dev`!

Diese Scripts starten Vite und Tauri separat, was das Problem umgeht.

---

## 📊 Geschätzte Zeiten

| Schritt | Dauer | Notizen |
|---------|-------|---------|
| Port Cleanup | ~1-2 sek | Schnell |
| Vite Start | ~2-3 sek | Cached |
| Tauri Compile (incremental) | ~5-15 sek | Bei kleinen Änderungen |
| Tauri Compile (full clean) | ~2-3 min | Bei --full-clean |
| **GESAMT (normal)** | **~10-20 sek** | Nach erstem Build |
| **GESAMT (full clean)** | **~3-4 min** | Löscht alle Caches |

---

## 🎯 Best Practices

### 1. **Normale Entwicklung**
```bash
npm run tauri:dev  # Schnell, zuverlässig
```

### 2. **Nach Git Pull oder großen Änderungen**
```bash
./start-clean.sh  # Stoppt alte Prozesse
```

### 3. **Bei unerklärlichen Fehlern**
```bash
./start-clean.sh --full-clean  # Nuclear Option
```

### 4. **Production Build**
```bash
npm run tauri build  # Erstellt .exe mit Signierung
```

---

## 📦 Verwendete Ports

| Port | Zweck | Konfiguration |
|------|-------|---------------|
| **1420** | Vite Dev Server | `vite.config.ts` |
| **1421** | Vite HMR (Hot Module Replacement) | `vite.config.ts` |

**⚠️ WICHTIG**: Diese Ports müssen in allen Configs konsistent sein!

- ✅ `vite.config.ts` → `port: 1420`
- ✅ `tauri.conf.json` → `devUrl: "http://localhost:1420"`
- ✅ `package.json` → `predev: kill-port 1420 1421`

**NIEMALS** Port-Overrides in Scripts verwenden (z.B. `--port 1423`)!

---

## 🆘 Hilfe

### Dokumentation
- **Claude Code Richtlinien**: `.claude/CLAUDE.md`
- **Tauri Docs**: https://tauri.app/v2/
- **Vite Docs**: https://vitejs.dev/

### Support
- GitHub Issues: [dpolg-booking-modern/issues](https://github.com/Maxwellbadger-1/dpolg-booking-modern/issues)

---

**Version**: 2.0 (2025-11-09)
**Zuletzt aktualisiert**: Nach Fixing von Windows "Running BeforeDevCommand" Hanging Issue
