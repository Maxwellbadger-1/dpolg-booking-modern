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
