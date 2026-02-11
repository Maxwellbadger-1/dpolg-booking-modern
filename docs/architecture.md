# System-Architektur

**Version:** 1.8.4
**Stand:** 2025-02-04
**Stack:** Tauri 2 + React 19 + TypeScript + PostgreSQL 16

---

## Metriken

| Metrik | Wert |
|--------|------|
| React Components | 60 |
| Tauri Commands | 174 |
| Datenbank-Tabellen | 32 |
| Repositories | 20 |
| Lines of Code (Rust) | ~6.200 |
| Lines of Code (React) | ~23.500 |

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
│  (174 Commands)    (20 Repos)        (deadpool-postgres) ││
│                                                          ││
│  listener.rs ←── LISTEN/NOTIFY ←── PostgreSQL Triggers  │┘
│  (Real-Time)      (Channels)         (AFTER INSERT/...)
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
│     32 Tabellen | Multi-User | NOTIFY Triggers          │
└─────────────────────────────────────────────────────────┘
```

---

## Frontend-Komponenten (60 total)

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
| Shared | 12 | ConfirmDialog, ContextMenu, PortalDropdown, QuickBookingFAB, UndoRedoButtons, CleaningSync, OfflineBanner, ErrorBoundary, etc. |

### Verzeichnisstruktur

```
src/
├── App.tsx                    # Haupt-Komponente
├── main.tsx                   # React Entry Point
├── context/
│   └── DataContext.tsx        # Zentraler State (Buchungen, Gäste, Zimmer)
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
│   └── useDebounce.ts
├── types/                     # TypeScript Types
└── lib/                       # Utilities
    ├── commandManager.ts      # Undo/Redo
    └── emailProviders.ts      # SMTP Provider
```

---

## Backend-Struktur (Rust)

### Repository Pattern

20 Repositories in `src-tauri/src/database_pg/repositories/`:

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

### Verzeichnisstruktur

```
src-tauri/src/
├── main.rs                    # Entry Point
├── lib_pg.rs                  # 174 Tauri Commands
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
- **Status:** Akzeptiert (v1.0.0)
- **Kontext:** Responsive UI trotz Netzwerk-Latenz
- **Entscheidung:** UI-Update vor Backend-Bestätigung
- **Konsequenzen:**
  - Sofortige UI-Reaktion
  - Rollback-Logik bei Fehlern nötig
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

**Optimistic Updates:**
```typescript
// 1. UI sofort updaten
setBookings(prev => prev.map(b => b.id === id ? {...b, status} : b))
// 2. Backend-Call
await invoke('update_booking_status_pg', { bookingId: id, status })
// 3. Bei Fehler: Rollback
```

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
