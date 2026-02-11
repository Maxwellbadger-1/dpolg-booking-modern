# Claude Code Projekt-Richtlinien
## DPolG Buchungssystem - Tauri 2 + React 19 + TypeScript + PostgreSQL 16

---

## Kritische Regeln

### 1. Tauri Parameter (WICHTIGSTE REGEL!)
```typescript
// Frontend: IMMER camelCase
invoke('update_booking', { bookingId: 1, guestName: "Max" })

// Backend: IMMER snake_case (Rust)
fn update_booking(booking_id: i64, guest_name: String)
```

### 2. Optimistic Updates
```typescript
// 1. UI sofort updaten
setEntities(prev => prev.map(...))
// 2. Backend-Call
await invoke('update_entity', {...})
// 3. Bei Fehler: Rollback
```
NIEMALS `refreshData()` nach erfolgreichen Operationen!

### 3. JSX Syntax
Sibling-Elemente MÜSSEN in `<>...</>` gewrapped sein.

### 4. VERBOTEN
- `any` Type in TypeScript
- `.unwrap()` in Rust Commands
- String concatenation für SQL
- Browser-Dialoge (`alert`, `confirm`)
- Englische UI-Texte (immer Deutsch)

### 5. Code Cleanup
Bei JEDER Änderung: Alten Code entfernen, ungenutzte Imports löschen.

---

## Befehle

```bash
# App starten (empfohlen)
npx tauri dev

# Oder mit Script (Windows)
.\dev-start.ps1

# Release erstellen
./quick-release.ps1 1.x.x

# Vor Commit
npm run lint && npm run build
```

---

## Vor Commit prüfen
- `npm run lint` - keine Warnings
- `npm run build` - keine Errors
- Drag & Drop funktioniert
- UI-Texte auf Deutsch

---

## Core-Dokumentation

Diese Docs MÜSSEN bei Änderungen aktuell gehalten werden:

| Dokument | Inhalt | Wann updaten |
|----------|--------|--------------|
| [CHANGELOG.md](../CHANGELOG.md) | Versionshistorie | Jeder Release |
| [docs/architecture.md](../docs/architecture.md) | Systemarchitektur, Metriken | Architektur-Änderungen |
| [docs/Project_Status.md](../docs/Project_Status.md) | Feature-Status, Issues, Roadmap | Feature-Änderungen |
| [docs/Project_Spec.md](../docs/Project_Spec.md) | Anforderungen, UI/UX | Anforderungs-Änderungen |

---

## Anleitungen

- [docs/DATABASE.md](../docs/DATABASE.md) - PostgreSQL, Connection
- [docs/RELEASE.md](../docs/RELEASE.md) - Release-Prozess
- [docs/EMAIL_SYSTEM.md](../docs/EMAIL_SYSTEM.md) - E-Mail Provider Setup
