# Claude Code Projekt-Richtlinien
## Stiftung der DPolG Buchungssystem - Modern Tauri React Application

---

## 🎯 Projekt-Vision

Ein modernes, performantes Hotel-Buchungssystem mit intuitiver Tape Chart Visualisierung, entwickelt mit Tauri 2 + React + TypeScript. Fokus auf Benutzerfreundlichkeit, Geschwindigkeit und Wartbarkeit.

---

## 🗄️ DATENBANK - MULTI-USER SETUP (ORACLE CLOUD)

**Status:** ✅ PostgreSQL Server läuft auf Oracle Cloud (Always Free)

### 📡 Production Database Server

**Oracle Cloud PostgreSQL 16.11:**
- **Host:** `141.147.3.123`
- **Port (pgBouncer):** `6432` (Connection Pooling - EMPFOHLEN!)
- **Port (Direct):** `5432` (Nur für Admin-Tasks)
- **Database:** `dpolg_booking`
- **User:** `dpolg_admin`
- **Password:** `DPolG2025SecureBooking`

**Connection String (für Backend):**
```
postgres://dpolg_admin:DPolG2025SecureBooking@141.147.3.123:6432/dpolg_booking
```

### 🔐 SSH Access (Server-Administration)

**SSH Credentials:**
- **Host:** `ubuntu@141.147.3.123`
- **Key:** `~/Downloads/ssh-key-2025-11-14.key`
- **OS:** Ubuntu 22.04.5 LTS (Oracle Cloud)

**SSH Verbinden:**
```bash
ssh -i ~/Downloads/ssh-key-2025-11-14.key ubuntu@141.147.3.123
```

### 🛠️ Server-Komponenten

| Komponente | Version | Port | Status |
|------------|---------|------|--------|
| PostgreSQL | 16.11 | 5432 | ✅ Running |
| pgBouncer | 1.25.0 | 6432 | ✅ Running |
| Ubuntu | 22.04.5 LTS | - | ✅ Active |

**pgBouncer Config:**
- Pool Mode: `transaction`
- Max Client Connections: `100`
- Default Pool Size: `20`
- Reserve Pool Size: `5`

### 🔥 Firewall Rules (Oracle Cloud + Ubuntu)

**Offene Ports:**
- `22` - SSH (Key-based Auth)
- `5432` - PostgreSQL (Direct Access)
- `6432` - pgBouncer (Connection Pooling)

### 📊 Datenbank-Struktur

**SQLite → PostgreSQL Migration:**
- Status: ✅ Completed (1,740 rows migrated across 23 tables)
- Methode: Custom Python script with Boolean type casting
- Original DB: `src-tauri/booking_system.db` (1.3 MB)

**Tabellen (alle migriert):**
- `rooms` (10 rows), `guests` (257 rows), `bookings` (323 rows)
- `additional_services` (392 rows), `discounts` (185 rows)
- `accompanying_guests` (52 rows), `guest_companions` (52 rows)
- `email_logs` (448 rows), `reminders`, `email_attachments`
- `service_templates`, `discount_templates`, `payment_recipients`
- Uvm. (insgesamt 23 Tabellen)

### 🏗️ Backend Architektur (Modern 2025)

**Repository Pattern mit Type-safe Queries:**
- `src-tauri/src/database_pg/` - PostgreSQL Layer
  - `pool.rs` - Connection Pooling (deadpool-postgres)
  - `error.rs` - Type-safe Error Handling
  - `models.rs` - Rust Structs für alle Entities
  - `repositories/` - Business Logic (CRUD Operations)
    - `room_repository.rs` - ✅ FERTIG (6 Methoden + 6 Commands)
    - `guest_repository.rs` - ✅ FERTIG (8 Methoden + 8 Commands)
    - `booking_repository.rs` - ✅ FERTIG (11 Methoden)
    - `additional_service_repository.rs` - ✅ FERTIG (7 Methoden)
    - `discount_repository.rs` - ✅ FERTIG (7 Methoden)
    - ~18 weitere Repositories (Email, Reminder, Companion, etc.)

**Environment Configuration:**
- `src-tauri/src/config.rs` - Dev/Prod Config
- Development: `.env` file (lokal, NOT in Git)
- Production: GitHub Secrets (CI/CD)

### 🚀 Nächste Schritte

1. ✅ PostgreSQL Server Setup (Oracle Cloud)
2. ✅ pgBouncer Connection Pooling
3. ✅ SQLite → PostgreSQL Migration (1,740 rows)
4. ✅ Modern Architecture Implementation (Repository Pattern)
5. ✅ Room + Guest + Booking Repositories (25 Methoden, 14 Commands)
6. ⏳ Service + Discount + Email + Reminder Repositories (~4 Repositories)
7. ⏳ Remaining ~18 Repositories erstellen
8. ⏳ Alle Commands zu lib_pg.rs migrieren (~86 Commands)
9. ⏳ Multi-User Testing (5-10 gleichzeitige User)
10. ⏳ Production Deployment via GitHub Actions

**Fortschritts-Tracking:**
- Repositories: 5/23 fertig (22%) ← **+2 heute!**
- Commands: 14/~100 fertig (14%)
- Datensätze: 1,167/1,740 (67%) ← **67% abgedeckt!**
- **Siehe:** [PROGRESS_SESSION_2.md](../PROGRESS_SESSION_2.md) für Details!

**Siehe auch:**
- [POSTGRESQL_ARCHITECTURE.md](../POSTGRESQL_ARCHITECTURE.md) - Vollständige Architektur-Dokumentation
- [DEVELOPMENT_WORKFLOW.md](../DEVELOPMENT_WORKFLOW.md) - HMR + GitHub CI/CD Workflow
- [GITHUB_SECRETS_SETUP.md](../GITHUB_SECRETS_SETUP.md) - Production Build Configuration
- [PROGRESS_SESSION_2.md](../PROGRESS_SESSION_2.md) - **NEU:** Detaillierter Fortschrittsbericht

---

## 🚀 RELEASE-PROZESS (AUTOMATISIERT - IMMER SO!)

**WICHTIG:** Bei JEDEM neuen Release IMMER diesen Prozess verwenden!

### ⚡ EIN-BEFEHL-RELEASE (SCHNELLSTER WEG!)

```bash
./quick-release.sh 1.7.5
```

**Das war's!** Dieser Befehl macht ALLES automatisch:

1. ✅ Bumpt Version in allen 3 Dateien (package.json, Cargo.toml, tauri.conf.json)
2. ✅ Committed Version Bump
3. ✅ Erstellt Git Tag (v1.7.5)
4. ✅ Pusht zu GitHub
5. ✅ Buildet lokal mit Signierung (~5-10 Min) **NSIS Installer**
6. ✅ Erstellt GitHub Release
7. ✅ Uploaded NSIS Installer (.exe Datei)
8. ✅ Uploaded .exe.sig Datei
9. ✅ Generiert + uploaded **latest.json** (für Tauri Auto-Update!)

**Voraussetzungen (einmalig):**
- `.github-token` Datei muss existieren (bereits vorhanden)
- `src-tauri/dpolg-signing.key` Datei muss existieren (bereits vorhanden)

### 📋 VOR jedem Release:

```bash
# 1. Alle Änderungen committen
git add <geänderte-files>
git commit -m "fix: Beschreibung der Änderung"

# 2. EIN Befehl macht den Rest:
./quick-release.sh 1.7.5
```

### 🔑 Wichtige Dateien (NIEMALS in Git committen!)

Diese Dateien sind **lokal** und bereits in `.gitignore`:

- `.github-token` - GitHub Personal Access Token
- `src-tauri/dpolg-signing.key` - Private Signing Key (passwordless)
- `*.db` - Datenbank-Dateien
- `latest.json` - Auto-generiert beim Release

### ✅ Was der Script automatisch macht (Tauri 2 Best Practices 2025)

**latest.json Generation:**
```json
{
  "version": "1.7.5",
  "notes": "Release v1.7.5",
  "pub_date": "2025-10-25T...",
  "platforms": {
    "windows-x86_64": {
      "signature": "<content-from-.exe.sig>",
      "url": "https://github.com/.../download/v1.7.5/...x64-setup.exe"
    }
  }
}
```

**Endpoint in tauri.conf.json:**
```json
{
  "plugins": {
    "updater": {
      "endpoints": [
        "https://github.com/Maxwellbadger-1/dpolg-booking-modern/releases/latest/download/latest.json"
      ]
    }
  }
}
```

### 🧪 Nach dem Release testen:

1. Öffne installierte App (ältere Version)
2. Update-Dialog sollte erscheinen
3. "Ja" klicken → Download + Installation
4. App neustartet automatisch

### ⏱️ Geschätzte Zeiten:

| Schritt | Dauer |
|---------|-------|
| Code committen | 1-2 min |
| `./quick-release.sh 1.7.5` | 7-12 min |
| **GESAMT** | **~10 min** |

### 🚨 NIEMALS:

- ❌ NIEMALS GitHub Actions verwenden (funktioniert nicht mit headless_chrome)
- ❌ NIEMALS manuell Dateien hochladen (Script macht das)
- ❌ NIEMALS Signing Key oder Token in Git committen
- ❌ NIEMALS latest.json vergessen (Auto-Update funktioniert sonst nicht!)

---

## 🚨 KRITISCHE REGELN

### 🔥 REGEL #1 - TAURI AUTO-KONVERTIERUNG (WICHTIGSTE REGEL!)

**⚠️ TAURI KONVERTIERT PARAMETER AUTOMATISCH!**

```typescript
// ✅ RICHTIG - Frontend IMMER camelCase:
invoke('sync_affected_dates', {
  bookingId: 81,           // → Tauri konvertiert zu: booking_id
  oldCheckout: "2025-10-28", // → Tauri konvertiert zu: old_checkout
  newCheckout: "2025-10-30"  // → Tauri konvertiert zu: new_checkout
})

// ❌ FALSCH - NIEMALS snake_case im Frontend:
invoke('sync_affected_dates', {
  booking_id: 81,        // ❌ FEHLER! Command missing required key
  old_checkout: "...",   // ❌ Bricht die Auto-Konvertierung!
  new_checkout: "..."
})
```

**Golden Rules (2025 Research-backed):**
1. **Frontend:** IMMER camelCase verwenden - keine Ausnahmen!
2. **Backend:** IMMER snake_case verwenden (Rust Convention)
3. **Tauri macht die Konvertierung automatisch** bei `invoke()` Aufrufen
4. **Für Structs:** `#[serde(rename_all = "camelCase")]` für nested fields
5. **NIEMALS** camelCase und snake_case mixen!

**Beispiel Struct:**
```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]  // ← Für nested fields!
pub struct BookingData {
    pub booking_id: i64,     // → bookingId im JSON
    pub guest_name: String,  // → guestName im JSON
    pub check_in: String,    // → checkIn im JSON
}
```

**Wie es funktioniert:**
- Tauri Command Parameter: **automatische Konvertierung**
- Nested Struct Fields: **benötigt serde annotation**
- Return Values: **benötigt serde annotation**

### 🔥 REGEL #2 - JSX SYNTAX (SUPER WICHTIG - JEDES MAL!)

**⚠️ JSX CLOSING TAGS MÜSSEN PERFEKT SEIN!**

**Häufigste Fehler (Research 2025):**

1. **Adjacent Elements MÜSSEN wrapped sein:**
```tsx
// ❌ FALSCH - Zwei sibling elements ohne Wrapper:
return (
  <div>Section 1</div>
  <div>Section 2</div>  // ❌ ERROR: Adjacent JSX elements must be wrapped
)

// ✅ RICHTIG - Mit Fragment wrapper:
return (
  <>
    <div>Section 1</div>
    <div>Section 2</div>
  </>
)
```

2. **Closing Tags in RICHTIGER REIHENFOLGE:**
```tsx
// ❌ FALSCH - Falsche Schließ-Reihenfolge:
<div>
  <button>
    <span>Text</div>  // ❌ span nicht geschlossen!
  </button>
</span>

// ✅ RICHTIG - Korrekte Reihenfolge (LIFO - Last In First Out):
<div>
  <button>
    <span>Text</span>  // ✅ Zuerst span
  </button>            // ✅ Dann button
</div>                 // ✅ Dann div
```

3. **Self-Closing Tags bei leeren Elementen:**
```tsx
// ❌ FALSCH:
<input>  // ❌ Muss self-closed sein!
<img>    // ❌ Muss self-closed sein!

// ✅ RICHTIG:
<input />
<img />
<br />
<hr />
```

4. **Conditional Rendering - Extra vorsichtig:**
```tsx
// ❌ GEFÄHRLICH - Leicht Fehler zu machen:
{condition && (
  <div>
    <span>...</span>
  </div>
  <div>...</div>  // ❌ Adjacent elements!
)}

// ✅ SICHER - Immer wrapper verwenden:
{condition && (
  <>
    <div>
      <span>...</span>
    </div>
    <div>...</div>
  </>
)}
```

**🛡️ SCHUTZ-STRATEGIE vor JSX-Fehlern:**

1. **Bei großen Edits:** IMMER erst komplette Struktur lesen
2. **Count Tags:** Jedes `<div>` MUSS ein `</div>` haben
3. **Indentation beachten:** Hilft beim Erkennen der Struktur
4. **Kleine Schritte:** Lieber 5 kleine Edits als 1 großer
5. **Testing:** Nach jeder Edit-Gruppe prüfen ob App läuft
6. **Git Checkpoint:** Vor großen Refactorings commit machen

**Tools die helfen:**
- ESLint + Prettier (auto-fix viele Fehler)
- VSCode Bracket Colorizer
- React Developer Tools

**MERKE:** JSX-Fehler brechen die GESAMTE App! Deshalb extra vorsichtig!

### 3. Optimistic Updates - IMMER!

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

### 6. SQLite Schema Changes - KRITISCH!

**⚠️ WICHTIG: SQLite erlaubt KEINE direkten CHECK constraint Änderungen!**

**Problem:**
```sql
-- ❌ FUNKTIONIERT NICHT:
ALTER TABLE reminders
ADD CONSTRAINT CHECK(reminder_type IN ('manual', 'auto_confirmation'));
```

**❌ FALSCHE Lösung: Datenbanken löschen**

Das funktioniert NICHT bei bestehenden Produktionsdaten! `CREATE TABLE IF NOT EXISTS` ändert existierende Tabellen NICHT.

**✅ RICHTIGE Lösung: Migration schreiben!**

**Schritt 1:** Migration-Funktion in `database.rs` schreiben:

```rust
// MIGRATION: Recreate table with updated CHECK constraint
fn migrate_reminders_table_check_constraint(conn: &Connection) -> Result<()> {
    // Test if old constraint exists
    let needs_migration = conn.execute(
        "INSERT INTO reminders (booking_id, reminder_type, title, due_date, priority)
         VALUES (NULL, 'auto_confirmation', 'TEST', '2025-01-01', 'low')",
        [],
    ).is_err();

    if needs_migration {
        println!("🔄 [MIGRATION] Recreating reminders table...");

        conn.execute("PRAGMA foreign_keys = OFF", [])?;
        conn.execute("BEGIN TRANSACTION", [])?;

        // Rename old table
        conn.execute("ALTER TABLE reminders RENAME TO reminders_old", [])?;

        // Create new table with updated CHECK constraint
        conn.execute("CREATE TABLE reminders (...new schema...)", [])?;

        // Copy data
        conn.execute("INSERT INTO reminders SELECT * FROM reminders_old", [])?;

        // Drop old table
        conn.execute("DROP TABLE reminders_old", [])?;

        conn.execute("COMMIT", [])?;
        conn.execute("PRAGMA foreign_keys = ON", [])?;

        println!("✅ [MIGRATION] Reminders table recreated!");
    } else {
        // Clean up test row
        conn.execute("DELETE FROM reminders WHERE title = 'TEST'", [])?;
    }

    Ok(())
}
```

**Schritt 2:** Migration in `init_database()` aufrufen:

```rust
pub fn init_database() -> Result<()> {
    // ... CREATE TABLE IF NOT EXISTS reminders ...

    // MIGRATION: Fix CHECK constraint
    migrate_reminders_table_check_constraint(&conn)?;

    // ... rest of init ...
}
```

**Schritt 3:** App neu kompilieren und starten:

```bash
# 1. App stoppen
taskkill //F //IM dpolg-booking-modern.exe

# 2. Rebuild
cd src-tauri && cargo build

# 3. App starten
npx tauri dev --no-watch
```

**Die Migration läuft automatisch beim ersten Start und:**
- ✅ Prüft ob Migration nötig ist (Test-Insert)
- ✅ Erstellt neue Tabelle mit korrektem Schema
- ✅ Migriert ALLE bestehenden Daten
- ✅ Läuft nur EINMAL (idempotent)
- ✅ Funktioniert mit Produktionsdaten

**NIEMALS mehr Datenbanken löschen!** Verwende IMMER Migrations!

**Web Search validated:** ✅ SQLite ALTER TABLE Limitations + Migration Best Practices (2025)

### 7. Web-Recherche - AUTOMATISCH!

**Trigger:** Nach 2-3 fehlgeschlagenen Lösungsversuchen SOFORT Web-Recherche durchführen!

**Queries:**
- "Technology + Problem + debugging + 2025"
- "github Library + Feature + example"
- "React/Rust + Specific Error Message"

### 8. Mobile App Deployment

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

## 🛡️ REGRESSION PREVENTION (2025 Best Practices)

### Warum Regressions Prevention?

**Problem:** Features die bereits funktioniert haben brechen nach neuen Änderungen wieder.

**Lösung:** Systematische Tests + Checklisten + Automated Testing

---

### 🔴 KRITISCHE REGEL - VOR JEDEM COMMIT

**IMMER diese Features manuell testen:**

1. **Drag & Drop im Tapechart** - Funktioniert es noch?
2. **Mobile App iframe** - Lädt die Putzplan-Vorschau?
3. **Dropdown z-index** - Werden UI-Elemente korrekt übereinander gelegt?
4. **Neue Buchung** - Dialog öffnet und speichert?
5. **TypeScript** - `npm run build` läuft ohne Fehler?

**Wenn eines dieser Features kaputt ist → NICHT committen!**

---

### 📋 Checklisten verwenden

**VOR JEDEM RELEASE:**
```bash
# 1. Öffne Checklist
code CRITICAL_FEATURES_CHECKLIST.md

# 2. Gehe systematisch durch ALLE Punkte

# 3. Markiere getestete Features als [x]

# 4. NUR wenn ALLES ✅ → Release machen
```

**Checkliste Location:** `CRITICAL_FEATURES_CHECKLIST.md`

---

### 🧪 Automated Regression Tests

**Playwright Tests laufen lassen:**

```bash
# Alle Tests
npx playwright test

# Nur Critical Regressions
npx playwright test tests/critical-regression.spec.ts

# Mit UI
npx playwright test --ui
```

**Test Files:**
- `tests/critical-regression.spec.ts` - Core Features (Tapechart, Drag&Drop, UI)
- `tests/mobile-app.spec.ts` - Mobile App Tests (Emojis, Filter, Tasks)

**Regel:** Neue kritische Features → SOFORT Test schreiben!

---

### 🔧 Pre-Commit Hooks

**Automatische Checks vor jedem Commit:**

```bash
# In package.json bereits konfiguriert
"husky": {
  "hooks": {
    "pre-commit": "npm run type-check && npm run lint"
  }
}
```

**Was wird geprüft:**
- TypeScript Type Errors
- ESLint Rules
- Code Formatting

**Wenn Hook fehlschlägt → Commit wird blockiert!**

---

### 📝 Feature Branch Workflow

**NIEMALS direkt auf `main` committen!**

```bash
# 1. Neuer Feature Branch
git checkout -b feature/neue-funktion

# 2. Änderungen machen
# ... Code schreiben ...

# 3. Lokale Tests
npm run build
npx playwright test tests/critical-regression.spec.ts

# 4. Checklist durchgehen
# CRITICAL_FEATURES_CHECKLIST.md

# 5. Commit
git add .
git commit -m "feat: Neue Funktion"

# 6. Merge NACH Testing
git checkout main
git merge feature/neue-funktion

# 7. Release
git tag v1.X.X
git push --tags
```

---

### ⚠️ Bekannte Probleme (IMMER prüfen!)

Diese Bugs sind schon mehrmals aufgetreten:

1. **JSX Syntax** - Adjacent Elements ohne Wrapper
   - ✅ Fix: Immer `<>...</>` wrapper verwenden

2. **Tauri Invoke** - snake_case statt camelCase
   - ✅ Fix: Frontend IMMER camelCase!

3. **z-index Conflicts** - Dropdowns unter Tapechart
   - ✅ Fix: Dropdowns z-[100], Tapechart z-0

4. **iframe CSP Blocking** - Mobile App lädt nicht
   - ✅ Fix: `frame-src` in tauri.conf.json

5. **Emoji Spalten fehlen** - Vercel cached alte Version
   - ✅ Fix: Dummy commit + `git push` triggert Redeploy

6. **Drag & Drop bricht** - z-index verhindert pointer events
   - ✅ Fix: Zellen ohne z-index, nur Dropdowns z-[100]

7. **SQLite CHECK Constraint Fehler** - Neue Enum-Werte werden abgelehnt
   - ❌ Fix: Datenbank löschen funktioniert NICHT (CREATE TABLE IF NOT EXISTS)
   - ✅ Fix: Migration schreiben (siehe Regel #6)
   - Beispiel: `auto_confirmation` zu reminder_type hinzufügen

8. **Cargo build hängt nach Vite-Start** - Windows + Tauri 2 Problem
   - ✅ Fix: `npx tauri dev --no-watch` verwenden
   - ✅ Fix: Bei Problemen `cargo clean` + Neustart
   - ⏱️ Cargo clean löscht ~2.8GB, neu kompilieren ~2-3 Min

---

### 🚨 Notfall-Recovery

**Wenn etwas kaputt geht:**

```bash
# 1. SOFORT Git Status checken
git status

# 2. Letzte funktionierende Version finden
git log --oneline -10

# 3. Zu funktionierender Version zurück
git checkout <commit-hash> -- <file>

# 4. Oder kompletten Revert
git revert <commit-hash>

# 5. Testen ob Feature wieder funktioniert

# 6. Commit
git commit -m "fix: Revert broken feature"
```

---

### 📊 Regression Testing Metrics

**Ziele (2025 Standards):**
- ✅ **99%** Critical Features immer funktionsfähig
- ✅ **< 1 Tag** bis Regression-Fix deployed
- ✅ **100%** Automated Tests für Core Features
- ✅ **0** Regressions in Production Releases

**Aktueller Status:**
- Manual Testing: ✅ Checklist vorhanden
- Automated Tests: ✅ Playwright Setup
- Pre-commit Hooks: ✅ TypeScript + Lint
- Feature Branches: ⚠️ Needs Enforcement

---

### 💡 Pro-Tipps

1. **Kleine Commits** - Leichter zu debuggen und reverten
2. **Descriptive Messages** - "fix: Tapechart drag & drop" nicht "fixed bug"
3. **Test after Merge** - Auch nach merge nochmal alles testen
4. **Screenshot before/after** - Bei UI-Änderungen Screenshots machen
5. **Ask before big refactors** - Große Änderungen mit Team absprechen

---

## 🚀 ENTWICKLUNGS-WORKFLOW (AUTOMATISIERT!)

### ⚡ APP STARTEN - IMMER SO!

**WICHTIG:** Bei JEDEM Start IMMER diesen Befehl verwenden:

```bash
npm run tauri:dev
```

**Das war's!** Dieser Befehl macht ALLES automatisch:

1. ✅ Killt alte Prozesse auf Port 1420 + 1421 (Vite + HMR)
2. ✅ Startet Vite Dev Server (Port 1420)
3. ✅ Startet Tauri Dev (mit Hot Reload)
4. ✅ Lädt IMMER die neuesten Code-Änderungen

**Vorteile:**
- ⚡ Keine Port-Konflikte mehr!
- ⚡ Keine veralteten Builds im Hintergrund!
- ⚡ Sofort einsatzbereit mit allen neuen Changes!
- ⚡ Cross-Platform (Windows, Mac, Linux)

### 📋 Alte manuelle Methode (NICHT MEHR VERWENDEN!)

```bash
# ❌ VERALTET - Funktioniert oft nicht wegen Port-Konflikten:
npm run tauri dev

# ❌ VERALTET - Manuelles Port-Killing:
netstat -ano | findstr :1423
taskkill //F //PID <pid>
```

### 🛠️ Technische Details (PRO-CONFIG 2025!)

**Kill-Port Package (2025 Industry Standard):**
- Verwendet: `kill-port` npm package
- Cross-Platform: Windows/Mac/Linux
- Pre-Dev Hook: `"predev": "kill-port 1420 1421 || echo 'Ports already free'"`

**Ports (EINHEITLICH KONFIGURIERT!):**
- **1420** - Vite Dev Server Port (vite.config.ts + tauri.conf.json)
- **1421** - Vite HMR Port (Hot Module Replacement)

**KRITISCHE REGEL: Port-Konsistenz (Web Search validated 2025)**

⚠️ **ALLE Konfigs MÜSSEN denselben Port verwenden!**

```typescript
// vite.config.ts
export default defineConfig({
  server: {
    port: 1420,              // ✅ Fester Port
    strictPort: true,        // ✅ WICHTIG! Fail-fast bei Port-Konflikt
    host: host || false,
    hmr: host ? {
      protocol: "ws",
      host,
      port: 1421,            // ✅ HMR Port
    } : undefined,
  }
})
```

```json
// tauri.conf.json
{
  "build": {
    "beforeDevCommand": "npm run dev",           // ✅ KEIN Port-Override!
    "devUrl": "http://localhost:1420"            // ✅ Gleicher Port wie Vite!
  }
}
```

```json
// package.json
{
  "scripts": {
    "predev": "kill-port 1420 1421 || echo 'Ports already free'",  // ✅ Beide Ports
    "tauri:dev": "npm run predev && tauri dev"
  }
}
```

**NIEMALS:**
- ❌ Port-Override in Scripts: `npm run dev -- --port 1423` (BAD!)
- ❌ Verschiedene Ports in verschiedenen Configs
- ❌ `strictPort: false` verwenden (maskiert Probleme!)

**Script-Kette:**
```json
{
  "predev": "kill-port 1420 1421 || echo 'Ports already free'",
  "dev": "vite",                    // ← Verwendet Port aus vite.config.ts
  "tauri:dev": "npm run predev && tauri dev"
}
```

### ✅ Was der Script automatisch macht

1. **Port Cleanup:** Killt alle Prozesse auf 1420 + 1421 (Vite + HMR)
2. **Fallback:** Falls Ports schon frei → Weiter ohne Fehler
3. **Tauri Start:** Startet clean dev environment mit EINHEITLICHEM Port
4. **Hot Reload:** Frontend + Backend Changes werden live geladen
5. **Fail-Fast:** `strictPort: true` verhindert Port-Konflikte sofort

### 🧪 Nach dem Start testen:

1. App öffnet sich automatisch
2. Änderungen im Code → Auto-Reload
3. Console zeigt Debug-Logs (z.B. Reminder-System)

### ⏱️ Geschätzte Zeiten:

| Schritt | Dauer |
|---------|-------|
| Port Cleanup | 1-2 sek |
| Vite Start | 2-3 sek |
| Tauri Compile | 5-10 sek |
| **GESAMT** | **~10 sek** |

### 🚨 NIEMALS:

- ❌ NIEMALS `npm run tauri dev` direkt verwenden (Port-Konflikte!)
- ❌ NIEMALS manuell Prozesse killen (Script macht das!)
- ❌ NIEMALS mehrere Dev-Server parallel starten
- ❌ NIEMALS alte Tauri-Builds im Hintergrund lassen

**Web Search validated:** ✅ 2025 Tauri 2 + npm kill-port Best Practices (Oktober 2025)

---

## 🤖 CLAUDE REMINDER - BEI JEDEM RELEASE!

**WENN DER USER NACH EINEM RELEASE FRAGT:**

1. ✅ **IMMER** fragen: "Hast du bereits Code-Änderungen committed?"
2. ✅ **IMMER** den automatisierten Prozess verwenden: `./quick-release.sh X.X.X`
3. ✅ **NIEMALS** manuell Dateien hochladen
4. ✅ **NIEMALS** GitHub Actions vorschlagen
5. ✅ **IMMER** `latest.json` Generation im Script sicherstellen
6. ✅ **IMMER** validieren dass `.github-token` und `src-tauri/dpolg-signing.key` existieren

**QUICK CHECKLIST:**
```bash
# Vorher: Code committed?
git status  # Sollte "nothing to commit" zeigen

# Release:
./quick-release.sh 1.7.5  # NUR DIESER EINE BEFEHL!

# Nachher: Testen
# → Installierte App öffnen
# → Update-Dialog sollte erscheinen
```

**Das Script macht ALLES:**
- Version Bump + Commit + Tag
- Build mit Signierung
- GitHub Release erstellen
- Upload: .msi + .msi.sig + latest.json

**Web Search validated:** ✅ 2025 Tauri 2 Best Practices (Oktober 2025)

---

**Version:** 2.2 (Mit Auto-Release Prozess)
**Letzte Aktualisierung:** 2025-10-25
**Status:** 🟢 Aktiv
