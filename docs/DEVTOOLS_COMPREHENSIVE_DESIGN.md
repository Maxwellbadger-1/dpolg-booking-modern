# Comprehensive DevTools Design
## Vollständige Test-Abdeckung für 113 Tauri Commands

**Status:** Design Phase
**Ziel:** JEDE Funktion im Programm mit einem Klick testbar machen

---

## 📊 Command-Inventar (113 Commands)

### 1. Bookings (25 Commands)
```typescript
// CREATE
- create_booking_pg

// READ
- get_all_bookings_pg
- get_booking_with_details_by_id_pg
- check_room_availability_pg

// UPDATE
- update_booking_pg
- update_booking_dates_and_room_pg
- update_booking_status_pg
- update_booking_statuses_pg
- update_booking_payment_pg

// DELETE
- delete_booking_pg
- delete_booking_tasks
- cancel_booking_command

// CALCULATIONS
- calculate_full_booking_price_pg
- calculate_nights_command
- link_service_template_to_booking_command
- link_discount_template_to_booking_command
```

### 2. Guests (10 Commands)
```typescript
// CREATE
- create_guest_pg

// READ
- get_all_guests_pg
- get_guest_by_id_pg
- search_guests_pg
- get_guests_by_membership_pg
- get_guest_count_pg
- get_guest_companions_command

// UPDATE
- update_guest_pg

// DELETE
- delete_guest_pg

// UTILITIES
- get_guest_credit_balance
```

### 3. Rooms (7 Commands)
```typescript
// CREATE
- create_room_pg

// READ
- get_all_rooms_pg
- get_room_by_id_pg
- search_rooms_pg

// UPDATE
- update_room_pg

// DELETE
- delete_room_pg

// UTILITIES
- get_room_occupancy_command (STUB)
```

### 4. Additional Services (8 Commands)
```typescript
// CREATE
- create_additional_service_pg

// READ
- get_all_additional_services_pg
- get_additional_service_by_id_pg
- get_additional_services_by_booking_pg

// UPDATE
- update_additional_service_pg

// DELETE
- delete_additional_service_pg

// CALCULATIONS
- calculate_additional_services_total_pg
```

### 5. Discounts (9 Commands)
```typescript
// CREATE
- create_discount_pg

// READ
- get_all_discounts_pg
- get_discount_by_id_pg
- get_discounts_by_booking_pg
- default_discount_type

// UPDATE
- update_discount_pg

// DELETE
- delete_discount_pg

// CALCULATIONS
- calculate_discounts_total_pg
```

### 6. Email System (15 Commands)
```typescript
// EMAIL LOGS
- create_email_log_pg
- get_all_email_logs_pg
- get_email_log_by_id_pg
- get_email_logs_by_booking_pg
- get_email_logs_by_guest_pg
- get_email_logs_by_status_pg
- get_failed_email_logs_pg
- update_email_log_status_pg
- delete_email_log_pg

// EMAIL TEMPLATES
- create_email_template_pg
- get_all_email_templates_pg
- get_active_email_templates_pg
- get_email_template_by_id_pg
- get_email_template_by_name_pg
- update_email_template_pg
- delete_email_template_pg
- toggle_email_template_active_pg

// EMAIL CONFIG
- get_email_config_pg
- update_email_config_pg
```

### 7. Reminders (8 Commands)
```typescript
// CREATE
- create_reminder_pg

// READ
- get_all_reminders_pg
- get_active_reminders_pg
- get_reminder_by_id_pg
- get_reminders_by_booking_pg

// UPDATE
- update_reminder_pg
- complete_reminder_pg
- snooze_reminder_pg

// DELETE
- delete_reminder_pg
```

### 8. Service Templates (6 Commands)
```typescript
// CREATE
- create_service_template_pg

// READ
- get_all_service_templates_pg
- get_active_service_templates_pg
- get_service_template_by_id_pg

// UPDATE
- update_service_template_pg
- toggle_service_template_active_pg

// DELETE
- delete_service_template_pg
```

### 9. Discount Templates (6 Commands)
```typescript
// CREATE
- create_discount_template_pg

// READ
- get_all_discount_templates_pg
- get_active_discount_templates_pg
- get_discount_template_by_id_pg

// UPDATE
- update_discount_template_pg
- toggle_discount_template_active_pg

// DELETE
- delete_discount_template_pg
```

### 10. Settings (8 Commands)
```typescript
// COMPANY
- get_company_settings_pg
- update_company_settings_pg

// PRICING
- get_pricing_settings_pg
- update_pricing_settings_pg

// PAYMENT
- get_payment_settings_pg
- update_payment_settings_pg

// NOTIFICATIONS
- get_notification_settings_pg
- update_notification_settings_pg
```

### 11. Accompanying Guests (7 Commands)
```typescript
// CREATE
- create_accompanying_guest_pg

// READ
- get_all_accompanying_guests_pg
- get_accompanying_guest_by_id_pg
- get_accompanying_guests_by_booking_pg

// UPDATE
- update_accompanying_guest_pg

// DELETE
- delete_accompanying_guest_pg
```

### 12. Payment Recipients (5 Commands)
```typescript
// CREATE
- create_payment_recipient_pg

// READ
- get_all_payment_recipients_pg
- get_active_payment_recipients_pg
- get_payment_recipient_by_id_pg

// UPDATE
- update_payment_recipient_pg

// DELETE
- delete_payment_recipient_pg
```

### 13. Utilities (4 Commands)
```typescript
// VALIDATION
- validate_email_command
- validate_date_range_command

// REPORTING
- get_report_stats_command (STUB)

// MONITORING
- get_pool_stats
```

---

## 🎨 UI Design (Accordion-basiert)

### Layout-Struktur:

```
┌─────────────────────────────────────────────────┐
│ 🛠️ DevTools - Comprehensive Testing Suite      │
│ [Close X]                                       │
├─────────────────────────────────────────────────┤
│ 📊 PostgreSQL Pool Stats (Auto-refresh)        │
│ ┌─────┬──────────┬─────────┬────────┬────────┐│
│ │Size │Available │Waiting  │Utiliz. │Status  ││
│ │5/20 │15        │0        │25%     │OK      ││
│ └─────┴──────────┴─────────┴────────┴────────┘│
├─────────────────────────────────────────────────┤
│ 🧪 Test Categories (113 Commands)              │
│                                                  │
│ ▼ 📋 Bookings (25 Commands)                    │
│   ├─ [Test All] [Clear Results]                │
│   ├─ CREATE                                     │
│   │   └─ [Test] Create Booking                 │
│   ├─ READ                                       │
│   │   ├─ [Test] Get All Bookings               │
│   │   ├─ [Test] Get Booking Details            │
│   │   └─ [Test] Check Room Availability        │
│   ├─ UPDATE                                     │
│   │   ├─ [Test] Update Booking                 │
│   │   ├─ [Test] Update Dates & Room            │
│   │   ├─ [Test] Update Status                  │
│   │   └─ [Test] Update Payment                 │
│   ├─ DELETE                                     │
│   │   ├─ [Test] Delete Booking                 │
│   │   └─ [Test] Cancel Booking                 │
│   └─ CALCULATIONS                               │
│       ├─ [Test] Calculate Full Price            │
│       └─ [Test] Calculate Nights                │
│                                                  │
│ ▼ 👥 Guests (10 Commands)                      │
│   └─ [Similar structure...]                     │
│                                                  │
│ ▶ 🏨 Rooms (7 Commands)                        │
│ ▶ 🧳 Services (8 Commands)                     │
│ ▶ 💰 Discounts (9 Commands)                    │
│ ▶ 📧 Email System (15 Commands)                │
│ ▶ ⏰ Reminders (8 Commands)                     │
│ ▶ 📋 Service Templates (6 Commands)            │
│ ▶ 🎟️ Discount Templates (6 Commands)           │
│ ▶ ⚙️ Settings (8 Commands)                      │
│ ▶ 👨‍👩‍👧‍👦 Accompanying Guests (7 Commands)         │
│ ▶ 💳 Payment Recipients (5 Commands)           │
│ ▶ 🔧 Utilities (4 Commands)                     │
│                                                  │
├─────────────────────────────────────────────────┤
│ 📋 Test Results (Auto-scroll)                  │
│ ┌───────────────────────────────────────────┐ │
│ │ ✅ Create Booking - Success                │ │
│ │    → Booking #145 created                 │ │
│ │ ✅ Get All Bookings - Success             │ │
│ │    → 323 bookings found                   │ │
│ │ ❌ Update Booking - Error                 │ │
│ │    → Database error: column not found     │ │
│ │ [Copy Error] [Details]                    │ │
│ └───────────────────────────────────────────┘ │
│                                                  │
│ [Export Test Report] [Clear All]                │
└─────────────────────────────────────────────────┘
```

---

## 🧪 Test Data Strategy

### Smart Test Data Generation

**Principle:** Verwende ECHTE Daten aus der Datenbank + generiere minimale Test-Daten

```typescript
// 1. Fetch real data first
const guests = await invoke('get_all_guests_pg');
const rooms = await invoke('get_all_rooms_pg');

// 2. Use real IDs when available
const testBookingData = {
  guestId: guests[0]?.id || null,  // Use real guest if available
  roomId: rooms[0]?.id || null,    // Use real room if available
  checkIn: new Date().toISOString().split('T')[0],
  checkOut: addDays(new Date(), 3).toISOString().split('T')[0],
  // ... rest of data
};

// 3. Fallback: Create minimal test entities if needed
if (!guests.length) {
  await testCreateGuest(); // Creates test guest first
}
```

### Test Data Templates

```typescript
const TEST_DATA = {
  guest: {
    vorname: 'DevTools',
    nachname: 'TestUser',
    email: 'devtools.test@example.com',
    telefon: '+49 000 000000',
    dpolgMitglied: true,
    strasse: 'Test Straße 1',
    plz: '00000',
    ort: 'Teststadt',
  },
  room: {
    roomNumber: 'DEV-999',
    roomType: 'Einzelzimmer',
    capacity: 1,
    pricePerNight: 50.0,
  },
  booking: {
    // Uses real guest/room IDs + generated dates
  },
  emailTemplate: {
    name: 'DevTools Test Template',
    subject: 'Test Email',
    body: '<p>Dies ist eine Test-Email</p>',
    isActive: false, // Don't interfere with production
  }
};
```

---

## 🔧 Implementation Strategy

### Phase 1: Scaffolding (30 min)
1. Create comprehensive DevTools.tsx structure
2. Implement Accordion component
3. Add category containers (13 categories)

### Phase 2: Command Integration (2 hours)
1. For each category, add all commands
2. Implement test functions with smart data generation
3. Add result logging and error handling

### Phase 3: Advanced Features (1 hour)
1. Test report export (JSON/CSV)
2. Command search/filter
3. Batch testing (run all tests in category)
4. Performance metrics (timing for each test)

### Phase 4: Documentation (30 min)
1. Add tooltips explaining each command
2. Show expected parameters
3. Show return value structure

---

## 📝 File Structure

```
src/components/
├── DevTools/
│   ├── DevTools.tsx                  (Main component - 3000+ lines)
│   ├── AccordionCategory.tsx         (Reusable accordion)
│   ├── CommandTest.tsx               (Single command test button)
│   ├── TestResults.tsx               (Results display)
│   ├── PoolStatsDisplay.tsx          (Pool monitoring)
│   └── testDataGenerators.ts         (Smart test data)
└── DevTools.tsx                      (Re-export for compatibility)
```

---

## ✅ Success Criteria

1. **100% Command Coverage:** Alle 113 Commands sind testbar
2. **Zero Manual Configuration:** Commands werden automatisch erkannt
3. **Smart Test Data:** Tests funktionieren mit echten + generierten Daten
4. **Clear Error Reports:** Jeder Fehler zeigt Command, Parameter, Error
5. **Performance Monitoring:** Pool Stats + Command execution time
6. **User-Friendly:** Kategorie-basiert, durchsuchbar, exportierbar

---

## 🚀 Next Steps

1. ✅ Analyse abgeschlossen (113 Commands, 49 Komponenten)
2. ⏳ Design dokumentiert (dieses File)
3. ⏳ Implementation Phase 1 (Scaffolding)
4. ⏳ Implementation Phase 2 (Command Integration)
5. ⏳ Implementation Phase 3 (Advanced Features)
6. ⏳ Testing & Bugfixing
7. ⏳ Documentation

---

**Geschätzte Implementierungszeit:** 4-5 Stunden
**Dateigröße:** ~3000-4000 Zeilen TypeScript
**Wartbarkeit:** Hoch (Auto-discovery + Template-basiert)
