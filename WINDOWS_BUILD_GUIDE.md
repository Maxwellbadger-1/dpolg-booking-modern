# 🪟 Windows Build Anleitung
## DPolG Buchungssystem - Windows Executable erstellen

---

## ✅ Voraussetzungen auf Windows PC

### 1. Node.js installieren
- Download: https://nodejs.org/en/download
- Version: LTS (Latest) - mindestens v18+
- **Wichtig:** Bei Installation "Add to PATH" anhaken!

### 2. Rust installieren
- Download: https://www.rust-lang.org/tools/install
- Führe `rustup-init.exe` aus
- Wähle: `1) Proceed with standard installation`
- **Wichtig:** Nach Installation Terminal NEU STARTEN!

### 3. Microsoft C++ Build Tools installieren
- Download: https://visualstudio.microsoft.com/visual-cpp-build-tools/
- Installiere "Desktop development with C++"
- **ODER:** Visual Studio 2022 Community Edition

### 4. WebView2 Runtime (sollte schon installiert sein)
- Windows 11: Vorinstalliert
- Windows 10: https://developer.microsoft.com/en-us/microsoft-edge/webview2/

### 5. Git installieren (optional, für Clone)
- Download: https://git-scm.com/download/win

---

## 📥 Schritt 1: Code auf Windows PC bekommen

### Option A: Via Git (empfohlen)
```bash
git clone https://github.com/Maxwellbadger-1/dpolg-booking-modern.git
cd dpolg-booking-modern
```

### Option B: Als ZIP-Datei
1. Download: https://github.com/Maxwellbadger-1/dpolg-booking-modern/archive/refs/heads/main.zip
2. Entpacke die ZIP-Datei
3. Öffne Ordner in Terminal/PowerShell

---

## 🔧 Schritt 2: Dependencies installieren

Öffne **PowerShell** (oder **cmd**) im Projekt-Ordner:

```bash
# Node.js Dependencies installieren
npm install --legacy-peer-deps

# Rust/Cargo Check (optional, zum Testen)
cd src-tauri
cargo check
cd ..
```

**Erwartete Dauer:** 2-5 Minuten

---

## 🏗️ Schritt 3: Production Build erstellen

```bash
npm run tauri build
```

**Erwartete Dauer:** 15-30 Minuten (beim ersten Mal)

---

## 📍 Schritt 4: Build-Artefakte finden

Nach erfolgreichem Build findest du die Windows Installer hier:

### .msi Installer (empfohlen)
```
src-tauri\target\release\bundle\msi\Stiftung der DPolG Buchungssystem_1.6.8_x64_en-US.msi
```

### .exe Installer (NSIS)
```
src-tauri\target\release\bundle\nsis\Stiftung der DPolG Buchungssystem_1.6.8_x64-setup.exe
```

### Portable .exe
```
src-tauri\target\release\dpolg-booking-modern.exe
```

---

## 🐛 Troubleshooting

### Problem: "cargo: command not found"
**Lösung:** Terminal nach Rust-Installation neu starten!

### Problem: "LINK : fatal error LNK1181: cannot open input file"
**Lösung:** Microsoft C++ Build Tools installieren (siehe oben)

### Problem: "Error: WebView2 not found"
**Lösung:** WebView2 Runtime installieren (siehe oben)

### Problem: npm ERR! peer dependency errors
**Lösung:** `--legacy-peer-deps` Flag verwenden (wie oben gezeigt)

### Problem: Compilation errors mit "unresolved module"
**Lösung:**
```bash
cd src-tauri
cargo clean
cd ..
npm run tauri build
```

---

## 🚀 Schnell-Setup Skript

Erstelle `build-windows.bat` im Projekt-Root:

```batch
@echo off
echo ========================================
echo   DPolG Buchungssystem - Windows Build
echo ========================================
echo.

echo [1/3] Installing Node.js dependencies...
call npm install --legacy-peer-deps
if %errorlevel% neq 0 (
    echo ERROR: npm install failed!
    pause
    exit /b 1
)

echo.
echo [2/3] Cleaning Rust build cache...
cd src-tauri
call cargo clean
cd ..

echo.
echo [3/3] Building Windows application...
call npm run tauri build
if %errorlevel% neq 0 (
    echo ERROR: Build failed!
    pause
    exit /b 1
)

echo.
echo ========================================
echo   BUILD SUCCESSFUL!
echo ========================================
echo.
echo Find your installers at:
echo   src-tauri\target\release\bundle\msi\
echo   src-tauri\target\release\bundle\nsis\
echo.
pause
```

Dann einfach ausführen:
```bash
build-windows.bat
```

---

## 📝 Nach dem Build

### Installer testen
1. Führe die `.msi` Datei aus
2. Installiere die App
3. Teste alle Funktionen

### Installer signieren (optional)
Für Production solltest du die `.msi` mit einem Code Signing Certificate signieren:
```bash
signtool sign /f MyCertificate.pfx /p password /tr http://timestamp.digicert.com /td sha256 /fd sha256 "Stiftung der DPolG Buchungssystem_1.6.8_x64_en-US.msi"
```

---

## 🔄 Für zukünftige Builds

Bei Code-Änderungen:
1. Ändere Version in `package.json` und `src-tauri/tauri.conf.json`
2. Führe `npm run tauri build` aus
3. Neue Installer sind fertig!

**Tipp:** Speichere `build-windows.bat` für schnelle Rebuilds!

---

## ❓ Support

Bei Problemen:
- Stelle sicher alle Prerequisites sind installiert
- Checke `cargo --version` und `node --version`
- Siehe Troubleshooting Sektion oben
