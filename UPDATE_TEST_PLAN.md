# Update-Test Plan für DPolG Buchungssystem

## 🎯 Ziel
Testen der automatischen Update-Funktion mit dem Tauri Updater Plugin.

## 📋 Test-Setup

### Schritt 1: GitHub Release für v1.6.9 erstellen

1. **Gehe zu GitHub:**
   ```
   https://github.com/Maxwellbadger-1/dpolg-booking-modern/releases/new
   ```

2. **Release Details:**
   - **Tag:** `v1.6.9`
   - **Title:** `Version 1.6.9 - Update Test`
   - **Description:**
     ```markdown
     ## Version 1.6.9 - Update Test Release

     ### Neue Features:
     - ✨ Automatische Update-Prüfung beim App-Start
     - 🔄 Manueller Update-Check in den Einstellungen
     - 📦 Nahtlose Installation von Updates

     ### Verbesserungen:
     - Performance-Optimierungen
     - Verbesserte Fehlerbehandlung
     - UI-Verbesserungen
     ```

3. **Assets hochladen:**
   - Die existierende MSI-Datei umbenennen:
     `Stiftung der DPolG Buchungssystem_1.6.8_x64_en-US.msi`
     → `Stiftung.der.DPolG.Buchungssystem_1.6.9_x64_en-US.msi`
   - `latest.json` (siehe unten)

### Schritt 2: latest.json erstellen

```json
{
  "version": "1.6.9",
  "notes": "## Version 1.6.9 - Update Test\\n\\n### Neue Features:\\n- ✨ Automatische Update-Prüfung\\n- 🔄 Manueller Update-Check\\n- 📦 Nahtlose Installation",
  "pub_date": "2025-10-23T20:30:00Z",
  "platforms": {
    "windows-x86_64": {
      "signature": "PLACEHOLDER_FOR_NOW",
      "url": "https://github.com/Maxwellbadger-1/dpolg-booking-modern/releases/download/v1.6.9/Stiftung.der.DPolG.Buchungssystem_1.6.9_x64_en-US.msi"
    }
  }
}
```

### Schritt 3: Test durchführen

1. **App v1.6.8 installieren** (falls nicht schon geschehen)
2. **App starten** → Automatischer Update-Check sollte v1.6.9 finden
3. **Manueller Test:** Einstellungen → "Nach Updates suchen"
4. **Erwartetes Verhalten:**
   - Update-Dialog erscheint: "Version 1.6.9 verfügbar"
   - Download startet nach Bestätigung
   - Progress-Anzeige
   - App-Neustart nach Installation

## 🔍 Debug-Informationen

### Console Logs prüfen:
- `🔍 Checking for updates...`
- `✅ Update available: 1.6.9`
- `Download started (X MB)`
- `Download finished`

### Endpoint prüfen:
```
https://github.com/Maxwellbadger-1/dpolg-booking-modern/releases/latest/download/latest.json
```

## ⚠️ Wichtige Hinweise

- Die Signatur ist nur ein Platzhalter für den Test
- Bei echten Releases müssen MSI-Dateien signiert werden
- Der Test funktioniert auch ohne echte Signatur (nur Warnung)
- GitHub muss die latest.json automatisch als "latest" Release markieren

## 📱 Nach dem Test

1. **Erfolg dokumentieren** ✅
2. **Version zurück auf 1.6.8 setzen** für normale Entwicklung
3. **Test-Release löschen** oder als "Pre-release" markieren