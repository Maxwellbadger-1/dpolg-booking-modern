# 🔄 Optimistic Update Pattern - KRITISCHE Regeln

## Das Problem: "Unbekannt" bei Gast/Zimmer

### Root Cause Analysis
**Symptom:** Nach dem Erstellen/Aktualisieren einer Buchung zeigt die UI "Unbekannt" für Gast und Zimmer.

**Ursache:** Backend gibt einfaches `Booking` struct zurück (nur `room_id`, `guest_id`), aber Frontend erwartet `BookingWithDetails` (mit nested `room: Room`, `guest: Guest` objects).

### Warum passiert das?

1. **Backend CREATE/UPDATE** gibt `Booking` zurück:
```rust
pub fn create_booking(...) -> Result<Booking> {
    // ...
    let booking = get_booking_by_id(id)?;  // ❌ nur IDs, keine nested objects!
    Ok(booking)
}
```

2. **Frontend Optimistic Update** erwartet vollständiges Objekt:
```typescript
const createBooking = async (data: any): Promise<Booking> => {
    const booking = await invoke<Booking>('create_booking_command', data);

    // ❌ Problem: booking.room und booking.guest sind undefined!
    setBookings(prev => [...prev, booking]);

    // Ergebnis: UI zeigt "Unbekannt"
};
```

---

## ✅ Die Lösung: Return BookingWithDetails

### Backend Pattern (Rust)

```rust
// ✅ RICHTIG: Return BookingWithDetails
pub fn create_booking(...) -> Result<BookingWithDetails> {
    let conn = Connection::open(get_db_path())?;

    // 1. INSERT INTO bookings ...
    let id = conn.last_insert_rowid();

    // 2. ✅ Load WITH nested objects
    let booking_with_details = get_booking_with_details_by_id(id)?;

    // 3. Transaction Log (use simple booking for JSON storage)
    let booking = get_booking_by_id(id)?;
    let booking_json = serde_json::to_string(&booking).unwrap_or_default();
    log_create("bookings", id, &booking_json, &format!("Buchung erstellt"));

    Ok(booking_with_details)  // ✅ Return with nested objects!
}

// ❌ FALSCH: Return Booking
pub fn create_booking(...) -> Result<Booking> {
    // ...
    let booking = get_booking_by_id(id)?;  // ❌ keine nested objects!
    Ok(booking)
}
```

### Frontend Pattern (TypeScript)

```typescript
// ✅ Interface muss nested objects haben
interface Booking {
  id: number;
  room_id: number;
  guest_id: number;
  // ... andere Felder
  room: Room;     // ✅ nested object
  guest: Guest;   // ✅ nested object
}

// ✅ Optimistic Update funktioniert
const createBooking = async (data: any): Promise<Booking> => {
    const booking = await invoke<Booking>('create_booking_command', data);

    // ✅ booking.room und booking.guest existieren!
    setBookings(prev => [...prev, booking]);

    return booking;
};
```

---

## 📋 Checkliste für ALLE CRUD Operations

### Backend (Rust)

Für JEDE Operation die ein Entity zurückgibt:

- [ ] **CREATE:** Return `EntityWithDetails` (mit nested objects)
- [ ] **UPDATE:** Return `EntityWithDetails` (mit nested objects)
- [ ] **GET_BY_ID:** Return `EntityWithDetails` (mit nested objects)
- [ ] **GET_ALL:** Return `Vec<EntityWithDetails>` (mit nested objects)
- [ ] **DELETE:** Kein Return nötig (oder verwende `()`)

### Tauri Commands (lib.rs)

```rust
#[tauri::command]
fn create_booking_command(...) -> Result<database::BookingWithDetails, String> {
    //                               ^^^^^^^^^^^^^^^^^^^^^^^^^^^ ✅ BookingWithDetails!
    database::create_booking(...)
        .map_err(|e| format!("Fehler: {}", e))
}
```

### Frontend (TypeScript)

```typescript
// ✅ Interface mit nested objects
interface Booking {
    room: Room;    // ✅ nicht nur room_id!
    guest: Guest;  // ✅ nicht nur guest_id!
}

// ✅ Invoke mit korrektem Type
const booking = await invoke<Booking>('create_booking_command', data);
```

---

## 🔍 Debugging: Wie man das Problem findet

### Symptome
- UI zeigt "Unbekannt" statt echten Namen
- `undefined` Fehler in Browser Console
- Empty fields wo Daten sein sollten

### Debugging Steps

1. **Browser Console öffnen:**
```typescript
console.log('Booking object:', booking);
console.log('Room:', booking.room);
console.log('Guest:', booking.guest);
```

2. **Prüfen ob nested objects existieren:**
```typescript
if (!booking.room || !booking.guest) {
    console.error('❌ Missing nested objects! Backend returns wrong type!');
}
```

3. **Backend Rust prüfen:**
```rust
// ❌ FALSCH - schau nach diesem Pattern:
pub fn create_something(...) -> Result<SimpleStruct> {
    let entity = get_by_id(id)?;  // ❌ nur IDs!
    Ok(entity)
}

// ✅ RICHTIG - sollte so aussehen:
pub fn create_something(...) -> Result<StructWithDetails> {
    let entity = get_with_details_by_id(id)?;  // ✅ mit nested objects!
    Ok(entity)
}
```

4. **SQL Query prüfen:**
```sql
-- ✅ RICHTIG: JOIN mit allen Feldern
SELECT
    b.id, b.room_id, b.guest_id, ...,
    r.id, r.name, r.gebaeude_typ, r.capacity, r.ort,  -- ✅ alle room fields
    g.id, g.vorname, g.nachname, g.email, g.telefon   -- ✅ alle guest fields
FROM bookings b
JOIN rooms r ON b.room_id = r.id
JOIN guests g ON b.guest_id = g.id
```

---

## 🚨 Common Mistakes

### Mistake 1: Backend gibt nur IDs zurück
```rust
// ❌ FALSCH
pub struct Booking {
    pub room_id: i64,   // ❌ nur ID!
    pub guest_id: i64,  // ❌ nur ID!
}

// ✅ RICHTIG
pub struct BookingWithDetails {
    pub room_id: i64,    // ✓ ID für Relations
    pub guest_id: i64,   // ✓ ID für Relations
    pub room: Room,      // ✅ nested object!
    pub guest: Guest,    // ✅ nested object!
}
```

### Mistake 2: Frontend refreshAll() nach CREATE
```typescript
// ❌ SCHLECHT: Unnötiger Reload, sichtbares Flickering
const createBooking = async (data: any) => {
    const booking = await invoke('create_booking_command', data);
    await refreshAll();  // ❌ Reload der GANZEN Liste!
};

// ✅ GUT: Optimistic Update, instant UI
const createBooking = async (data: any) => {
    const booking = await invoke('create_booking_command', data);
    setBookings(prev => [...prev, booking]);  // ✅ Instant!
};
```

### Mistake 3: TypeScript Interface passt nicht zu Backend
```typescript
// ❌ FALSCH: Interface erwartet nur IDs
interface Booking {
    room_id: number;  // ❌ Backend sendet aber room: Room!
    guest_id: number; // ❌ Backend sendet aber guest: Guest!
}

// ✅ RICHTIG: Interface matched Backend
interface Booking {
    room_id: number;   // ✓ für Relations
    guest_id: number;  // ✓ für Relations
    room: Room;        // ✅ nested object
    guest: Guest;      // ✅ nested object
}
```

---

## 📚 Best Practices

### 1. **Konsequente Namensgebung**
```rust
// Einfaches Struct (nur IDs)
pub struct Booking { ... }
pub fn get_booking_by_id() -> Result<Booking> { ... }

// Struct mit nested objects
pub struct BookingWithDetails { ... }
pub fn get_booking_with_details_by_id() -> Result<BookingWithDetails> { ... }
```

### 2. **Transaction Log Separation**
```rust
// Für Transaction Log: Simple Booking (kleineres JSON)
let booking = get_booking_by_id(id)?;
let json = serde_json::to_string(&booking)?;
log_create(..., &json, ...);

// Für Frontend Return: BookingWithDetails (vollständig)
let booking_with_details = get_booking_with_details_by_id(id)?;
Ok(booking_with_details)
```

### 3. **Type Safety überall**
```typescript
// ✅ Explizite Types in invoke()
const booking = await invoke<BookingWithDetails>('create_booking_command', data);
//                            ^^^^^^^^^^^^^^^^^^^ Type!

// ✅ Explizite Return Types
const createBooking = async (data: any): Promise<BookingWithDetails> => {
    //                                           ^^^^^^^^^^^^^^^^^^^^ Type!
```

---

## 🧪 Testing Checklist

Nach JEDEM CREATE/UPDATE Command testen:

1. [ ] **Erstelle Entity** (z.B. Stiftungsfall-Buchung)
2. [ ] **Überprüfe UI:** Werden Gast/Zimmer-Namen angezeigt?
3. [ ] **Browser Console:** Keine `undefined` Fehler?
4. [ ] **Inspect Object:** Hat das Objekt nested `room` und `guest`?
5. [ ] **Refresh Page:** Daten bleiben korrekt?

---

## 📖 Related Patterns

- **Optimistic Updates:** `CLAUDE.md` Section "Optimistic Updates"
- **SQL JOIN Best Practices:** Siehe Web-Recherche Ergebnisse
- **Error Handling:** CLAUDE.md Section "Error Handling"

---

**Erstellt:** 2025-10-10
**Letztes Problem:** Stiftungsfall-Buchungen zeigten "Unbekannt"
**Fix:** `create_booking` und `update_booking` geben jetzt `BookingWithDetails` zurück

---

## 🎯 Quick Reference

```rust
// ✅ Template für neue CRUD Operations
pub fn create_entity(...) -> Result<EntityWithDetails> {
    // 1. INSERT/UPDATE
    // 2. Get WITH details
    let entity = get_entity_with_details_by_id(id)?;
    // 3. Transaction Log (optional)
    // 4. Return WITH details
    Ok(entity)
}
```

```typescript
// ✅ Template für Frontend CRUD
const createEntity = async (data: any): Promise<EntityWithDetails> => {
    const entity = await invoke<EntityWithDetails>('create_entity_command', data);
    setEntities(prev => [...prev, entity]);  // Optimistic Update
    return entity;
};
```
