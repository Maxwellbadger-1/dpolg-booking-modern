# System-Architektur

**Version:** 1.9.0 (Unreleased)
**Stand:** 2025-02-12
**Stack:** Tauri 2 + React 19 + TypeScript + PostgreSQL 16

---

## Metriken

| Metrik | Wert | Seit v1.8.4 |
|--------|------|-------------|
| React Components | 63 | +3 (LockBadge, ConflictResolutionDialog, AuditLogViewer) |
| Tauri Commands | 182 | +8 (6 Lock + 2 Audit) |
| Datenbank-Tabellen | 33 | +1 (active_locks) |
| Repositories | 21 | +1 (LockRepository) |
| Custom Hooks | 7 | +1 (useLockManager) |
| Lines of Code (Rust) | ~6.800 | +600 |
| Lines of Code (React) | ~24.800 | +1.300 |

---

## Übersicht

```
┌─────────────────────────────────────────────────────────┐
│              Frontend (React + TypeScript)               │
│     TailwindCSS 4 | Vite 7 | date-fns | Lucide Icons    │
└────────────────────────┬────────────────────────────────┘
                         │ invoke('command', { params })
                         │
                         │ ◄───── listen('db-change') ─────┐
                         ↓                                  │
┌─────────────────────────────────────────────────────────┐│
│                  Tauri 2 (Rust Backend)                  ││
│                                                          ││
│  lib_pg.rs ─────→ Repositories ─────→ Database Pool     ││
│  (182 Commands)    (21 Repos)        (deadpool-postgres) ││
│                                                          ││
│  listener.rs ←── LISTEN/NOTIFY ←── PostgreSQL Triggers  │┘
│  (Real-Time)      (5 Channels)       (booking/guest/room/table/lock)
└────────────────────────┬────────────────────────────────┘
                         │
                         ↓
┌─────────────────────────────────────────────────────────┐
│              pgBouncer (Port 6432)                       │
│         Transaction Mode | 100 Max Connections           │
└────────────────────────┬────────────────────────────────┘
                         │
                         ↓
┌─────────────────────────────────────────────────────────┐
│           PostgreSQL 16.11 (Oracle Cloud)                │
│  33 Tabellen | Multi-User | Advisory Locks | Audit Log  │
│  NOTIFY Triggers | active_locks | audit_log             │
└─────────────────────────────────────────────────────────┘
```

---

## Frontend-Komponenten (63 total)

### Komponenten-Inventar

| Modul | Anzahl | Dateien |
|-------|--------|---------|
| BookingManagement | 14 | BookingList, BookingSidebar, BookingDetails, CompanionSelector, PaymentDateDialog, SearchableGuestPicker, SearchableRoomPicker, StatusDropdown, PaymentDropdown, InvoiceDropdown, CancellationConfirmDialog, EmailSelectionDialog, FilterDatePicker, BookingDatePicker |
| TapeChart | 6 | TapeChart, TapeChartFilters, ChangeConfirmationDialog, QuickActions, TapeChartHelpers, TodayLine |
| GuestManagement | 3 | GuestList, GuestDialog, GuestDetails |
| RoomManagement | 2 | RoomList, RoomDialog |
| Settings | 9 | SettingsDialog + 8 Tabs (General, Payment, Pricing, EmailConfig, EmailTemplates, PaymentRecipients, Notifications, Backup) |
| Email | 2 | EmailHistoryView, EmailConfigDialog |
| Reminders | 3 | RemindersView, BookingReminders, ReminderDropdown |
| Reports | 1 | ReportsView |
| Statistics | 1 | StatisticsView |
| TemplatesManagement | 5 | TemplatesManagement, ServiceTemplateList, ServiceTemplateDialog, DiscountTemplateList, DiscountTemplateDialog |
| DevTools | 2 | ComprehensiveDevTools, IntegrationTestSuite |
| **Shared** | **15** | ConfirmDialog, ContextMenu, PortalDropdown, QuickBookingFAB, UndoRedoButtons, CleaningSync, OfflineBanner, ErrorBoundary, **LockBadge**, **ConflictResolutionDialog**, **AuditLogViewer**, etc. |

### Verzeichnisstruktur

```
src/
├── App.tsx                    # Haupt-Komponente
├── main.tsx                   # React Entry Point
├── context/
│   ├── DataContext.tsx        # Zentraler State (Buchungen, Gäste, Zimmer)
│   ├── UserContext.tsx        # User Identity (Multi-User)
│   └── OnlineContext.tsx      # Online/Offline Detection
├── components/
│   ├── BookingManagement/     # Buchungsverwaltung
│   ├── GuestManagement/       # Gästeverwaltung
│   ├── RoomManagement/        # Zimmerverwaltung
│   ├── TapeChart/             # Timeline-Visualisierung
│   ├── Settings/              # Einstellungen (8 Tabs)
│   ├── Reminders/             # Erinnerungen
│   ├── Reports/               # Berichte
│   ├── Statistics/            # Statistiken
│   └── DevTools/              # Entwickler-Tools
├── hooks/                     # Custom Hooks
│   ├── usePriceCalculation.ts
│   ├── useBookingSync.ts
│   ├── useBatchPriceCalculation.ts
│   ├── useGlobalReminderUpdates.ts
│   ├── useLockManager.ts      # Advisory Locks Lifecycle
│   └── useDebounce.ts
├── types/                     # TypeScript Types
└── lib/                       # Utilities
    ├── commandManager.ts      # Undo/Redo
    └── emailProviders.ts      # SMTP Provider
```

---

## Backend-Struktur (Rust)

### Repository Pattern

21 Repositories in `src-tauri/src/database_pg/repositories/`:

| Repository | Tabelle | Hauptmethoden |
|------------|---------|---------------|
| booking_repository | bookings | get_all, get_by_id, create, update, delete, update_status |
| guest_repository | guests | get_all, get_by_id, create, update, delete, search |
| room_repository | rooms | get_all, get_by_id, create, update, delete |
| accompanying_guest_repository | accompanying_guests | get_by_booking, create, update, delete |
| additional_services_repository | additional_services | get_by_booking, create, update, delete |
| discount_repository | discounts | get_by_booking, create, update, delete |
| reminder_repository | reminders | get_all, get_by_booking, create, update, delete, complete, snooze |
| email_config_repository | email_config | get, update |
| email_template_repository | email_templates | get_all, get_by_name, create, update, delete |
| email_log_repository | email_log | create, get_by_booking, get_recent |
| scheduled_email_repository | scheduled_emails | get_pending, create, update_status |
| company_settings_repository | company_settings | get, update |
| pricing_settings_repository | pricing_settings | get, update |
| payment_settings_repository | payment_settings | get, update |
| notification_settings_repository | notification_settings | get, update |
| payment_recipients_repository | payment_recipients | get_all, create, update, delete |
| service_template_repository | service_templates | get_all, create, update, delete, toggle_active |
| discount_template_repository | discount_templates | get_all, create, update, delete, toggle_active |
| cleaning_tasks_repository | cleaning_tasks | get_by_date_range, create, delete |
| guest_credit_repository | guest_credit_transactions | get_balance, get_transactions, add_credit, use_credit |
| **lock_repository** | **active_locks** | **acquire_lock, release_lock, update_heartbeat, get_all_locks, get_lock_for_booking** |

### Verzeichnisstruktur

```
src-tauri/src/
├── main.rs                    # Entry Point
├── lib_pg.rs                  # 182 Tauri Commands
├── config.rs                  # Umgebungs-Konfiguration
│
├── database_pg/               # PostgreSQL Layer
│   ├── mod.rs                 # Public API
│   ├── pool.rs                # Connection Pooling (deadpool)
│   ├── error.rs               # Fehlerbehandlung
│   ├── models.rs              # Datenmodelle
│   └── repositories/          # 20 Repository-Dateien
│
├── pdf_generator_html.rs      # HTML→PDF (headless_chrome)
├── cleaning_timeline_pdf.rs   # Putzplan PDF
├── email_scheduler.rs         # E-Mail-Planung
├── turso_sync.rs              # Mobile App Sync
└── invoice_html.rs            # Rechnungs-Templates
```

---

## Datenbank-Schema (32 Tabellen)

### Kern-Tabellen (7)

| Tabelle | Spalten | Beschreibung |
|---------|---------|--------------|
| bookings | 15 | Buchungen mit Status, Zeitraum, Preisen |
| guests | 35 | Gästedaten, DPolG-Mitgliedschaft |
| rooms | 14 | Zimmerdaten, Preise nach Saison |
| accompanying_guests | 6 | Begleitpersonen zu Buchungen |
| additional_services | 8 | Zusatzleistungen pro Buchung |
| discounts | 5 | Rabatte pro Buchung |
| reminders | 11 | Erinnerungen mit Snooze |

### Settings-Tabellen (10)

| Tabelle | Beschreibung |
|---------|--------------|
| company_settings | Firmenname, Adresse, Logo |
| pricing_settings | Saisonzeiten, Basispreise |
| payment_settings | Zahlungsfristen, Bankdaten |
| email_config | SMTP-Einstellungen |
| email_templates | E-Mail-Vorlagen |
| notification_settings | Erinnerungs-Präferenzen |
| payment_recipients | Zahlungsempfänger |
| service_templates | Zusatzleistungs-Vorlagen |
| discount_templates | Rabatt-Vorlagen |
| backup_settings | Backup-Konfiguration |

### Kommunikation (3)

| Tabelle | Beschreibung |
|---------|--------------|
| email_log | E-Mail-Protokoll |
| scheduled_emails | Geplante E-Mails |
| guest_companions | Wiederverwendbare Begleitpersonen |

### System-Tabellen (12)

| Tabelle | Beschreibung |
|---------|--------------|
| cleaning_tasks | Reinigungsaufgaben (Mobile Sync) |
| guest_credit_transactions | Gäste-Guthaben Ledger |
| seasons | Saison-Definitionen |
| room_prices | Historische Zimmerpreise |
| price_history | Preis-Änderungsprotokoll |
| audit_log | Änderungs-Tracking |
| booking_attributes | Custom Buchungs-Attribute |
| booking_attribute_assignments | Attribut-Zuweisungen |
| booking_discounts | Buchung-Rabatt-Verknüpfung |
| booking_services | Buchung-Service-Verknüpfung |

---

## Architektur-Entscheidungen (ADR)

### ADR-001: PostgreSQL statt SQLite
- **Status:** Akzeptiert (v1.6.0)
- **Kontext:** Multi-User-Support für gleichzeitige Benutzer benötigt
- **Entscheidung:** Migration zu PostgreSQL 16 auf Oracle Cloud
- **Konsequenzen:**
  - 100+ gleichzeitige Benutzer möglich
  - pgBouncer für Connection Pooling erforderlich
  - Höhere Latenz (Cloud vs. lokal)

### ADR-002: Repository Pattern
- **Status:** Akzeptiert (v1.6.0)
- **Kontext:** Bessere Testbarkeit, Wartbarkeit, Trennung von Concerns
- **Entscheidung:** Ein Repository pro Entität
- **Konsequenzen:**
  - Konsistente API für alle Datenzugriffe
  - Einfachere Unit-Tests möglich
  - Mehr Boilerplate-Code

### ADR-003: Optimistic Updates
- **Status:** Akzeptiert (v1.0.0), **Modernisiert (v1.9.0)**
- **Kontext:** Responsive UI trotz Netzwerk-Latenz
- **Entscheidung:** UI-Update vor Backend-Bestätigung
- **Modernisierung (Phase 1 - v1.9.0):**
  - Backend Commands geben updated Entity zurück (`Promise<Booking>`)
  - Context Functions nutzen Return Value für immediate State-Update
  - Dual-Update: Optimistic (Command Pattern) + Backend Sync (Return Value)
  - Latenz: < 100ms (vorher: bis zu 3s durch Polling/NOTIFY)
  - Beispiel: `updateBookingPayment`, `updateBookingStatus`, `markInvoiceSent`
- **Konsequenzen:**
  - ✅ Sofortige UI-Reaktion (< 10ms durch Command Pattern)
  - ✅ Sofortiger Context-Sync (< 100ms durch Return Value)
  - ✅ Keine visual jumps mehr (kein Warten auf LISTEN/NOTIFY)
  - Rollback-Logik bei Fehlern nötig (Command Pattern `undo()`)
  - Komplexere State-Verwaltung

### ADR-004: Tauri statt Electron
- **Status:** Akzeptiert (v1.0.0)
- **Kontext:** Native Performance, kleinere Bundle-Größe
- **Entscheidung:** Tauri 2 mit Rust Backend
- **Konsequenzen:**
  - ~80% kleinere Installer
  - Native Performance
  - Rust-Lernkurve für Team

---

## Datenfluss

### Buchung erstellen

```
1. User klickt "Neue Buchung"
   │
2. BookingSidebar öffnet sich
   │
3. User füllt Formular aus
   │
4. "Speichern" → invoke('create_booking_pg', { ... })
   │
5. Tauri Command empfängt Daten
   │
6. BookingRepository::create() führt SQL aus
   │
7. PostgreSQL speichert Daten
   │
8. Response zurück an Frontend
   │
9. DataContext aktualisiert State (Optimistic Update)
   │
10. TapeChart rendert neue Buchung
```

---

## Externe Verbindungen

### PostgreSQL (Oracle Cloud)
- **Host:** 141.147.3.123
- **pgBouncer Port:** 6432 (empfohlen)
- **Direct Port:** 5432 (nur Admin)
- **Pool Mode:** Transaction
- **Max Connections:** 100

### Mobile App (Optional)
- Vercel Frontend
- Turso SQLite (Edge)
- Sync mit Hauptdatenbank

---

## Sicherheit

- TLS/SSL für alle DB-Verbindungen
- Passwörter nur in .env (nie in Git)
- Prepared Statements (kein SQL-Injection)
- Input-Validierung in Rust

---

## Multi-User Architektur

### Real-Time Synchronisation

```
User A ändert Buchung
        │
        ↓
┌─────────────────────────┐
│ PostgreSQL INSERT/UPDATE │
└───────────┬─────────────┘
            │
            ↓
┌─────────────────────────┐
│ NOTIFY Trigger auslösen │ → booking_changes Channel
└───────────┬─────────────┘
            │
            ↓
┌─────────────────────────┐
│ listener.rs empfängt    │ (poll_message)
└───────────┬─────────────┘
            │
            ↓
┌─────────────────────────┐
│ app.emit("db-change")   │ → Tauri Event System
└───────────┬─────────────┘
            │
            ↓
┌─────────────────────────┐
│ DataContext aktualisiert│ → User B sieht Änderung
└─────────────────────────┘
```

**Channels:**
- `booking_changes` - Buchungen
- `guest_changes` - Gäste
- `room_changes` - Zimmer
- `table_changes` - Andere Tabellen

### Doppelbuchungs-Schutz

```rust
// Transaktionale Verfügbarkeitsprüfung mit Row-Level Locking
pub async fn create_with_availability_check(pool, ...) {
    // 1. Transaction starten (SERIALIZABLE)
    let transaction = client.build_transaction()
        .isolation_level(IsolationLevel::Serializable)
        .start().await?;

    // 2. Überlappende Buchungen sperren
    transaction.query(
        "SELECT id FROM bookings WHERE room_id = $1
         AND checkin_date < $3 AND checkout_date > $2
         FOR UPDATE",  // ← Row-Level Lock
        &[&room_id, &checkin_date, &checkout_date]
    ).await?;

    // 3. Verfügbarkeit prüfen
    if !rows.is_empty() {
        return Err(DoubleBookingError("Zimmer bereits gebucht"));
    }

    // 4. Buchung einfügen
    transaction.query_one("INSERT INTO bookings ...").await?;

    // 5. Commit
    transaction.commit().await?;
}
```

**Fehlerbehandlung Frontend:**
```typescript
if (errorStr.includes('DOUBLE_BOOKING:')) {
    toast.error('🚫 Doppelbuchung verhindert: ' + message);
}
```

---

## Performance

| Aspekt | Implementierung |
|--------|-----------------|
| Connection Pooling | deadpool-postgres (20 Connections) |
| UI Updates | Optimistic Updates |
| Real-Time Sync | PostgreSQL LISTEN/NOTIFY (< 1s Latenz) |
| Fallback Sync | Polling alle 3 Sekunden |
| Große Listen | react-window Virtualisierung |
| Caching | DataContext hält State |
| Batch Operations | useBatchPriceCalculation |
| Race Prevention | Row-Level Locking (FOR UPDATE) |

---

## Key Patterns

### Frontend

**Optimistic Updates (Modernized Pattern - Phase 1):**
```typescript
// MODERNES PATTERN (seit v1.9.0)
// 1. UI sofort updaten (Command Pattern macht das automatisch)
// 2. Backend-Call mit Return Value
const updated = await updateBookingPayment(id, isPaid, zahlungsmethode, paymentDate);
// 3. Sofort Context/Local State updaten (kein Polling/LISTEN warten!)
setBooking(updated); // < 100ms Latenz

// ALTES PATTERN (deprecated)
// 1. UI sofort updaten
setBookings(prev => prev.map(b => b.id === id ? {...b, status} : b))
// 2. Backend-Call
await invoke('update_booking_status_pg', { bookingId: id, status })
// 3. Warten auf LISTEN/NOTIFY oder Polling (bis zu 3s Latenz) ❌
```

**Dual-Update Strategy:**
- **Optimistic Update**: Command Pattern updated UI sofort (< 10ms)
- **Backend Sync**: Return Value (`Promise<Booking>`) ermöglicht immediate Context-Update (< 100ms)
- **Fallback**: LISTEN/NOTIFY + Polling für Multi-User Sync (< 1s)
- **Vorteil**: Keine visual jumps, keine stale data, minimale Latenz

**Parameter-Konvention:**
```typescript
// Frontend: IMMER camelCase
invoke('update_booking', { bookingId: 1, guestName: "Max" })

// Backend: IMMER snake_case
fn update_booking(booking_id: i64, guest_name: String)
```

### Backend

**Repository Pattern:**
```rust
impl BookingRepository {
    pub async fn get_all(pool: &DbPool) -> DbResult<Vec<Booking>>
    pub async fn get_by_id(pool: &DbPool, id: i64) -> DbResult<Booking>
    pub async fn create(pool: &DbPool, data: CreateBooking) -> DbResult<Booking>
    pub async fn update(pool: &DbPool, data: UpdateBooking) -> DbResult<Booking>
    pub async fn delete(pool: &DbPool, id: i64) -> DbResult<()>
}
```

### Database Patterns

**Reminder Update System (Migration 017):**

Pattern für automatische Updates von Auto-Reminders bei Buchungsänderungen.

**Partial Unique Constraint:**
```sql
CREATE UNIQUE INDEX unique_active_reminder_per_booking_type
ON reminders (booking_id, reminder_type)
WHERE is_completed = FALSE;
```
- Erlaubt Historie (mehrere completed Reminders)
- Verhindert Duplikate (nur ein aktiver Reminder pro Typ)
- Basis für ON CONFLICT DO UPDATE Pattern

**ON CONFLICT DO UPDATE Pattern:**
```sql
INSERT INTO reminders (booking_id, reminder_type, title, description, due_date, ...)
VALUES (...)
ON CONFLICT (booking_id, reminder_type) WHERE is_completed = FALSE
DO UPDATE SET
    due_date = EXCLUDED.due_date,
    description = EXCLUDED.description,
    title = EXCLUDED.title,
    updated_at = CURRENT_TIMESTAMP;
```
- Idempotent: Funktioniert bei INSERT und UPDATE
- Automatische Aktualisierung bei Buchungsänderungen
- Vorbild: Email-System (Migration 005)

**Selective Trigger (Performance):**
```sql
CREATE TRIGGER trg_auto_create_reminders
    AFTER INSERT OR UPDATE OF
        checkin_date,           -- Payment/Checkin due_date
        guest_id,               -- Incomplete data + description
        status,                 -- Cancellation
        bezahlt,                -- Payment completion
        mahnung_gesendet_am,    -- Payment skip
        rechnung_versendet_am   -- Invoice completion
    ON bookings
    FOR EACH ROW
    EXECUTE FUNCTION auto_create_reminders_for_booking();
```
- Feuert nur bei relevanten Feldänderungen
- ~60% weniger Trigger-Ausführungen
- Vorbild: Email-System Trigger

**Cancellation Handler:**
```sql
IF NEW.status IN ('storniert', 'cancelled') THEN
    UPDATE reminders
    SET is_completed = true,
        description = description || ' [Buchung storniert]'
    WHERE booking_id = NEW.id
      AND is_completed = false
      AND reminder_type LIKE 'auto_%';
END IF;
```
- Auto-complete aller Auto-Reminders bei Stornierung
- Audit-Trail mit Suffix im description

**ON DELETE CASCADE:**
```sql
ALTER TABLE reminders
ADD CONSTRAINT reminders_booking_id_fkey
    FOREIGN KEY (booking_id)
    REFERENCES bookings(id)
    ON DELETE CASCADE;
```
- Automatisches Löschen von Reminders bei Buchungslöschung
- Verhindert verwaiste Reminders

---

## Preisberechnung-Architektur

### Zwei-Modus-System

Die Preisanzeige unterscheidet zwischen View-Mode und Edit-Mode:

```
┌─────────────────────────────────────────────────────────────────┐
│                    BookingSidebar                                │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  VIEW-MODE (bestehende Buchung anzeigen)                        │
│  ├─ Quelle: booking.grundpreis, booking.gesamtpreis             │
│  ├─ Zeigt: GESPEICHERTE Preise (historisch korrekt)             │
│  └─ Konsistent mit: Invoice/Rechnung                            │
│                                                                  │
│  EDIT/CREATE-MODE (neue Buchung oder bearbeiten)                │
│  ├─ Quelle: usePriceCalculation Hook                            │
│  ├─ Berechnet: DYNAMISCH mit aktuellen Zimmerpreisen            │
│  └─ Berücksichtigt: Saison, DPolG-Rabatt, Services, Discounts   │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

### Preisberechnung (Two-Pass Algorithm)

```
Pass 1: Services die auf overnight_price basieren
        └─ base = zimmerpreis * nächte

Pass 2: Services die auf total_price basieren
        └─ base = Pass1-Ergebnis (inkl. Services aus Pass 1)

Rabatte: Immer auf Subtotal (nach allen Services)
```

### Datentypen

| Feld | Typ | Warum |
|------|-----|-------|
| Alle Preisfelder | `f64` / `double precision` | Präzision für Währungsberechnungen |
| service_price | `double precision` | Berechneter Endpreis |
| original_value | `double precision` | Eingabewert (€ oder %) |
| discount_value | `double precision` | Berechneter Rabattbetrag |

### Automatische Preis-Neuberechnung (v1.9.1)

**Problem:** Nach Änderungen von Daten, Zimmer oder Gast waren gespeicherte Preise veraltet.

**Lösung: Backend-Triggered Price Recalculation**

```
┌─────────────────────────────────────────────────────────────────┐
│                    Automatischer Flow                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  1. Frontend: updateBooking({ checkoutDate: "2026-03-15" })     │
│  2. Backend: update_booking_pg erkennt Datumsänderung           │
│  3. Backend: recalculate_and_save_booking_prices()              │
│      ├─ Lädt: Booking, Guest, Room, Services, Discounts         │
│      ├─ Per-Night Berechnung: Hauptsaison/Nebensaison           │
│      ├─ Endreinigung: Automatisch wenn nicht in Services        │
│      ├─ Services: Two-Pass Algorithm (Fixed → Percent)          │
│      ├─ Discounts: DPolG-Rabatt wenn Mitglied                   │
│      ├─ Speichert: grundpreis, services_preis, gesamtpreis      │
│      └─ Speichert: discounts.calculated_amount für jeden Rabatt │
│  4. Backend: Reload Booking mit neuen Preisen                   │
│  5. Frontend: Response enthält aktualisierte Preise             │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

**Trigger-Bedingungen (Preis-Neuberechnung erfolgt wenn):**
- `checkin_date` geändert wird (Saisonwechsel)
- `checkout_date` geändert wird (Anzahl Nächte)
- `room_id` geändert wird (andere Zimmerpreise)
- `guest_id` geändert wird (DPolG-Mitgliedsstatus)

**Keine Neuberechnung bei:**
- Status-Änderungen (bestätigt → eingecheckt)
- Zahlungs-Updates (bezahlt, bezahlt_am)
- Bemerkungen-Änderungen

**Vorteile:**
- ✅ View-Mode und Rechnungen zeigen IMMER korrekte Preise
- ✅ Keine manuellen Preis-Korrekturen nötig
- ✅ Kein extra API-Call für Reload (Response enthält Preise)
- ✅ Automatisch konsistent für alle Anzeige-Bereiche

**Backend-Funktion:**
```rust
async fn recalculate_and_save_booking_prices(
    pool: &DbPool,
    booking_id: i32,
) -> Result<(), String>
```

---

## Enterprise Multi-User System (v1.9.0)

### Advisory Locks System

**Zweck:** Verhindert gleichzeitige Bearbeitung derselben Buchung durch mehrere Benutzer.

**Architektur:**
```
User öffnet Booking-Dialog
        │
        ↓
┌─────────────────────────────────────┐
│ useLockManager Hook                 │
│ ├─ acquireLock(bookingId, userName) │
│ ├─ Start Heartbeat (30s Interval)   │
│ └─ Auto-Release on unmount          │
└─────────────┬───────────────────────┘
              │
              ↓
┌─────────────────────────────────────┐
│ Backend: lock_repository.rs         │
│ ├─ INSERT INTO active_locks         │
│ ├─ ON CONFLICT → ConflictError      │
│ └─ NOTIFY lock_changes               │
└─────────────┬───────────────────────┘
              │
              ↓
┌─────────────────────────────────────┐
│ PostgreSQL active_locks Tabelle     │
│ ├─ booking_id (UNIQUE)              │
│ ├─ user_name                        │
│ ├─ last_heartbeat (Auto-Cleanup)    │
│ └─ locked_at (Timestamp)            │
└─────────────────────────────────────┘
```

**Cleanup-Mechanismus:**
```sql
CREATE FUNCTION cleanup_stale_locks() RETURNS void AS $$
BEGIN
    DELETE FROM active_locks
    WHERE last_heartbeat < NOW() - INTERVAL '5 minutes';
END;
$$ LANGUAGE plpgsql;
```

**Frontend Hook:**
```typescript
const { lockStatus, acquireLock, releaseLock } = useLockManager({
    bookingId,
    onLockConflict: (lockedBy) => {
        toast.warning(`🔒 Wird bereits bearbeitet von ${lockedBy}`);
    }
});

// Auto-acquire on mount
useEffect(() => {
    if (bookingId) acquireLock();
    return () => releaseLock(); // Auto-release on unmount
}, [bookingId]);
```

**Heartbeat-System:**
- Interval: 30 Sekunden
- Timeout: 5 Minuten Inaktivität → Auto-Unlock
- Verhindert "vergessene" Locks nach Browser-Crash

---

### Presence System

**Zweck:** Visuelles Feedback welcher User gerade was bearbeitet.

**UI-Komponenten:**

1. **LockBadge Component:**
```typescript
<LockBadge lockStatus={lockStatus} />

// Rendert:
// ✅ "Von Ihnen bearbeitet (seit 2 Min)" - Grün
// 🔒 "In Bearbeitung von Max Müller" - Amber
```

2. **Integration in BookingSidebar:**
```typescript
// View-Mode: Zeigt wer gerade bearbeitet
{viewMode && currentLock && (
    <LockBadge lockStatus={{
        isLocked: true,
        lockedBy: currentLock.userName,
        lockedSince: new Date(currentLock.lockedAt)
    }} />
)}

// Edit-Mode: Warnung falls Lock nicht erworben
{!viewMode && lockStatus.isLocked && !lockStatus.isOwnLock && (
    <div className="bg-red-50 border border-red-200 p-3 rounded">
        ⚠️ Achtung: Wird gerade von {lockStatus.lockedBy} bearbeitet
    </div>
)}
```

**Real-Time Updates:**
```typescript
// Listener in useLockManager Hook
useEffect(() => {
    const unlisten = listen<LockChangeEvent>('lock-change', (event) => {
        if (event.payload.bookingId === bookingId) {
            if (event.payload.action === 'ACQUIRED') {
                setLockStatus({
                    isLocked: true,
                    lockedBy: event.payload.userName,
                    lockedSince: new Date(event.payload.lockedAt!)
                });
            } else {
                setLockStatus({ isLocked: false, isOwnLock: false });
            }
        }
    });
    return () => { unlisten.then(fn => fn()); };
}, [bookingId]);
```

---

### Conflict Resolution System

**Zweck:** Benutzerfreundliche Auflösung von Bearbeitungskonflikten.

**Konflikt-Erkennung:**
```typescript
// Bei Save mit Optimistic Locking
try {
    await invoke('update_booking_pg', {
        id: bookingId,
        ...bookingData,
        expectedUpdatedAt: booking.updatedAt  // ← Version Check
    });
} catch (error) {
    if (error.includes('CONFLICT:')) {
        // Konflikt erkannt → Dialog öffnen
        const currentData = await invoke('get_booking_by_id_pg', { id: bookingId });
        const conflicts = detectConflicts(yourData, currentData, fieldLabels);
        setShowConflictDialog(true);
    }
}
```

**ConflictResolutionDialog Component:**
```
┌─────────────────────────────────────────────────────────────┐
│ Konflikt erkannt bei Buchung #123                           │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│ ┌─────────────┬─────────────────┬────────────────────────┐  │
│ │ Feld        │ Ihre Änderung   │ Aktuelle Daten        │  │
│ ├─────────────┼─────────────────┼────────────────────────┤  │
│ │ Gast        │ Max Müller      │ Max Müller            │  │
│ │ Check-in    │ 10.02.2025  [●] │ 11.02.2025        [ ] │  │ ← Radio Buttons
│ │ Check-out   │ 15.02.2025      │ 15.02.2025            │  │
│ │ Status      │ Bestätigt       │ Bestätigt             │  │
│ └─────────────┴─────────────────┴────────────────────────┘  │
│                                                              │
│ ⚠️ Geänderte Felder: Check-in                               │
│                                                              │
│ ┌──────────────┐ ┌──────────────┐ ┌───────────────────┐    │
│ │ Überschreiben│ │ Verwerfen    │ │ Ausgewählte Mergen│    │
│ └──────────────┘ └──────────────┘ └───────────────────┘    │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

**Auflösungs-Optionen:**
1. **Force (Überschreiben):** Ignoriere Konflikte, speichere deine Änderungen
2. **Discard (Verwerfen):** Verwerfe deine Änderungen, behalte aktuelle Daten
3. **Merge (Mergen):** Wähle Field-by-Field per Radio Buttons

**Konflikt-Erkennung Logic:**
```typescript
export function detectConflicts(
    yourData: Record<string, any>,
    currentData: Record<string, any>,
    fieldLabels: Record<string, string>
): FieldConflict[] {
    const conflicts: FieldConflict[] = [];

    for (const key of Object.keys(yourData)) {
        if (JSON.stringify(yourData[key]) !== JSON.stringify(currentData[key])) {
            conflicts.push({
                field: key,
                label: fieldLabels[key] || key,
                yourValue: yourData[key],
                currentValue: currentData[key]
            });
        }
    }

    return conflicts;
}
```

---

### Audit Log System

**Zweck:** Vollständige Change-History für Compliance und Debugging.

**Datenbank-Architektur:**
```sql
CREATE TABLE audit_log (
    id SERIAL PRIMARY KEY,
    table_name VARCHAR(50) NOT NULL,
    record_id INT NOT NULL,
    action VARCHAR(10) NOT NULL,  -- INSERT, UPDATE, DELETE
    old_values JSONB,
    new_values JSONB,
    user_name VARCHAR(100),
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Trigger-Funktion (Beispiel für bookings)
CREATE TRIGGER audit_booking_changes
    AFTER INSERT OR UPDATE OR DELETE ON bookings
    FOR EACH ROW EXECUTE FUNCTION log_booking_changes();
```

**Trigger-Implementierung:**
```sql
CREATE OR REPLACE FUNCTION log_booking_changes()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'UPDATE') THEN
        INSERT INTO audit_log (table_name, record_id, action, old_values, new_values, user_name)
        VALUES ('bookings', OLD.id, 'UPDATE', to_jsonb(OLD), to_jsonb(NEW),
                COALESCE(NEW.updated_by, 'system'));
        RETURN NEW;
    ELSIF (TG_OP = 'DELETE') THEN
        INSERT INTO audit_log (table_name, record_id, action, old_values, new_values, user_name)
        VALUES ('bookings', OLD.id, 'DELETE', to_jsonb(OLD), NULL,
                COALESCE(OLD.updated_by, 'system'));
        RETURN OLD;
    ELSIF (TG_OP = 'INSERT') THEN
        INSERT INTO audit_log (table_name, record_id, action, old_values, new_values, user_name)
        VALUES ('bookings', NEW.id, 'INSERT', NULL, to_jsonb(NEW),
                COALESCE(NEW.created_by, 'system'));
        RETURN NEW;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;
```

**Backend Commands:**
```rust
#[tauri::command]
async fn get_audit_log_pg(
    pool: State<'_, DbPool>,
    table_name: Option<String>,
    record_id: Option<i32>,
    limit: Option<i64>
) -> Result<Vec<AuditLog>, String> {
    // Flexible Query mit Filtern
}

#[tauri::command]
async fn get_booking_audit_log_pg(
    pool: State<'_, DbPool>,
    booking_id: i32
) -> Result<Vec<AuditLog>, String> {
    // Spezifisch für eine Buchung
}
```

**AuditLogViewer Component:**
```typescript
export function AuditLogViewer({ bookingId }: AuditLogViewerProps) {
    const [logs, setLogs] = useState<AuditLog[]>([]);
    const [expandedLogs, setExpandedLogs] = useState<Set<number>>(new Set());

    // Lädt Audit-Historie
    useEffect(() => {
        const result = bookingId
            ? await invoke('get_booking_audit_log_pg', { bookingId })
            : await invoke('get_audit_log_pg', { tableName: null, recordId: null, limit: 100 });
        setLogs(result);
    }, [bookingId]);

    // Rendert Timeline mit:
    // - Timestamp
    // - User-Name
    // - Action (INSERT/UPDATE/DELETE)
    // - Expandable JSON-Diff der geänderten Felder
}
```

**UI-Timeline:**
```
┌─────────────────────────────────────────────────────────┐
│ Änderungsverlauf für Buchung #123                       │
├─────────────────────────────────────────────────────────┤
│                                                          │
│ ● 12.02.2025 14:32  -  Max Müller                       │
│   UPDATE: Status geändert                               │
│   [▼ Details anzeigen]                                  │
│     ├─ status: "Anfrage" → "Bestätigt"                  │
│     └─ updated_at: "..." → "..."                        │
│                                                          │
│ ● 11.02.2025 09:15  -  Anna Schmidt                     │
│   UPDATE: Check-in Datum geändert                       │
│   [▼ Details anzeigen]                                  │
│     └─ checkin_date: "2025-02-10" → "2025-02-11"        │
│                                                          │
│ ● 10.02.2025 16:45  -  Max Müller                       │
│   INSERT: Buchung erstellt                              │
│   [Details anzeigen]                                    │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

---

### User Context System

**Zweck:** Multi-User-Identifikation ohne komplexes Auth-System.

**UserContext Implementation:**
```typescript
interface UserContextType {
    userName: string;
    setUserName: (name: string) => void;
}

export function UserProvider({ children }: { children: React.ReactNode }) {
    const [userName, setUserName] = useState<string>(() => {
        // Persistent via LocalStorage
        return localStorage.getItem('dpolg_user_name') || '';
    });

    useEffect(() => {
        if (userName) {
            localStorage.setItem('dpolg_user_name', userName);
        }
    }, [userName]);

    return (
        <UserContext.Provider value={{ userName, setUserName }}>
            {children}
        </UserContext.Provider>
    );
}
```

**Integration in CRUD-Operationen:**
```typescript
// BookingSidebar beim Speichern
const { userName } = useUser();

await invoke('create_booking_pg', {
    ...bookingData,
    createdBy: userName  // ← Audit Trail
});

await invoke('update_booking_pg', {
    ...bookingData,
    updatedBy: userName  // ← Audit Trail
});
```

**Erstmalige Einrichtung:**
```typescript
// App.tsx - Modal beim ersten Start
useEffect(() => {
    if (!userName) {
        // Zeige Dialog: "Bitte geben Sie Ihren Namen ein"
        setShowUserNameDialog(true);
    }
}, [userName]);
```

---

## Multi-User Testing Guide

### Lokales Testing (Entwicklung)

**Setup:**
```bash
# Build Release
npm run tauri build

# Starte mehrere Instanzen
cd src-tauri/target/release
./dpolg-booking.exe  # Instanz 1
./dpolg-booking.exe  # Instanz 2
```

**Test-Szenarien:**

1. **Advisory Lock Testing:**
   - Instanz 1: Öffne Buchung #123 → Sollte Lock erwerben
   - Instanz 2: Öffne Buchung #123 → Sollte "In Bearbeitung von User1" anzeigen
   - Instanz 1: Schließe Dialog → Lock wird freigegeben
   - Instanz 2: Sollte Badge verschwinden sehen (Real-Time)

2. **Conflict Resolution Testing:**
   - Instanz 1: Öffne Buchung #123, ändere Check-in auf 10.02
   - Instanz 2: Öffne Buchung #123, ändere Check-in auf 11.02
   - Instanz 1: Speichern → Erfolg
   - Instanz 2: Speichern → ConflictResolutionDialog öffnet sich
   - Wähle Merge-Option → Speichern erfolgreich

3. **Audit Log Testing:**
   - Instanz 1: Ändere Buchung → Status auf "Bestätigt"
   - Instanz 2: Öffne BookingSidebar → Klick "Änderungsverlauf"
   - Sollte sehen: "User1 hat Status geändert" mit Timestamp

4. **Real-Time Sync Testing:**
   - Instanz 1: Erstelle neue Buchung
   - Instanz 2: Sollte Buchung SOFORT im TapeChart sehen (< 1 Sekunde)
   - Keine manuelle Aktualisierung nötig

### PostgreSQL NOTIFY-Channels

**5 Channels aktiv:**
```
booking_changes   - INSERT/UPDATE/DELETE auf bookings
guest_changes     - INSERT/UPDATE/DELETE auf guests
room_changes      - INSERT/UPDATE/DELETE auf rooms
table_changes     - INSERT/UPDATE/DELETE auf andere Tabellen
lock_changes      - ACQUIRE/RELEASE von Advisory Locks
```

**Monitoring:**
```sql
-- Aktive Locks anzeigen
SELECT * FROM active_locks;

-- Audit-Log der letzten 10 Änderungen
SELECT * FROM audit_log ORDER BY timestamp DESC LIMIT 10;

-- Stale Locks manuell aufräumen
SELECT cleanup_stale_locks();
```

---

## Entwickler-Workflows

### Dev-Scripts (PowerShell)

Automatisierte Scripts für effiziente Entwicklung (Windows):

#### dev-start.ps1
**Zweck:** Startet die App im Dev-Modus mit automatischem Port-Management

**Features:**
- Prüft ob Port 1420 blockiert ist
- Killt alte `tauri-dev` Prozesse automatisch
- Startet `npx tauri dev` mit sauberem State
- Farbige Console-Ausgabe für bessere Lesbarkeit

**Verwendung:**
```powershell
.\dev-start.ps1
```

**Wann verwenden:**
- Nach einem App-Crash (automatisches Cleanup)
- Beim ersten Start des Tages
- Wenn Port-Blockade-Fehler auftreten

#### dev-stop.ps1
**Zweck:** Stoppt alle laufenden Dev-Prozesse

**Features:**
- Beendet Vite Dev-Server
- Killt Tauri Dev-Prozesse
- Gibt Ports 1420/5173 frei

**Verwendung:**
```powershell
.\dev-stop.ps1
```

**Wann verwenden:**
- Vor dem Herunterfahren des Rechners
- Bei hängenden Prozessen
- Vor einem Clean-Restart

#### cleanup.ps1
**Zweck:** Bereinigt Build-Artefakte und Caches

**Features:**
- Entfernt `node_modules/`, `target/`, `dist/`
- Löscht Vite/Tauri Caches
- **Achtung:** ~5-6 GB werden gelöscht, danach `npm install` erforderlich

**Verwendung:**
```powershell
.\cleanup.ps1
```

**Wann verwenden:**
- Bei mysteriösen Build-Fehlern
- Nach Updates von Dependencies
- Vor Device-Switch (Platzeinsparung)
- **NICHT** zwischen normalen Dev-Sessions (zu langsam)

---

## Gelöste Probleme & Lessons Learned

### PostgreSQL Timezone Format Bug (v1.9.0)

**Problem:**
- Email-Verlauf zeigte "Ungültiges Datum" bei neu versendeten E-Mails
- Nur alte E-Mails (ohne Mikrosekunden) funktionierten korrekt

**Root Cause:**
```
PostgreSQL gibt Timestamps zurück als:
"2026-02-12 04:47:23.736642+00"

JavaScript Date-Parser erwartet:
"2026-02-12T04:47:23.736642+00:00"  ✅ (mit Doppelpunkt)
"2026-02-12T04:47:23.736642Z"       ✅ (UTC Abkürzung)
"2026-02-12T04:47:23.736642+00"     ❌ (UNGÜLTIG!)
```

**Lösung:**
```typescript
// Frontend: EmailHistoryView.tsx formatDateTime()
// PostgreSQL gibt manchmal +00 statt +00:00 oder Z
if (isoString.endsWith('+00')) {
  isoString = isoString.replace(/\+00$/, 'Z');
}
```

**Zusätzliche Fixes:**
- TypeScript Interface korrigiert: `sent_at: string | null` (Type Mismatch mit Rust `Option<String>`)
- Backend WHERE-Klausel: `sent_at IS NOT NULL` für konsistente Filterung

**Debugging-Methode:**
- Massives Debug-Logging an allen kritischen Punkten (Backend + Frontend)
- Debug-Logs zeigten exakt welcher Wert problematisch war
- Nach Fix: Alle Debug-Logs wieder entfernt

**Lesson Learned:**
- PostgreSQL `::text` Cast kann unterschiedliche Formate produzieren (mit/ohne Mikrosekunden, mit/ohne Timezone)
- JavaScript Date ist strikt bei Timezone-Formaten (+00 vs +00:00)
- Immer beide Seiten der Serialization testen (Backend → JSON → Frontend)
- Type Mismatches zwischen Rust `Option<T>` und TypeScript `T` können zu Runtime-Fehlern führen
