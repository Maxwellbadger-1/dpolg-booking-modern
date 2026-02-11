# Development Workflow

## App starten

```bash
pkill -9 -f "tauri|cargo|vite|node|npm" 2>/dev/null || true; sleep 2; npm run tauri:dev
```

**Immer zuerst killen, dann starten!**

---

## Ports

- **1420** - Vite Dev Server
- **1421** - HMR (Hot Module Replacement)

**Config-Dateien:**
- vite.config.ts
- tauri.conf.json
- package.json (predev script)

---

## Scripts

```json
{
  "predev": "kill-port 1420 1421 || echo 'Ports already free'",
  "dev": "vite",
  "tauri:dev": "npm run predev && tauri dev"
}
```

---

## Wichtige Regeln

### Port-Konsistenz
ALLE Configs MÜSSEN denselben Port (1420) verwenden!

```typescript
// vite.config.ts
server: {
  port: 1420,
  strictPort: true  // Fail-fast!
}
```

### NIEMALS
- `npm run tauri dev` direkt (Port-Konflikte)
- Mehrere Dev-Server parallel
- `strictPort: false`

---

## Troubleshooting

### Port blockiert
```bash
pkill -9 -f "tauri|cargo|vite|node|npm"
```

### Cargo hängt
```bash
cd src-tauri && cargo clean
# Dann neu starten (~2-3 Min Build)
```

---

## Regression Testing

### Vor jedem Commit prüfen
1. Drag & Drop im Tapechart
2. Neue Buchung erstellen
3. `npm run build` ohne Fehler

### Playwright Tests
```bash
npx playwright test
npx playwright test tests/critical-regression.spec.ts
```

---

## Git Workflow

```bash
# Feature Branch
git checkout -b feature/neue-funktion

# Entwickeln + Testen
npm run build
npx playwright test

# Merge
git checkout main
git merge feature/neue-funktion
```

---

## 🔄 Multi-Geräte-Entwicklung

### Quick Reference: Gerätewechsel

```bash
# Vor Wechsel (auf aktuellem Gerät):
git add .
git commit -m "feat: Beschreibung"
git push origin main

# Nach Wechsel (auf neuem Gerät):
git pull origin main
npm run tauri:dev
```

**Dauer:** ~1-2 Minuten (statt 10+ Min manuelles Kopieren)

### Plattform-spezifische Setup-Anleitungen:

- **Windows:** [docs/WINDOWS_BUILD_GUIDE.md](WINDOWS_BUILD_GUIDE.md)
- **macOS:** [docs/MACOS_SETUP.md](MACOS_SETUP.md)
- **Workflow-Details:** [docs/DEVICE_SYNC_WORKFLOW.md](DEVICE_SYNC_WORKFLOW.md)

### Wichtig:

**✅ Was wird via Git synchronisiert:**
- Source Code (TypeScript, Rust, React)
- Config-Dateien (package.json, Cargo.toml)
- Dokumentation

**❌ Was bleibt lokal:**
- `node_modules/` (~323 MB) - via `npm install` generiert
- `src-tauri/target/` (~11 GB) - via `cargo build` generiert
- `.env` - bleibt lokal (Credentials!)
- `dist/` - Build-Output

### Troubleshooting Multi-Device:

```bash
# Dependencies nach git pull aktualisieren:
npm install

# Bei Cargo-Problemen:
cd src-tauri && cargo clean && cd ..

# Alte Prozesse killen:
pkill -9 -f "tauri|cargo|vite|node|npm"
```
