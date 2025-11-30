# Claude Code Projekt-Richtlinien
## DPolG Buchungssystem - Tauri 2 + React + TypeScript

---

## Tech-Stack
- **Backend:** Tauri 2, Rust, PostgreSQL (Oracle Cloud)
- **Frontend:** React 18, TypeScript, TailwindCSS 4, Vite
- **DB:** PostgreSQL 16.11 via pgBouncer (Port 6432)

**Details:** [docs/DATABASE.md](../docs/DATABASE.md)

---

## Kritische Regeln

### 1. Tauri Parameter (WICHTIGSTE REGEL!)
```typescript
// Frontend: IMMER camelCase
invoke('update_booking', { bookingId: 1, guestName: "Max" })

// Backend: IMMER snake_case (Rust)
fn update_booking(booking_id: i64, guest_name: String)
```
Tauri konvertiert automatisch. NIEMALS snake_case im Frontend!

### 2. JSX Syntax
- Sibling-Elemente MÜSSEN in `<>...</>` gewrapped sein
- Closing Tags: LIFO (Last In First Out)
- Leere Elemente: `<input />`, `<img />`

### 3. Optimistic Updates
```typescript
// 1. UI sofort updaten (kein await)
setEntities(prev => prev.map(...))
// 2. Backend-Call
await invoke('update_entity', {...})
// 3. Bei Fehler: Rollback
```
NIEMALS `refreshData()` nach erfolgreichen Operationen!

### 4. Code Cleanup
Bei JEDER Änderung: Alten Code entfernen, ungenutzte Imports löschen.

### 5. UI-Integration (KRITISCH!)
**ALLE Backend-Funktionen MÜSSEN mit der UI verknüpft sein!**
- Neue Tauri Commands → Button/Dialog in UI erstellen
- Bestehende Funktionen → UI-Anbindung überprüfen
- NIEMALS "Stub"-Funktionen ohne UI-Zugang
- Test: Jede Funktion muss per UI erreichbar sein

---

## Befehle

### App starten
```bash
pkill -9 -f "tauri|cargo|vite|node|npm" 2>/dev/null || true; sleep 2; npm run tauri:dev
```

### Release erstellen
```bash
./quick-release.sh 1.7.5
```
**Details:** [docs/RELEASE.md](../docs/RELEASE.md)

---

## Code-Konventionen

### Naming
- **Rust:** `snake_case` Funktionen, `PascalCase` Structs
- **TypeScript:** `camelCase` Variablen, `PascalCase` Components

### Verboten
- `any` Type in TypeScript
- `.unwrap()` in Rust Commands
- String concatenation für SQL
- Browser-Dialoge (`alert`, `confirm`)
- Englische UI-Texte (immer Deutsch)

### Error Handling
```rust
fn my_command() -> Result<T, String> { }  // IMMER Result
```
```typescript
const data = await invoke<BookingType>('cmd', {});  // Explizite Types
```

---

## UI/UX

### Farben
- Backgrounds: `bg-slate-800`, `bg-slate-900`
- Primary: `bg-blue-600`
- Success: `bg-emerald-500`
- Error: `bg-red-500`

### Formate
- Datum: DD.MM.YYYY
- Währung: 1.234,56 €
- Sprache: Deutsch

---

## Git

### Commit Format
```
feat|fix|refactor(scope): Beschreibung
```

### Vor Commit prüfen
- `npm run lint` - ESLint Warnings
- `npm run build` - TypeScript Errors
- Drag & Drop funktioniert
- UI-Texte auf Deutsch

---

## Bekannte Probleme

| Problem | Fix |
|---------|-----|
| Tauri Parameter falsch | Frontend camelCase, Backend snake_case |
| JSX Adjacent Elements | `<>...</>` Wrapper |
| z-index Konflikte | Dropdowns z-[100] |
| Port blockiert | `pkill` vor Start |

---

## Dokumentation

- [docs/DATABASE.md](../docs/DATABASE.md) - PostgreSQL, SSH, Connection
- [docs/RELEASE.md](../docs/RELEASE.md) - Release-Prozess
- [docs/DEVELOPMENT.md](../docs/DEVELOPMENT.md) - Dev-Workflow, Ports

---

**Version:** 3.0 (Optimiert)
