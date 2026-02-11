# 🔄 Multi-Device Development Workflow
## Git-basierter Workflow zwischen Windows PC und MacBook

---

## 🚀 Quick Start: Gerätewechsel (1-2 Minuten)

### Vor dem Gerätewechsel:
```bash
# Änderungen committen und pushen
git add .
git commit -m "feat: Beschreibung der Änderungen"
git push origin main
```

### Auf neuem Gerät (MacBook/Windows):
```bash
# Neueste Änderungen holen
git pull origin main

# Development starten
npm run tauri:dev
```

**Das war's!** Keine manuellen Kopien, kein USB-Stick, keine ZIP-Dateien.

---

## 📖 Detaillierte Anleitung

### Erste Einrichtung auf neuem Gerät

#### Windows PC (erstes Setup):
1. **Repository klonen:**
   ```bash
   git clone https://github.com/Maxwellbadger-1/dpolg-booking-modern.git
   cd dpolg-booking-modern
   ```

2. **Environment-Datei erstellen:**
   ```bash
   # .env aus .env.example erstellen
   copy .env.example .env
   # Jetzt .env editieren und DATABASE_URL eintragen!
   ```

3. **Dependencies installieren:**
   ```bash
   npm install
   ```

4. **Starten:**
   ```bash
   npm run tauri:dev
   ```
   Beim ersten Mal: ~5-10 Min (Rust-Build)

**Vollständige Anleitung:** [docs/WINDOWS_BUILD_GUIDE.md](WINDOWS_BUILD_GUIDE.md)

---

#### MacBook (erstes Setup):
1. **Repository klonen:**
   ```bash
   git clone https://github.com/Maxwellbadger-1/dpolg-booking-modern.git
   cd dpolg-booking-modern
   ```

2. **Environment-Datei erstellen:**
   ```bash
   # .env aus .env.example erstellen
   cp .env.example .env
   # Jetzt .env editieren und DATABASE_URL eintragen!
   ```

3. **Dependencies installieren:**
   ```bash
   npm install
   ```

4. **Starten:**
   ```bash
   npm run tauri:dev
   ```
   Beim ersten Mal: ~5-10 Min (Rust-Build)

**Vollständige Anleitung:** [docs/MACOS_SETUP.md](MACOS_SETUP.md)

---

## 🔁 Täglicher Workflow

### Morgens (Arbeit beginnen):
```bash
# 1. Neueste Änderungen holen
git pull origin main

# 2. Development starten
npm run tauri:dev
```

### Abends (Arbeit beenden):
```bash
# 1. Änderungen stagen
git add .

# 2. Commit erstellen
git commit -m "feat: Beschreibung der Arbeit"

# 3. Push zu GitHub
git push origin main
```

### Gerät wechseln (z.B. Windows → MacBook):
```bash
# Auf Windows PC:
git add .
git commit -m "WIP: Arbeit auf MacBook fortsetzen"
git push origin main

# Auf MacBook:
git pull origin main
npm run tauri:dev
```

**Dauer:** ~1-2 Minuten (statt 10+ Min manuelles Kopieren!)

---

## ✅ Wichtige Regeln

### ✅ Was wird via Git synchronisiert:
- ✅ Source Code (TypeScript, Rust, React)
- ✅ Konfigurationsdateien (package.json, Cargo.toml, etc.)
- ✅ Dokumentation (.md Dateien)
- ✅ Icons und Assets (public/)

### ❌ Was wird NICHT synchronisiert:
- ❌ `node_modules/` (~323 MB) - wird via `npm install` generiert
- ❌ `src-tauri/target/` (~11 GB) - wird via `cargo build` generiert
- ❌ `.env` - bleibt lokal (Credentials!)
- ❌ `dist/` - Build-Output

**Grund:** Diese Dateien werden lokal generiert und sind plattform-spezifisch.

---

## 💡 Best Practices

### Commit Messages (Deutsch):
```bash
git commit -m "feat: Neue Buchungsfunktion für Gäste"
git commit -m "fix: Validierung bei Raumbuchung korrigiert"
git commit -m "refactor: BookingList optimiert"
git commit -m "docs: MACOS_SETUP.md hinzugefügt"
```

### Branch-Strategie:
```bash
# Feature entwickeln
git checkout -b feature/neue-funktion
# ... Arbeit ...
git commit -m "feat: Neue Funktion implementiert"

# Merge zu main
git checkout main
git merge feature/neue-funktion
git push origin main
```

### .env synchronisieren:
```bash
# ❌ NIEMALS .env committen!
# ✅ Stattdessen: .env.example anpassen und committen

# Auf Windows PC:
# .env.example mit neuen Variablen erweitern
git add .env.example
git commit -m "docs: .env.example erweitert"
git push

# Auf MacBook:
git pull
# Jetzt .env manuell anpassen
```

---

## 🐛 Troubleshooting

### Problem: "Merge Conflict"
**Ursache:** Beide Geräte haben dieselbe Datei geändert.

**Lösung:**
```bash
git pull origin main
# Konflikt in der Datei manuell lösen
git add .
git commit -m "fix: Merge conflict resolved"
git push origin main
```

### Problem: "Your branch is behind 'origin/main'"
**Lösung:**
```bash
git pull origin main
```

### Problem: "Changes not staged for commit"
**Lösung:**
```bash
git add .
git commit -m "WIP: Changes before switch"
git push origin main
```

### Problem: Dependencies fehlen nach git pull
**Lösung:**
```bash
# Wenn package.json geändert wurde:
npm install

# Wenn Cargo.toml geändert wurde:
cd src-tauri
cargo check
cd ..
```

---

## 📊 Vergleich: Vorher vs. Nachher

| Aspekt | Vorher (Manuelles Kopieren) | Nachher (Git-Workflow) |
|--------|----------------------------|------------------------|
| **Projekt-Größe** | 11 GB (mit Build-Artefakten) | ~7 MB (nur Source Code) |
| **Transfer-Zeit** | 10-15 Min (USB/Kopieren) | 1-2 Min (git pull) |
| **Fehleranfälligkeit** | Hoch (vergessene Dateien) | Gering (Git tracked alles) |
| **Erstes Setup** | 15+ Min (Kopieren + Setup) | 5-10 Min (Clone + npm install) |
| **Täglicher Wechsel** | 10-15 Min (Kopieren) | 1-2 Min (git pull/push) |
| **.env Sync** | Manuell kopieren | Bleibt lokal (sicherer!) |

**Zeit-Ersparnis:** ~90% bei täglichem Gerätewechsel!

---

## 🔗 Weiterführende Dokumentation

- **Windows Setup:** [docs/WINDOWS_BUILD_GUIDE.md](WINDOWS_BUILD_GUIDE.md)
- **macOS Setup:** [docs/MACOS_SETUP.md](MACOS_SETUP.md)
- **Development Workflow:** [docs/DEVELOPMENT.md](DEVELOPMENT.md)
- **Release Process:** [docs/RELEASE.md](RELEASE.md)

---

## 💾 Lokales Cleanup (optional)

Wenn du Speicherplatz freigeben willst:

```bash
# Windows:
.\cleanup.ps1

# macOS/Linux:
./cleanup.sh
```

Dies löscht ~11 GB Build-Artefakte (`src-tauri/target/`).
Beim nächsten `npm run tauri:dev` werden sie neu generiert.

**Empfehlung:** Nur bei Speicherplatzmangel ausführen!

---

## ❓ FAQ

### Muss ich immer committen bevor ich das Gerät wechsle?
**Ja!** Git tracked nur committete Änderungen.

### Was passiert mit .env Dateien?
**Sie bleiben lokal.** Jedes Gerät hat seine eigene .env Datei. Das ist sicherer!

### Kann ich auch ohne Internet entwickeln?
**Ja!** Git funktioniert offline. Nur push/pull braucht Internet.

### Was wenn ich aus Versehen .env committed habe?
```bash
# .env aus Git History entfernen
git rm --cached .env
git commit -m "fix: Remove .env from git"
git push origin main
```

### Wie groß ist das Git Repository?
**~10.94 MB** (ohne Build-Artefakte). Sehr klein!

---

## 📝 Cheat Sheet

```bash
# MORGENS
git pull origin main
npm run tauri:dev

# ABENDS
git add .
git commit -m "feat: Beschreibung"
git push origin main

# GERÄTEWECHSEL
# Altes Gerät:
git push origin main

# Neues Gerät:
git pull origin main

# BEI PROBLEMEN
git status              # Was ist geändert?
git pull                # Neueste Version holen
npm install             # Dependencies aktualisieren
cargo check             # Rust Dependencies prüfen
```

---

## ✨ Zusammenfassung

**Git-basierter Workflow = Schneller, sicherer, professioneller!**

- ✅ Nur ~7 MB Source Code synchronisieren
- ✅ 1-2 Min statt 10+ Min pro Gerätewechsel
- ✅ Keine vergessenen Dateien
- ✅ .env bleibt lokal (sicherer)
- ✅ Versionskontrolle inklusive (Rollbacks möglich!)
- ✅ Professioneller Workflow wie in der Industrie

**Happy Coding! 🚀**
