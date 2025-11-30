# Auto-Update System - GitHub Releases Workflow

## Übersicht

Das DPolG Buchungssystem verwendet das **Tauri Updater Plugin** für automatische Updates via **GitHub Releases**. Die App prüft beim Start automatisch auf neue Versionen und bietet dem User an, das Update zu installieren.

---

## ✅ Was wurde bereits konfiguriert

### 1. NPM Packages installiert
- `@tauri-apps/plugin-updater`
- `@tauri-apps/plugin-dialog`
- `@tauri-apps/plugin-process`

### 2. Rust Dependencies hinzugefügt (Cargo.toml)
```toml
tauri-plugin-updater = "2"
tauri-plugin-process = "2"
```

### 3. Plugins registriert (lib.rs)
```rust
.plugin(tauri_plugin_updater::Builder::new().build())
.plugin(tauri_plugin_process::init())
```

### 4. Updater konfiguriert (tauri.conf.json)
```json
{
  "plugins": {
    "updater": {
      "active": true,
      "endpoints": [
        "https://github.com/YOUR_USERNAME/dpolg-booking-modern/releases/latest/download/latest.json"
      ],
      "dialog": true,
      "pubkey": "YOUR_PUBLIC_KEY_WILL_BE_GENERATED_HERE"
    }
  }
}
```

### 5. Frontend Update-Check implementiert (App.tsx)
- Automatischer Check beim App-Start
- Nur in Production-Builds aktiv
- Progress-Anzeige beim Download
- Automatischer Neustart nach Installation

---

## 🔑 Signing Keys generieren (MANUELL durchführen!)

**WICHTIG:** Du musst die Signing Keys einmalig manuell generieren!

### Schritt 1: Keys erstellen

```bash
cd /Users/maximilianfegg/Desktop/Sicherungskopie\ DPolG\ Buchungssystem.nosynch/Claude\ Code/dpolg-booking-modern

# Erstelle .tauri Ordner im Home-Verzeichnis
mkdir -p ~/.tauri

# Generiere Signing Keys
npx @tauri-apps/cli signer generate -w ~/.tauri/dpolg-booking.key
```

**Du wirst nach einem Passwort gefragt:**
- Wähle ein sicheres Passwort
- Schreibe es auf! (Du brauchst es bei jedem Build)
- Das Passwort schützt den Private Key

**Output:**
```
Your keypair was generated successfully
Private: /Users/maximilianfegg/.tauri/dpolg-booking.key (Keep this secret!)
Public: dne8sZm... (Long string)
```

### Schritt 2: Public Key in tauri.conf.json eintragen

1. Kopiere den **Public Key** aus dem Output
2. Öffne `src-tauri/tauri.conf.json`
3. Ersetze `"YOUR_PUBLIC_KEY_WILL_BE_GENERATED_HERE"` mit dem Public Key:

```json
{
  "plugins": {
    "updater": {
      "active": true,
      "endpoints": [
        "https://github.com/maximilianfegg/dpolg-booking-modern/releases/latest/download/latest.json"
      ],
      "dialog": true,
      "pubkey": "dne8sZm...DEIN_PUBLIC_KEY_HIER"
    }
  }
}
```

4. Ersetze auch `YOUR_USERNAME` mit deinem GitHub-Username

### Schritt 3: Private Key sichern

**KRITISCH:** Der Private Key (`~/.tauri/dpolg-booking.key`) ist **GEHEIM**!

- ✅ **NIEMALS** in Git committen
- ✅ **NIEMALS** teilen oder veröffentlichen
- ✅ Backup an sicherem Ort (z.B. verschlüsselter USB-Stick)
- ✅ Bei Verlust: Neue Keys generieren & alle User müssen neu installieren

---

## 🚀 Release-Workflow (Updates bereitstellen)

### Vorbereitung

1. **Version erhöhen** in `src-tauri/tauri.conf.json`:
   ```json
   {
     "productName": "DPolG Buchungssystem",
     "version": "1.7.0",  // <- Erhöhen (z.B. 1.6.0 → 1.7.0)
     ...
   }
   ```

2. **Changelog erstellen** (optional aber empfohlen):
   - Erstelle `CHANGELOG.md` oder notiere Änderungen für Release Notes

### Production Build erstellen

```bash
cd /Users/maximilianfegg/Desktop/Sicherungskopie\ DPolG\ Buchungssystem.nosynch/Claude\ Code/dpolg-booking-modern

# Build für Production
npm run tauri build
```

**Während des Builds wirst du nach dem Passwort gefragt** (das du beim Key-Generieren gewählt hast)

**Output:**
Die Binaries werden erstellt in:
- macOS: `src-tauri/target/release/bundle/dmg/`
- Windows: `src-tauri/target/release/bundle/msi/`
- Universal: `src-tauri/target/release/bundle/`

**Wichtig:** Tauri erstellt automatisch auch die `latest.json` Datei für den Updater!

### GitHub Release erstellen

#### Option A: Via GitHub Web-Interface (Empfohlen für Anfang)

1. Gehe zu: https://github.com/maximilianfegg/dpolg-booking-modern/releases
2. Klicke auf **"Draft a new release"**
3. **Tag version:** `v1.7.0` (mit "v" Prefix!)
4. **Release title:** `Version 1.7.0` oder `v1.7.0 - Kurzbeschreibung`
5. **Describe this release:**
   ```markdown
   ## Was ist neu in v1.7.0

   - ✨ Feature: Neues Feature X
   - 🐛 Bugfix: Problem Y behoben
   - ⚡ Performance: Verbesserte Ladezeiten

   ## Installation

   Für neue User: DMG/MSI herunterladen und installieren
   Für bestehende User: Automatisches Update beim nächsten App-Start
   ```

6. **Attach binaries:**
   - Drag & Drop die `.dmg` Datei (macOS)
   - Drag & Drop die `.msi` Datei (Windows)
   - Drag & Drop die `latest.json` Datei (WICHTIG!)
   - **Optional:** Auch die `.app.tar.gz` und `.sig` Dateien hochladen

7. ✅ **Publish release** klicken

#### Option B: Via GitHub CLI (Schneller für fortgeschrittene)

```bash
# GitHub CLI installieren (falls noch nicht)
brew install gh

# Authentifizieren
gh auth login

# Release erstellen mit allen Files
gh release create v1.7.0 \
  --title "Version 1.7.0 - Beschreibung" \
  --notes "Release Notes hier..." \
  src-tauri/target/release/bundle/dmg/*.dmg \
  src-tauri/target/release/bundle/macos/*.app.tar.gz \
  src-tauri/target/release/bundle/macos/*.app.tar.gz.sig \
  src-tauri/target/release/latest.json
```

**WICHTIG:** Die `latest.json` muss im Release enthalten sein! Ohne sie funktioniert der Auto-Updater nicht!

---

## 🔄 Was passiert beim User?

### Beim nächsten App-Start:

1. **App startet** → Update-Check läuft im Hintergrund
2. **Neue Version gefunden:**
   - Confirmation Dialog erscheint:
     ```
     Update verfügbar: Version 1.7.0

     [Release Notes werden angezeigt]

     Jetzt installieren? Die App wird nach der Installation neu gestartet.

     [Abbrechen]  [Installieren]
     ```
3. **User klickt "Installieren":**
   - Toast: "Update wird heruntergeladen... 25%"
   - Download läuft (mit Progress)
   - Toast: "Update erfolgreich heruntergeladen!"
   - Toast: "Update installiert! App wird neu gestartet..."
   - Nach 2 Sekunden: App startet neu mit neuer Version

4. **User klickt "Abbrechen":**
   - Kein Update
   - Nächster Check beim nächsten App-Start

---

## 📋 Checkliste für jeden Release

- [ ] Version in `tauri.conf.json` erhöhen
- [ ] Production Build erstellen (`npm run tauri build`)
- [ ] Passwort für Private Key eingeben beim Build
- [ ] Binaries testen (lokal installieren und prüfen)
- [ ] GitHub Release erstellen mit:
  - [ ] Tag im Format `vX.Y.Z` (z.B. `v1.7.0`)
  - [ ] DMG/MSI Dateien hochladen
  - [ ] **`latest.json` hochladen** (KRITISCH!)
  - [ ] Release Notes schreiben
- [ ] Release veröffentlichen
- [ ] Testen: App auf einem Gerät mit alter Version starten → Update sollte angeboten werden

---

## 🐛 Troubleshooting

### Problem: "No update available" obwohl neue Version existiert

**Ursachen:**
- `latest.json` fehlt im GitHub Release
- GitHub Username falsch in `tauri.conf.json`
- Endpoint URL falsch
- Tag-Format falsch (muss `vX.Y.Z` sein)

**Lösung:**
```bash
# Console im Browser öffnen (Cmd+Option+I) und prüfen:
# "🔍 Checking for updates..."
# Fehler-Meldung lesen
```

### Problem: "Invalid signature" beim Update

**Ursachen:**
- Public Key in `tauri.conf.json` stimmt nicht mit Private Key überein
- Build wurde mit anderem Key signiert

**Lösung:**
- Neue Keys generieren
- Public Key in `tauri.conf.json` aktualisieren
- Neuen Build erstellen

### Problem: Download schlägt fehl

**Ursachen:**
- Firewall blockiert
- Keine Internet-Verbindung
- GitHub Release ist "Draft" statt "Published"

**Lösung:**
- Release als "Published" markieren
- Internet-Verbindung prüfen

---

## 🔐 Sicherheit

### Warum Signing Keys?

- **Verhindert Manipulation:** Nur mit dem Private Key signierte Updates werden installiert
- **Man-in-the-Middle Schutz:** Angreifer kann kein gefälschtes Update einspielen
- **Vertrauen:** User kann sicher sein dass Update von dir kommt

### Best Practices:

1. **Private Key niemals committen** (ist bereits in `.gitignore`)
2. **Passwort sicher aufbewahren** (z.B. Passwort-Manager)
3. **Bei Key-Verlust:** Alle User müssen App neu installieren
4. **GitHub Repository:** Private oder Public ist egal (Public ist sogar transparenter)

---

## 📝 Beispiel: Vollständiger Release-Workflow

```bash
# 1. Version erhöhen
# In tauri.conf.json: "version": "1.7.0"

# 2. Build erstellen
npm run tauri build
# → Passwort eingeben wenn gefragt

# 3. Binaries prüfen
open src-tauri/target/release/bundle/dmg/
# → DMG installieren und testen

# 4. GitHub Release via Web-Interface:
#    - https://github.com/maximilianfegg/dpolg-booking-modern/releases/new
#    - Tag: v1.7.0
#    - Upload: .dmg, .app.tar.gz, .app.tar.gz.sig, latest.json
#    - Publish

# 5. Testen
#    - App mit alter Version starten
#    - Update sollte angeboten werden
```

---

## 🎯 Nächste Schritte

1. **Signing Keys generieren** (siehe oben - muss manuell gemacht werden)
2. **Public Key in tauri.conf.json eintragen**
3. **GitHub Username in tauri.conf.json ersetzen**
4. **Ersten Production Build erstellen** (`npm run tauri build`)
5. **Ersten GitHub Release erstellen** (v1.6.0)
6. **Update-System testen** (alte Version → neue Version)

---

## 📚 Weitere Ressourcen

- [Tauri Updater Dokumentation](https://v2.tauri.app/plugin/updater/)
- [GitHub Releases Dokumentation](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases)
- [Tauri CLI Dokumentation](https://v2.tauri.app/reference/cli/)
