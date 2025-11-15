# 📦 GitHub Release Upload - v1.7.4

## ✅ Dateien bereit zum Upload

**Location:** `src-tauri/target/release/bundle/msi/`

**Dateien:**
1. ✅ `Stiftung der DPolG Buchungssystem_1.7.4_x64_en-US.msi` (8.7 MB)
2. ✅ `Stiftung der DPolG Buchungssystem_1.7.4_x64_en-US.msi.sig` (452 bytes)

---

## 🚀 Schritt-für-Schritt Anleitung

### 1. GitHub Releases öffnen
**URL:** https://github.com/Maxwellbadger-1/dpolg-booking-modern/releases/new

### 2. Release Details ausfüllen

**Tag:** `v1.7.4` (sollte bereits vorhanden sein)

**Title:** `Stiftung der DPolG Buchungssystem v1.7.4`

**Description:**
```markdown
## 🎉 Änderungen in v1.7.4

- ✅ QR-Code wird jetzt zentriert in Rechnungen angezeigt (keine "für Zahlung" Text mehr)
- ✅ Month-spanning Buchungen funktionieren über Monatsgrenzen hinweg
- ✅ Visuelle Indikatoren (Pfeile) für fortlaufende Buchungen
- ✅ Template-Dateien bereinigt und optimiert

## 📥 Installation
**Windows:** Laden Sie die `.msi` Datei herunter und installieren Sie die App.

## 🔄 Auto-Update
Wenn Sie bereits eine ältere Version installiert haben, wird automatisch ein Update-Dialog angezeigt.

## 🔑 Wichtig: Neuer Signing Key
Dieses Release verwendet einen neuen Signing Key ohne Passwort für einfachere zukünftige Updates.
```

### 3. Dateien hochladen

**Drag & Drop** die beiden Dateien in den "Attach binaries" Bereich:

From: `C:\Users\maxfe\Desktop\DPolG Buchungssystem\dpolg-booking-modern\src-tauri\target\release\bundle\msi\`

1. `Stiftung der DPolG Buchungssystem_1.7.4_x64_en-US.msi`
2. `Stiftung der DPolG Buchungssystem_1.7.4_x64_en-US.msi.sig`

⚠️ **WICHTIG:** Beide Dateien MÜSSEN hochgeladen werden!

### 4. Release Settings

- ✅ **Set as the latest release** (AKTIVIERT)
- ❌ **Set as a pre-release** (DEAKTIVIERT)
- ❌ **Create a discussion** (Optional)

### 5. Publish Release

Klicken Sie auf **"Publish release"** (NICHT "Save draft"!)

---

## ✅ Nach dem Release

### Release URL prüfen:
https://github.com/Maxwellbadger-1/dpolg-booking-modern/releases/tag/v1.7.4

### Dateien sollten erreichbar sein unter:
- MSI: `https://github.com/Maxwellbadger-1/dpolg-booking-modern/releases/download/v1.7.4/Stiftung.der.DPolG.Buchungssystem_1.7.4_x64_en-US.msi`
- SIG: `https://github.com/Maxwellbadger-1/dpolg-booking-modern/releases/download/v1.7.4/Stiftung.der.DPolG.Buchungssystem_1.7.4_x64_en-US.msi.sig`

(Leerzeichen werden in URLs zu Punkten!)

---

## 🧪 Auto-Update testen

1. Installierte App öffnen (v1.7.3 oder älter)
2. Update-Dialog sollte erscheinen: "Update verfügbar: v1.7.4"
3. "Ja" klicken → App lädt Update herunter
4. App neustartet → v1.7.4 läuft
5. QR-Code in Rechnung testen → sollte zentriert sein

---

**Erstellt:** 2025-10-25
**Build:** Erfolgreich ✅
**Signierung:** Erfolgreich ✅
