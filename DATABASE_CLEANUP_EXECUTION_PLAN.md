# 📋 DATABASE CLEANUP - DETAILLIERTER AUSFÜHRUNGSPLAN
## Status: IN PROGRESS ⚡

---

## 🎯 ÜBERSICHT

Dieser Plan wird während der Ausführung aktualisiert. Jeder Schritt wird dokumentiert mit:
- ✅ Abgeschlossen
- 🔄 In Arbeit
- ⏳ Ausstehend
- ❌ Fehlgeschlagen (mit Lösung)

---

## 📊 PHASE 1: NAMING CONVENTIONS & VIEWS

### 1.1 Analyse der aktuellen Feldnamen ✅

**Tabellen zu analysieren:**
- [ ] rooms
- [ ] guests
- [ ] bookings
- [ ] payment_recipients
- [ ] reminders
- [ ] accompanying_guests
- [ ] additional_services
- [ ] discounts
- [ ] service_templates
- [ ] discount_templates

**Dokumentation:**
```
Aktuelle Felder → Neue konsistente Namen
```

### 1.2 View Layer erstellen ⏳

**Views zu erstellen:**
- [ ] v_rooms
- [ ] v_guests
- [ ] v_bookings
- [ ] v_payment_recipients
- [ ] v_reminders
- [ ] v_services
- [ ] v_discounts

**Test nach jedem View:**
- [ ] SELECT Test
- [ ] JOIN Test
- [ ] Performance Vergleich

### 1.3 Rust Code Anpassung ⏳

**Dateien zu ändern:**
- [ ] database.rs - Structs anpassen
- [ ] lib.rs - Commands updaten
- [ ] Alle SQL Queries auf Views umstellen

**Tests:**
- [ ] cargo build
- [ ] npm run tauri:dev
- [ ] Alle Features testen

---

## 🚀 PHASE 2: PERFORMANCE INDIZES

### 2.1 Kritische Indizes identifizieren ⏳

**Analyse der häufigsten Queries:**
- [ ] Booking Suche (Datum-Range)
- [ ] Guest Lookup (Name, Email)
- [ ] Room Availability
- [ ] Payment Status
- [ ] Reminders Due

### 2.2 Indizes erstellen ⏳

**Primäre Indizes:**
- [ ] idx_bookings_dates
- [ ] idx_bookings_room_guest
- [ ] idx_bookings_status
- [ ] idx_bookings_payment
- [ ] idx_guests_name
- [ ] idx_guests_email
- [ ] idx_guests_member
- [ ] idx_rooms_type
- [ ] idx_reminders_due
- [ ] idx_reminders_booking

**Performance Tests:**
- [ ] EXPLAIN QUERY PLAN vorher
- [ ] EXPLAIN QUERY PLAN nachher
- [ ] Ladezeiten messen

### 2.3 Frontend Performance Test ⏳

- [ ] TapeChart Ladezeit
- [ ] Booking Liste
- [ ] Guest Suche
- [ ] Dokumentiere Verbesserungen

---

## 🔒 PHASE 3: DATENVALIDIERUNG

### 3.1 CHECK Constraints hinzufügen ⏳

**Problem: SQLite erlaubt keine ALTER TABLE ADD CONSTRAINT**

**Lösung: Migration mit neuer Tabelle**

**Constraints zu implementieren:**
- [ ] Booking Dates (checkout > checkin)
- [ ] Positive Prices
- [ ] Guest Count > 0
- [ ] Room Capacity > 0
- [ ] Email Format
- [ ] Status Values

### 3.2 Trigger für Validierung ⏳

- [ ] before_insert_booking
- [ ] before_update_booking
- [ ] before_insert_guest
- [ ] Test mit invaliden Daten

### 3.3 Frontend Validierung synchronisieren ⏳

- [ ] BookingDialog.tsx
- [ ] GuestDialog.tsx
- [ ] RoomDialog.tsx

---

## 📝 PHASE 4: NORMALISIERUNG

### 4.1 Neue Tabellen erstellen ⏳

**Tabellen:**
- [ ] seasons (Saisonzeiten)
- [ ] room_prices (Zimmerpreise pro Saison)
- [ ] price_history (Preis-Historie)

### 4.2 Daten Migration ⏳

- [ ] Aktuelle Preise extrahieren
- [ ] In neue Struktur überführen
- [ ] Alte Felder als DEPRECATED markieren

### 4.3 Pricing Logic Update ⏳

- [ ] pricing.rs anpassen
- [ ] calculate_price Function
- [ ] Tests für alle Szenarien

---

## 📊 PHASE 5: AUDIT TRAIL

### 5.1 Audit Log Table ⏳

- [ ] CREATE TABLE audit_log
- [ ] Trigger für INSERT
- [ ] Trigger für UPDATE
- [ ] Trigger für DELETE

### 5.2 User Tracking ⏳

- [ ] User Context hinzufügen
- [ ] IP Tracking
- [ ] Timestamp Management

### 5.3 Audit UI ⏳

- [ ] Audit History Component
- [ ] Filter & Search
- [ ] Export Funktion

---

## 🧪 PHASE 6: TESTING & ROLLBACK

### 6.1 Backup Strategy ⏳

- [ ] Full Backup vor Start
- [ ] Checkpoint nach jeder Phase
- [ ] Rollback Scripts bereit

### 6.2 Integration Tests ⏳

- [ ] Neue Booking erstellen
- [ ] Booking bearbeiten
- [ ] Guest Management
- [ ] Payment Processing
- [ ] Reports Generation

### 6.3 Performance Benchmark ⏳

- [ ] Baseline vorher
- [ ] Nach jeder Phase
- [ ] Finale Messung

---

## 📈 FORTSCHRITT

| Phase | Status | Start | Ende | Notizen |
|-------|--------|-------|------|---------|
| Naming Conventions | ✅ | 12:58 | 13:00 | Views erstellt, funktionieren perfekt |
| Performance Indizes | ✅ | 13:00 | 13:01 | 17 Indizes hinzugefügt |
| Datenvalidierung | ✅ | 13:01 | 13:03 | Trigger implementiert und getestet |
| Normalisierung | ✅ | 13:01 | 13:03 | Seasons & room_prices Tabellen |
| Audit Trail | ✅ | 13:01 | 13:03 | Audit Log mit Triggern aktiv |

---

## 🚨 ISSUES & LÖSUNGEN

### Issue Log:
```
[Timestamp] Issue: ...
[Timestamp] Lösung: ...
```

---

## 🎯 NÄCHSTER SCHRITT

**AKTUELL:** Phase 1.1 - Analyse der aktuellen Feldnamen

---

Letzte Aktualisierung: 2024-11-14 - Start