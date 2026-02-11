# 🍎 macOS Setup Anleitung
## DPolG Buchungssystem - Development & Build auf macOS

---

## ✅ Voraussetzungen auf macOS

### 1. Homebrew installieren (Package Manager)
```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
```

Nach Installation:
```bash
brew --version  # Sollte Version anzeigen
```

### 2. Node.js installieren
```bash
# Via Homebrew (empfohlen)
brew install node

# Oder Download: https://nodejs.org/en/download
# Version: LTS (Latest) - mindestens v18+
```

Prüfen:
```bash
node --version   # Sollte v18+ anzeigen
npm --version
```

### 3. Rust installieren
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

Wähle: `1) Proceed with standard installation`

Nach Installation:
```bash
source $HOME/.cargo/env
rustc --version   # Sollte Version anzeigen
cargo --version
```

### 4. Xcode Command Line Tools installieren
```bash
xcode-select --install
```

**Wichtig:** Dies installiert C/C++ Compiler, die für Rust/Tauri nötig sind.

Prüfen:
```bash
xcode-select -p
# Sollte /Library/Developer/CommandLineTools anzeigen
```

### 5. Git installieren (meist vorinstalliert)
```bash
git --version

# Falls nicht installiert:
brew install git
```

---

## 📥 Schritt 1: Repository klonen

```bash
# Via HTTPS (empfohlen)
git clone https://github.com/Maxwellbadger-1/dpolg-booking-modern.git
cd dpolg-booking-modern

# Oder via SSH (wenn eingerichtet)
git clone git@github.com:Maxwellbadger-1/dpolg-booking-modern.git
cd dpolg-booking-modern
```

---

## 🔧 Schritt 2: Environment-Datei einrichten

```bash
# .env aus Template erstellen
cp .env.example .env

# .env editieren (z.B. mit nano)
nano .env
```

**Wichtig:** Trage die PostgreSQL Connection String ein:
```env
DATABASE_URL=postgres://dpolg_admin:DEIN_PASSWORT@141.147.3.123:6432/dpolg_booking
```

Speichern und schließen:
- `Ctrl + X` → `Y` → `Enter`

**Hinweis:** Die .env Datei bleibt lokal und wird NICHT via Git synchronisiert!

---

## 🏗️ Schritt 3: Dependencies installieren

```bash
# Node.js Dependencies installieren
npm install

# Rust Dependencies prüfen (optional)
cd src-tauri
cargo check
cd ..
```

**Erwartete Dauer:** 2-5 Minuten

---

## 🚀 Schritt 4: Development starten

```bash
npm run tauri:dev
```

**Erwartete Dauer (erstes Mal):** 5-10 Minuten
- Rust-Dependencies werden kompiliert
- Tauri-App wird gebaut
- Vite Dev-Server startet

**Ab dem zweiten Mal:** ~30 Sekunden (Incremental Builds)

---

## 📦 Production Build erstellen (optional)

```bash
npm run tauri build
```

**Erwartete Dauer:** 15-30 Minuten (beim ersten Mal)

### Build-Artefakte finden:

#### .dmg Installer (empfohlen)
```
src-tauri/target/release/bundle/dmg/Stiftung der DPolG Buchungssystem_1.8.2_universal.dmg
```

#### .app Bundle
```
src-tauri/target/release/bundle/macos/Stiftung der DPolG Buchungssystem.app
```

#### Portable Binary
```
src-tauri/target/release/dpolg-booking-modern
```

---

## 🐛 Troubleshooting

### Problem: "rustc: command not found"
**Lösung:** Terminal nach Rust-Installation neu starten oder:
```bash
source $HOME/.cargo/env
```

### Problem: "xcode-select error"
**Lösung:** Xcode Command Line Tools installieren:
```bash
xcode-select --install
```

### Problem: "xcrun: error: invalid active developer path"
**Lösung:** Nach macOS-Update CLT neu installieren:
```bash
xcode-select --install
```

### Problem: npm ERR! peer dependency errors
**Lösung:** Legacy peer deps verwenden:
```bash
npm install --legacy-peer-deps
```

### Problem: Port 1420 already in use
**Lösung:** Alte Prozesse killen:
```bash
pkill -9 -f "tauri|cargo|vite|node|npm"
sleep 2
npm run tauri:dev
```

### Problem: Compilation errors mit "unresolved module"
**Lösung:**
```bash
cd src-tauri
cargo clean
cd ..
npm run tauri:dev
```

### Problem: PostgreSQL Connection refused
**Lösung:** Prüfe .env Datei:
```bash
cat .env
# DATABASE_URL sollte korrekte Credentials enthalten
```

---

## 🚀 Schnell-Setup Skript

Erstelle `setup.sh` im Projekt-Root:

```bash
#!/bin/bash
set -e

echo "=========================================="
echo "  DPolG Buchungssystem - macOS Setup"
echo "=========================================="
echo

echo "[1/4] Checking prerequisites..."
command -v node >/dev/null 2>&1 || { echo "❌ Node.js not found. Install with: brew install node"; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "❌ Rust not found. Install from: https://rustup.rs"; exit 1; }
echo "✅ Prerequisites OK"
echo

echo "[2/4] Installing Node.js dependencies..."
npm install
echo

echo "[3/4] Setting up environment file..."
if [ ! -f .env ]; then
    cp .env.example .env
    echo "⚠️  Please edit .env and add your DATABASE_URL!"
    echo "   Then run: npm run tauri:dev"
else
    echo "✅ .env already exists"
fi
echo

echo "[4/4] Checking Rust setup..."
cd src-tauri
cargo check
cd ..
echo

echo "=========================================="
echo "  SETUP COMPLETE!"
echo "=========================================="
echo
echo "Next steps:"
echo "  1. Edit .env and add DATABASE_URL"
echo "  2. Run: npm run tauri:dev"
echo
```

Ausführbar machen und starten:
```bash
chmod +x setup.sh
./setup.sh
```

---

## 🔄 Multi-Geräte-Workflow

### Zwischen Windows und macOS wechseln:

```bash
# Auf macOS: Änderungen pushen
git add .
git commit -m "feat: Änderungen beschreiben"
git push origin main

# Auf Windows: Änderungen holen
git pull origin main
npm run tauri:dev
```

**Mehr Details:** [docs/DEVICE_SYNC_WORKFLOW.md](DEVICE_SYNC_WORKFLOW.md)

---

## 📝 Nach dem Setup

### Täglicher Workflow:
```bash
# Morgens: Neueste Änderungen holen
git pull origin main

# Development starten
npm run tauri:dev

# Abends: Änderungen committen
git add .
git commit -m "feat: Beschreibung"
git push origin main
```

### Projekt aufräumen (optional):
```bash
# Cleanup-Script erstellen
cat > cleanup.sh << 'EOF'
#!/bin/bash
echo "Cleaning up build artifacts..."
cd src-tauri
cargo clean --release
cd ..
rm -rf .vite dist
echo "Cleanup complete! (~11 GB freed)"
EOF

chmod +x cleanup.sh
./cleanup.sh
```

---

## 🔗 Weiterführende Dokumentation

- **Multi-Device Workflow:** [docs/DEVICE_SYNC_WORKFLOW.md](DEVICE_SYNC_WORKFLOW.md)
- **Windows Setup:** [docs/WINDOWS_BUILD_GUIDE.md](WINDOWS_BUILD_GUIDE.md)
- **Development Workflow:** [docs/DEVELOPMENT.md](DEVELOPMENT.md)
- **Release Process:** [docs/RELEASE.md](RELEASE.md)

---

## ⚡ Performance-Tipps

### Schnellerer Rust-Build:
```bash
# ~/.cargo/config.toml erstellen (optional)
mkdir -p ~/.cargo
cat > ~/.cargo/config.toml << 'EOF'
[build]
jobs = 8  # Anzahl CPU-Cores anpassen

[profile.dev]
split-debuginfo = "unpacked"
EOF
```

### Node.js Performance:
```bash
# npm Cache prüfen
npm cache verify

# Bei Problemen: Cache leeren
npm cache clean --force
```

---

## 🍎 macOS-spezifische Hinweise

### Apple Silicon (M1/M2/M3) vs. Intel:
- **Apple Silicon:** Builds sind ~2x schneller
- **Intel:** Vollständig unterstützt
- **Universal Binary:** `npm run tauri build` erstellt automatisch Universal Binaries

### Firewall & Security:
Bei erstem Start:
1. macOS fragt nach **Keychain-Zugriff** → Erlauben
2. macOS fragt nach **Netzwerk-Zugriff** → Erlauben

### Signing (optional):
Für Distribution außerhalb App Store:
```bash
# Mit Apple Developer Account
codesign --sign "Developer ID" "Stiftung der DPolG Buchungssystem.app"
```

---

## ❓ Support

Bei Problemen:
- Stelle sicher alle Prerequisites sind installiert
- Prüfe: `node --version`, `cargo --version`, `xcode-select -p`
- Siehe Troubleshooting Sektion oben
- Konsultiere [docs/DEVELOPMENT.md](DEVELOPMENT.md)

---

## ✅ Cheat Sheet

```bash
# SETUP (einmalig)
brew install node
curl https://sh.rustup.rs | sh
xcode-select --install
git clone https://github.com/Maxwellbadger-1/dpolg-booking-modern.git
cd dpolg-booking-modern
cp .env.example .env
npm install

# DEVELOPMENT (täglich)
git pull
npm run tauri:dev

# BUILD (für Release)
npm run tauri build

# CLEANUP (bei Speicherplatzmangel)
cd src-tauri && cargo clean && cd ..

# TROUBLESHOOTING
pkill -9 -f "tauri|cargo|vite|node|npm"
cargo clean
npm install
```

**Happy Coding! 🚀**
