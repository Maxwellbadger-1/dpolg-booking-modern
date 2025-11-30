# Mobile App Architecture & Troubleshooting Guide

**Last Updated:** 2025-11-27
**Purpose:** Dokumentation der Mobile App Architektur und häufige Probleme

---

## Architektur Übersicht

### Stack
- **Frontend:** Static HTML/JS (Vanilla JavaScript)
- **Deployment:** Vercel (https://dpolg-cleaning-mobile.vercel.app)
- **API:** Vercel Serverless Function (`/api/tasks`)
- **Database:** Turso SQLite Cloud (libsql)
- **Sync:** PostgreSQL → Turso (via Tauri Desktop App)

### Datenfluss

```
Desktop App (PostgreSQL)
  ↓ Tauri Command: cleanup_cleaning_tasks (stündlich)
  ↓ sync_week_ahead (3 Monate)
Turso Cloud Database
  ↓ Vercel Serverless Function: /api/tasks
  ↓ HTTP POST Request
Mobile App (Browser)
```

---

## Datenbankschema

### PostgreSQL: `cleaning_tasks` Tabelle

```sql
CREATE TABLE cleaning_tasks (
    id SERIAL PRIMARY KEY,
    booking_id INTEGER NOT NULL,
    room_id INTEGER NOT NULL,
    task_date DATE NOT NULL,
    checkout_time TIME,
    checkin_time TIME,
    priority VARCHAR(20),
    has_dog BOOLEAN,
    change_bedding BOOLEAN,
    guest_count INTEGER,
    guest_name VARCHAR(200),
    status VARCHAR(20),
    completed_at TIMESTAMP,
    completed_by VARCHAR(100),
    notes TEXT,

    -- WICHTIG: UNIQUE Constraint für Duplikate-Prävention
    CONSTRAINT cleaning_tasks_unique UNIQUE (booking_id, room_id, task_date)
);
```

### Turso: `cleaning_tasks` Tabelle

```sql
CREATE TABLE cleaning_tasks (
    id TEXT PRIMARY KEY,  -- ⚠️ TEXT in Turso, INTEGER in PostgreSQL!
    booking_id TEXT,
    reservierungsnummer TEXT,
    date TEXT,  -- ⚠️ 'date' in Turso, 'task_date' in PostgreSQL!
    room_name TEXT,
    room_id TEXT,
    room_location TEXT,
    guest_name TEXT,
    checkout_time TEXT,
    checkin_time TEXT,
    priority TEXT,
    notes TEXT,
    status TEXT,
    guest_count INTEGER,
    extras TEXT,  -- JSON string mit original_checkin, original_checkout
    emojis_start TEXT,
    emojis_end TEXT,
    has_dog BOOLEAN,
    change_bedding BOOLEAN,
    completed_at TEXT,
    completed_by TEXT
);
```

**⚠️ WICHTIG:** Spaltenname-Unterschiede beachten!
- PostgreSQL: `task_date` → Turso: `date`
- PostgreSQL: `id` (INTEGER) → Turso: `id` (TEXT)

---

## Cleaning Tasks Erstellung

### Database Trigger (PostgreSQL)

```sql
CREATE TRIGGER trg_generate_cleaning_task_update
    AFTER UPDATE OF checkout_date, room_id, status ON bookings
    FOR EACH ROW
    EXECUTE FUNCTION generate_cleaning_task_for_booking();
```

**Wann wird der Trigger gefeuert:**
- Booking UPDATE mit Änderung von `checkout_date`, `room_id` oder `status`
- Booking INSERT (neues Booking)

**Was der Trigger macht:**
```sql
INSERT INTO cleaning_tasks (...)
ON CONFLICT (booking_id, room_id, task_date)  -- ⚠️ Benötigt UNIQUE Constraint!
DO UPDATE SET ...
```

---

## Häufige Probleme & Lösungen

### Problem 1: Duplicate Cleaning Tasks

**Symptome:**
- Mobile App zeigt gleiche Tasks 4-7x
- PostgreSQL hat mehrere Task-IDs für gleiche booking_id/room_id/task_date Kombination

**Root Cause:**
1. **Fehlender UNIQUE Constraint:** Trigger verwendet `ON CONFLICT DO NOTHING`, aber ohne UNIQUE Constraint funktioniert das nicht!
2. **Multiple Trigger Executions:** Bei jedem `UPDATE bookings` wird ein NEUER cleaning_task erstellt

**Lösung:**
```sql
-- 1. UNIQUE Constraint hinzufügen
CREATE UNIQUE INDEX idx_cleaning_tasks_unique_booking_room_date
    ON cleaning_tasks (booking_id, room_id, task_date);

-- 2. Trigger-Funktion auf UPSERT ändern
INSERT INTO cleaning_tasks (...)
ON CONFLICT (booking_id, room_id, task_date)
DO UPDATE SET ...  -- Statt DO NOTHING!

-- 3. Bestehende Duplikate löschen
DELETE FROM cleaning_tasks
WHERE id IN (
    SELECT ct.id
    FROM cleaning_tasks ct
    INNER JOIN (
        SELECT booking_id, room_id, task_date, MAX(id) as max_id
        FROM cleaning_tasks
        GROUP BY booking_id, room_id, task_date
        HAVING COUNT(*) > 1
    ) dups ON ct.booking_id = dups.booking_id
          AND ct.room_id = dups.room_id
          AND ct.task_date = dups.task_date
    WHERE ct.id < dups.max_id
);
```

**Dateien:**
- Fix: `/migrations/006_fix_cleaning_task_duplicates.sql`
- Cleanup: `/fix-duplicate-cleaning-tasks.sql`

### Problem 2: Timeline View zeigt falsche Duplikate

**Symptome:**
- Debug-Logs zeigen "Processing task for room X" mehrmals
- ABER: `✅ No duplicate IDs in filteredTasks`

**Root Cause:**
- `filteredTasks.forEach()` läuft nur EINMAL pro Task
- Aber die `while`-Schleife innerhalb des forEach erstellt mehrere Tage-Einträge für EINEN Task
- Debug-Log war an falscher Stelle (vor der while-Schleife)

**Lösung:**
- **NICHT** das Debug-Log, sondern die **Datenbank** war das Problem (siehe Problem 1)
- Debug-Logs sollten INNERHALB der while-Schleife sein, um Tage-Erstellung zu tracken

### Problem 3: Mobile App cached Duplikate

**Symptome:**
- Backend ist gefixt, aber Mobile App zeigt noch Duplikate

**Root Cause:**
1. Vercel Deployment dauert 1-2 Minuten
2. Browser cached alte Daten

**Lösung:**
```bash
# 1. Git push triggert Auto-Deployment
git add . && git commit -m "fix: ..." && git push

# 2. Warte 1-2 Minuten für Vercel Deployment

# 3. Hard Reload im Browser
Cmd + Shift + R (Mac)
Ctrl + Shift + R (Windows)
```

### Problem 4: Turso Sync zeigt keine Duplikate, aber Mobile App schon

**Symptome:**
```
✅ No duplicate IDs in PostgreSQL result
✅ No duplicate IDs in Turso data
```
Aber Mobile App zeigt trotzdem Duplikate!

**Root Cause:**
- Turso `INSERT OR REPLACE` nutzt PRIMARY KEY (`id`)
- Wenn PostgreSQL mehrere Tasks mit verschiedenen IDs hat, werden ALLE in Turso eingefügt
- `INSERT OR REPLACE` ersetzt nur bei GLEICHEM ID, nicht bei gleichem (booking_id, room_id, date)!

**Lösung:**
- Problem ist in PostgreSQL, nicht in Turso Sync
- Fix: Problem 1 (UNIQUE Constraint + Duplikate löschen)

---

## Sync-Prozess Details

### Desktop App: `cleanup_cleaning_tasks` Command

**Wann:** Stündlich (via cron/scheduler)

**Was:**
1. Löscht alte Tasks aus PostgreSQL (< 7 Tage in Vergangenheit)
2. Reinitialisiert Turso Schema (DROP + CREATE TABLE)
3. Ruft `sync_week_ahead` auf

### Desktop App: `sync_week_ahead` Command

**Wann:** Nach `cleanup_cleaning_tasks` oder manuell

**Was:**
```rust
// 1. Query PostgreSQL
let tasks = CleaningTaskRepository::get_for_period(pool, start_date, end_date).await?;

// 2. DELETE FROM Turso (alle alten Tasks)
DELETE FROM cleaning_tasks;

// 3. INSERT OR REPLACE INTO Turso
for task in tasks {
    INSERT OR REPLACE INTO cleaning_tasks (...) VALUES (...);
}
```

**⚠️ WICHTIG:**
- `INSERT OR REPLACE` nutzt PRIMARY KEY (`id`)
- Wenn PostgreSQL Duplikate hat (verschiedene IDs), werden ALLE eingefügt!

---

## Debug-Checkliste

Bei Duplicate-Problemen in Mobile App:

1. **PostgreSQL prüfen:**
```sql
SELECT booking_id, room_id, task_date, COUNT(*) as count
FROM cleaning_tasks
GROUP BY booking_id, room_id, task_date
HAVING COUNT(*) > 1;
```

2. **Turso prüfen:**
```bash
node -e "
async function check() {
  const response = await fetch('https://dpolg-cleaning-mobile.vercel.app/api/tasks', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ action: 'getTasks' })
  });
  const data = await response.json();
  const tasks = data.results[0].response.result.rows;
  console.log('Total tasks:', tasks.length);

  // Check for duplicate IDs
  const ids = tasks.map(row => row[0].value);
  const uniqueIds = new Set(ids);
  console.log('Unique IDs:', uniqueIds.size);
  if (ids.length !== uniqueIds.size) {
    console.log('❌ DUPLICATES FOUND!');
  } else {
    console.log('✅ No duplicates');
  }
}
check();
"
```

3. **UNIQUE Constraint prüfen:**
```sql
SELECT indexname, indexdef
FROM pg_indexes
WHERE tablename = 'cleaning_tasks'
  AND indexname LIKE '%unique%';
```

4. **Trigger-Funktion prüfen:**
```sql
SELECT prosrc
FROM pg_proc
WHERE proname = 'generate_cleaning_task_for_booking';
```

---

## Performance Optimierungen

### 1. Turso Sync nur für geänderte Tasks

**Aktuell:**
```rust
// DELETE all, INSERT all (ineffizient!)
DELETE FROM cleaning_tasks;
INSERT OR REPLACE INTO cleaning_tasks ...;
```

**Besser:**
```rust
// Nur geänderte Tasks syncen
INSERT OR REPLACE INTO cleaning_tasks ...;
// Alte Tasks (> 3 Monate) löschen
DELETE FROM cleaning_tasks WHERE date < ?;
```

### 2. Mobile App: Debouncing

**Aktuell:** `displayTasks()` mit 50ms Debounce

**Funktioniert gut!** Keine weiteren Optimierungen nötig.

### 3. PostgreSQL: Partial Indexes

```sql
-- Nur pending/in_progress Tasks indexieren (90% schneller!)
CREATE INDEX idx_cleaning_active_tasks
    ON cleaning_tasks (task_date DESC, room_id)
    WHERE status IN ('pending', 'in_progress');
```

---

## Testing

### Unit Tests (PostgreSQL Trigger)

```sql
-- Test 1: INSERT booking erstellt Task
INSERT INTO bookings (...) VALUES (...);
SELECT COUNT(*) FROM cleaning_tasks WHERE booking_id = <id>;
-- Erwartung: 1

-- Test 2: UPDATE booking updated Task (kein Duplikat!)
UPDATE bookings SET checkout_date = '2025-12-01' WHERE id = <id>;
SELECT COUNT(*) FROM cleaning_tasks WHERE booking_id = <id>;
-- Erwartung: 1 (NICHT 2!)

-- Test 3: Cancelled booking löscht Task
UPDATE bookings SET status = 'cancelled' WHERE id = <id>;
SELECT COUNT(*) FROM cleaning_tasks WHERE booking_id = <id>;
-- Erwartung: 0
```

### Integration Tests (Mobile App)

```javascript
// Test 1: Load tasks
const response = await fetch('/api/tasks', {
  method: 'POST',
  body: JSON.stringify({ action: 'getTasks' })
});
const data = await response.json();
assert(data.results.length > 0);

// Test 2: No duplicates
const ids = data.results[0].response.result.rows.map(r => r[0].value);
const uniqueIds = new Set(ids);
assert(ids.length === uniqueIds.size);
```

---

## Deployment

### Vercel Deployment

**Auto-Deploy:**
- Push zu `main` branch → Auto-Deploy
- Dauer: ~1-2 Minuten

**Manual Deploy:**
```bash
cd dpolg-cleaning-mobile
vercel --prod
```

**Environment Variables:**
- `TURSO_DATABASE_URL`: libsql://...
- `TURSO_AUTH_TOKEN`: eyJ...

---

## Zukünftige Verbesserungen

### Option 1: pg_turso (PostgreSQL → Turso CDC)

**Status:** Evaluierung für später (2-3 Monate)

**Was ist pg_turso?**
- PostgreSQL Logical Replication Plugin von Turso
- Change Data Capture (CDC) - automatisch bei jedem INSERT/UPDATE/DELETE
- Real-Time Sync (< 1 Sekunde Latenz)
- Open Source: https://github.com/tursodatabase/pg_turso

**Vorteile:**
- Kein manueller Sync mehr nötig
- Real-Time Updates in Mobile App
- Automatisches Toast-Sync bei Booking-Updates
- Keine Duplikate (CDC tracked exact changes)

**Setup-Schritte:**
```sql
-- 1. PostgreSQL konfigurieren
-- postgresql.conf:
wal_level = logical
max_replication_slots = 4
max_wal_senders = 4

-- 2. Publication erstellen
CREATE PUBLICATION turso_pub FOR TABLE cleaning_tasks;

-- 3. Replication Slot erstellen
SELECT pg_create_logical_replication_slot('turso_slot', 'pgoutput');

-- 4. pg_turso Plugin installieren und konfigurieren
-- Siehe: https://github.com/tursodatabase/pg_turso
```

**Nachteile/Risiken:**
- Noch Beta (Stand: 2025-11)
- Benötigt PostgreSQL Server-Konfiguration (Oracle Cloud)
- Setup-Aufwand: 3-4 Stunden
- Mehr Abhängigkeiten

**Wann implementieren:**
- Wenn stündlicher Sync zu langsam wird
- Wenn Real-Time Updates kritisch werden
- Wenn mehr als 50 Tasks/Tag erstellt werden

### Option 2: PowerSync

**Status:** Nicht empfohlen (kostenpflichtig)

**Was ist PowerSync?**
- Production-Ready PostgreSQL ↔ SQLite Sync Framework
- Bi-directional Sync
- Automatic Conflict Resolution
- Website: https://www.powersync.com

**Kosten:** $39+/Monat

**Wann evaluieren:**
- Wenn Bi-directional Sync benötigt wird (Mobile → PostgreSQL)
- Wenn Budget vorhanden ($468/Jahr)
- Wenn Support vom Hersteller wichtig ist

---

## Weitere Dokumentation

- [DATABASE.md](./DATABASE.md) - PostgreSQL Schema & Migrations
- [RELEASE.md](./RELEASE.md) - Release-Prozess
- [PUTZPLAN_FIX_COMPLETE.md](../PUTZPLAN_FIX_COMPLETE.md) - Duplicate Fix Details
