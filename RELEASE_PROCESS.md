# 🚀 Release-Prozess (LOKALER BUILD)

**WICHTIG:** Wir builden IMMER lokal (nicht GitHub Actions)!

---

## 📋 Standard-Ablauf für JEDES Update

### ✅ Schritt 1: Code-Änderungen committen

```bash
# Dateien stagen
git add <geänderte-files>

# Commit mit aussagekräftiger Message
git commit -m "fix: QR-Code zentriert in Rechnung"
# ODER
git commit -m "feat: Month-spanning bookings im TapeChart"
```

---

### ✅ Schritt 2: Version bumpen

**3 Dateien müssen aktualisiert werden:**

1. **package.json**
   ```json
   "version": "1.7.4"
   ```

2. **src-tauri/Cargo.toml**
   ```toml
   version = "1.7.4"
   ```

3. **src-tauri/tauri.conf.json**
   ```json
   "version": "1.7.4"
   ```

**Dann committen:**
```bash
git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
git commit -m "chore: Bump version to 1.7.4"
```

---

### ✅ Schritt 3: Git Tag erstellen

```bash
# Tag mit Beschreibung
git tag -a v1.7.4 -m "Release v1.7.4 - QR-Code Fix & Month-Spanning Bookings

Changes:
- Fixed QR code display in invoices
- Added month-spanning booking support
- Improved visual indicators"

# Pushen (Code + Tags)
git push && git push --tags
```

---

### ✅ Schritt 4: Dev-Server stoppen

**WICHTIG:** Alle laufenden Dev-Server beenden, sonst Port-Konflikt!

```bash
# Prüfen welche Server laufen
netstat -ano | findstr :1423

# ALLE stoppen (oder manuell Terminals schließen)
```

---

### ✅ Schritt 5: LOKAL BUILDEN 🏗️

```bash
npm run tauri build
```

**Dauer:** ~5-10 Minuten

**Output-Verzeichnis:** `src-tauri/target/release/bundle/`

**Wichtige Dateien:**
```
msi/
├── dpolg-booking-modern_1.7.4_x64_en-US.msi          ← Installer
├── dpolg-booking-modern_1.7.4_x64_en-US.msi.zip      ← Update-Package
└── dpolg-booking-modern_1.7.4_x64_en-US.msi.zip.sig  ← Signatur
```

⚠️ **ALLE 3 DATEIEN werden für Auto-Update benötigt!**

---

### ✅ Schritt 6: GitHub Release erstellen

#### Option A: Mit `gh` CLI (schneller)

```bash
cd src-tauri/target/release/bundle/msi

gh release create v1.7.4 \
  --title "Stiftung der DPolG Buchungssystem v1.7.4" \
  --notes "## 🎉 Änderungen in v1.7.4

- ✅ QR-Code wird jetzt zentriert in Rechnungen angezeigt
- ✅ Month-spanning Buchungen funktionieren über Monatsgrenzen
- ✅ Visuelle Indikatoren für fortlaufende Buchungen

## 📥 Installation
**Windows:** Laden Sie die \`.msi\` Datei herunter und installieren Sie die App.

## 🔄 Auto-Update
Wenn Sie bereits eine ältere Version installiert haben, wird automatisch ein Update-Dialog angezeigt." \
  *.msi \
  *.msi.zip \
  *.msi.zip.sig
```

#### Option B: Manuell über GitHub Web UI

1. Öffnen: https://github.com/Maxwellbadger-1/dpolg-booking-modern/releases/new

2. **Tag auswählen:** `v1.7.4`

3. **Title:** `Stiftung der DPolG Buchungssystem v1.7.4`

4. **Beschreibung:**
   ```markdown
   ## 🎉 Änderungen in v1.7.4

   - ✅ QR-Code wird jetzt zentriert in Rechnungen angezeigt
   - ✅ Month-spanning Buchungen funktionieren über Monatsgrenzen
   - ✅ Visuelle Indikatoren für fortlaufende Buchungen

   ## 📥 Installation
   **Windows:** Laden Sie die `.msi` Datei herunter und installieren Sie die App.

   ## 🔄 Auto-Update
   Wenn Sie bereits eine ältere Version installiert haben, wird automatisch ein Update-Dialog angezeigt.
   ```

5. **Dateien hochladen (Drag & Drop):**
   - ✅ `dpolg-booking-modern_1.7.4_x64_en-US.msi`
   - ✅ `dpolg-booking-modern_1.7.4_x64_en-US.msi.zip`
   - ✅ `dpolg-booking-modern_1.7.4_x64_en-US.msi.zip.sig`

6. **"Publish release"** klicken

⚠️ **NICHT als Draft speichern!** Sonst funktioniert Auto-Update nicht.

---

### ✅ Schritt 7: Auto-Update testen

1. **Installierte App öffnen** (ältere Version z.B. 1.7.3)

2. **Update-Dialog sollte erscheinen:**
   ```
   🔄 Update verfügbar: v1.7.4
   Möchten Sie jetzt aktualisieren?
   ```

3. **"Ja" klicken** → App lädt Update herunter

4. **App neustartet** → Version 1.7.4 läuft

5. **Änderungen testen** (z.B. QR-Code in Rechnung generieren)

---

## 🚨 Häufige Fehler vermeiden

### ❌ FEHLER: Port bereits belegt
**Symptom:** `Error: Port 1423 already in use`
**Lösung:** Alle Dev-Server stoppen vor Build

### ❌ FEHLER: Dateien fehlen im Release
**Symptom:** Auto-Update funktioniert nicht
**Lösung:** ALLE 3 Dateien hochladen (.msi + .msi.zip + .msi.zip.sig)

### ❌ FEHLER: Version nicht erhöht
**Symptom:** Update wird nicht erkannt
**Lösung:** Immer Version in ALLEN 3 Dateien bumpen (package.json, Cargo.toml, tauri.conf.json)

### ❌ FEHLER: Release als Draft
**Symptom:** Auto-Updater findet kein Update
**Lösung:** Release MUSS published sein, nicht Draft!

---

## 📝 Checkliste vor jedem Release

- [ ] Alle Änderungen committed
- [ ] Version in 3 Dateien erhöht (package.json, Cargo.toml, tauri.conf.json)
- [ ] Git tag erstellt + gepusht
- [ ] Dev-Server gestoppt
- [ ] `npm run tauri build` erfolgreich
- [ ] 3 Dateien vorhanden (*.msi, *.msi.zip, *.msi.zip.sig)
- [ ] GitHub Release erstellt
- [ ] Alle 3 Dateien hochgeladen
- [ ] Release published (nicht Draft!)
- [ ] Auto-Update in installierter App getestet

---

## 🎯 Versionsnummern-Schema

```
v1.7.4
  │ │ │
  │ │ └─ Patch: Bugfixes (QR-Code Fix, kleine Änderungen)
  │ └─── Minor: Neue Features (Month-spanning Bookings)
  └───── Major: Breaking Changes (komplettes Redesign)
```

**Beispiele:**
- Bugfix → `1.7.3` → `1.7.4`
- Neues Feature → `1.7.4` → `1.8.0`
- Breaking Change → `1.8.0` → `2.0.0`

---

## ⏱️ Geschätzte Zeiten

| Schritt | Dauer |
|---------|-------|
| Code committen | 1-2 min |
| Version bumpen | 1 min |
| Tag erstellen + pushen | 30 sec |
| Build lokal | 5-10 min |
| GitHub Release erstellen | 2-3 min |
| Auto-Update testen | 2-3 min |
| **GESAMT** | **~15 min** |

---

**Erstellt:** 2025-10-24
**Letzte Aktualisierung:** 2025-10-24
