# Real-Time Multi-User System - Implementierung

**Datum:** 2025-11-30
**Status:** In Arbeit
**Ziel:** Zwei Rechner können gleichzeitig arbeiten und sehen Änderungen sofort

---

## 🎯 Übersicht

### Was wird implementiert:

1. **~~PostgreSQL LISTEN/NOTIFY~~** - ❌ Zu komplex, stattdessen: **Polling-basierte Updates**
2. **Frontend Event-Handling** - UI aktualisiert sich automatisch alle 3 Sekunden
3. **Konflikt-Warnung** - User sehen wenn jemand anderes gerade bearbeitet (Optional)
4. **Offline-Detection** - Banner wenn keine Verbindung (✅ FERTIG)

### ⚠️ Architektur-Änderung: Polling statt NOTIFY

**Grund:** PostgreSQL LISTEN/NOTIFY mit deadpool-postgres ist zu komplex.

**Neue Lösung:**
- Frontend fragt alle 3 Sekunden: "Gab's Updates seit Timestamp X?"
- Backend vergleicht `updated_at` Timestamps
- Nur geänderte Entities werden zurückgegeben
- **Latenz:** 1-3 Sekunden (völlig OK für 2-3 User)

---

## 📋 Implementierungsplan

### Phase 1: Backend - Polling API erstellen

**Status:** ✅ FERTIG

**Was passiert:**
```
User A ändert Buchung 123 um 10:00:00
    ↓
DB: updated_at = 2025-11-30 10:00:00
    ↓
User B Frontend fragt um 10:00:02:
"Gib mir alle Updates seit 10:00:00"
    ↓
Backend: SELECT * WHERE updated_at > '10:00:00'
    ↓
User B sieht Update nach 2 Sekunden
```

**Dateien:**
- `src-tauri/src/lib_pg.rs` - Neue Commands:
  - `get_updates_since` - Gibt alle geänderten Entities zurück

**Aufgaben:**
- [x] Offline-Detection (3 Sek Timeout)
- [x] NOTIFY Migration erstellt (für später)
- [x] `get_updates_since` Command erstellen
- [x] Frontend Polling alle 3 Sekunden

---

### Phase 2: Frontend - Event-Handling

**Status:** ✅ FERTIG

**Was passiert:**
```typescript
// Frontend pollt alle 3 Sekunden für Updates
useEffect(() => {
  const pollForUpdates = async () => {
    const response = await invoke('get_updates_since', {
      sinceTimestamp: lastTimestamp
    });

    // Merge updates into state
    if (response.bookings.length > 0) {
      setBookings(prev => mergeUpdates(prev, response.bookings));
    }
  };

  const interval = setInterval(pollForUpdates, 3000);
  return () => clearInterval(interval);
}, [hasLoadedOnce]);
```

**Dateien:**
- `src/context/DataContext.tsx` - Polling alle 3 Sekunden
- Components nutzen automatisch aktualisierten State

**Aufgaben:**
- [x] Polling-Mechanismus in DataContext
- [x] Optimistic Updates beibehalten
- [x] Merge-Strategie: Neue Daten überschreiben Alte

---

### Phase 3: Konflikt-Warnung (Optional)

**Status:** ⏳ Pending

**Szenario:**
```
User A öffnet Buchung 123 um 10:00
User B öffnet Buchung 123 um 10:01
→ User B sieht Warnung: "⚠️ User A bearbeitet gerade diese Buchung"
```

**Implementierung:**
- Tracking welcher User welche Entity bearbeitet
- Warnung anzeigen (nicht blockieren!)
- Auto-Refresh wenn anderer User speichert

**Aufgaben:**
- [ ] "Editing Lock" Tabelle in DB (optional)
- [ ] Toast-Warnung bei Konflikt
- [ ] Auto-Reload nach fremdem Save

---

### Phase 4: Testing

**Status:** ⏳ Pending

**Test-Szenarien:**

1. **Create Test:**
   - User A erstellt Buchung
   - User B sieht neue Buchung SOFORT (ohne F5)

2. **Update Test:**
   - User A ändert Buchung
   - User B sieht Änderung SOFORT

3. **Delete Test:**
   - User A löscht Buchung
   - User B sieht Löschung SOFORT

4. **Konflikt Test:**
   - Beide bearbeiten gleichzeitig
   - Konflikt-Warnung erscheint

**Aufgaben:**
- [ ] 2 Instanzen der App starten
- [ ] Parallel Änderungen machen
- [ ] Prüfen: Updates ohne F5 sichtbar?
- [ ] Performance: Lags bei vielen Events?

---

## 🔧 Technische Details

### PostgreSQL NOTIFY Format

```sql
-- Trigger sendet JSON Event:
NOTIFY booking_changes, '{
  "table": "bookings",
  "action": "UPDATE",
  "id": 123,
  "timestamp": "2025-11-30T10:30:00Z"
}'
```

### Rust Event Struct

```rust
pub struct DbChangeEvent {
    pub table: String,      // "bookings", "guests", "rooms"
    pub action: String,     // "INSERT", "UPDATE", "DELETE"
    pub id: i32,           // Affected record ID
    pub timestamp: String,
}
```

### Frontend Event Types

```typescript
type TauriEvent =
  | { type: 'booking-created', payload: Booking }
  | { type: 'booking-updated', payload: Booking }
  | { type: 'booking-deleted', payload: { id: number } }
  | { type: 'guest-updated', payload: Guest }
  | { type: 'room-updated', payload: Room };
```

---

## ⚠️ Herausforderungen & Lösungen

### Problem 1: Event-Storm
**Problem:** 1000 Events in 1 Sekunde → UI laggt
**Lösung:** Debouncing (max 1 Update pro Sekunde pro Entity)

### Problem 2: Offline während Event
**Problem:** User ist offline, verpasst Events
**Lösung:** Bei Reconnect: Full Refresh

### Problem 3: Optimistic Updates vs. Server Events
**Problem:** User A speichert → Optimistic Update → Server Event kommt → Doppel-Update?
**Lösung:** Event ignorieren wenn `updated_by = current_user`

### Problem 4: Race Conditions
**Problem:** Event kommt bevor Save-Response
**Lösung:** Version-Check / `updated_at` Timestamp

---

## 📊 Performance-Ziele

- **Event-Latenz:** < 500ms (User A speichert → User B sieht Update)
- **UI-Update:** < 100ms (Event empfangen → UI gerendert)
- **Keine Lags:** Auch bei 10+ Events/Sekunde
- **Memory:** Keine Event-Leaks (Listener cleanup!)

---

## ✅ Akzeptanzkriterien

### Muss funktionieren:
- [x] Offline-Banner nach 3 Sekunden
- [ ] User B sieht neue Buchung von User A ohne F5
- [ ] User B sieht Änderungen von User A ohne F5
- [ ] User B sieht Löschung von User A ohne F5
- [ ] Keine Doppel-Updates (Optimistic + Event)
- [ ] Kein Lag bei vielen Events

### Nice-to-have:
- [ ] Konflikt-Warnung bei gleichzeitiger Bearbeitung
- [ ] "User X bearbeitet gerade..." Anzeige
- [ ] Auto-Reload bei fremdem Save

---

## 🚀 Nächste Schritte

1. ✅ Offline-Detection fertig (3 Sek Timeout)
2. ✅ Backend Polling API (`get_updates_since`) implementiert
3. ✅ Frontend Polling (alle 3 Sekunden) implementiert
4. ⏳ Testing mit 2 Clients - NÄCHSTER SCHRITT!
5. ⏹️ PostgreSQL Triggers (Optional - falls wir später auf LISTEN/NOTIFY wechseln)

---

## 📝 Notizen

- Bestehender Code in `database_pg/listener.rs` bereits vorhanden!
- Triggers sollten aus früherer Migration existieren
- Tauri Events API: `app.emit()` für Frontend-Broadcasting
- DataContext ist zentraler Punkt für Event-Handling

---

**Letzte Aktualisierung:** 2025-11-30 12:00

---

## ✅ Fertiggestellt (2025-11-30)

### Backend-Implementierung

1. **`get_updates_since` Command** ([src-tauri/src/lib_pg.rs](src-tauri/src/lib_pg.rs:2811-2897))
   - Akzeptiert `since_timestamp` Parameter
   - Gibt alle Bookings/Guests/Rooms zurück die seit dem Timestamp geändert wurden
   - Verwendet `Booking::from(row)`, `Guest::from(row)`, `Room::from(row)` für Row-Parsing
   - Gibt neuen Timestamp zurück für nächsten Poll

2. **Offline Detection** ([src-tauri/src/lib_pg.rs](src-tauri/src/lib_pg.rs))
   - `check_db_connection` Command mit 3-Sekunden-Timeout
   - Frontend zeigt roten Banner bei Verbindungsabbruch

### Frontend-Implementierung

1. **Polling-Mechanismus** ([src/context/DataContext.tsx](src/context/DataContext.tsx:168-229))
   - Poll alle 3 Sekunden für Updates (startet nach Initial Load)
   - Tracked `lastTimestamp` für inkrementelle Updates
   - Merged Updates in bestehenden State (filtert alte Versionen raus)
   - Console-Logs zeigen Anzahl aktualisierter Entities

2. **Merge-Strategie**
   - Server-Daten überschreiben lokale Daten
   - Optimistic Updates bleiben erhalten bis Server antwortet
   - Keine doppelten Updates durch ID-basiertes Filtering

### Kompilierung

- ✅ Backend kompiliert erfolgreich (nur Warnings, keine Errors)
- ✅ Frontend kompiliert erfolgreich
- ✅ App läuft und pollt im Hintergrund

### Tests

- ⏳ Noch kein Multi-Client-Test durchgeführt
- ⏳ Polling läuft theoretisch, muss mit 2 Instanzen getestet werden
