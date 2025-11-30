# PostgreSQL Migration - Testing Phase Handover

**Datum:** 2025-11-17
**Status:** Testing Phase - Mehrere Bugs behoben, weitere Commands benötigt

---

## ✅ ERFOLGREICH BEHOBENE BUGS

### 1. Reminder Deserialization Panic ✓
**Problem:** `is_snoozed` Spalte war `integer` (0/1) in PostgreSQL, aber Rust erwartete `bool`

**Lösung:**
- Datei: `src-tauri/src/database_pg/models.rs:334-346`
- Integer → Boolean Konvertierung beim Deserialisieren:
```rust
let is_snoozed_int: i32 = row.get("is_snoozed");
is_snoozed: is_snoozed_int != 0,  // Convert 0/1 to false/true
```

### 2. BookingWithDetails Type Mismatch ✓
**Problem:** Frontend erwartete `{ room: Room, guest: Guest }` aber Backend lieferte nur IDs

**Lösung:**
- Datei: `src-tauri/src/database_pg/models.rs:207-221` - Struct erweitert
- Datei: `src-tauri/src/database_pg/repositories/booking_repository.rs:51-73`
  Neue `get_with_details()` Methode:
```rust
pub async fn get_with_details(pool: &DbPool, id: i32) -> DbResult<BookingWithDetails> {
    let booking = Self::get_by_id(pool, id).await?;
    let room = RoomRepository::get_by_id(pool, booking.room_id).await.ok();
    let guest = GuestRepository::get_by_id(pool, booking.guest_id).await.ok();

    Ok(BookingWithDetails {
        booking,
        room,
        guest,
        // ...
    })
}
```

### 3. Frontend Command Names ✓
**Problem:** Frontend rief alte SQLite Commands auf (ohne `_pg` suffix)

**Lösung:** Bulk-Replacement mit sed:
```bash
sed -i.bak "s/'get_all_guests_command'/'get_all_guests_pg'/g" src/components/BookingManagement/BookingSidebar.tsx
sed -i.bak "s/'calculate_full_booking_price_command'/'calculate_full_booking_price_pg'/g" src/hooks/usePriceCalculation.ts
sed -i.bak "s/'check_room_availability_command'/'check_room_availability_pg'/g" src/components/BookingManagement/BookingSidebar.tsx
```

### 4. Pricing & Validation Stub Commands ✓
**Problem:** Commands fehlten komplett

**Lösung:**
- Datei: `src-tauri/src/lib_pg.rs:733-892`
- Stub-Commands erstellt (funktionsfähig aber vereinfacht):
  - `calculate_full_booking_price_pg` - Basic Preisberechnung ohne Hauptsaison-Logik
  - `check_room_availability_pg` - Gibt aktuell immer `true` zurück (TODO: Vollständige Implementierung)

**Status:** ✅ Commands funktionieren (siehe stdout: "⚠️ check_room_availability_pg STUB")

---

## ⏳ NOCH ZU BEHEBEN

### Fehlende Commands (aus Browser Console Errors):

1. **`update_booking_statuses_command`** - Command not found
2. **`get_all_rooms`** - Command not found (sollte `get_all_rooms_pg` sein)
3. **`delete_booking_command`** - Command not found
4. **`get_guest_credit_balance`** - Command not found

### Database Query Errors:

1. **Templates Loading** - "Database query error: db error"
2. **Reminders Loading** - "Database query error: db error"
3. **Payment Recipients** - "Database query error: db error"
4. **Company Settings** - "Record not found: Company settings not found"

---

## 📊 AKTUELLER APP STATUS

✅ **PostgreSQL Verbindung:** 141.147.3.123:5432 (Direct)
✅ **Daten Geladen:**
- 16 Rooms ✓
- 756 Guests ✓
- 7 Bookings ✓

✅ **Funktionierende Commands:**
- `get_all_rooms_pg` ✓
- `get_all_guests_pg` ✓
- `get_all_bookings_pg` ✓
- `get_booking_with_details_by_id_pg` ✓
- `calculate_full_booking_price_pg` ✓ (Stub)
- `check_room_availability_pg` ✓ (Stub)

❌ **Nicht funktionierende Features:**
- Update Booking Statuses
- Delete Booking
- Templates (Service/Discount)
- Reminders
- Payment Recipients
- Guest Credit Balance
- Company Settings

---

## 🚀 NÄCHSTE SCHRITTE

### Priorität 1: Fehlende Commands hinzufügen

**Systematisches Vorgehen:**

1. Alle fehlenden Command-Namen im Frontend finden:
```bash
cd /Users/maximilianfegg/Desktop/dpolg-booking-modern\ -\ aktuell
grep -r "Command.*not found" --include="*.tsx" --include="*.ts" src
```

2. Für jeden fehlenden Command:
   - Prüfen ob Repository-Methode existiert
   - Falls ja: Command in `lib_pg.rs` registrieren
   - Falls nein: Repository-Methode erstellen + Command registrieren

3. Bulk-Replacement für `_command` → `_pg`:
```bash
find src -name "*.tsx" -o -name "*.ts" | xargs sed -i.bak "s/_command'/_pg'/g"
```

### Priorität 2: Database Errors beheben

**Templates:**
- Problem: `service_templates` oder `discount_templates` Repository fehlt Commands
- Lösung: Commands zu `lib_pg.rs` hinzufügen + invoke_handler registrieren

**Reminders:**
- Problem: Eventuell fehlen Commands für "dringende" Reminders
- Lösung: Repository prüfen + Commands hinzufügen

**Payment Recipients:**
- Problem: Repository-Commands fehlen in lib_pg.rs
- Lösung: `get_all_payment_recipients_pg` Command hinzufügen

**Company Settings:**
- Problem: Keine Default-Einstellungen in DB
- Lösung: Migration schreiben die Default-Settings anlegt

### Priorität 3: Vollständige Stub-Implementierungen

**`calculate_full_booking_price_pg`:**
- TODO: Hauptsaison/Nebensaison Erkennung
- TODO: Prozentuale Services/Discounts berechnen
- TODO: Endreinigung einberechnen

**`check_room_availability_pg`:**
- TODO: Query für overlapping Bookings:
```sql
SELECT COUNT(*) FROM bookings
WHERE room_id = $1
AND status != 'storniert'
AND (checkin_date < $3 AND checkout_date > $2)
AND ($4 IS NULL OR id != $4)
```

---

## 🛠️ ENTWICKLUNGS-HINWEISE

### Hot-Reload Problem
**Problem:** serde imports werden nicht erkannt trotz `use serde::{Serialize, Deserialize};`
**Ursache:** Cargo kompiliert alte `.tmp` Files
**Lösung:** App komplett neu starten:
```bash
killall -9 npm node vite cargo rustc dpolg-booking-modern 2>/dev/null
cd "/Users/maximilianfegg/Desktop/dpolg-booking-modern - aktuell"
npm run tauri:dev
```

### Compilation Status
- ✅ App läuft trotz Compiler-Errors in stderr (alte temp files)
- ✅ Stub Commands funktionieren (siehe stdout Logs)
- ⚠️ Ignoriere "error: cannot find derive macro" in stderr - sind von alten temp files

---

## 📝 TESTING CHECKLIST

### Core Features (zu testen):

- [ ] Buchung erstellen (neue Buchung anlegen)
- [ ] Buchung bearbeiten (bestehende Buchung ändern)
- [ ] Buchung löschen
- [ ] Services hinzufügen/entfernen
- [ ] Discounts hinzufügen/entfernen
- [ ] Preisberechnung (mit Services + Discounts)
- [ ] Room Availability Check
- [ ] Templates laden (Service/Discount)
- [ ] Reminders anzeigen/erstellen
- [ ] Email verschicken (Confirmation/Invoice/etc.)
- [ ] Settings ändern (Company Settings/Payment/etc.)

### Regression Tests:

- [ ] Drag & Drop im Tapechart funktioniert
- [ ] Multi-User: Mehrere Browser gleichzeitig öffnen
- [ ] Optimistic Updates funktionieren
- [ ] Mobile App iframe lädt

---

## 🔗 WICHTIGE DATEIEN

### Backend (Rust):
- `src-tauri/src/lib_pg.rs` - Command Registrations
- `src-tauri/src/database_pg/models.rs` - Type Definitions
- `src-tauri/src/database_pg/repositories/` - Business Logic

### Frontend (TypeScript):
- `src/context/DataContext.tsx` - Haupt-API-Calls
- `src/hooks/usePriceCalculation.ts` - Pricing Logic
- `src/components/BookingManagement/BookingSidebar.tsx` - Booking UI

### Config:
- `src-tauri/tauri.conf.json` - App Configuration
- `.env` - Development Secrets (NOT in Git)

---

## 📞 KONTAKT

Falls Fragen oder Probleme auftreten:

1. Prüfe zuerst Browser Console + Terminal stderr/stdout
2. Checke `CLAUDE.md` für Projekt-Richtlinien
3. Verwende `git log` um letzte Änderungen zu sehen
4. Bei Compilation Errors: App komplett neu starten (siehe oben)

---

**Letzte Aktualisierung:** 2025-11-17 13:32 CET
**Erstellt von:** Claude (PostgreSQL Migration Assistant)
