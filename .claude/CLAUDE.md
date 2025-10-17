# Claude Code Projekt-Richtlinien
## DPolG Buchungssystem - Modern Tauri React Application

---

## 🎯 Projekt-Vision

Ein modernes, performantes Hotel-Buchungssystem mit intuitiver Tape Chart Visualisierung, entwickelt mit Tauri 2 + React + TypeScript. Fokus auf Benutzerfreundlichkeit, Geschwindigkeit und Wartbarkeit.

---

## 🚨 KRITISCHE REGEL #1: Tauri + Serde camelCase/snake_case - VOLLSTÄNDIGER LEITFADEN

**⚠️ DIESES PROBLEM TRITT IMMER WIEDER AUF! LIES DIESE REGEL AUFMERKSAM!**

Diese Regel basiert auf Web-Recherche und realen Debugging-Sessions. Sie deckt ALLE Aspekte der camelCase/snake_case Konvertierung in Tauri ab.

---

## 📋 Inhaltsverzeichnis

1. [Das Grundproblem](#das-grundproblem)
2. [Frontend → Backend (invoke Parameter)](#frontend--backend-invoke-parameter)
3. [Backend → Frontend (Serde Serialization)](#backend--frontend-serde-serialization)
4. [Struct vs Individuelle Parameter](#struct-vs-individuelle-parameter)
5. [Debugging Checklist](#debugging-checklist)
6. [Komplette Beispiele](#komplette-beispiele)

---

## 🔍 Das Grundproblem

**Rust Convention:** `snake_case` für Variablen/Felder
**JavaScript/TypeScript Convention:** `camelCase` für Variablen/Properties

**Tauri's Rolle:**
- Konvertiert **automatisch** Parameter-Namen zwischen beiden Welten
- **ABER:** Nur unter bestimmten Bedingungen!
- **ACHTUNG:** Unterschiedliches Verhalten für Parameter vs Struct-Properties!

---

## 1️⃣ Frontend → Backend (invoke Parameter)

### Wie Tauri Parameter konvertiert:

**Tauri konvertiert automatisch Top-Level Parameter von camelCase → snake_case**

```typescript
// Frontend
await invoke('update_room', {
  roomId: 1,        // → Rust: room_id
  roomName: "Test"  // → Rust: room_name
});

// Rust Backend empfängt:
fn update_room(room_id: i64, room_name: String) { }
```

### ❌ KRITISCHER FEHLER: Parameter-Mixing

**Wenn auch nur EIN Parameter in snake_case ist, BRICHT die Konvertierung für ALLE!**

```typescript
// ❌ FALSCH - Mixed Naming
const payload = {
  roomId: 1,              // camelCase
  guestId: 1,             // camelCase
  payment_recipient_id: 1 // snake_case - FEHLER!
};

await invoke('update_booking', payload);

// Backend erhält:
// room_id: Some(1)           ← Funktioniert
// guest_id: Some(1)          ← Funktioniert
// payment_recipient_id: None ← VERLOREN!
```

### ✅ RICHTIG: Konsistentes camelCase

```typescript
// ✅ RICHTIG - Alles camelCase
const payload = {
  roomId: 1,                // ✅
  guestId: 1,               // ✅
  paymentRecipientId: 1     // ✅ Konsistent!
};

await invoke('update_booking', payload);

// Backend erhält:
// room_id: Some(1)           ← Funktioniert
// guest_id: Some(1)          ← Funktioniert
// payment_recipient_id: Some(1) ← Funktioniert!
```

### 🚨 REGEL für Frontend → Backend:

**ALLE Parameter die von Frontend → Backend gesendet werden MÜSSEN camelCase sein!**

**KEINE AUSNAHMEN! KEIN MIXING!**

---

## 2️⃣ Backend → Frontend (Serde Serialization)

### Das Problem:

**Tauri konvertiert NICHT automatisch Rust Struct Properties beim Zurückgeben!**

Rust serialisiert Structs mit Serde → JSON → Frontend

```rust
// Rust Struct
#[derive(Serialize)]
pub struct Room {
    pub id: i64,
    pub street_address: Option<String>,  // snake_case
    pub postal_code: Option<String>,     // snake_case
}
```

```json
// JSON an Frontend (DEFAULT)
{
  "id": 1,
  "street_address": "Hauptstraße 1",  // snake_case ❌
  "postal_code": "12345"               // snake_case ❌
}
```

```typescript
// TypeScript erwartet:
interface Room {
  id: number;
  streetAddress?: string;  // camelCase!
  postalCode?: string;     // camelCase!
}

// room.streetAddress  → undefined ❌
// room.street_address → "Hauptstraße 1" (falsch!)
```

### ✅ LÖSUNG 1: Serde `#[serde(rename)]` für einzelne Felder

**Verwende dies wenn NUR EINIGE neue Felder zu einem bestehenden Struct hinzugefügt werden:**

```rust
#[derive(Serialize, Deserialize)]
pub struct Room {
    pub id: i64,
    pub name: String,
    pub gebaeude_typ: String,           // Alte Felder bleiben snake_case
    pub price_member: f64,              // (historische Gründe)
    #[serde(rename = "streetAddress")]  // ← Einzelfeld rename!
    pub street_address: Option<String>,
    #[serde(rename = "postalCode")]     // ← Einzelfeld rename!
    pub postal_code: Option<String>,
    pub city: Option<String>,           // bleibt "city"
}
```

**Wann verwenden:**
- ✅ Bestehende Structs mit gemischten Konventionen
- ✅ Nur einzelne neue Felder hinzufügen
- ✅ Backwards-Kompatibilität mit Frontend nötig

### ✅ LÖSUNG 2: Serde `#[serde(rename_all = "camelCase")]` für ganze Struct

**Verwende dies für NEUE Structs oder komplette Refactorings:**

```rust
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]  // ← Ganzes Struct!
pub struct NewEntity {
    pub id: i64,
    pub first_name: String,         // → firstName
    pub last_name: String,          // → lastName
    pub street_address: String,     // → streetAddress
    pub postal_code: String,        // → postalCode
}
```

**Wann verwenden:**
- ✅ Neue Structs von Anfang an
- ✅ Alle Felder sollen konvertiert werden
- ✅ Keine gemischten Konventionen

### ⚠️ WICHTIG: `rename_all` überschreibt ALLE Felder!

```rust
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Example {
    pub some_field: String,       // → someField ✅
    pub another_field: i64,       // → anotherField ✅
    pub old_snake_name: String,   // → oldSnakeName ⚠️  (wird auch konvertiert!)
}
```

### 🚨 REGEL für Backend → Frontend:

**Neue Struct-Felder MÜSSEN mit `#[serde(rename = "camelCase")]` annotiert werden!**

**Bestehende gemischte Structs:** Nur neue Felder mit `#[serde(rename)]` annotieren
**Neue Structs:** Gesamtes Struct mit `#[serde(rename_all = "camelCase")]` annotieren

---

## 3️⃣ Struct vs Individuelle Parameter

### Das große Missverständnis:

**Tauri konvertiert nur Top-Level Parameter-Namen, NICHT Struct-Properties!**

### ❌ FALSCH: Nested Struct Properties

```typescript
// Frontend - Struct als Parameter
const roomData = {
  name: "Zimmer 1",
  street_address: "Test",  // ❌ snake_case in nested object
  postal_code: "12345"     // ❌
};

await invoke('update_room', { roomData });  // ← Struct as Parameter

// Rust Command erwartet:
#[tauri::command]
fn update_room(room_data: RoomData) -> Result<Room, String> {
    // room_data.street_address → None ❌
    // room_data.postal_code    → None ❌
}
```

**Warum geht das nicht?**
- Tauri konvertiert `roomData` → `room_data` (Top-Level ✅)
- **ABER:** Properties INNERHALB von `roomData` werden NICHT konvertiert!
- Struct benötigt `#[serde(rename_all = "camelCase")]` oder individuelle `#[serde(rename)]`

### ✅ RICHTIG Option A: Individuelle Parameter

```typescript
// Frontend - Alle Parameter einzeln übergeben
await invoke('update_room', {
  id: 1,
  name: "Zimmer 1",
  gebaeudeTyp: "App",         // camelCase!
  capacity: 2,
  priceMember: 51,            // camelCase!
  priceNonMember: 60,         // camelCase!
  streetAddress: "Test",      // camelCase!
  postalCode: "12345",        // camelCase!
  city: "Berlin"
});

// Rust Command:
#[tauri::command]
fn update_room(
    id: i64,
    name: String,
    gebaeude_typ: String,    // ← Tauri konvertiert automatisch
    capacity: i32,
    price_member: f64,       // ← Tauri konvertiert automatisch
    price_non_member: f64,   // ← Tauri konvertiert automatisch
    street_address: Option<String>,
    postal_code: Option<String>,
    city: Option<String>,
) -> Result<Room, String> { }
```

**Vorteil:** Tauri's automatische Konvertierung funktioniert! ✅

### ✅ RICHTIG Option B: Struct mit Serde Annotation

```rust
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]  // ← WICHTIG!
pub struct UpdateRoomRequest {
    pub id: i64,
    pub name: String,
    pub gebaeude_typ: String,
    pub street_address: Option<String>,
    pub postal_code: Option<String>,
}

#[tauri::command]
fn update_room(request: UpdateRoomRequest) -> Result<Room, String> {
    // Jetzt funktioniert's! ✅
}
```

```typescript
// Frontend - Struct als Parameter
const request = {
  id: 1,
  name: "Test",
  gebaeudeTyp: "App",      // camelCase → gebaeude_typ
  streetAddress: "Test",   // camelCase → street_address
  postalCode: "12345"      // camelCase → postal_code
};

await invoke('update_room', { request });
```

### 🚨 REGEL: Struct vs Individuelle Parameter

**Option 1: Individuelle Parameter (Empfohlen für bestehende Commands)**
- ✅ Tauri konvertiert automatisch
- ✅ Keine Serde-Annotations nötig
- ❌ Viele Parameter = lange Signatur

**Option 2: Struct (Empfohlen für neue Commands mit vielen Parametern)**
- ✅ Saubere Command-Signatur
- ✅ Einfach zu erweitern
- ⚠️ Benötigt `#[serde(rename_all = "camelCase")]` oder individuelle `#[serde(rename)]`

---

## 4️⃣ Debugging Checklist

### Symptom: Parameter kommt als `None`/`undefined` im Backend an

**Checklist:**
1. ✅ Ist der Parameter im Frontend in camelCase geschrieben?
2. ✅ Sind ALLE anderen Parameter auch camelCase? (kein Mixing!)
3. ✅ Wird ein Struct als Parameter übergeben?
   - JA → Hat das Rust-Struct `#[serde(rename_all = "camelCase")]`?
   - NEIN → Sind alle Parameter Top-Level im invoke-Object?
4. ✅ Console-Log: Welche Daten sendet das Frontend wirklich?
5. ✅ Rust println!: Was empfängt das Backend wirklich?

### Symptom: Frontend empfängt snake_case statt camelCase

**Checklist:**
1. ✅ Hat das Rust-Struct `#[derive(Serialize)]`?
2. ✅ Hat es `#[serde(rename = "fieldName")]` für betroffene Felder?
   - ODER: `#[serde(rename_all = "camelCase")]` für das ganze Struct?
3. ✅ Console-Log: Was gibt das Backend zurück (JSON.stringify)?
4. ✅ TypeScript Interface: Matched es die erwarteten camelCase Namen?

### Debug-Tools:

```typescript
// Frontend: Zeige EXAKT was gesendet wird
console.log('📤 Sending to Tauri:', JSON.stringify(payload, null, 2));
const result = await invoke('command', payload);
console.log('📥 Received from Tauri:', JSON.stringify(result, null, 2));
```

```rust
// Backend: Zeige EXAKT was empfangen wird
#[tauri::command]
fn my_command(param1: String, param2: Option<String>) -> Result<Data, String> {
    println!("🔍 Received param1: {:?}", param1);
    println!("🔍 Received param2: {:?}", param2);
    // ...
}
```

---

## 5️⃣ Komplette Beispiele

### Beispiel 1: Room Update (Individuelle Parameter - EMPFOHLEN)

```typescript
// Frontend: RoomDialog.tsx
const roomData = {
  name: formData.name,
  gebaeudeTyp: formData.gebaeude_typ,        // camelCase!
  capacity: formData.capacity,
  priceMember: formData.price_member,        // camelCase!
  priceNonMember: formData.price_non_member, // camelCase!
  streetAddress: formData.streetAddress,     // camelCase!
  postalCode: formData.postalCode,           // camelCase!
  city: formData.city,
};

await invoke('update_room_command', {
  id: room.id,
  ...roomData  // Alle Properties auf Top-Level!
});
```

```rust
// Backend: lib.rs
#[tauri::command]
fn update_room_command(
    id: i64,
    name: String,
    gebaeude_typ: String,         // ← Tauri konvertiert von gebaeudeTyp
    capacity: i32,
    price_member: f64,            // ← Tauri konvertiert von priceMember
    price_non_member: f64,
    street_address: Option<String>, // ← Tauri konvertiert von streetAddress
    postal_code: Option<String>,
    city: Option<String>,
) -> Result<Room, String> {
    database::update_room(
        id, name, gebaeude_typ, capacity,
        price_member, price_non_member,
        street_address, postal_code, city
    )
}
```

```rust
// Backend: database.rs - Room Struct
#[derive(Serialize, Deserialize)]
pub struct Room {
    pub id: i64,
    pub name: String,
    pub gebaeude_typ: String,  // Alte Felder bleiben snake_case
    pub capacity: i32,
    pub price_member: f64,
    pub price_non_member: f64,
    #[serde(rename = "streetAddress")]  // ← Nur neue Felder mit rename!
    pub street_address: Option<String>,
    #[serde(rename = "postalCode")]
    pub postal_code: Option<String>,
    pub city: Option<String>,
}
```

### Beispiel 2: Neue Entity (Struct mit rename_all)

```typescript
// Frontend
const guestData = {
  firstName: "Max",
  lastName: "Müller",
  emailAddress: "max@test.de",
  phoneNumber: "+49123",
  streetAddress: "Teststr. 1",
  postalCode: "12345",
  cityName: "Berlin"
};

await invoke('create_guest', { guestData });
```

```rust
// Backend
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]  // ← Für ganzes Struct!
pub struct CreateGuestRequest {
    pub first_name: String,       // ← von firstName
    pub last_name: String,        // ← von lastName
    pub email_address: String,
    pub phone_number: String,
    pub street_address: String,
    pub postal_code: String,
    pub city_name: String,
}

#[tauri::command]
fn create_guest(guest_data: CreateGuestRequest) -> Result<Guest, String> {
    // guest_data.first_name ✅
    // guest_data.street_address ✅
    // Alles funktioniert!
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]  // ← Auch für Response!
pub struct Guest {
    pub id: i64,
    pub first_name: String,  // → firstName im JSON
    pub last_name: String,   // → lastName im JSON
    pub email_address: String,
}
```

---

## 📚 Zusammenfassung & Quick Reference

### Frontend → Backend:
✅ **IMMER** alle Parameter in **camelCase** schreiben
✅ **Individuelle Parameter** bevorzugen (Tauri konvertiert automatisch)
⚠️ Bei Structs: Rust-Struct braucht `#[serde(rename_all = "camelCase")]`

### Backend → Frontend:
✅ **Neue Felder:** Mit `#[serde(rename = "fieldName")]` annotieren
✅ **Neue Structs:** Mit `#[serde(rename_all = "camelCase")]` annotieren
✅ **Bestehende Structs:** Nur neue Felder mit `#[serde(rename)]`

### Debugging:
✅ Console-Logs im Frontend (JSON.stringify)
✅ println! im Backend
✅ Prüfen: camelCase consistency
✅ Prüfen: Serde annotations vorhanden

### Golden Rules:
1. **Frontend sendet IMMER camelCase** (keine Ausnahmen!)
2. **Backend annotiert IMMER neue Felder** mit Serde rename
3. **Individuelle Parameter > Structs** (wenn möglich)
4. **Bei Structs:** `#[serde(rename_all = "camelCase")]` nicht vergessen!
5. **Testen:** Mit Console-Logs BEIDE Seiten prüfen

---

## ⚡ Optimistic Updates - KRITISCHE REGEL!

**WICHTIG:** ALLE Datenänderungen MÜSSEN Optimistic Updates verwenden - NIEMALS `refreshBookings()`, `refreshGuests()` oder `refreshRooms()` nach erfolgreichen Operationen aufrufen!

### Warum Optimistic Updates?
- ✅ **Instant UI Response** - User sieht Änderung SOFORT (< 10ms statt 100ms+ mit Backend-Roundtrip)
- ✅ **Keine sichtbaren Reloads** - Kein Flackern, kein "Loading..." Zustand
- ✅ **Bessere UX** - App fühlt sich nativ und responsive an
- ❌ **OHNE Optimistic Update:** Jede Änderung = sichtbarer Page Reload = schlechte UX!

### Pattern für ALLE CRUD-Operationen:

```typescript
// ✅ RICHTIG - Optimistic Update Pattern
const updateEntity = useCallback(async (id: number, newData: any): Promise<Entity> => {
  // 1. Backup für Rollback
  const oldEntity = entities.find(e => e.id === id);

  // 2. SOFORT im UI ändern (KEIN await, KEIN refresh!)
  setEntities(prev => prev.map(e =>
    e.id === id ? { ...e, ...newData } : e
  ));

  try {
    // 3. Backend Update
    const result = await invoke('update_entity_command', { id, ...newData });

    // 4. NUR refresh-data Event triggern (für Undo-Button)
    window.dispatchEvent(new CustomEvent('refresh-data'));

    return result;
  } catch (error) {
    // 5. Rollback bei Fehler
    if (oldEntity) {
      setEntities(prev => prev.map(e =>
        e.id === id ? oldEntity : e
      ));
    }
    throw error;
  }
}, [entities]); // WICHTIG: Keine refresh-Funktionen in Dependencies!

// ❌ FALSCH - Mit refresh (verursacht Page Reload!)
const updateEntityBad = useCallback(async (id: number, newData: any) => {
  const result = await invoke('update_entity_command', { id, ...newData });
  await refreshEntities(); // ❌ VERBOTEN! Verursacht sichtbaren Reload!
  return result;
}, [refreshEntities]);
```

### Optimistic Update für CREATE:

```typescript
const createEntity = useCallback(async (data: any): Promise<Entity> => {
  try {
    // 1. Backend Create
    const newEntity = await invoke<Entity>('create_entity_command', data);

    // 2. SOFORT zum State hinzufügen (KEIN refresh!)
    setEntities(prev => [...prev, newEntity]);

    // 3. Event für Undo-Button
    window.dispatchEvent(new CustomEvent('refresh-data'));

    return newEntity;
  } catch (error) {
    // Kein Rollback nötig - Entity wurde noch nicht hinzugefügt
    throw error;
  }
}, []);
```

### Optimistic Update für DELETE:

```typescript
const deleteEntity = useCallback(async (id: number): Promise<void> => {
  // 1. Backup für Rollback
  const deletedEntity = entities.find(e => e.id === id);

  // 2. SOFORT aus UI entfernen (KEIN refresh!)
  setEntities(prev => prev.filter(e => e.id !== id));

  try {
    // 3. Backend Delete
    await invoke('delete_entity_command', { id });

    // 4. Event für Undo-Button
    window.dispatchEvent(new CustomEvent('refresh-data'));
  } catch (error) {
    // 5. Rollback - Entity wiederherstellen
    if (deletedEntity) {
      setEntities(prev => [...prev, deletedEntity]);
    }
    throw error;
  }
}, [entities]);
```

### Wann refresh() erlaubt ist:

**NUR in diesen Fällen:**
1. ✅ Initial Load (`useEffect` beim Mount)
2. ✅ User klickt explizit auf "Refresh" Button
3. ✅ Nach Undo-Operation (globaler Refresh via `refresh-data` Event)
4. ✅ Bei Fehlern die inkonsistenten State verursachen könnten

**NIEMALS:**
- ❌ Nach erfolgreichen CREATE/UPDATE/DELETE Operationen
- ❌ Nach Status-Änderungen
- ❌ Nach Zahlungs-Updates
- ❌ Nach irgendeiner User-Aktion die funktioniert hat

### Event System:

**KRITISCH:** Zwei verschiedene Events für unterschiedliche Zwecke!

```typescript
// 1️⃣ 'refresh-data' Event - Nach normalen CRUD-Operationen
// Wird NUR von UndoRedoButtons gehört (um Transaction-Liste zu aktualisieren)
// Löst KEIN globales refresh aus! (damit Optimistic Updates erhalten bleiben)
window.dispatchEvent(new CustomEvent('refresh-data'));

// UndoRedoButtons Component:
useEffect(() => {
  const handleRefresh = () => {
    loadTransactions(); // NUR Transaction-Liste neu laden!
  };
  window.addEventListener('refresh-data', handleRefresh);
  return () => window.removeEventListener('refresh-data', handleRefresh);
}, []);

// 2️⃣ 'undo-executed' Event - Nach UNDO-Operation
// Wird von DataContext gehört (löst VOLLSTÄNDIGEN refresh aus)
// Nötig weil Backend-State nach Undo komplett neu geladen werden muss
window.dispatchEvent(new CustomEvent('undo-executed'));

// DataContext:
useEffect(() => {
  const handleUndoRefresh = () => {
    refreshAll(); // Vollständiger Reload nach Undo
  };
  window.addEventListener('undo-executed', handleUndoRefresh);
  return () => window.removeEventListener('undo-executed', handleUndoRefresh);
}, [refreshAll]);
```

**Warum zwei Events?**
- ❌ Problem: `refresh-data` → `refreshAll()` überschreibt Optimistic Updates!
- ✅ Lösung: `refresh-data` nur für Transaction-Liste, `undo-executed` für echten Reload

### Checkliste für jede CRUD-Operation:

- [ ] State-Update SOFORT (BEVOR Backend-Call oder SOFORT danach bei CREATE)
- [ ] Backend-Call mit try/catch
- [ ] Bei Erfolg: NUR `refresh-data` Event dispatchen, NIEMALS refresh() aufrufen!
- [ ] Bei Fehler: Rollback zum alten State
- [ ] Dependencies: KEINE refresh-Funktionen (nur entities array)!
- [ ] ACHTUNG: `refresh-data` löst KEIN globales refresh aus (nur Transaction-Liste wird aktualisiert)

---

## 🐛 Debugging-Workflow (KRITISCH!)

**WICHTIG:** Bei React White Screen, TypeError oder unerklärlichen Fehlern IMMER dieser Workflow:

### 1. **Error Boundary SOFORT einbauen**
```typescript
// src/components/ErrorBoundary.tsx - IMMER um komplexe Komponenten wrappen!
class ErrorBoundary extends Component {
  componentDidCatch(error, errorInfo) {
    console.error('ERROR BOUNDARY:', error, errorInfo);
    // Zeigt EXAKTE Fehlermeldung + Stack Trace statt White Screen
  }
}

// Usage in Parent:
<ErrorBoundary>
  <ComplexComponent />
</ErrorBoundary>
```

**Warum:** Fängt Fehler ab, zeigt exakte Fehlermeldung mit Zeilennummer statt White Screen!

### 2. **Extensive Debug Logging**
```typescript
// ❌ FALSCH - zu wenig Info
console.log('loading...');

// ✅ RICHTIG - komplette Struktur-Analyse
console.log('═══════════════════════════════════════');
console.log('📦 RAW DATA:', JSON.stringify(data, null, 2));
console.log('🔍 Structure check:');
console.log('  - data.id:', data?.id);
console.log('  - data.guest:', data?.guest);
console.log('  - data.guest type:', typeof data?.guest);
console.log('  - data.guest.vorname:', data?.guest?.vorname);
console.log('═══════════════════════════════════════');

// Validierung BEVOR render
if (!data.guest) {
  throw new Error('❌ data.guest is missing! Got: ' + typeof data.guest);
}
```

**Warum:** JSON.stringify zeigt ALLE Properties, typeof zeigt ob undefined/object/number etc.

### 3. **Web-Recherche für Best Practices**
```
Query-Beispiele:
- "React white screen TypeError debugging 2025"
- "React component briefly loads then goes white"
- "Error Boundary best practices React"
```

**Warum:** Community hat diese Probleme schon gelöst, spart Stunden!

### 4. **Type-Safety Backend ↔ Frontend prüfen**
```rust
// Backend Rust struct
#[derive(Serialize)]
pub struct BookingWithDetails {
    pub grundpreis: f64,
    pub guest: Guest,  // ← Nested object!
}
```

```typescript
// Frontend TypeScript interface - MUSS EXAKT MATCHEN!
interface Booking {
  grundpreis: number;  // ← Muss vorhanden sein!
  guest: Guest;        // ← Muss nested object sein, nicht nur guest_id!
}
```

**Häufiger Fehler:** Backend gibt `guest_id: i64`, Frontend erwartet `guest: Guest` → TypeError!

### 5. **Systematische Fehlersuche**

**Bei TypeError "undefined is not an object (evaluating 'x.y')":**
1. ✅ Error Boundary zeigt EXAKTE Zeile (z.B. line 974: `booking.grundpreis.toFixed`)
2. ✅ Debug Log zeigt: `booking.grundpreis: undefined`
3. ✅ Backend struct prüfen: Fehlt `grundpreis` im struct?
4. ✅ SQL Query prüfen: Wird `grundpreis` im SELECT gelesen?
5. ✅ Frontend Interface prüfen: Ist `grundpreis` typisiert?

**Debugging-Checkliste:**
- [ ] Error Boundary eingebaut?
- [ ] JSON.stringify des kompletten Objekts geloggt?
- [ ] Struktur-Validierung (if !data.field throw Error)?
- [ ] Backend struct hat alle Felder?
- [ ] SQL Query liest alle Felder?
- [ ] Frontend Interface matched Backend struct?
- [ ] Web-Recherche für Best Practices gemacht?

### 6. **Nach dem Fix: Debug Code entfernen**
```typescript
// Entferne excessive Logs nach erfolgreichem Fix
// Behalte nur wichtige Error-Handling
```

**Warum:** Production Code soll sauber sein, aber Error Boundaries bleiben!

---

## 🔄 React State & Lifecycle Debugging (KRITISCH!)

**WICHTIG:** Bei State-Problemen wo Daten nicht angezeigt werden IMMER dieses systematische Pattern verwenden!

### Das Problem-Pattern erkennen:
```typescript
// SYMPTOM: Komponente zeigt Daten nicht an, obwohl sie im Objekt vorhanden sind
booking.payment_recipient_id: 1 ✅  // Daten sind da!
currentPaymentRecipient: null ❌    // State ist leer!
{currentPaymentRecipient && <Component />}  // Wird nicht gerendert ❌
```

**ROOT CAUSE:** State wird nur beim **Initial Load** gesetzt, aber NICHT bei Updates!

### 1. **UI-Debug-Box SOFORT einbauen** (Der Game-Changer! 🎯)

```typescript
{/* 🚨 IMMER ZUERST: Visuelle Debug-Box einbauen */}
<div className="border-2 border-orange-500 rounded-lg p-4 bg-orange-50 mb-4">
  <h3 className="text-lg font-bold text-orange-900 mb-3">🚨 DEBUG: State Check</h3>
  <div className="space-y-2 font-mono text-sm">
    <p><span className="font-bold">someObject.someId:</span> {String(someObject?.someId ?? 'undefined')} (Type: {typeof someObject?.someId})</p>
    <p><span className="font-bold">currentState:</span> {currentState ? 'TRUTHY ✅' : 'FALSY ❌'}</p>
    <p><span className="font-bold">currentState value:</span> {currentState === null ? 'NULL' : currentState === undefined ? 'UNDEFINED' : 'HAS VALUE'}</p>
    <p><span className="font-bold">Conditional evaluates to:</span> {currentState ? '✅ TRUE - Will render' : '❌ FALSE - Will NOT render'}</p>
    {currentState && (
      <div className="mt-2 p-2 bg-white rounded border border-orange-300">
        <p className="font-bold">State Data:</p>
        <pre className="text-xs overflow-auto">{JSON.stringify(currentState, null, 2)}</pre>
      </div>
    )}
  </div>
</div>
```

**Warum UI-Debug-Box > Console Logs:**
- ✅ **Visuell sofort sichtbar** - Kein Tab-Wechsel nötig
- ✅ **User kann Screenshots schicken** - Remote Debugging möglich
- ✅ **Zeigt Problem in Sekunden** - Diskrepanz zwischen Objekt und State sofort erkennbar
- ✅ **Immer sichtbar** - Kein Scrollen durch Console nötig

### 2. **Systematische React Lifecycle Checks**

**Bei JEDEM State-Problem diese Fragen durchgehen:**

```typescript
// ✅ CHECKLISTE FÜR REACT STATE BUGS:

1. Wird der State initial gesetzt?
   → Test: Initial Load funktioniert?

2. Wird der State bei Updates aktualisiert?
   → Test: Objekt ändert sich, wird State nachgeladen?
   → HÄUFIGSTER FEHLER: Nur Initial Load, keine Updates!

3. useEffect Dependencies korrekt?
   → Hört useEffect auf die richtige Dependency?
   → Ist die Dependency im Dependency-Array?

4. State wird auf null gesetzt wenn nötig?
   → Was passiert wenn die ID/Daten fehlen?
   → Wird alter State überschrieben?

5. Gibt es Race Conditions?
   → Mehrere Updates gleichzeitig?
   → Async Calls überschreiben sich?
```

### 3. **Die Lösung: useEffect mit korrekten Dependencies**

```typescript
// ❌ FALSCH - Lädt nur beim Initial Mount
useEffect(() => {
  const loadData = async () => {
    if (someObject.someId) {
      const data = await invoke('get_data', { id: someObject.someId });
      setCurrentState(data);
    }
  };
  loadData();
}, []); // ← FEHLER! Leeres Dependency Array!

// ✅ RICHTIG - Lädt bei JEDEM Update
useEffect(() => {
  const loadData = async () => {
    // Prüfe ob Daten vorhanden
    if (!someObject || mode !== 'view') return;

    if (someObject.someId) {
      try {
        const data = await invoke('get_data', { id: someObject.someId });
        setCurrentState(data);
      } catch (error) {
        console.error('Error loading data:', error);
        setCurrentState(null);
      }
    } else {
      // KRITISCH: State auf null setzen wenn keine ID!
      setCurrentState(null);
    }
  };

  loadData();
}, [someObject?.someId, mode]); // ← Richtige Dependencies!
```

**Warum das funktioniert:**
- ✅ useEffect wird bei **JEDER Änderung** von `someObject.someId` getriggert
- ✅ State wird automatisch nachgeladen
- ✅ State wird auf `null` gesetzt wenn keine ID vorhanden
- ✅ Conditional Rendering funktioniert korrekt

### 4. **Web-Recherche Pattern**

```typescript
// TRIGGER: Problem nach 2-3 Versuchen NICHT gelöst
if (attempts >= 3 && problem.still_exists) {
  // SOFORT Web-Recherche durchführen
  await webSearch({
    query: "React useState not updating component debugging 2025",
    query: "React useEffect dependencies best practices 2025",
    query: "React conditional rendering not showing component 2025"
  });
}
```

**Beispiel-Queries:**
- "React state not updating after initial load"
- "React useEffect dependencies explained"
- "React component not re-rendering when state changes"
- "React conditional rendering debugging"

### 5. **Complete Debugging Workflow**

```typescript
// PHASE 1: Diagnose (5 Minuten)
1. ✅ UI-Debug-Box einbauen (zeigt ALLE relevanten Werte)
2. ✅ Screenshot vom User anfordern
3. ✅ Identifiziere: Objekt hat Daten, State ist leer?

// PHASE 2: Root Cause (5 Minuten)
1. ✅ Initial Load funktioniert?
2. ✅ Nachgeladen bei Updates? ← HIER ist meist das Problem!
3. ✅ useEffect Dependencies prüfen
4. ✅ State wird auf null gesetzt?

// PHASE 3: Fix (5 Minuten)
1. ✅ useEffect mit korrekten Dependencies hinzufügen
2. ✅ State auf null setzen wenn keine ID
3. ✅ Testen mit Daten MIT und OHNE ID

// PHASE 4: Cleanup (5 Minuten)
1. ✅ Debug-Box entfernen
2. ✅ Excessive Logs entfernen
3. ✅ Nur Error-Handling behalten
```

### 6. **Common Mistakes zu vermeiden**

```typescript
// ❌ FEHLER 1: Leeres Dependency Array
useEffect(() => {
  loadData();
}, []); // Lädt nur einmal beim Mount!

// ❌ FEHLER 2: Dependency fehlt
useEffect(() => {
  if (booking.payment_recipient_id) {
    loadData();
  }
}, [booking]); // ← Sollte [booking?.payment_recipient_id] sein!

// ❌ FEHLER 3: Kein else-Branch
useEffect(() => {
  if (someObject.someId) {
    loadData();
  }
  // ❌ Was wenn keine ID? State bleibt alt!
}, [someObject?.someId]);

// ✅ RICHTIG: Mit else-Branch
useEffect(() => {
  if (someObject.someId) {
    loadData();
  } else {
    setCurrentState(null); // ← State zurücksetzen!
  }
}, [someObject?.someId]);
```

### 7. **Testing Checklist**

Nach dem Fix IMMER testen:

- [ ] Komponente MIT Daten laden → State wird gesetzt ✅
- [ ] Komponente OHNE Daten laden → State ist null ✅
- [ ] Von MIT zu OHNE wechseln → State wird auf null gesetzt ✅
- [ ] Von OHNE zu MIT wechseln → State wird geladen ✅
- [ ] Mehrfaches Wechseln → Funktioniert durchgehend ✅

### 8. **Lessons Learned - Quick Reference**

```typescript
// 🎯 BEI JEDEM REACT STATE BUG:

1. ✅ UI-Debug-Box SOFORT einbauen (nicht erst nach 10 Versuchen!)
2. ✅ Web-Recherche nach 2-3 fehlgeschlagenen Versuchen
3. ✅ Systematisch checken:
   - Initial Load funktioniert?
   - Nachgeladen bei Updates?
   - useEffect Dependencies korrekt?
   - State wird auf null gesetzt?
4. ✅ Mit User testen (MIT und OHNE Daten)
5. ✅ Debug-Code entfernen nach Fix

// ⏱️ ZEITERSPARNIS:
// Mit diesem Pattern: ~20 Minuten
// Ohne Pattern: 60+ Minuten (Trial & Error)
```

### 9. **Anti-Pattern: Was NICHT zu tun**

```typescript
// ❌ NICHT: Blind Trial & Error
"Vielleicht hilft setState nochmal?"
"Vielleicht muss ich forceUpdate()?"
"Vielleicht ist es ein Cache-Problem?"

// ✅ STATTDESSEN: Systematisch debuggen
1. UI-Debug-Box zeigt Problem
2. useEffect Dependencies prüfen
3. Fix implementieren
4. Testen
```

---

## 🛡️ Regression Prevention - KRITISCHE REGELN

**WICHTIG:** Diese Regeln verhindern dass funktionierende Features kaputt gehen!

### 1. **Minimal-Change-Prinzip**
```
✅ RICHTIG: Nur die notwendigen Änderungen machen
❌ FALSCH: "Gleich auch das andere refactoren..."

Beispiel:
- Aufgabe: "Invoice Status im TapeChart anzeigen"
- NUR ändern: TapeChart.tsx (Booking interface + JSX)
- NICHT ändern: DataContext.tsx, BookingList.tsx (wenn nicht absolut nötig!)
```

### 2. **Defensive Coding** (KRITISCH!)
```typescript
// ✅ IMMER prüfen ob Daten existieren
if (booking.guest && booking.room) {
  const name = booking.guest.vorname;
}

// ✅ Optional Chaining verwenden
const guestName = booking.guest?.vorname || 'Unbekannt';
const roomName = booking.room?.name || 'Unbekannt';

// ✅ Type Guards für sicherheit
if (!booking.guest) {
  console.error('Guest data missing!', booking);
  return null; // oder default value
}

// ❌ NIEMALS direkter Zugriff ohne Prüfung
const name = booking.guest.vorname; // TypeError wenn guest undefined!
```

### 3. **Read-Before-Write** (Verstärkt!)
```typescript
// ✅ IMMER zuerst KOMPLETTE Datei lesen
await Read('/path/to/file.tsx');

// ✅ Verstehen WAS die Datei macht
// ✅ Prüfen: Wird diese Änderung andere Features brechen?
// ✅ Nur dann: Minimale Änderung vornehmen

// ❌ NIEMALS: Blind Funktionen ändern ohne Kontext
```

### 4. **Small Focused Commits**
```bash
# ✅ Kleine, fokussierte Commits (Ein Feature = Ein Commit)
git commit -m "feat: Add invoice status to TapeChart only"

# ❌ Große Mixed Commits
git commit -m "fix various things and add features"

Vorteil: Bei Bugs kann man EXAKT den Commit finden der es kaputt gemacht hat
```

### 5. **TypeScript als Schutz-Netz**
```typescript
// ✅ Nutze TypeScript strict mode
interface Booking {
  guest: Guest;    // Required! TypeScript warnt wenn fehlt
  room: Room;      // Required!
  rechnung_versendet_am?: string; // Optional
}

// ✅ Explicit Return Types
function getBooking(id: number): Booking | null {
  // TypeScript zwingt zur Behandlung von null-Fall
}

// ❌ NIEMALS 'any' verwenden - umgeht alle Checks!
const data: any = await invoke('get_booking'); // VERBOTEN!
```

### 6. **Smoke Tests nach App-Rebuild** (KRITISCH!)
```
Nach JEDER größeren Änderung + App-Neustart:

□ App startet ohne Crash
□ Hauptfeatures laden (Buchungen, Gäste, Zimmer)
□ NEUES Feature funktioniert
□ Ein ALTES Feature stichprobenartig testen (z.B. Buchung anlegen)
□ Keine TypeScript Errors
```

### 7. **Error Boundaries als Sicherheitsnetz**
```typescript
// ✅ Error Boundaries um komplexe Komponenten
<ErrorBoundary>
  <BookingList />
</ErrorBoundary>

// Zeigt exakte Fehlermeldung statt White Screen
// User kann anderen Features weiter nutzen
```

### Regression Test Checklist (Nach JEDEM Feature):
- [ ] Neues Feature funktioniert
- [ ] Alte Features funktionieren noch
- [ ] Nur minimal nötige Änderungen gemacht
- [ ] Keine TypeScript Errors
- [ ] App läuft ohne Crashes
- [ ] Defensive Checks eingebaut (null/undefined)

---

## 🧪 Test-Driven Development (TDD)

**KRITISCH:** Alle Features MÜSSEN nach TDD-Prinzipien entwickelt werden!

### TDD-Workflow (Red-Green-Refactor):

1. **🔴 RED - Test schreiben (der fehlschlägt)**
   - Test schreiben BEVOR der Code existiert
   - Test muss fehlschlagen (weil Feature noch nicht implementiert)
   - Test beschreibt gewünschtes Verhalten

2. **🟢 GREEN - Implementierung (Test besteht)**
   - Minimale Implementation schreiben um Test zu bestehen
   - Code muss Test grün machen
   - Noch keine Optimierung

3. **🔵 REFACTOR - Code verbessern**
   - Code aufräumen und optimieren
   - Tests müssen weiterhin grün bleiben
   - Bessere Struktur, Performance, Lesbarkeit

### TDD für Rust Backend:

```rust
// 1. RED: Test schreiben
#[cfg(test)]
mod tests {
    #[test]
    fn test_create_guest_returns_guest_with_id() {
        let guest = create_guest(
            "Max".to_string(),
            "Mustermann".to_string(),
            "max@test.de".to_string(),
            "+49123".to_string(),
            true,
            None, None, None, None, None
        ).unwrap();

        assert!(guest.id > 0);
        assert_eq!(guest.vorname, "Max");
    }
}

// 2. GREEN: Implementation schreiben (minimal)
pub fn create_guest(...) -> Result<Guest> {
    // Implementation...
}

// 3. REFACTOR: Code optimieren (wenn nötig)
```

### TDD für Frontend:

```typescript
// 1. RED: Test schreiben
test('create guest button calls backend command', async () => {
  const { getByText } = render(<GuestDialog />);
  const button = getByText('Speichern');
  fireEvent.click(button);

  expect(invoke).toHaveBeenCalledWith('create_guest_command', ...);
});

// 2. GREEN: Komponente implementieren
function GuestDialog() {
  const handleSave = () => invoke('create_guest_command', data);
  // ...
}

// 3. REFACTOR: Code verbessern
```

### App automatisch starten:

**KRITISCH:** Nach JEDER Code-Änderung die App starten, damit der User sofort testen kann!

```bash
# App starten (im Background)
cd "/path/to/project" && npm run tauri dev > /dev/null 2>&1 &

# App neustarten (alte Prozesse killen, dann starten)
pkill -f "tauri dev" 2>/dev/null; sleep 2; npm run tauri dev > /dev/null 2>&1 &
```

**Wann App starten:**
- ✅ Nach Backend-Änderungen (neue Commands)
- ✅ Nach Frontend-Änderungen (neue Komponenten)
- ✅ Nach Datenbank-Schema-Änderungen (DB löschen + neu starten)
- ✅ Nach Bug-Fixes
- ✅ Nach JEDEM Feature

**Workflow:**
1. **App IMMER im Hintergrund laufen lassen** (Hot Reload für Frontend)
2. Feature implementieren
3. Frontend-Änderungen → automatisch live im Browser
4. Backend-Änderungen → App automatisch neu kompilieren
5. User kann SOFORT testen
6. Feedback bekommen
7. Iterieren

**Best Practice:**
- App läuft kontinuierlich im Background
- Vite Hot Module Replacement (HMR) für Frontend
- Tauri recompile für Rust-Änderungen
- User sieht Änderungen LIVE ohne manuellen Neustart

---

### Frontend Test-UI für Backend Features:

**WICHTIG:** Jedes neue Backend-Feature bekommt SOFORT eine einfache Test-UI im Frontend!

Workflow:
1. Backend Command implementieren
2. Einfache Test-Komponente im Frontend bauen
3. User kann Feature sofort testen
4. Später durch "echte" UI ersetzen

Beispiel Test-UI:
```typescript
// src/components/DevTools/TestCommands.tsx
export function TestCommands() {
  return (
    <div className="p-4 bg-yellow-100 border-2 border-yellow-500">
      <h2>🧪 Test Commands (DEV)</h2>
      <button onClick={() => testCreateGuest()}>Test Create Guest</button>
      <button onClick={() => testSearchGuests()}>Test Search Guests</button>
      {/* Mehr Test-Buttons... */}
    </div>
  );
}
```

### Test-Pyramide:

```
       /\
      /  \  E2E Tests (wenige, wichtige User-Flows)
     /    \
    /      \ Integration Tests (API + DB + Frontend)
   /        \
  /          \ Unit Tests (viele, schnelle Tests)
 /____________\
```

- **Unit Tests:** 70% - Einzelne Funktionen testen
- **Integration Tests:** 20% - Zusammenspiel testen
- **E2E Tests:** 10% - Komplette User-Flows

### Test-Naming (Deutsch):

```rust
#[test]
fn test_create_guest_mit_validen_daten_erstellt_gast() { }

#[test]
fn test_create_guest_mit_leerem_namen_gibt_fehler() { }

#[test]
fn test_delete_guest_mit_buchungen_schlaegt_fehl() { }
```

### Wann TDD überspringen? NIEMALS!

Ausnahmen gibt es nicht. Auch für:
- ✅ UI-Komponenten → Visual Regression Tests oder Component Tests
- ✅ Datenbank-Queries → Unit Tests mit In-Memory DB
- ✅ Validierung → Unit Tests für alle Edge Cases
- ✅ Business Logic → Unit Tests ZUERST!

---

## 📱 Mobile App Deployment (KRITISCH!)

**WICHTIG:** IMMER wenn Änderungen an der Mobile Cleaning App (`dpolg-cleaning-mobile`) gemacht werden, MUSS sofort ein Vercel Deployment durchgeführt werden!

### Warum sofort deployen?
- ✅ **Keine Cache-Probleme** - Neue Version überschreibt alle Cache-Layers (Vercel CDN + Browser)
- ✅ **Sofort testbar** - Änderungen sind innerhalb von 30 Sekunden live
- ✅ **Kein manuelles Cache-Clearing** - User muss nicht F5 drücken oder Cache leeren
- ✅ **Production-ready** - Putzkräfte sehen die Änderung SOFORT auf allen Geräten

### Workflow für JEDE Mobile App Änderung:

```bash
# 1️⃣ In Mobile App Directory wechseln
cd "/Users/maximilianfegg/Desktop/Sicherungskopie DPolG Buchungssystem.nosynch/Claude Code/dpolg-cleaning-mobile"

# 2️⃣ Git Commit & Push (wie gewohnt)
git add index.html  # oder andere geänderte Dateien
git commit -m "fix: Beschreibung der Änderung"
git push

# 3️⃣ SOFORT Vercel Production Deployment
vercel --prod --yes
```

### Deployment-Regel:

**IMMER in DIESER Reihenfolge:**
1. ✅ Änderung an `index.html` oder anderen Mobile App Dateien
2. ✅ Git commit
3. ✅ Git push
4. ✅ **SOFORT** Vercel deployment (`vercel --prod --yes`)
5. ✅ Nach 30 Sekunden: Mobile App URL testen (`https://dpolg-cleaning-mobile.vercel.app`)

### Wann deployen?

**IMMER nach diesen Änderungen:**
- ✅ UI-Anpassungen (HTML, CSS)
- ✅ JavaScript Logic Changes
- ✅ Emoji-Darstellung Fixes
- ✅ Layout-Änderungen
- ✅ Timeline-View Updates
- ✅ Irgendeine Änderung an `index.html` oder anderen Frontend-Dateien

**NICHT nötig nach:**
- ❌ Backend-Änderungen (`supabase.rs`, `database.rs`) - diese laufen lokal in der Desktop App
- ❌ Datenbank-Schema-Änderungen - diese sind in Turso Cloud

### Authentifizierung:

Falls `vercel --prod --yes` fehlschlägt mit "token is not valid":

```bash
# Einmalig authentifizieren
vercel login

# Dann nochmal deployen
vercel --prod --yes
```

### Deployment-Bestätigung:

Nach erfolgreichem Deployment:
```
✅ Production: https://dpolg-cleaning-mobile.vercel.app [30s]
```

**SOFORT testen:**
1. Mobile App URL im Browser öffnen (mit Shift+Cmd+R für Hard Reload)
2. Änderung verifizieren
3. User informieren dass Deployment live ist

### Checkliste für Mobile App Changes:

- [ ] Änderung an Mobile App Datei(en) gemacht
- [ ] `git add` + `git commit` + `git push`
- [ ] **SOFORT** `vercel --prod --yes` ausführen
- [ ] Deployment-URL testen (nach 30s)
- [ ] User informieren: "Deployment ist live!"

**WICHTIG:** Diese Regel gilt für ALLE Mobile App Änderungen - keine Ausnahmen!

---

## 🤖 Arbeitsweise mit Subagents

### Subagent-Strategie:
**WICHTIG:** Für JEDEN Task müssen die passenden Subagents identifiziert und eingebunden werden, um die besten Ergebnisse zu erzielen.

### Verfügbare Subagents:
1. **database-architect** - Datenbank-Schema, Migrations, SQL-Optimierung
2. **rust-backend-dev** - Tauri Commands, Backend-Logik, rusqlite
3. **react-component-builder** - UI-Komponenten, TypeScript, TailwindCSS
4. **validation-expert** - Validierungen, Error Handling, Business Rules
5. **pdf-email-specialist** - PDF-Generierung, Email-System
6. **testing-qa** - Testing, Code Review, Performance

### Subagent-Auswahl Regeln:
```
Task-Typ                           → Subagents
────────────────────────────────────────────────────────────
Datenbank-Schema ändern            → database-architect + rust-backend-dev
Neues Tauri Command                → rust-backend-dev + validation-expert
React Komponente erstellen         → react-component-builder
Formular mit Validierung           → react-component-builder + validation-expert
CRUD Operations                    → rust-backend-dev + database-architect
PDF/Email Features                 → pdf-email-specialist + rust-backend-dev
Bugfix                             → testing-qa + [relevanter Spezialist]
Performance Problem                → testing-qa + [relevanter Spezialist]
Code Review                        → testing-qa
```

### Workflow mit Subagents:
1. **Analyse:** Task verstehen und Komplexität einschätzen
2. **Subagent-Auswahl:** Passende Spezialisten identifizieren (min. 1, max. 3)
3. **Delegation:** Task an Subagent(s) delegieren mit klarem Kontext
4. **Integration:** Ergebnisse zusammenführen und testen
5. **Review:** testing-qa für Qualitätssicherung einbeziehen (bei wichtigen Features)
6. **Roadmap Update:** ROADMAP.md aktualisieren - erledigte Tasks abhaken ✅
7. **App starten:** IMMER die App starten damit User testen kann!

### Beispiel-Delegation:
```
User: "Erstelle die Buchungsverwaltung mit Formular und Validierung"

Claude Planung:
1. database-architect: Prüfe DB-Schema für Buchungen
2. validation-expert: Implementiere Validierungslogik (Rust + TS)
3. rust-backend-dev: Erstelle create_booking Command
4. react-component-builder: Baue BookingDialog Komponente
5. testing-qa: Schreibe Tests und Review

Dann: Subagents parallel/sequentiell einsetzen
```

---

## 📋 Roadmap-Tracking-Regel

**KRITISCH:** Nach JEDEM abgeschlossenen Task MUSS die ROADMAP.md aktualisiert werden!

### Roadmap Update Workflow:
1. **Nach Task-Completion:** Sofort entsprechende Checkbox in ROADMAP.md abhaken
2. **Format:** `- [x]` für erledigte Tasks (war vorher `- [ ]`)
3. **Kontext hinzufügen:** Bei Bedarf Notizen unter dem Task hinzufügen
4. **Commit zusammen:** Roadmap-Update im selben Commit wie die Feature-Implementation

### Beispiel:
```markdown
Vorher:
- [ ] **Tabelle `accompanying_guests` hinzufügen**

Nachher:
- [x] **Tabelle `accompanying_guests` hinzufügen**
  ✅ Implementiert in src-tauri/src/database.rs:123
```

### Verantwortlichkeit:
- **Hauptagent (Claude):** Muss nach jedem Task die Roadmap aktualisieren
- **Subagents:** Erwähnen in ihrem Report welche Roadmap-Items erledigt wurden
- **User:** Kann jederzeit den aktuellen Stand in ROADMAP.md einsehen

---

## 🔍 Web-Recherche Strategie (AUTOMATISCH!)

**KRITISCH:** Führe automatisch eine Deep Web Search durch wenn du bei einem Problem nicht weiterkommst!

### Trigger Bedingungen - AUTOMATISCHE Web-Recherche:

Führe automatisch eine **Deep Web Search** durch wenn:

1. **Du bei einem Problem nicht weiterkommst** nach 2-3 Versuchen
2. **Eine Implementierung unklar ist** trotz Code-Analyse
3. **Ein Bug auftritt** der nicht offensichtlich ist
4. **Eine Library-spezifische Funktion** nicht wie erwartet funktioniert
5. **Best Practices** für ein Pattern gefragt sind
6. **Hartnäckige UI/CSS Probleme**: Wenn mehrere Lösungsversuche fehlschlagen

### 4-Phasen Such-Strategie:

#### Phase 1: Tech Stack Spezifische Suche
```
Suche nach: "[Library Name] [Feature] [Current Year]"
Beispiel: "dnd-kit drag drop resize 2025"
```

#### Phase 2: GitHub Repository & Code Search
```
Suche nach: "github [Library Name] [Feature] example"
Beispiel: "github dnd-timeline resize implementation"
```

#### Phase 3: Kombinierte Setup-Suche
```
Suche nach: "[Tech1] + [Tech2] + [Feature] implementation"
Beispiel: "react typescript dnd-kit calendar drag drop"
```

#### Phase 4: Problem-Spezifische Suche
```
Suche nach: "[Error Message]" OR "[Specific Behavior]"
Beispiel: "dnd-kit drag not working after resize"
```

### Recherche-Pattern (Code-Trigger):

```typescript
// AUTOMATISCHER TRIGGER
if (task.involves("neue_library") || task.involves("best_practices") || task.involves("security")) {
  await webSearch({
    query: "specific technology + best practices 2025",
    focus: "recent documentation, benchmarks, security"
  });
}

// KRITISCH: Nach 2-3 fehlgeschlagenen Versuchen
if (attempts >= 3 && problem.still_exists) {
  // SOFORT Web-Recherche - STOP weitere Trial & Error Versuche!
  await webSearch({
    query: "specific problem description + debugging + solution 2025",
    focus: "StackOverflow, official docs, debugging guides"
  });
}
```

### Web-Recherche Trigger-Regel:
**Wenn ein Problem nach 2-3 Lösungsversuchen NICHT gelöst ist:**
1. ✅ **STOP** weitere Versuche ohne Recherche
2. ✅ Führe **Deep Web-Recherche** durch mit präziser Problembeschreibung
3. ✅ Suche nach: Problem + Technologie + "debugging" + "2025"
4. ✅ Fokus: StackOverflow Answers, Browser DevTools Guides, Official Docs
5. ✅ **Dokumentiere** Findings und angewendete Lösung

### Pflicht Web Searches für TapeChart-bezogene Tasks:

Bei TapeChart-bezogenen Aufgaben **IMMER** folgende Quellen prüfen:

1. **dnd-timeline GitHub Repository**
   - URL: https://github.com/samuelarbibe/dnd-timeline
   - Suche nach: Code patterns, Issues, Discussions

2. **@dnd-kit Documentation**
   - Aktuelle Version: 6.3.1
   - Suche nach: PointerSensor, useDraggable, useDroppable

3. **Stack Overflow**
   - Tags: `[dnd-kit]`, `[react-dnd]`, `[drag-and-drop]`
   - Suche nach: ähnlichen Problemen mit unserem Setup

4. **GitHub Code Search**
   - Suche nach: Repositories die dnd-kit für Calendar/Timeline verwenden
   - Filter: Updated in last year

### Beispiel: Komplexes Problem Workflow

**Scenario**: Drag funktioniert nicht mehr nach Density Mode Update

**Automatischer Workflow:**

1. **Sofort**: Lies `TAPECHART_CONTEXT.md` für bekannte Lösungen (falls vorhanden)
2. **Web Search 1**: "dnd-kit drag not working listeners 2025"
3. **Web Search 2**: "github dnd-kit conditional listeners"
4. **Web Search 3**: "dnd-timeline drag resize conflict solution"
5. **Code Analysis**: Vergleiche gefundene Patterns mit aktuellem Code
6. **Implementation**: Wende beste Lösung an
7. **Documentation**: Update Context-Dokumente mit neuer Lösung

### Output Format - Nach jeder Web Search:

Nach jeder Web Search dokumentiere:

```markdown
## Web Search Results: [Problem Description]

**Search Query**: "[exact query]"
**Source**: [URL oder "Multiple sources"]
**Key Findings**:
- Finding 1
- Finding 2
- Finding 3

**Applied Solution**: [Beschreibung der implementierten Lösung]
**File**: [path/to/file.tsx:line_number]
```

### Beispiel-Queries:
- "Tauri 2 SQLite connection pooling best practices 2025"
- "Rust lettre email attachment modern example"
- "React TailwindCSS modal accessibility 2025"
- "rusqlite transaction performance optimization"
- "dnd-kit drag drop resize calendar implementation 2025"
- "React state not updating useEffect dependencies 2025"

### TapeChart Spezialist Modus:

**WICHTIG:** Wenn der User über TapeChart spricht oder Änderungen anfragt:

#### Automatische Aktionen:
1. **Lies TAPECHART_CONTEXT.md** vor jeder Antwort (falls vorhanden)
2. **Prüfe dnd-timeline GitHub** für ähnliche Implementierungen
3. **Verwende bekannte Patterns** aus unserer Historie
4. **Dokumentiere alle Änderungen** im Context-Dokument (falls vorhanden)

#### TapeChart Wissen & Context:
Du bist Experte für:
- **@dnd-kit/core v6.3.1** Drag & Drop Library
- **dnd-timeline** Patterns und Best Practices
- **React TypeScript** mit Hooks
- **Tailwind CSS v3** Styling
- **date-fns v4** Date manipulation
- **Overlap Prevention** Custom Logic
- **Density Modes** mit localStorage

#### TapeChart Code Style:
Halte dich an diese Standards:
- **TypeScript strict mode** aktiviert
- **Functional components** mit Hooks (keine Class Components)
- **useCallback** für Event Handler
- **useMemo** für teure Berechnungen
- **cn()** utility für Tailwind classnames
- **select-none** für Drag/Resize Elemente

### Problemlösung Priorität (Workflow):

Bei Bugs oder Fehlern folge dieser Reihenfolge:

1. **Check Context-Dokumente** - Bekannte Lösungen (TAPECHART_CONTEXT.md, etc.)
2. **Web Search** - Aktuelle Implementierungen/Lösungen
3. **GitHub Repository** - Issues & Discussions (dnd-timeline, etc.)
4. **Code Analysis** - Vergleich mit Working Patterns
5. **Experimentation** - Nur wenn keine Lösung gefunden
6. **Documentation** - Update Context mit Lösung

---

## 🏗️ Technologie-Stack

### Backend:
- **Tauri 2** (Latest) - Desktop App Framework
- **Rust** (Edition 2021) - Backend Language
- **rusqlite** - SQLite Database Access
- **serde** - Serialization/Deserialization
- **chrono** - Date/Time Handling

### Frontend:
- **React 18+** - UI Framework
- **TypeScript 5+** - Type Safety
- **Vite** - Build Tool & Dev Server
- **TailwindCSS 4** - Styling
- **lucide-react** - Icons
- **dnd-kit** - Drag & Drop

### Datenbank:
- **SQLite** - Local Database (via rusqlite)
- Stored in: App Data Directory

### Warum diese Stack?
- ✅ **Tauri:** Kleiner Binary, natives Performance, sichere IPC
- ✅ **Rust:** Memory Safety, Concurrency, keine Runtime
- ✅ **SQLite:** Embedded, kein Server, ACID-compliant, perfekt für Desktop
- ✅ **React + TS:** Type-safe, große Community, schnelle Development
- ✅ **TailwindCSS:** Utility-first, konsistentes Design, kleine Bundle-Size

---

## 📁 Projekt-Struktur

```
dpolg-booking-modern/
├── .claude/
│   ├── CLAUDE.md              # Diese Datei - Projekt-Regeln
│   └── agents/                # Subagent Konfigurationen
│       ├── database-architect.md
│       ├── rust-backend-dev.md
│       ├── react-component-builder.md
│       ├── validation-expert.md
│       ├── pdf-email-specialist.md
│       └── testing-qa.md
├── ROADMAP.md                 # Feature-Liste & Implementierungs-Plan
├── MIGRATION_GUIDE.md         # Python → Tauri Migration Guide
├── src/
│   ├── App.tsx                # Main App Component
│   ├── main.tsx               # React Entry Point
│   ├── components/
│   │   ├── TapeChart.tsx      # Tape Chart Visualisierung
│   │   ├── BookingManagement/ # Phase 3.1
│   │   ├── GuestManagement/   # Phase 3.2
│   │   ├── RoomManagement/    # Phase 3.3
│   │   ├── Layout/            # Navigation, Header, Sidebar
│   │   ├── Search/            # Phase 4.1
│   │   ├── Filters/           # Phase 4.2
│   │   ├── Reports/           # Phase 5
│   │   └── Email/             # Phase 6
│   ├── lib/
│   │   ├── utils.ts           # Utility Functions
│   │   └── validation.ts      # Frontend Validation Helpers
│   └── types/
│       └── booking.ts         # TypeScript Type Definitions
├── src-tauri/
│   ├── src/
│   │   ├── main.rs            # Tauri Entry Point
│   │   ├── lib.rs             # Tauri App Setup & Command Registration
│   │   ├── database.rs        # Database Operations & Models
│   │   ├── validation.rs      # Phase 2.1 - Validierungslogik
│   │   ├── pricing.rs         # Phase 2.2 - Preisberechnung
│   │   ├── email.rs           # Phase 6.1 - Email System
│   │   └── pdf_generator.rs   # Phase 7.1 - PDF Generierung
│   ├── Cargo.toml             # Rust Dependencies
│   └── tauri.conf.json        # Tauri Configuration
└── package.json               # Node Dependencies
```

### Datei-Namenskonventionen:
- **React Components:** PascalCase (z.B. `BookingDialog.tsx`)
- **Rust Files:** snake_case (z.B. `database.rs`, `pdf_generator.rs`)
- **TypeScript Utils:** camelCase (z.B. `validation.ts`, `utils.ts`)
- **Komponenten-Ordner:** PascalCase (z.B. `BookingManagement/`)
- **Config Files:** kebab-case (z.B. `tauri.conf.json`)

---

## 💻 Code-Konventionen

### UI/UX Dialoge & Bestätigungen:

**KRITISCH:** NIEMALS Browser-Standard-Dialoge (`alert()`, `confirm()`, `prompt()`) verwenden!

```typescript
// ❌ VERBOTEN - Browser-Standard-Dialoge
const confirmed = confirm('Wirklich löschen?');
alert('Erfolgreich gespeichert!');
const input = prompt('Name eingeben:');

// ✅ RICHTIG - Eigene Custom Dialoge
const [showDialog, setShowDialog] = useState(false);
const [dialogData, setDialogData] = useState<SomeType | null>(null);

// Dialog Component mit modernem Design
{showDialog && (
  <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
    <div className="bg-gradient-to-br from-slate-800 to-slate-900 rounded-2xl shadow-2xl p-6 max-w-md w-full mx-4">
      {/* Dialog Content */}
      <div className="flex items-start gap-4 mb-6">
        <div className="p-3 bg-amber-500/10 rounded-full">
          <AlertTriangle className="w-6 h-6 text-amber-400" />
        </div>
        <div className="flex-1">
          <h3 className="text-xl font-bold text-white mb-2">Bestätigung erforderlich</h3>
          <p className="text-slate-300 text-sm">Beschreibung der Aktion...</p>
        </div>
      </div>

      {/* Buttons */}
      <div className="flex gap-3">
        <button onClick={() => setShowDialog(false)} className="...">Abbrechen</button>
        <button onClick={handleConfirm} className="...">Bestätigen</button>
      </div>
    </div>
  </div>
)}
```

**Warum eigene Dialoge?**
- ✅ Konsistentes Design mit der App
- ✅ Bessere UX (Icons, Farben, Layout)
- ✅ Mehr Kontrolle (Loading States, Details anzeigen)
- ✅ Accessibility-Features
- ✅ Funktionieren zuverlässig (keine Browser-Unterschiede)

**Dialog-Typen:**
- **Bestätigung (Confirm):** Amber/Warning-Icon, "Abbrechen" + "Bestätigen"
- **Fehler (Alert Error):** Red-Icon, nur "OK" Button
- **Erfolg (Alert Success):** Green-Icon, nur "OK" Button oder auto-close
- **Löschen:** Red/Trash-Icon, "Abbrechen" + "Löschen" (rot)
- **Warnung:** Amber/AlertTriangle-Icon, Details + deutliche Warning-Box

### Rust (Backend):

#### Naming:
```rust
// Functions & Variables: snake_case
fn create_booking(booking_data: CreateBookingRequest) -> Result<Booking, String>
let checkin_date = "2025-01-10";

// Structs & Enums: PascalCase
struct BookingWithDetails { ... }
enum BookingStatus { ... }

// Constants: SCREAMING_SNAKE_CASE
const MAX_GUESTS_PER_ROOM: i32 = 10;

// Modules: snake_case
mod database;
mod validation;
```

#### Error Handling:
```rust
// ✅ IMMER Result<T, E> verwenden
#[tauri::command]
fn create_booking(data: CreateBookingRequest) -> Result<Booking, String> {
    validate_booking_data(&data)?;
    let booking = insert_booking(data)?;
    Ok(booking)
}

// ❌ NIEMALS unwrap() oder expect() in Production Code
let booking = get_booking(id).unwrap(); // VERBOTEN!

// ✅ Stattdessen proper error handling
let booking = get_booking(id)
    .map_err(|e| format!("Buchung nicht gefunden: {}", e))?;
```

#### Database Operations:
```rust
// ✅ IMMER Prepared Statements verwenden (SQL Injection Prevention)
conn.execute(
    "INSERT INTO bookings (room_id, guest_id) VALUES (?1, ?2)",
    params![room_id, guest_id],
)?;

// ❌ NIEMALS String Concatenation für SQL
let query = format!("SELECT * FROM bookings WHERE id = {}", id); // VERBOTEN!

// ✅ IMMER Transactions für Multi-Step Operations
let tx = conn.transaction()?;
tx.execute("INSERT INTO bookings ...", params![...])?;
tx.execute("INSERT INTO services ...", params![...])?;
tx.commit()?;
```

#### Kommentare:
```rust
// Deutsche Kommentare für Business Logic
// Prüfe ob Zimmer verfügbar ist im gewählten Zeitraum
fn check_room_availability(...) -> Result<bool, String> {
    // ...
}

// Englische Kommentare für technische Details sind auch ok
// Check for overlapping date ranges using SQL query
```

### TypeScript (Frontend):

#### Naming:
```typescript
// Variables & Functions: camelCase
const bookingData = { ... };
const createBooking = async (data: BookingData) => { ... };

// Components: PascalCase
export default function BookingDialog({ ... }) { ... }

// Interfaces & Types: PascalCase
interface BookingWithDetails { ... }
type BookingStatus = 'reserviert' | 'bestätigt' | ...;

// Constants: SCREAMING_SNAKE_CASE
const MAX_FILE_SIZE = 5 * 1024 * 1024;
```

#### Type Safety:
```typescript
// ✅ IMMER explizite Types
interface CreateBookingRequest {
  room_id: number;
  guest_id: number;
  checkin_date: string;
  checkout_date: string;
}

// ❌ NIEMALS 'any' verwenden
const data: any = await invoke('get_booking'); // VERBOTEN!

// ✅ Stattdessen proper typing
const data = await invoke<BookingWithDetails>('get_booking', { id });

// ✅ Type Guards verwenden
if (typeof value === 'string') { ... }
if (error instanceof Error) { ... }
```

#### React Patterns:
```typescript
// ✅ Functional Components mit TypeScript
interface BookingDialogProps {
  booking?: Booking;
  onSave: (booking: Booking) => void;
  onClose: () => void;
}

export default function BookingDialog({ booking, onSave, onClose }: BookingDialogProps) {
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Event handlers
  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    // ...
  };

  return (
    // JSX
  );
}

// ✅ Custom Hooks für wiederverwendbare Logik
function useBookings() {
  const [bookings, setBookings] = useState<Booking[]>([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    loadBookings();
  }, []);

  return { bookings, loading, refetch: loadBookings };
}
```

#### Tauri Invoke Pattern:
```typescript
import { invoke } from '@tauri-apps/api/core';

// ✅ Mit Type-Safety
const createBooking = async (data: CreateBookingRequest): Promise<Booking> => {
  try {
    const result = await invoke<Booking>('create_booking', { bookingData: data });
    return result;
  } catch (error) {
    // User-friendly error message in German
    throw new Error(
      error instanceof Error ? error.message : 'Fehler beim Erstellen der Buchung'
    );
  }
};
```

---

## 🎨 UI/UX Richtlinien

### Design-System:

#### Farbpalette:
```css
/* Primary Background */
from-slate-800 to-slate-900  /* Gradient für Header, Modals */
bg-slate-800                  /* Main Background */
bg-slate-900                  /* Darker Sections */
bg-slate-700                  /* Inputs, Secondary Elements */

/* Text */
text-white                    /* Primary Text */
text-slate-300                /* Secondary Text */
text-slate-400                /* Tertiary Text, Placeholders */

/* Accent Colors */
bg-blue-600                   /* Primary Action (Buttons) */
bg-blue-500                   /* Hover Accent */
bg-emerald-500                /* Success States */
bg-red-500                    /* Danger/Error States */
bg-purple-500                 /* Info/Stats */
bg-amber-500                  /* Warning States */

/* Borders */
border-slate-600              /* Default Borders */
border-slate-700              /* Subtle Borders */

/* Hover States */
hover:bg-blue-700             /* Primary Button Hover */
hover:bg-slate-600            /* Secondary Hover */
```

#### Spacing & Layout:
```css
/* Padding */
p-2, p-4, p-6, p-8            /* Verwende 4px-Schritte */

/* Gaps */
gap-2, gap-3, gap-4           /* Für Flexbox/Grid */

/* Rounded Corners */
rounded-lg                    /* Standard (8px) */
rounded-xl                    /* Dialoge, Cards (12px) */
rounded-2xl                   /* Große Elemente (16px) */

/* Shadows */
shadow-xl                     /* Standard Cards */
shadow-2xl                    /* Elevated Elements (Modals) */
```

#### Komponenten-Patterns:

##### Button:
```tsx
// Primary Button
<button className="px-4 py-3 bg-blue-600 hover:bg-blue-700 text-white font-semibold rounded-lg transition-colors disabled:bg-slate-600">
  Speichern
</button>

// Secondary Button
<button className="px-4 py-3 bg-slate-700 hover:bg-slate-600 text-white font-semibold rounded-lg transition-colors">
  Abbrechen
</button>

// Danger Button
<button className="px-4 py-3 bg-red-600 hover:bg-red-700 text-white font-semibold rounded-lg transition-colors">
  Löschen
</button>
```

##### Input Field:
```tsx
<div className="space-y-2">
  <label htmlFor="email" className="block text-sm font-medium text-slate-300">
    E-Mail-Adresse *
  </label>
  <input
    id="email"
    type="email"
    required
    className="w-full px-4 py-3 bg-slate-700 border border-slate-600 rounded-lg text-white placeholder-slate-400 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
    placeholder="beispiel@email.de"
  />
</div>
```

##### Filter Section (Search + Dropdowns):
```tsx
{/* Filter Container - mit Hintergrund und Border */}
<div className="p-3 bg-slate-50 rounded-lg border border-slate-200">
  <div className="grid grid-cols-4 gap-3">
    {/* Search Input */}
    <div className="col-span-4 sm:col-span-1 relative">
      <Search className="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 text-slate-400" />
      <input
        type="text"
        placeholder="Suche..."
        value={searchQuery}
        onChange={(e) => setSearchQuery(e.target.value)}
        className="w-full pl-10 pr-3 py-3 bg-white border border-slate-300 rounded-lg text-sm text-slate-700 placeholder-slate-400 shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 focus:shadow-md transition-all"
      />
    </div>

    {/* Dropdown Filter */}
    <select
      value={filter}
      onChange={(e) => setFilter(e.target.value)}
      className="px-4 py-3 bg-white border border-slate-300 rounded-lg text-sm text-slate-700 font-medium shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 focus:shadow-md transition-all cursor-pointer hover:border-slate-400 hover:shadow"
    >
      <option value="all">Alle</option>
      <option value="option1">Option 1</option>
      <option value="option2">Option 2</option>
    </select>
  </div>
</div>
```

**Filter Design-Regeln:**
- **Container:** `p-3 bg-slate-50 rounded-lg border border-slate-200` (oder `bg-blue-50/50 border-blue-200` für blaue Variante)
- **Grid Layout:** `grid grid-cols-4 gap-3`
- **Search Input:** Icon links (`left-4`, `w-5 h-5`), `pl-12 py-3.5 rounded-xl text-base`, `shadow-sm`, Focus: `shadow-md`
- **Dropdowns:** `px-5 py-3.5 rounded-xl text-base`, `shadow-sm`, Hover: `hover:shadow-md`
- **Konsistenz:** Alle Filter-Sections im Projekt müssen diesem Pattern folgen!
- **Höhe:** Search Input und Dropdowns haben IDENTISCHE Höhe (`py-3.5`)

##### Standard Dropdown (Unified Design):
```tsx
{/* Universal Dropdown Pattern - Gilt für ALLE Dropdowns im Projekt */}
<select
  value={selectedValue}
  onChange={(e) => setSelectedValue(e.target.value)}
  className="w-full px-5 py-3.5 bg-white border border-slate-300 rounded-xl text-base text-slate-700 font-normal appearance-none cursor-pointer shadow-sm hover:border-slate-400 hover:shadow-md focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-blue-500 transition-all"
  style={{
    backgroundImage: `url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='%23475569' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpolyline points='6 9 12 15 18 9'%3E%3C/polyline%3E%3C/svg%3E")`,
    backgroundRepeat: 'no-repeat',
    backgroundPosition: 'right 0.75rem center',
    backgroundSize: '1.5rem',
    paddingRight: '3rem'
  }}
>
  <option value="">Alle Status</option>
  <option value="option1">Option 1</option>
  <option value="option2">Option 2</option>
</select>
```

**Dropdown Design-Regeln (VERBINDLICH):**
- ✅ **Padding:** `px-5 py-3.5` (horizontal 1.25rem, vertikal 0.875rem)
- ✅ **Schriftgröße:** `text-base` (16px) für bessere Lesbarkeit
- ✅ **Font-Weight:** `font-normal` (nicht bold, außer aktiv)
- ✅ **Border-Radius:** `rounded-xl` (12px) für moderne Optik
- ✅ **Hintergrund:** `bg-white` mit `border-slate-300`
- ✅ **Schatten:** `shadow-sm` standard, `hover:shadow-md` on hover
- ✅ **Custom Dropdown-Pfeil:** Via `backgroundImage` (SVG), rechts positioniert
- ✅ **Padding-Right:** `3rem` (48px) damit Text nicht unter Pfeil gerät
- ✅ **Appearance:** `appearance-none` (entfernt Browser-Standard-Pfeil)
- ✅ **Hover:** `hover:border-slate-400` für subtiles Feedback
- ✅ **Focus:** `focus:ring-2 focus:ring-blue-500`
- ✅ **Transitions:** `transition-all` für smooth animations

**WICHTIG:** Dieses Pattern gilt für ALLE Dropdowns:
- Buchungsstatus-Filter
- Zimmer-Auswahl
- Gast-Auswahl
- Jahr/Ort-Filter
- Settings-Dropdowns
- Alle zukünftigen Dropdowns

##### Modal/Dialog:
```tsx
<div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50">
  <div className="bg-gradient-to-br from-slate-800 to-slate-900 rounded-2xl shadow-2xl p-8 max-w-2xl w-full mx-4">
    {/* Header */}
    <div className="flex items-center justify-between mb-6">
      <h2 className="text-2xl font-bold text-white">Dialog Titel</h2>
      <button className="p-2 hover:bg-slate-700 rounded-lg transition-colors">
        <X className="w-5 h-5 text-slate-300" />
      </button>
    </div>
    {/* Content */}
  </div>
</div>
```

### Accessibility-Anforderungen:

```tsx
// ✅ Semantic HTML
<button type="submit">Speichern</button>  // nicht <div onClick={...}>
<nav>...</nav>
<main>...</main>

// ✅ ARIA Labels für Icons
<button aria-label="Schließen">
  <X className="w-5 h-5" />
</button>

// ✅ Keyboard Navigation
onKeyDown={(e) => {
  if (e.key === 'Escape') onClose();
  if (e.key === 'Enter') handleSubmit();
}}

// ✅ Focus Management in Modals
useEffect(() => {
  if (isOpen) {
    firstInputRef.current?.focus();
  }
}, [isOpen]);

// ✅ Color Contrast (mindestens WCAG AA)
text-white on bg-slate-800  ✅
text-slate-400 on bg-slate-900  ✅
```

### Responsive Design:
```tsx
// ✅ Mobile-First Approach
<div className="
  px-4           // Mobile
  sm:px-6        // Small screens (640px+)
  md:px-8        // Medium screens (768px+)
  lg:px-12       // Large screens (1024px+)
">
```

### Sprache & Formatierung:

#### Sprache:
- **UI-Texte:** Deutsch
- **Code-Kommentare:** Deutsch für Business Logic, Englisch/Deutsch für technische Details
- **Error Messages:** Deutsch, benutzerfreundlich

#### Datumsformat:
```typescript
// Display: DD.MM.YYYY
const formatDate = (date: string): string => {
  // Input: "2025-01-15" (ISO 8601 from database)
  // Output: "15.01.2025" (German format)
  return new Date(date).toLocaleDateString('de-DE');
};
```

#### Währungsformat:
```typescript
// Format: 1.234,56 €
const formatPrice = (price: number): string => {
  return new Intl.NumberFormat('de-DE', {
    style: 'currency',
    currency: 'EUR',
  }).format(price);
};
```

#### Deutsche UI-Texte:
```typescript
const UI_TEXT = {
  // Actions
  save: 'Speichern',
  cancel: 'Abbrechen',
  delete: 'Löschen',
  edit: 'Bearbeiten',
  create: 'Erstellen',
  search: 'Suchen',

  // Status
  loading: 'Lädt...',
  saving: 'Speichert...',
  success: 'Erfolgreich gespeichert',
  error: 'Ein Fehler ist aufgetreten',

  // Booking
  checkin: 'Check-in',
  checkout: 'Check-out',
  nights: 'Nächte',
  guests: 'Gäste',
  room: 'Zimmer',
  price: 'Preis',
  total: 'Gesamt',
};
```

---

## 🔒 Sicherheits-Richtlinien

### Input Validation:
```rust
// ✅ IMMER validieren auf Backend
#[tauri::command]
fn create_booking(data: CreateBookingRequest) -> Result<Booking, String> {
    // 1. Input Validation
    validate_email(&data.guest_email)?;
    validate_booking_dates(&data.checkin, &data.checkout)?;

    // 2. Business Logic Validation
    check_room_availability(data.room_id, &data.checkin, &data.checkout)?;

    // 3. Process
    insert_booking(data)
}
```

### SQL Injection Prevention:
```rust
// ✅ IMMER Prepared Statements
conn.execute(
    "SELECT * FROM bookings WHERE guest_id = ?1",
    params![guest_id],
)?;

// ❌ NIEMALS String Concatenation
let query = format!("SELECT * FROM bookings WHERE guest_id = {}", guest_id); // VERBOTEN!
```

### Password/Secret Storage:
```rust
// ✅ SMTP Passwords verschlüsselt speichern
use base64::{Engine as _, engine::general_purpose};

fn encrypt_password(password: &str) -> String {
    // TODO: Use proper encryption (AES-256)
    general_purpose::STANDARD.encode(password.as_bytes())
}

// ❌ NIEMALS Passwords in Plain Text
conn.execute(
    "INSERT INTO config (smtp_password) VALUES (?1)",
    params![password], // VERBOTEN! Erst verschlüsseln!
)?;
```

### Error Messages:
```rust
// ✅ User-friendly, keine technischen Details
Err("Zimmer ist nicht verfügbar".to_string())

// ❌ Interne Details nicht nach außen geben
Err(format!("Database error: {}", e)) // VERBOTEN!
```

---

## ⚡ Performance-Richtlinien

### Datenbank:
```rust
// ✅ Indexes für häufige Queries
CREATE INDEX idx_bookings_dates ON bookings(checkin_date, checkout_date);
CREATE INDEX idx_bookings_room ON bookings(room_id);
CREATE INDEX idx_guests_email ON guests(email);

// ✅ Transactions für Multiple Operations
let tx = conn.transaction()?;
// ... multiple operations
tx.commit()?;

// ✅ Prepared Statements wiederverwenden
let mut stmt = conn.prepare("SELECT * FROM bookings WHERE room_id = ?1")?;
for room_id in room_ids {
    let bookings = stmt.query_map([room_id], |row| { ... })?;
}
```

### React:
```tsx
// ✅ React.memo für teure Komponenten
export default React.memo(BookingCard);

// ✅ useMemo für berechnete Werte
const filteredBookings = useMemo(() => {
  return bookings.filter(b => b.status !== 'storniert');
}, [bookings]);

// ✅ useCallback für Event Handlers in Listen
const handleDelete = useCallback((id: number) => {
  deleteBooking(id);
}, []);

// ✅ Virtualisierung für lange Listen (>100 Items)
import { FixedSizeList } from 'react-window';
```

### Bundle Size:
```typescript
// ✅ Lazy Loading für Routes
const Reports = lazy(() => import('./components/Reports'));

// ✅ Code Splitting
// Vite macht das automatisch für dynamische Imports
```

---

## 🧪 Testing-Anforderungen

### Unit Tests (Rust):
```rust
// ✅ Für ALLE Validierungsfunktionen
#[cfg(test)]
mod tests {
    #[test]
    fn test_validate_booking_dates() {
        assert!(validate_booking_dates("2025-01-10", "2025-01-15").is_ok());
        assert!(validate_booking_dates("2025-01-15", "2025-01-10").is_err());
    }
}

// ✅ Für Preisberechnungen
#[test]
fn test_calculate_total_price() {
    let total = calculate_total_price(100.0, 20.0, 10.0);
    assert_eq!(total.unwrap(), 110.0);
}

// ✅ Für Business Logic
#[test]
fn test_room_availability() {
    let conn = setup_test_db();
    // ... test availability logic
}
```

### Test Coverage Ziele:
- **Kritische Business Logic:** 90%+
- **Validierungsfunktionen:** 100%
- **Preisberechnungen:** 100%
- **Database Operations:** 80%+

### Testing Commands:
```bash
# Rust Tests
cd src-tauri
cargo test
cargo test -- --nocapture  # Mit Output

# Frontend Tests (später)
npm test
npm run test:coverage
```

---

## 📝 Git & Commit-Konventionen

### Branch-Strategie:
```
main                    # Production-ready Code
├── develop             # Development Branch
    ├── feature/phase-1-database
    ├── feature/phase-2-validation
    ├── feature/phase-3-ui-components
    └── fix/booking-date-validation
```

### Commit-Message Format:
```
<type>(<scope>): <subject>

[optional body]

[optional footer]
```

**Types:**
- `feat`: Neue Feature
- `fix`: Bugfix
- `refactor`: Code-Refactoring
- `style`: Code-Style Änderungen
- `test`: Tests hinzufügen/ändern
- `docs`: Dokumentation
- `chore`: Build/Config Änderungen

**Beispiele:**
```
feat(database): Add accompanying_guests table

- Create schema with foreign key to bookings
- Add migration function
- Update BookingWithDetails struct

Refs: ROADMAP.md Phase 1.1
```

```
fix(validation): Check-out date must be after check-in

Fixed validation bug where same-day bookings were allowed.
Now properly enforces minimum 1-night stay.

Fixes #42
```

---

## 🚫 Verbotene Praktiken

### Rust:
- ❌ `.unwrap()` oder `.expect()` in Tauri commands
- ❌ String concatenation für SQL queries
- ❌ Blocking operations im Main Thread
- ❌ Unverschlüsselte Passwörter speichern
- ❌ Technische Errors an Frontend weitergeben

### TypeScript:
- ❌ `any` Type verwenden
- ❌ Englische Texte im UI
- ❌ Inline Styles (immer TailwindCSS)
- ❌ Unhandled Promise Rejections
- ❌ Fehlende Loading/Error States

### Allgemein:
- ❌ Code ohne Tests committen (für kritische Features)
- ❌ Hardcoded Credentials oder API Keys
- ❌ Console.logs in Production Code
- ❌ Unkommentierte komplexe Logik
- ❌ Fehlende Type Definitions

---

## ✅ Best Practices Checkliste

### Vor jedem Commit:
- [ ] Code kompiliert ohne Warnings
- [ ] Alle Tests laufen durch
- [ ] TypeScript Errors behoben
- [ ] UI-Texte auf Deutsch
- [ ] Error Handling implementiert
- [ ] Kommentare für komplexe Logik
- [ ] Keine hardcoded Werte
- [ ] Accessibility geprüft

### Code Review Checklist:
- [ ] Folgt Naming-Konventionen
- [ ] Proper Error Handling
- [ ] Input Validation vorhanden
- [ ] Performance berücksichtigt
- [ ] Security best practices befolgt
- [ ] Tests vorhanden (für kritische Features)
- [ ] Dokumentation aktualisiert (wenn nötig)

---

## 📚 Referenzen

### Dokumentation:
- [Tauri 2 Docs](https://v2.tauri.app/)
- [Rust Book](https://doc.rust-lang.org/book/)
- [React Docs](https://react.dev/)
- [TypeScript Handbook](https://www.typescriptlang.org/docs/)
- [TailwindCSS Docs](https://tailwindcss.com/docs)
- [rusqlite Docs](https://docs.rs/rusqlite/)

### Interne Dokumente:
- `ROADMAP.md` - Feature-Implementierungsplan
- `MIGRATION_GUIDE.md` - Python → Tauri Migration
- `.claude/agents/*.md` - Subagent Spezifikationen

---

## 🎯 Qualitätsziele

### Performance:
- App Start: < 2 Sekunden
- Database Queries: < 100ms (für normale Queries)
- UI Interaktionen: < 50ms Response Time
- Tape Chart Rendering: < 500ms (für 50 Zimmer × 31 Tage)

### Code Quality:
- Rust: `cargo clippy` ohne Warnings
- TypeScript: Strict Mode aktiviert
- Test Coverage: > 80% für kritische Pfade

### User Experience:
- Alle UI-Texte auf Deutsch
- Klare, actionable Error Messages
- Loading States für alle async Operations
- Keyboard Navigation funktioniert überall

---

**Version:** 1.0
**Letzte Aktualisierung:** 2025-09-30
**Status:** 🟢 Aktiv

---

## 🔄 Diese Datei wird gelesen von:
- Claude (Main Agent)
- Alle Subagents
- Entwicklern für Referenz

**Bei Änderungen:** Alle Beteiligten informieren und ROADMAP.md synchronisieren.