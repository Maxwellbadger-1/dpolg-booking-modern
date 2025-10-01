# 🗺️ Roadmap: Python → Modern Tauri React Migration

## Aktueller Stand
✅ **Bereits implementiert:**
- **Tape Chart Visualisierung** (TapeChart.tsx)
  - Drag & Drop mit @dnd-kit/core v6.3.1
  - Resize Funktionalität (virtuelle Handles, 30px Breite)
  - Overlap Prevention (Bookings können sich berühren aber nicht überlappen)
  - Density Modes (Compact/Comfortable/Spacious) mit localStorage
  - Visuelle Feedback (rot bei Overlap, blau bei gültiger Position)
  - Monatliche Kalenderansicht mit Navigation
  - Status-basierte Farbcodierung (5 Status-Typen)
  - "Heute" Button mit Auto-Scroll
  - Wochenend-Hervorhebung
- **Datenbank** (SQLite via Rusqlite)
  - Vollständiges Schema (rooms, guests, bookings, accompanying_guests, additional_services, discounts)
  - Foreign Key Enforcement + CASCADE DELETE
  - Performance-Indexes für alle wichtigen Queries
  - Migration-safe ALTER TABLE Statements
- **Backend** (Rust/Tauri 2)
  - 25 Database Functions
  - 24 Tauri Commands (CRUD für Bookings, Guests, Rooms, Services, Discounts)
  - Validation Module (Email, Phone, Dates, Room Availability)
  - Pricing Module (Nights, Base Price, Services, Discounts, Total)
  - Deutsche Error Messages
- **Frontend** (React 18 + TypeScript)
  - GuestDialog Component (Create/Edit Gäste)
  - DevTools Panel mit Test-Buttons
  - Moderne UI mit TailwindCSS v3
  - Header mit Stats (Zimmer, Aktive Buchungen, Auslastung)

---

## Phase 1: Datenbank-Erweiterung & Backend-Fundament
**Ziel:** Vollständiges Datenbank-Schema und CRUD-Operationen

### 1.1 Datenbank-Schema erweitern ✅ ERLEDIGT
**Dateien:** `src-tauri/src/database.rs`

- [x] **Tabelle `accompanying_guests` hinzufügen**
  ```sql
  - id: INTEGER PRIMARY KEY
  - booking_id: INTEGER (FK → bookings)
  - vorname: TEXT NOT NULL
  - nachname: TEXT NOT NULL
  - geburtsdatum: TEXT
  ```
  ✅ Implementiert in `database.rs:134-143` mit CASCADE DELETE

- [x] **Tabelle `additional_services` hinzufügen**
  ```sql
  - id: INTEGER PRIMARY KEY
  - booking_id: INTEGER (FK → bookings)
  - service_name: TEXT NOT NULL
  - service_price: REAL NOT NULL
  - created_at: TEXT DEFAULT CURRENT_TIMESTAMP
  ```
  ✅ Implementiert in `database.rs:147-156` mit CASCADE DELETE + CHECK constraint

- [x] **Tabelle `discounts` hinzufügen**
  ```sql
  - id: INTEGER PRIMARY KEY
  - booking_id: INTEGER (FK → bookings)
  - discount_name: TEXT NOT NULL
  - discount_type: TEXT (percent/fixed)
  - discount_value: REAL NOT NULL
  - created_at: TEXT DEFAULT CURRENT_TIMESTAMP
  ```
  ✅ Implementiert in `database.rs:160-170` mit CASCADE DELETE + CHECK constraints

- [x] **Tabelle `guests` erweitern**
  ```sql
  Felder hinzufügen:
  - strasse: TEXT
  - plz: TEXT
  - ort: TEXT
  - mitgliedsnummer: TEXT
  - notizen: TEXT
  - created_at: TEXT DEFAULT CURRENT_TIMESTAMP
  ```
  ✅ Implementiert mit ALTER TABLE in `database.rs:119-121` (migration-safe)

- [x] **Tabelle `bookings` erweitern**
  ```sql
  Felder hinzufügen:
  - anzahl_begleitpersonen: INTEGER DEFAULT 0
  - grundpreis: REAL
  - services_preis: REAL DEFAULT 0
  - rabatt_preis: REAL DEFAULT 0
  - anzahl_naechte: INTEGER
  - created_at: TEXT DEFAULT CURRENT_TIMESTAMP
  - updated_at: TEXT DEFAULT CURRENT_TIMESTAMP
  ```
  ✅ Implementiert mit ALTER TABLE in `database.rs:124-129` (migration-safe)

- [x] **Rust Structs erstellt**
  - `AccompanyingGuest` (`database.rs:56-62`)
  - `AdditionalService` (`database.rs:66-72`)
  - `Discount` (`database.rs:76-83`)
  - Extended `Guest` struct (`database.rs:18-31`)
  - Extended `Booking` struct (`database.rs:34-52`)

- [x] **Performance-Indexes erstellt** (`database.rs:209-248`)
  - `idx_bookings_dates` - Datumsbereich-Abfragen
  - `idx_bookings_room` - Foreign key room_id
  - `idx_bookings_guest` - Foreign key guest_id
  - `idx_accompanying_booking` - Foreign key für Begleitpersonen
  - `idx_services_booking` - Foreign key für Services
  - `idx_discounts_booking` - Foreign key für Rabatte
  - `idx_guests_email` - Email-Suche

- [x] **Foreign Key Enforcement** aktiviert
  - `PRAGMA foreign_keys = ON` für alle Connections (`database.rs:68, 186, 206`)
  - CASCADE DELETE für abhängige Daten
  - CHECK constraints für Datenvalidierung

### 1.2 Rust Backend Commands erweitern ✅ ERLEDIGT
**Dateien:** `src-tauri/src/lib.rs`, `src-tauri/src/database.rs`

- [x] **Guest Management Commands** (6 Functions + 6 Commands)
  ```rust
  - create_guest() / create_guest_command
  - update_guest() / update_guest_command
  - delete_guest() / delete_guest_command
  - get_guest_by_id() / get_guest_by_id_command
  - search_guests(query) / search_guests_command
  - get_all_guests() / get_all_guests_command
  ```
  ✅ Implementiert in `database.rs:446-631` und `lib.rs:42-122`

- [x] **Booking Management Commands** (5 Functions + 5 Commands)
  ```rust
  - create_booking() / create_booking_command
  - update_booking() / update_booking_command
  - delete_booking() / delete_booking_command
  - cancel_booking() / cancel_booking_command
  - get_booking_by_id() / get_booking_by_id_command
  ```
  ✅ Implementiert in `database.rs:748-911` und `lib.rs:190-278`

- [x] **Room Management Commands** (4 Functions + 4 Commands)
  ```rust
  - create_room() / create_room_command
  - update_room() / update_room_command
  - delete_room() / delete_room_command
  - get_room_by_id() / get_room_by_id_command
  ```
  ✅ Implementiert in `database.rs:637-742` und `lib.rs:128-184`

- [x] **Additional Services Commands** (3 Functions + 3 Commands)
  ```rust
  - add_service_to_booking() / add_service_command
  - delete_service() / delete_service_command
  - get_booking_services() / get_booking_services_command
  ```
  ✅ Implementiert in `database.rs:917-990` und `lib.rs:284-304`

- [x] **Accompanying Guests Commands** (3 Functions + 3 Commands)
  ```rust
  - add_accompanying_guest() / add_accompanying_guest_command
  - delete_accompanying_guest() / delete_accompanying_guest_command
  - get_booking_accompanying_guests() / get_booking_accompanying_guests_command
  ```
  ✅ Implementiert in `database.rs:996-1070` und `lib.rs:310-331`

- [x] **Discounts Commands** (3 Functions + 3 Commands)
  ```rust
  - add_discount_to_booking() / add_discount_command
  - delete_discount() / delete_discount_command
  - get_booking_discounts() / get_booking_discounts_command
  ```
  ✅ Implementiert in `database.rs:1076-1152` und `lib.rs:337-358`

**Zusammenfassung:**
- ✅ 25 Database Functions implementiert
- ✅ 24 Tauri Commands implementiert + registriert
- ✅ Foreign Key Enforcement überall aktiviert
- ✅ Deutsche Error Messages
- ✅ Proper rows_affected checks
- ✅ Kompiliert ohne Fehler

---

## Phase 2: Business Logic & Validierungen ✅ ERLEDIGT
**Ziel:** Buchungslogik, Preisberechnung, Verfügbarkeits-Checks

### 2.1 Validierungs-Modul erstellen ✅ ERLEDIGT
**Datei:** `src-tauri/src/validation.rs`

- [x] **Datumsvalidierung**
  ```rust
  - validate_date_format(date_str) // YYYY-MM-DD parsing
  - validate_date_range(checkin, checkout) // checkout > checkin, min 1 night
  - is_date_in_future(date_str) // Check if date is future
  ```
  ✅ Implementiert mit chrono::NaiveDate + 3 Tests

- [x] **Verfügbarkeits-Check**
  ```rust
  - check_room_availability(room_id, start, end, exclude_booking_id, conn)
  - Overlap detection: checkin < new_checkout AND checkout > new_checkin
  - Ignores cancelled bookings
  ```
  ✅ Implementiert mit SQLite queries + 7 Tests

- [x] **Gast-Validierung**
  ```rust
  - validate_email(email) // RFC5322 compliant regex
  - validate_phone_number(phone) // German formats
  - validate_guest_count(count, capacity) // Capacity check
  - validate_reservation_number(nummer) // Format validation
  ```
  ✅ Implementiert mit regex + once_cell::Lazy + 9 Tests

**Tauri Commands hinzugefügt** (`lib.rs:363-388`):
- ✅ validate_email_command
- ✅ validate_date_range_command
- ✅ check_room_availability_command

**Test Coverage:**
- ✅ 20 Unit Tests (alle bestanden)
- ✅ Edge cases abgedeckt
- ✅ Deutsche Error Messages

### 2.2 Preisberechnungs-Modul ✅ ERLEDIGT
**Datei:** `src-tauri/src/pricing.rs`

- [x] **Nächte-Berechnung**
  ```rust
  - calculate_nights(checkin, checkout) // Date difference
  ```
  ✅ Implementiert mit chrono + 8 Tests

- [x] **Grundpreis-Berechnung**
  ```rust
  - calculate_base_price(nights, price_per_night)
  - Simple: nights × price
  ```
  ✅ Implementiert + 6 Tests

- [x] **Services-Berechnung**
  ```rust
  - calculate_services_total(services) // Sum of service prices
  ```
  ✅ Implementiert + 4 Tests

- [x] **Rabatt-Berechnung**
  ```rust
  - apply_discount_percentage(price, percentage) // 0-100%
  - apply_discount_fixed(price, discount_amount) // Fixed amount
  ```
  ✅ Implementiert + 13 Tests

- [x] **Gesamtpreis-Berechnung**
  ```rust
  - calculate_total_price(grundpreis, services, rabatte)
  - Formula: grundpreis + services - rabatte (min 0.0)
  - calculate_booking_total() // Main comprehensive function
  ```
  ✅ Implementiert mit Database integration + 19 Tests

**Tauri Commands hinzugefügt** (`lib.rs:390-422`):
- ✅ calculate_nights_command
- ✅ calculate_booking_price_command (returns JSON with all components)

**Test Coverage:**
- ✅ 50 Unit Tests (alle bestanden)
- ✅ Database integration tests
- ✅ Edge cases (zero, negative, excessive values)
- ✅ Member vs Non-member pricing
- ✅ Percentage and fixed discounts
- ✅ Multiple discounts combined

**DevTools Tests hinzugefügt** (`src/components/DevTools.tsx`):
- ✅ testValidateEmail() - Email validation test
- ✅ testValidateDateRange() - Date range validation test
- ✅ testCheckAvailability() - Room availability test
- ✅ testCalculateNights() - Nights calculation test
- ✅ testCalculatePrice() - Price calculation test
- ✅ Integrated into runCompleteTest() suite
- ✅ New buttons: "Validate Email", "Calculate Nights"

---

## Phase 3: Frontend UI-Komponenten
**Ziel:** React-Komponenten für alle Verwaltungsfunktionen

### 3.1 Buchungsverwaltung ✅ ERLEDIGT
**Ordner:** `src/components/BookingManagement/`

- [x] **BookingDialog.tsx** - Neue Buchung erstellen/bearbeiten
  ```tsx
  Felder:
  ✅ Gast auswählen (Dropdown mit allen Gästen, zeigt Mitgliedschaft)
  ✅ Check-in Datum (DatePicker mit HTML5 date input)
  ✅ Check-out Datum (DatePicker mit HTML5 date input)
  ✅ Zimmer auswählen (Dropdown mit Kapazität-Info)
  ✅ Anzahl Gäste (Number Input mit Validierung)
  ✅ Status (Select: 5 Status-Optionen mit Farben)
  ✅ Bemerkungen (Textarea)
  ✅ Live-Preisberechnung (zeigt Nächte, Preis/Nacht, Grundpreis)
  ✅ Mitglied vs Nicht-Mitglied Preise
  ✅ Validierung (Kapazität, Datumsbereich, Pflichtfelder)
  ✅ Verfügbarkeits-Check mit Live-Anzeige (grün/rot/checking)
  ```
  ⏳ TODO: Begleitpersonen, Services, Rabatte (Dynamic Lists)

- [x] **BookingList.tsx** - Übersicht aller Buchungen
  ```tsx
  Features:
  ✅ Tabellen-Ansicht mit hover effects
  ✅ Suche nach Reservierungsnummer, Gastname, Zimmer
  ✅ Filter nach Status (6 Optionen)
  ✅ Status-Badges mit Icons & Farben
  ✅ Formatierte Daten (DD.MM.YYYY)
  ✅ Preis-Anzeige
  ✅ Edit Actions (funktional)
  ✅ Connected mit BookingDialog
  ✅ Delete Actions mit Bestätigung
  ```
  ⏳ TODO: Filter nach Zeitraum
  ⏳ TODO: Sortierung

- [ ] **BookingDetails.tsx** - Detailansicht einer Buchung
  ```tsx
  Anzeige:
  - Alle Buchungsdaten
  - Gast-Details
  - Zimmer-Details
  - Begleitpersonen
  - Services mit Preisen
  - Rabatte
  - Preisaufschlüsselung
  - Aktionen: Bearbeiten, Stornieren, Email senden
  ```

### 3.2 Gästeverwaltung ✅ ERLEDIGT
**Ordner:** `src/components/GuestManagement/`

- [x] **GuestDialog.tsx** - Gast erstellen/bearbeiten
  ```tsx
  Felder:
  ✅ Vorname* (required)
  ✅ Nachname* (required)
  ✅ Email* (mit Validierung)
  ✅ Telefon (mit Format-Validierung)
  ✅ Straße
  ✅ PLZ (mit Validierung)
  ✅ Ort
  ✅ DPolG Mitglied (Checkbox)
  ✅ Mitgliedsnummer (wenn Mitglied)
  ✅ Notizen (Textarea)
  ```
  ✅ Vollständig implementiert mit Toast-Benachrichtigungen

- [x] **GuestList.tsx** - Übersicht aller Gäste
  ```tsx
  Features:
  ✅ Tabellen-Ansicht mit hover effects
  ✅ Suche nach Name, Email, Telefon
  ✅ Filter: Alle/Mitglieder/Nicht-Mitglieder
  ✅ Edit/Delete Actions mit Bestätigung
  ✅ Membership Badges mit Icons
  ✅ Kontaktdaten mit Icons (Mail, Phone)
  ✅ Adress-Display
  ```
  ✅ Implementiert mit Live-Daten via `get_all_guests_command`

- [ ] **GuestDetails.tsx** - Detailansicht eines Gastes
  ```tsx
  Anzeige:
  - Alle Gast-Daten
  - Buchungshistorie
  - Gesamtumsatz
  - Letzte Buchung
  ```

### 3.3 Zimmerverwaltung ✅ ERLEDIGT
**Ordner:** `src/components/RoomManagement/`

- [x] **RoomDialog.tsx** - Zimmer erstellen/bearbeiten
  ```tsx
  Felder:
  ✅ Zimmername* (required)
  ✅ Gebäude/Typ* (required)
  ✅ Kapazität* (number, 1-20)
  ✅ Preis Mitglied* (number mit € Symbol)
  ✅ Preis Nicht-Mitglied* (number mit € Symbol)
  ✅ Ort* (required)
  ✅ Schlüsselcode (optional, monospace font)
  ```
  ✅ Vollständig implementiert mit Error Handling

- [x] **RoomList.tsx** - Übersicht aller Zimmer
  ```tsx
  Features:
  ✅ Card-basierte Grid-Ansicht (responsive)
  ✅ Filter nach Ort (dynamisch aus Daten)
  ✅ Suche nach Name, Gebäude, Ort
  ✅ Edit/Delete Actions mit Bestätigung
  ✅ Gradient-Header mit Icons
  ✅ Preisanzeige (Mitglieder vs Nicht-Mitglieder)
  ✅ Kapazitäts-Display mit Icon
  ✅ Schlüsselcode-Anzeige (wenn vorhanden)
  ```
  ✅ Implementiert mit Live-Daten via `get_all_rooms`

### 3.4 Navigation & Layout ✅ ERLEDIGT
**Dateien:** `src/components/Layout/`, `src/App.tsx`

- [x] **Tab Navigation** implementieren
  ```tsx
  Tabs:
  ✅ Dashboard (Tape Chart)
  ✅ Buchungen (BookingList mit Live-Daten)
  ✅ Gäste (GuestList mit Live-Daten)
  ✅ Zimmer (RoomList mit Live-Daten, Grid Layout)
  ```
  ✅ Implementiert in `App.tsx:199-231`
  - Type-safe tab switching mit `type Tab = 'dashboard' | 'bookings' | 'guests' | 'rooms'`
  - Visual active state mit Tailwind
  - Icons von Lucide React
  - Smooth transitions

- [x] **Header** fertig
  ```tsx
  Features:
  ✅ Logo + Title
  ✅ Stats (Zimmer, Aktive Buchungen, Auslastung)
  ✅ Action Buttons (Neuer Gast)
  ✅ Datums-Anzeige
  ```

- [x] **List Views mit CRUD** implementiert
  ```tsx
  ✅ BookingList.tsx - Tabelle mit Status-Badges, Suche, Filter
  ✅ GuestList.tsx - Tabelle mit Membership-Badges, Suche, Filter
  ✅ RoomList.tsx - Card-basiertes Grid mit Pricing, Suche, Filter
  ```

- [x] **Dialog Components verbunden**
  ```tsx
  ✅ RoomDialog - Create/Edit Zimmer (connected zu RoomList)
  ✅ GuestDialog - Create/Edit Gäste (connected zu GuestList)
  ```

---

## Phase 4: Such- & Filterfunktionen
**Ziel:** Leistungsstarke Suche und Filter für alle Bereiche

### 4.1 Globale Such-Komponente
**Neue Datei:** `src/components/Search/GlobalSearch.tsx`

- [ ] **Globale Suche** implementieren
  ```tsx
  Features:
  - Suche über Buchungen, Gäste, Zimmer
  - Keyboard Shortcut (Cmd+K / Ctrl+K)
  - Live-Ergebnisse mit Kategorien
  - Schnellnavigation zu Ergebnis
  ```

### 4.2 Filter-Komponenten
**Neuer Ordner:** `src/components/Filters/`

- [ ] **DateRangeFilter.tsx** - Zeitraum-Filter
- [ ] **StatusFilter.tsx** - Status-Filter mit Checkboxes
- [ ] **RoomFilter.tsx** - Zimmer-Filter (Multi-Select)
- [ ] **MembershipFilter.tsx** - Mitgliedschaft-Filter

---

## Phase 5: Reports & Statistiken
**Ziel:** Auswertungen und Export-Funktionen

### 5.1 Report-Komponenten
**Neuer Ordner:** `src/components/Reports/`

- [ ] **OccupancyReport.tsx** - Belegungsstatistik
  ```tsx
  Features:
  - Zeitraum wählbar
  - Auslastung pro Zimmer (Chart)
  - Auslastung pro Monat (Chart)
  - Export als PDF/Excel
  ```

- [ ] **RevenueReport.tsx** - Umsatzübersicht
  ```tsx
  Features:
  - Gesamtumsatz pro Zeitraum
  - Aufschlüsselung nach Zimmern
  - Services-Umsatz
  - Mitglieder vs. Nicht-Mitglieder
  - Chart-Visualisierung
  - Export-Funktion
  ```

- [ ] **GuestReport.tsx** - Gäste-Statistik
  ```tsx
  Features:
  - Anzahl Buchungen pro Gast
  - Top-Gäste nach Umsatz
  - Neue Gäste pro Monat
  - Mitglieder-Statistik
  - Export als CSV/Excel
  ```

### 5.2 Export-Funktionalität
**Neue Datei:** `src-tauri/src/export.rs`

- [ ] **PDF-Export** (mit Rust PDF Library)
  ```rust
  - export_booking_confirmation(booking_id)
  - export_invoice(booking_id)
  - export_report_pdf(report_data)
  ```

- [ ] **Excel/CSV-Export**
  ```rust
  - export_bookings_csv(filters)
  - export_guests_csv(filters)
  - export_revenue_xlsx(filters)
  ```

---

## Phase 6: Email-System
**Ziel:** Automatische Emails und Templates

### 6.1 Email Backend
**Neue Datei:** `src-tauri/src/email.rs`

- [ ] **SMTP-Konfiguration**
  ```rust
  Tabelle: email_config
  - smtp_server: TEXT
  - smtp_port: INTEGER
  - smtp_username: TEXT
  - smtp_password: TEXT (verschlüsselt)
  - from_email: TEXT
  - from_name: TEXT
  - use_tls: INTEGER (boolean)
  ```

- [ ] **Email-Templates**
  ```rust
  Tabelle: email_templates
  - id: INTEGER PRIMARY KEY
  - template_name: TEXT (confirmation/reminder/invoice)
  - subject: TEXT
  - body: TEXT (mit Platzhaltern)
  - created_at: TEXT
  ```

- [ ] **Email-Funktionen**
  ```rust
  - send_confirmation_email(booking_id)
  - send_reminder_email(booking_id)
  - send_invoice_email(booking_id)
  - send_custom_email(guest_id, subject, body)
  ```

### 6.2 Email Frontend
**Neuer Ordner:** `src/components/Email/`

- [ ] **EmailTemplateEditor.tsx**
  ```tsx
  Features:
  - Template bearbeiten
  - Platzhalter-Liste anzeigen
  - Preview-Funktion
  - Test-Email senden
  ```

- [ ] **EmailConfigDialog.tsx**
  ```tsx
  Features:
  - SMTP-Einstellungen konfigurieren
  - Verbindung testen
  - Standard-Absender festlegen
  ```

### 6.3 Template-Platzhalter
- [ ] **Platzhalter definieren**
  ```
  {gast_vorname}
  {gast_nachname}
  {gast_email}
  {buchung_reservierungsnummer}
  {zimmer_name}
  {checkin_date}
  {checkout_date}
  {anzahl_naechte}
  {anzahl_gaeste}
  {gesamtpreis}
  {grundpreis}
  {services_preis}
  {rabatt_preis}
  ```

---

## Phase 7: PDF-Generierung
**Ziel:** Professionelle PDF-Dokumente

### 7.1 PDF-Layouts
**Neue Datei:** `src-tauri/src/pdf_generator.rs`

- [ ] **Buchungsbestätigung**
  ```rust
  Inhalt:
  - Header mit Logo
  - Gastdaten
  - Zimmerdaten
  - Check-in/Check-out Daten
  - Anzahl Gäste/Nächte
  - Preisaufschlüsselung
  - Begleitpersonen
  - Services
  - Rabatte
  - Gesamtpreis
  - Zahlungsinformationen
  - Footer mit Kontaktdaten
  ```

- [ ] **Rechnung**
  ```rust
  Inhalt:
  - Rechnungsnummer (generiert)
  - Rechnungsdatum
  - Alle Details aus Buchungsbestätigung
  - MwSt.-Berechnung (19% / 7%)
  - Netto/Brutto-Ausweisung
  - Zahlungskonditionen
  - Bankverbindung
  ```

### 7.2 PDF Frontend
**Komponente:** `src/components/PDF/`

- [ ] **PDF-Viewer** integrieren
  ```tsx
  Features:
  - PDF in Modal anzeigen
  - Download-Button
  - Print-Button
  - Per Email versenden
  ```

---

## Phase 8: Erweiterte Features
**Ziel:** Nice-to-have Features und UX-Verbesserungen

### 8.1 Benutzerrechte-System (Optional)
**Neue Datei:** `src-tauri/src/auth.rs`

- [ ] **User-Tabelle**
  ```sql
  - id: INTEGER PRIMARY KEY
  - username: TEXT UNIQUE
  - password_hash: TEXT
  - email: TEXT
  - role: TEXT (admin/manager/reception/readonly)
  - created_at: TEXT
  ```

- [ ] **Permissions-System**
  ```rust
  Rechte:
  - booking_create
  - booking_edit
  - booking_delete
  - booking_cancel
  - guest_manage
  - room_manage
  - config_manage
  - reports_view
  - reports_export
  ```

- [ ] **Login-Screen**
  ```tsx
  Features:
  - Login-Formular
  - Session Management
  - Password Reset
  ```

### 8.2 Dashboard-Erweiterungen

- [ ] **Dashboard-Widgets** hinzufügen
  ```tsx
  Widgets:
  - Check-ins heute
  - Check-outs heute
  - Neue Buchungen (letzte 7 Tage)
  - Umsatz aktueller Monat vs. Vormonat
  - Top 5 Zimmer nach Auslastung
  - Anstehende Buchungen (nächste 7 Tage)
  ```

- [ ] **Quick Actions** implementieren
  ```tsx
  Actions:
  - Neue Buchung (Floating Action Button)
  - Neuer Gast (Quick Add)
  - Check-in durchführen
  - Check-out durchführen
  ```

### 8.3 Benachrichtigungen & Reminders

- [ ] **Benachrichtigungs-System**
  ```rust
  Features:
  - Check-in Reminder (1 Tag vorher)
  - Check-out Reminder (am Tag)
  - Überfällige Zahlungen
  - Doppelbuchungen vermeiden
  ```

- [ ] **Automatische Reminder-Emails**
  ```rust
  - Scheduler für automatische Emails
  - X Tage vor Check-in Reminder senden
  - Nach Check-out Feedback-Email
  ```

### 8.4 Spezielle Features

- [ ] **Stammgäste-Rabatt**
  ```rust
  - Automatischer Rabatt nach X Buchungen
  - Treueprogramm-Integration
  ```

- [ ] **Saisonale Preise**
  ```rust
  Tabelle: seasonal_pricing
  - room_id
  - start_date
  - end_date
  - price_multiplier
  ```

- [ ] **Zimmer-Kategorien**
  ```rust
  - Kategorien definieren (Standard, Komfort, Superior)
  - Filter nach Kategorie
  - Preisgruppen
  ```

- [ ] **Blacklist für Gäste**
  ```rust
  Tabelle: guest_blacklist
  - guest_id
  - reason
  - blocked_date
  - notes
  ```

---

## Phase 9: Testing & Optimierung
**Ziel:** Stabile, performante Anwendung

### 9.1 Testing

- [ ] **Unit Tests** für Rust Backend
  ```rust
  - Validierungsfunktionen testen
  - Preisberechnungen testen
  - Datenbankoperationen testen
  ```

- [ ] **Integration Tests** für Frontend
  ```tsx
  - Komponenten-Tests mit React Testing Library
  - E2E-Tests mit Playwright
  ```

### 9.2 Performance-Optimierung

- [ ] **Backend**
  ```rust
  - Datenbank-Indizes optimieren
  - Query-Performance messen
  - Caching implementieren (falls nötig)
  ```

- [ ] **Frontend**
  ```tsx
  - React.memo für teure Komponenten
  - Virtualisierung für lange Listen
  - Lazy Loading für Routes
  - Image Optimization
  ```

### 9.3 Error Handling

- [ ] **Fehlerbehandlung verbessern**
  ```rust/tsx
  - Aussagekräftige Fehlermeldungen
  - Error Boundaries in React
  - Toast Notifications für Fehler
  - Logging-System
  ```

---

## Phase 10: Deployment & Dokumentation
**Ziel:** Produktionsreife Anwendung

### 10.1 Build & Deployment

- [ ] **Build-Prozess optimieren**
  ```bash
  - Production Build testen
  - Bundle-Size optimieren
  - App-Icons & Splash Screens
  ```

- [ ] **Installer erstellen**
  ```bash
  - Windows Installer (.exe)
  - macOS Installer (.dmg)
  - Linux Package (.deb/.AppImage)
  ```

### 10.2 Dokumentation

- [ ] **Benutzer-Dokumentation**
  ```markdown
  - Installationsanleitung
  - Benutzerhandbuch
  - FAQ
  - Troubleshooting
  ```

- [ ] **Entwickler-Dokumentation**
  ```markdown
  - Code-Struktur
  - API-Dokumentation
  - Setup-Anleitung
  - Contribution Guide
  ```

---

## Prioritäten-Übersicht

### 🔴 Kritisch (Phase 1-3)
Ohne diese Features ist die App nicht verwendbar
- Datenbank-Erweiterung
- CRUD-Operationen für Buchungen/Gäste/Zimmer
- Validierungen & Preisberechnung
- UI-Komponenten für Verwaltung

### 🟡 Wichtig (Phase 4-6)
Diese Features machen die App produktionsreif
- Such- und Filterfunktionen
- Reports und Statistiken
- Email-System

### 🟢 Nice-to-have (Phase 7-8)
Diese Features verbessern die UX erheblich
- PDF-Generierung
- Benutzerrechte
- Erweiterte Dashboard-Features

### ⚪ Optional (Phase 9-10)
Diese Features sind für Stabilität und Wartbarkeit wichtig
- Testing
- Performance-Optimierung
- Deployment & Dokumentation

---

## Technologie-Stack Zusammenfassung

| Bereich | Technologie |
|---------|-------------|
| Frontend Framework | React 18 + TypeScript |
| UI Library | TailwindCSS 4 |
| Drag & Drop | dnd-kit |
| Backend Framework | Tauri 2 (Rust) |
| Datenbank | SQLite (via Rusqlite) |
| Email | Rust `lettre` crate |
| PDF | Rust `printpdf` oder `genpdf` |
| Charts | Recharts oder Chart.js |
| Date Picker | react-datepicker |
| Forms | React Hook Form |
| State Management | React Context / Zustand (bei Bedarf) |

---

## Nächste Schritte

1. **Start mit Phase 1.1:** Datenbank-Schema erweitern
2. **Parallel dazu Phase 1.2:** Rust Backend Commands implementieren
3. **Dann Phase 2:** Validierungen und Business Logic
4. **Anschließend Phase 3:** UI-Komponenten bauen

**Geschätzte Gesamtdauer:** 4-6 Wochen bei Vollzeit-Entwicklung

---

## Notizen & Entscheidungen

- **Design:** Modernes, dunkles Design beibehalten (Slate-Farben)
- **Icons:** Lucide-React verwenden (bereits im Projekt)
- **Responsive:** Mobile-First Ansatz (auch wenn Desktop-App)
- **Sprache:** UI komplett auf Deutsch
- **Datumsformat:** DD.MM.YYYY (deutsches Format)
- **Währung:** EUR mit deutschem Format (1.234,56 €)

---

**Stand:** {{ heute }}
**Version:** 1.0
**Status:** 📋 Bereit für Implementierung