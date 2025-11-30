# 🔄 Übergabeprotokoll: Mac → Windows

**Datum:** 2025-11-30
**Version:** v1.8.0
**Feature:** Multi-User Real-Time Updates

---

## ✅ Was auf dem Mac fertig ist

### 1. Code-Implementierung
- ✅ Backend: `get_updates_since` Command implementiert
- ✅ Frontend: Auto-Polling alle 3 Sekunden in DataContext
- ✅ Offline Detection mit 3-Sekunden-Timeout
- ✅ OfflineBanner + OnlineContext Components
- ✅ Dokumentation: `REALTIME_MULTIUSER_IMPLEMENTATION.md`

### 2. Git & Release
- ✅ Commit erstellt: `feat: Implement Multi-User Real-Time Updates with Polling`
- ✅ Version gebumpt: `1.7.6` → `1.8.0`
- ✅ Git Tag: `v1.8.0` erstellt
- ✅ GitHub Release: https://github.com/Maxwellbadger-1/dpolg-booking-modern/releases/tag/v1.8.0
- ✅ Gepusht zu GitHub: `main` Branch + Tag

### 3. Kompilierung
- ✅ Backend kompiliert (nur Warnings, keine Errors)
- ✅ Frontend kompiliert
- ✅ Dev-Server läuft (aber ohne Headless Chrome für PDFs)

---

## 🎯 Was am Windows-PC zu tun ist

### Schritt 1: Projekt aktualisieren

```powershell
cd "C:\Pfad\zu\dpolg-booking-modern"

# Neueste Version holen
git fetch --all
git pull origin main

# Verifizieren dass v1.8.0 da ist
git log --oneline -5
# Sollte zeigen:
# b15351a chore: Bump version to 1.8.0
# b04b1be feat: Implement Multi-User Real-Time Updates with Polling
```

### Schritt 2: Dependencies aktualisieren

```powershell
# Node Packages
npm install

# Rust Dependencies (falls nötig)
cd src-tauri
cargo clean
cargo build
cd ..
```

### Schritt 3: Production Build erstellen

```powershell
# Windows .exe + Installer bauen
npm run tauri:build

# Oder direkt mit Tauri CLI
cd src-tauri
cargo tauri build --config tauri.conf.json
cd ..
```

**Build-Dauer:** Ca. 5-15 Minuten (je nach PC)

**Build-Output Pfad:**
```
src-tauri/target/release/
├── dpolg-booking-modern.exe          # Standalone .exe
└── bundle/
    └── nsis/
        └── dpolg-booking-modern_1.8.0_x64-setup.exe  # Installer
```

### Schritt 4: Testing (WICHTIG!)

#### Test 1: PDF-Generierung (Headless Chrome)
```
1. App starten
2. Buchung erstellen
3. Rechnung generieren → Muss funktionieren!
4. Putzplan PDF generieren → Muss funktionieren!
```

#### Test 2: Multi-User Real-Time Updates
```
1. App 2x starten (2 Fenster)
2. Fenster A: Neue Buchung erstellen
3. Fenster B: Nach 3 Sekunden muss Buchung erscheinen (automatisch!)
4. Fenster B: Buchung ändern
5. Fenster A: Nach 3 Sekunden muss Änderung erscheinen
```

#### Test 3: Offline Detection
```
1. App starten
2. WLAN ausschalten / Netzwerkkabel ziehen
3. Nach 3 Sekunden: Roter Banner "KEINE VERBINDUNG" muss erscheinen
4. WLAN wieder an
5. Banner muss verschwinden
```

### Schritt 5: Release-Assets hochladen

```powershell
# GitHub CLI installieren (falls nicht vorhanden)
# https://cli.github.com/

# In das Projekt-Verzeichnis
cd "C:\Pfad\zu\dpolg-booking-modern"

# .exe zum Release hochladen
gh release upload v1.8.0 `
  "src-tauri/target/release/bundle/nsis/dpolg-booking-modern_1.8.0_x64-setup.exe" `
  --clobber

# Falls du auch die Standalone .exe hochladen willst
gh release upload v1.8.0 `
  "src-tauri/target/release/dpolg-booking-modern.exe" `
  --clobber
```

**Alternativ:** Manuell auf GitHub hochladen:
1. Gehe zu: https://github.com/Maxwellbadger-1/dpolg-booking-modern/releases/tag/v1.8.0
2. "Edit release" klicken
3. .exe Dateien per Drag & Drop hochladen
4. "Update release" klicken

---

## 📋 Checkliste Windows-Build

- [ ] Git Pull ausgeführt (v1.8.0 vorhanden)
- [ ] `npm install` ausgeführt
- [ ] `npm run tauri:build` erfolgreich
- [ ] .exe Datei existiert in `src-tauri/target/release/bundle/nsis/`
- [ ] PDF-Generierung getestet (Rechnung + Putzplan)
- [ ] Multi-User Updates getestet (2 Fenster)
- [ ] Offline Detection getestet (WLAN aus/an)
- [ ] .exe zu GitHub Release hochgeladen
- [ ] Release verifiziert auf GitHub

---

## 🐛 Häufige Probleme & Lösungen

### Problem: Build schlägt fehl
```powershell
# Lösung 1: Clean Build
npm run tauri:build:clean

# Lösung 2: Cargo Cache löschen
cd src-tauri
cargo clean
cd ..
npm run tauri:build
```

### Problem: Headless Chrome fehlt
```powershell
# Sollte automatisch installiert sein, falls nicht:
npm install puppeteer
```

### Problem: "error: linker `link.exe` not found"
```
Visual Studio 2022 Build Tools installieren:
https://visualstudio.microsoft.com/downloads/
→ "Build Tools for Visual Studio 2022"
→ "Desktop development with C++"
```

### Problem: Multi-User Updates funktionieren nicht
```
1. Console öffnen (F12 in DevTools)
2. Logs checken:
   - "📊 Real-Time: X booking(s) updated" sollte alle 3 Sek erscheinen
   - Falls nicht: Backend-Fehler checken
```

---

## 📊 Erwartete Test-Ergebnisse

### Multi-User Polling
```
Fenster A (10:00:00): Buchung erstellen
    ↓
DB: updated_at = 2025-11-30 10:00:00
    ↓
Fenster B (10:00:02): Poll-Request
    ↓
Backend: "1 booking updated since 10:00:00"
    ↓
Fenster B (10:00:02): Buchung erscheint automatisch!
    ✅ Console: "📊 Real-Time: 1 booking(s) updated"
```

### Offline Detection
```
WLAN AUS
    ↓ (3 Sekunden)
Roter Banner: "⚠️ KEINE VERBINDUNG ZUR DATENBANK"
    ↓
WLAN AN
    ↓ (3 Sekunden)
Banner verschwindet
```

---

## 📝 Nach erfolgreichem Build

### Release Notes für User
```
v1.8.0 ist jetzt live! 🚀

Neue Features:
✅ Multi-User fähig - Mehrere Rechner gleichzeitig nutzen
✅ Automatische Updates alle 3 Sekunden
✅ Offline-Warnung wenn Verbindung abbricht

Installation:
1. Download: dpolg-booking-modern_1.8.0_x64-setup.exe
2. Installer ausführen
3. Fertig!

Testen:
- Starte App auf 2 Rechnern
- Ändere Buchung auf Rechner A
- Rechner B zeigt Update automatisch nach 3 Sek
```

---

## 🔗 Wichtige Links

- **GitHub Release:** https://github.com/Maxwellbadger-1/dpolg-booking-modern/releases/tag/v1.8.0
- **Repo:** https://github.com/Maxwellbadger-1/dpolg-booking-modern
- **Dokumentation:** `REALTIME_MULTIUSER_IMPLEMENTATION.md` im Projekt

---

## ✉️ Bei Problemen

Falls Probleme auftreten:
1. Fehler-Message kopieren
2. Console-Logs checken (F12 → Console)
3. Issue auf GitHub erstellen mit:
   - Windows Version
   - Fehler-Message
   - Build-Logs

---

**Status:** Bereit für Windows-Build! 🚀

**Ziel:** Production-Ready .exe mit Multi-User Updates + PDF-Generierung

**Geschätzter Zeitaufwand:** 30-45 Minuten (Build + Tests)
