# 🚀 Release-Prozess (LOKALER BUILD)

**WICHTIG:** Wir builden IMMER lokal (nicht GitHub Actions)!

---

## ⚡ SCHNELLSTER WEG - VOLLAUTOMATISCH (EMPFOHLEN!)

**Ein Befehl macht ALLES:**
```bash
./quick-release.sh 1.7.5
```

**Was passiert automatisch:**
1. ✅ Version-Nummern in allen 3 Dateien aktualisiert
2. ✅ Änderungen committed ("chore: Bump version to X.X.X")
3. ✅ Git Tag erstellt und gepusht
4. ✅ Lokaler Build mit Signierung (~5-10 Minuten)
5. ✅ GitHub Release erstellt
6. ✅ NSIS Installer (.exe + .sig) hochgeladen
7. ✅ latest.json für Auto-Update generiert und hochgeladen
8. ✅ Release published (kein Draft!)

**Ergebnis:** Release ist sofort verfügbar und Auto-Update funktioniert!

### 🔑 GitHub Token Setup (Einmalig)

**GitHub Token gespeichert in:** `.github-token` (lokal, nicht in Git!)

Das Token ist bereits in der Datei `.github-token` gespeichert.

**Falls die Datei verloren geht:**
1. Token aus vorheriger Session/Backup holen
2. Neue Datei erstellen: `echo "YOUR_TOKEN_HERE" > .github-token`
3. Token-Format: `ghp_...` (GitHub Personal Access Token)

⚠️ **WICHTIG:** VORHER Code-Änderungen committen (siehe Schritt 1 unten)!

### 🖥️ Cross-Platform Support (Windows, Linux, Mac)

Das `quick-release.sh` Script funktioniert VOLLAUTOMATISCH auf allen Plattformen:

**Alle Plattformen (Windows, Linux, Mac):**
- ✅ Verwendet `curl` mit GitHub REST API (Official Best Practice 2025)
- ✅ Korrekte Behandlung von Dateinamen mit Leerzeichen
- ✅ Funktioniert in Git Bash, WSL, Linux, macOS
- ✅ **Keine manuelle Eingabe nötig!**

**Technische Details:**
- Methode: `curl --data-binary @filename` mit GitHub Upload API
- Endpoint: `https://uploads.github.com/repos/OWNER/REPO/releases/ID/assets`
- Headers: `Authorization: Bearer TOKEN`, `Content-Type: application/octet-stream`
- Vorteile: Zuverlässig, plattformunabhängig, keine externen Dependencies

**Warum nicht gh CLI?**
- ❌ gh CLI hat bekannten Bug mit Leerzeichen in Dateinamen (Issue #10585)
- ❌ Ersetzt Leerzeichen mit Punkten → "Stiftung der DPolG" → "Stiftung.der.DPolG"
- ✅ curl funktioniert perfekt mit Spaces in Filenames

---

## 📋 Standard-Ablauf für JEDES Update (Manuell)

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
   "version": "1.7.5"
   ```

2. **src-tauri/Cargo.toml**
   ```toml
   version = "1.7.5"
   ```

3. **src-tauri/tauri.conf.json**
   ```json
   "version": "1.7.5"
   ```

**Dann committen:**
```bash
git add package.json src-tauri/Cargo.toml src-tauri/tauri.conf.json
git commit -m "chore: Bump version to 1.7.5"
```

---

### ✅ Schritt 3: Git Tag erstellen

```bash
# Tag mit Beschreibung
git tag -a v1.7.5 -m "Release v1.7.5"

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

**WICHTIG:** Build MUSS mit Signierung erfolgen, sonst fehlen die Update-Dateien!

```bash
# Mit Signierung (KORREKT - IMMER SO!)
export TAURI_SIGNING_PRIVATE_KEY="$(cat src-tauri/dpolg-signing.key)"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="dpolg2025"
npm run tauri build
```

**Signing Key Location:** `src-tauri/dpolg-signing.key`
**Password:** `dpolg2025`
**Public Key:** `dpolg-signing.key.pub` (für Verifizierung)

⚠️ **Ohne diese Environment Variables:**
- ❌ Build erstellt NUR `.exe` Datei
- ❌ KEINE `.exe.sig` (Signatur)
- ❌ **Auto-Update funktioniert NICHT!**

**Dauer:** ~5-10 Minuten

**Output-Verzeichnis:** `src-tauri/target/release/bundle/nsis/`

**Wichtige Dateien (nach erfolgreichem Build):**
```
nsis/
├── Stiftung der DPolG Buchungssystem_1.7.5_x64-setup.exe       ← NSIS Installer (6-8 MB)
└── Stiftung der DPolG Buchungssystem_1.7.5_x64-setup.exe.sig   ← Signatur (452 bytes)
```

⚠️ **BEIDE DATEIEN werden für Auto-Update benötigt!**

---

### ✅ Schritt 6: GitHub Release erstellen

#### Option A: Mit `gh` CLI (schneller)

```bash
cd src-tauri/target/release/bundle/nsis

gh release create v1.7.5 \
  --title "Stiftung der DPolG Buchungssystem v1.7.5" \
  --notes "## 🎉 Änderungen in v1.7.5

- ✅ [Beschreibung der Änderungen hier einfügen]

## 📥 Installation
**Windows:** Laden Sie die \`-setup.exe\` Datei herunter und installieren Sie die App.

## 🔄 Auto-Update
Wenn Sie bereits eine ältere Version installiert haben, wird automatisch ein Update-Dialog angezeigt." \
  *.exe \
  *.exe.sig
```

#### Option B: Manuell über GitHub Web UI

1. Öffnen: https://github.com/Maxwellbadger-1/dpolg-booking-modern/releases/new

2. **Tag auswählen:** `v1.7.5`

3. **Title:** `Stiftung der DPolG Buchungssystem v1.7.5`

4. **Beschreibung:**
   ```markdown
   ## 🎉 Änderungen in v1.7.5

   - ✅ [Beschreibung der Änderungen hier einfügen]

   ## 📥 Installation
   **Windows:** Laden Sie die `-setup.exe` Datei herunter und installieren Sie die App.

   ## 🔄 Auto-Update
   Wenn Sie bereits eine ältere Version installiert haben, wird automatisch ein Update-Dialog angezeigt.
   ```

5. **Dateien hochladen (Drag & Drop):**
   - ✅ `Stiftung der DPolG Buchungssystem_1.7.5_x64-setup.exe`
   - ✅ `Stiftung der DPolG Buchungssystem_1.7.5_x64-setup.exe.sig`

6. **"Publish release"** klicken

⚠️ **NICHT als Draft speichern!** Sonst funktioniert Auto-Update nicht.

---

### ✅ Schritt 7: Auto-Update testen

1. **Installierte App öffnen** (ältere Version z.B. 1.7.4)

2. **Update-Dialog sollte erscheinen:**
   ```
   🔄 Update verfügbar: v1.7.5
   Möchten Sie jetzt aktualisieren?
   ```

3. **"Ja" klicken** → App lädt Update herunter und installiert

4. **App neustartet automatisch** → Version 1.7.5 läuft

5. **Änderungen testen**

---

## 🚨 Häufige Fehler vermeiden

### ❌ FEHLER: Port bereits belegt
**Symptom:** `Error: Port 1423 already in use`
**Lösung:** Alle Dev-Server stoppen vor Build

### ❌ FEHLER: Dateien fehlen im Release
**Symptom:** Auto-Update funktioniert nicht
**Lösung:** BEIDE Dateien hochladen (.exe + .exe.sig)

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
- [ ] 2 Dateien vorhanden (*.exe, *.exe.sig)
- [ ] GitHub Release erstellt
- [ ] Beide Dateien hochgeladen
- [ ] Release published (nicht Draft!)
- [ ] Auto-Update in installierter App getestet

---

## 🎯 Versionsnummern-Schema

```
v1.7.5
  │ │ │
  │ │ └─ Patch: Bugfixes (QR-Code Fix, kleine Änderungen)
  │ └─── Minor: Neue Features (Month-spanning Bookings)
  └───── Major: Breaking Changes (komplettes Redesign)
```

**Beispiele:**
- Bugfix → `1.7.4` → `1.7.5`
- Neues Feature → `1.7.5` → `1.8.0`
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

## 💡 Warum NSIS statt MSI?

- ✅ **Auto-Update funktioniert besser** - Weniger Probleme mit UAC
- ✅ **Kleinere Dateigröße** - ~6 MB statt ~9 MB
- ✅ **Schnellere Installation**
- ✅ **Bessere User Experience** beim Update

**Hinweis:** Bestehende MSI-User müssen einmalig manuell auf NSIS umsteigen (alte Version deinstallieren, neue installieren). Danach funktioniert Auto-Update perfekt!

---

---

## 📤 Manueller File-Upload (Falls Quick-Release fehlschlägt)

Falls das automatische Quick-Release Script fehlschlägt, können die Dateien manuell hochgeladen werden:

### Option 1: Mit curl Script (EMPFOHLEN)

```bash
# Script erstellen
cat > upload-release.sh << 'EOF'
#!/bin/bash
# Upload files to GitHub Release
set -e

GITHUB_TOKEN=$(cat .github-token)
RELEASE_ID="YOUR_RELEASE_ID"  # z.B. 257283329
VERSION="YOUR_VERSION"         # z.B. 1.7.5
REPO="Maxwellbadger-1/dpolg-booking-modern"

cd "src-tauri/target/release/bundle/nsis"

# Upload .exe
curl -L -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer ${GITHUB_TOKEN}" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  -H "Content-Type: application/octet-stream" \
  "https://uploads.github.com/repos/${REPO}/releases/${RELEASE_ID}/assets?name=Stiftung.der.DPolG.Buchungssystem_${VERSION}_x64-setup.exe" \
  --data-binary "@Stiftung der DPolG Buchungssystem_${VERSION}_x64-setup.exe"

# Upload .sig
curl -L -X POST \
  -H "Accept: application/vnd.github+json" \
  -H "Authorization: Bearer ${GITHUB_TOKEN}" \
  -H "X-GitHub-Api-Version: 2022-11-28" \
  -H "Content-Type: application/octet-stream" \
  "https://uploads.github.com/repos/${REPO}/releases/${RELEASE_ID}/assets?name=Stiftung.der.DPolG.Buchungssystem_${VERSION}_x64-setup.exe.sig" \
  --data-binary "@Stiftung der DPolG Buchungssystem_${VERSION}_x64-setup.exe.sig"
EOF

chmod +x upload-release.sh
./upload-release.sh
```

### Option 2: GitHub Web UI (Drag & Drop)

1. Öffnen: https://github.com/Maxwellbadger-1/dpolg-booking-modern/releases
2. Release finden und auf "Edit" klicken
3. Dateien per Drag & Drop hochladen:
   - `Stiftung der DPolG Buchungssystem_X.X.X_x64-setup.exe`
   - `Stiftung der DPolG Buchungssystem_X.X.X_x64-setup.exe.sig`
4. "Update release" klicken

---

**Erstellt:** 2025-10-24
**Letzte Aktualisierung:** 2025-10-26 (curl mit GitHub API - Official Best Practice)
