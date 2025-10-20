# Claude Code Projekt-Richtlinien
## Stiftung der DPolG Buchungssystem - Modern Tauri React Application

---

## 🎯 Projekt-Vision

Ein modernes, performantes Hotel-Buchungssystem mit intuitiver Tape Chart Visualisierung, entwickelt mit Tauri 2 + React + TypeScript. Fokus auf Benutzerfreundlichkeit, Geschwindigkeit und Wartbarkeit.

---

## 🚨 KRITISCHE REGELN

### 1. Tauri + Serde camelCase/snake_case

**Golden Rules:**
- Frontend sendet IMMER camelCase (keine Ausnahmen!)
- Backend annotiert neue Felder mit `#[serde(rename = "camelCase")]`
- NIEMALS camelCase und snake_case mixen - bricht die Auto-Konvertierung!
- Bei neuen Structs: `#[serde(rename_all = "camelCase")]` verwenden

**Beispiel:**
```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NewEntity {
    pub first_name: String,  // → firstName im JSON
    pub postal_code: String, // → postalCode im JSON
}
```

### 2. Optimistic Updates - IMMER!

**Pattern:**
```typescript
const updateEntity = useCallback(async (id: number, newData: any) => {
  const oldEntity = entities.find(e => e.id === id);

  // 1. SOFORT UI updaten (KEIN await!)
  setEntities(prev => prev.map(e => e.id === id ? { ...e, ...newData } : e));

  try {
    // 2. Backend Update
    const result = await invoke('update_entity_command', { id, ...newData });

    // 3. Event für Undo (KEIN refresh!)
    window.dispatchEvent(new CustomEvent('refresh-data'));

    return result;
  } catch (error) {
    // 4. Rollback bei Fehler
    if (oldEntity) {
      setEntities(prev => prev.map(e => e.id === id ? oldEntity : e));
    }
    throw error;
  }
}, [entities]);
```

**NIEMALS:** `refreshBookings()`, `refreshGuests()` oder `refreshRooms()` nach erfolgreichen Operationen aufrufen!

### 3. UI/UX Dialoge

**VERBOTEN:** Browser-Standard-Dialoge (`alert()`, `confirm()`, `prompt()`)

**IMMER:** Eigene Custom Dialoge mit modernem Design verwenden

### 4. React State & Lifecycle

**Häufigster Fehler:** State wird nur beim Initial Load gesetzt, aber nicht bei Updates!

**Lösung:**
```typescript
useEffect(() => {
  const loadData = async () => {
    if (someObject?.someId) {
      const data = await invoke('get_data', { id: someObject.someId });
      setCurrentState(data);
    } else {
      setCurrentState(null); // ← WICHTIG: State zurücksetzen!
    }
  };
  loadData();
}, [someObject?.someId]); // ← Richtige Dependencies!
```

### 5. Regression Prevention

- ✅ Minimal-Change-Prinzip: Nur notwendige Änderungen
- ✅ Defensive Coding: Immer `?.` optional chaining verwenden
- ✅ Read-Before-Write: Erst Datei lesen, dann ändern
- ✅ Small Focused Commits: Ein Feature = Ein Commit

### 6. Web-Recherche - AUTOMATISCH!

**Trigger:** Nach 2-3 fehlgeschlagenen Lösungsversuchen SOFORT Web-Recherche durchführen!

**Queries:**
- "Technology + Problem + debugging + 2025"
- "github Library + Feature + example"
- "React/Rust + Specific Error Message"

### 7. Mobile App Deployment

**WICHTIG:** Bei JEDER Änderung an `dpolg-cleaning-mobile`:

```bash
cd "/path/to/dpolg-cleaning-mobile"
git add .
git commit -m "fix: Beschreibung"
git push
vercel --prod --yes  # ← SOFORT nach Push!
```

### 8. Roadmap-Tracking

**Nach JEDEM abgeschlossenen Task:**
```markdown
- [x] Task erledigt
  ✅ Implementiert in src/path/to/file.tsx:123
```

---

## 🤖 Subagent-Strategie

**Verfügbare Subagents:**
- `database-architect` - Schema, Migrations, SQL
- `rust-backend-dev` - Tauri Commands, Backend
- `react-component-builder` - UI-Komponenten, TypeScript
- `validation-expert` - Validierungen, Error Handling
- `pdf-email-specialist` - PDF/Email Features
- `testing-qa` - Testing, Code Review

**Workflow:** Analyse → Subagent-Auswahl → Delegation → Integration → Review → Roadmap Update → App starten

---

## 🏗️ Technologie-Stack

### Backend:
- **Tauri 2** - Desktop App Framework
- **Rust 2021** - Backend Language
- **rusqlite** - SQLite Database
- **serde** - Serialization
- **chrono** - Date/Time

### Frontend:
- **React 18+** - UI Framework
- **TypeScript 5+** - Type Safety
- **Vite** - Build Tool
- **TailwindCSS 4** - Styling
- **lucide-react** - Icons
- **dnd-kit** - Drag & Drop

### Datenbank:
- **SQLite** - Local Database (via rusqlite)

---

## 💻 Code-Konventionen

### Naming:
```rust
// Rust
fn create_booking()              // snake_case
struct BookingWithDetails {}     // PascalCase
const MAX_GUESTS: i32 = 10;      // SCREAMING_SNAKE_CASE
```

```typescript
// TypeScript
const bookingData = {};          // camelCase
function BookingDialog() {}      // PascalCase (Components)
interface BookingData {}         // PascalCase
const MAX_FILE_SIZE = 5000;      // SCREAMING_SNAKE_CASE
```

### Error Handling:
```rust
// ✅ IMMER Result<T, E>
fn create_booking() -> Result<Booking, String> { }

// ❌ NIEMALS unwrap() in Production
let booking = get_booking(id).unwrap(); // VERBOTEN!
```

### Database:
```rust
// ✅ IMMER Prepared Statements
conn.execute("INSERT INTO bookings VALUES (?1, ?2)", params![id, name])?;

// ❌ NIEMALS String Concatenation
let query = format!("SELECT * FROM bookings WHERE id = {}", id); // VERBOTEN!
```

### TypeScript Type Safety:
```typescript
// ✅ Explizite Types
const data = await invoke<BookingWithDetails>('get_booking', { id });

// ❌ NIEMALS 'any'
const data: any = await invoke('get_booking'); // VERBOTEN!
```

---

## 🎨 UI/UX Richtlinien

### Design-System:

**Farbpalette:**
```css
/* Backgrounds */
bg-slate-800, bg-slate-900       /* Main Backgrounds */
from-slate-800 to-slate-900      /* Gradient für Modals */

/* Text */
text-white, text-slate-300       /* Primary/Secondary Text */

/* Accent Colors */
bg-blue-600                      /* Primary Action */
bg-emerald-500                   /* Success */
bg-red-500                       /* Danger/Error */
bg-amber-500                     /* Warning */
```

**Standard Dropdown Pattern (VERBINDLICH):**
```tsx
<select
  className="w-full px-5 py-3.5 bg-white border border-slate-300 rounded-xl text-base text-slate-700 font-normal appearance-none cursor-pointer shadow-sm hover:border-slate-400 hover:shadow-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-all"
  style={{
    backgroundImage: `url("data:image/svg+xml,%3Csvg...")`,
    backgroundRepeat: 'no-repeat',
    backgroundPosition: 'right 0.75rem center',
    backgroundSize: '1.5rem',
    paddingRight: '3rem'
  }}
>
  <option value="">Option</option>
</select>
```

**Filter Section Pattern:**
```tsx
<div className="p-3 bg-slate-50 rounded-lg border border-slate-200">
  <div className="grid grid-cols-4 gap-3">
    {/* Search + Dropdowns */}
  </div>
</div>
```

### Accessibility:
- ✅ Semantic HTML (`<button>`, `<nav>`, `<main>`)
- ✅ ARIA Labels für Icons
- ✅ Keyboard Navigation
- ✅ Focus Management in Modals

### Sprache & Formatierung:
- **UI-Texte:** Deutsch
- **Datumsformat:** DD.MM.YYYY
- **Währung:** 1.234,56 €

---

## 🔒 Sicherheits-Richtlinien

- ✅ IMMER Input Validation auf Backend
- ✅ IMMER Prepared Statements (SQL Injection Prevention)
- ✅ Passwords verschlüsselt speichern
- ❌ NIEMALS technische Error Details an Frontend

---

## 📝 Git & Commit-Konventionen

### Commit-Message Format:
```
<type>(<scope>): <subject>

[optional body]
```

**Types:**
- `feat`: Neue Feature
- `fix`: Bugfix
- `refactor`: Code-Refactoring
- `style`: Code-Style
- `docs`: Dokumentation
- `chore`: Build/Config

**Beispiel:**
```
feat(database): Add accompanying_guests table

- Create schema with foreign key to bookings
- Add migration function
- Update BookingWithDetails struct

Refs: ROADMAP.md Phase 1.1
```

---

## 🚫 Verbotene Praktiken

### Rust:
- ❌ `.unwrap()` / `.expect()` in Tauri commands
- ❌ String concatenation für SQL queries
- ❌ Unverschlüsselte Passwörter

### TypeScript:
- ❌ `any` Type
- ❌ Englische Texte im UI
- ❌ Inline Styles (immer TailwindCSS)
- ❌ Unhandled Promise Rejections

---

## ✅ Checkliste vor jedem Commit

- [ ] Code kompiliert ohne Warnings
- [ ] TypeScript Errors behoben
- [ ] UI-Texte auf Deutsch
- [ ] Error Handling implementiert
- [ ] Keine hardcoded Werte
- [ ] Accessibility geprüft

---

## 🎯 Qualitätsziele

### Performance:
- App Start: < 2 Sekunden
- Database Queries: < 100ms
- UI Interaktionen: < 50ms

### Code Quality:
- Rust: `cargo clippy` ohne Warnings
- TypeScript: Strict Mode aktiviert

### User Experience:
- Alle UI-Texte auf Deutsch
- Klare Error Messages
- Loading States für async Operations
- Keyboard Navigation

---

**Version:** 2.0 (Kompakt)
**Letzte Aktualisierung:** 2025-10-20
**Status:** 🟢 Aktiv
