-- ========================================================================
-- DATABASE CLEANUP PHASE 1: NAMING CONVENTIONS & VIEWS
-- ========================================================================
-- Status: IN PROGRESS
-- Datum: 2024-11-14
--
-- WICHTIG: Diese SQL-Befehle werden schrittweise ausgeführt und getestet!
-- ========================================================================

-- ========================================================================
-- PHASE 1.1: ANALYSE DER FELDNAMEN
-- ========================================================================

-- MAPPING TABELLE: Alt -> Neu (Dokumentation)
/*
ROOMS:
- gebaeude_typ -> building_type
- nebensaison_preis -> off_season_price
- hauptsaison_preis -> main_season_price
- endreinigung -> cleaning_fee
- ort -> location
- schluesselcode -> key_code
- notizen -> notes

GUESTS:
- vorname -> first_name
- nachname -> last_name
- telefon -> phone
- dpolg_mitglied -> is_dpolg_member
- strasse -> street
- plz -> postal_code
- ort -> city
- mitgliedsnummer -> member_number
- notizen -> notes
- beruf -> occupation
- bundesland -> state
- dienststelle -> department
- telefon_geschaeftlich -> phone_business
- telefon_privat -> phone_private
- telefon_mobil -> phone_mobile
- geburtsdatum -> date_of_birth
- geburtsort -> place_of_birth
- sprache -> language
- nationalitaet -> nationality
- identifikationsnummer -> identification_number
- debitorenkonto -> debtor_account
- kennzeichen -> license_plate
- rechnungs_email -> invoice_email
- marketing_einwilligung -> marketing_consent
- leitweg_id -> routing_id
- kostenstelle -> cost_center
- automail_sprache -> automail_language

BOOKINGS:
- reservierungsnummer -> reservation_number
- anzahl_gaeste -> guest_count
- gesamtpreis -> total_price
- bemerkungen -> remarks
- anzahl_begleitpersonen -> companion_count
- grundpreis -> base_price
- services_preis -> services_price
- rabatt_preis -> discount_amount
- anzahl_naechte -> nights_count
- bezahlt -> is_paid
- bezahlt_am -> paid_at
- zahlungsmethode -> payment_method
- mahnung_gesendet_am -> reminder_sent_at
- rechnung_versendet_am -> invoice_sent_at
- rechnung_versendet_an -> invoice_sent_to
- ist_stiftungsfall -> is_foundation_case
- ist_dpolg_mitglied -> is_dpolg_member

ACCOMPANYING_GUESTS:
- vorname -> first_name
- nachname -> last_name
- geburtsdatum -> date_of_birth

GUEST_COMPANIONS:
- vorname -> first_name
- nachname -> last_name
- geburtsdatum -> date_of_birth
- beziehung -> relationship
- notizen -> notes
*/

-- ========================================================================
-- PHASE 1.2: VIEW LAYER MIT KONSISTENTEN NAMEN
-- ========================================================================

-- VIEW 1: v_rooms - Konsistente englische Feldnamen für Räume
DROP VIEW IF EXISTS v_rooms;
CREATE VIEW v_rooms AS
SELECT
    id,
    name,
    gebaeude_typ as building_type,
    capacity,
    price_member,
    price_non_member,
    nebensaison_preis as off_season_price,
    hauptsaison_preis as main_season_price,
    endreinigung as cleaning_fee,
    ort as location,
    schluesselcode as key_code,
    street_address,
    postal_code,
    city,
    notizen as notes
FROM rooms;

-- VIEW 2: v_guests - Konsistente englische Feldnamen für Gäste
DROP VIEW IF EXISTS v_guests;
CREATE VIEW v_guests AS
SELECT
    id,
    vorname as first_name,
    nachname as last_name,
    email,
    telefon as phone,
    dpolg_mitglied as is_dpolg_member,
    strasse as street,
    plz as postal_code,
    ort as city,
    mitgliedsnummer as member_number,
    notizen as notes,
    beruf as occupation,
    bundesland as state,
    dienststelle as department,
    created_at,
    -- Erweiterte Felder
    anrede as salutation,
    geschlecht as gender,
    land as country,
    telefon_geschaeftlich as phone_business,
    telefon_privat as phone_private,
    telefon_mobil as phone_mobile,
    fax,
    geburtsdatum as date_of_birth,
    geburtsort as place_of_birth,
    sprache as language,
    nationalitaet as nationality,
    identifikationsnummer as identification_number,
    debitorenkonto as debtor_account,
    kennzeichen as license_plate,
    rechnungs_email as invoice_email,
    marketing_einwilligung as marketing_consent,
    leitweg_id as routing_id,
    kostenstelle as cost_center,
    tags,
    automail,
    automail_sprache as automail_language
FROM guests;

-- VIEW 3: v_bookings - Konsistente englische Feldnamen für Buchungen
DROP VIEW IF EXISTS v_bookings;
CREATE VIEW v_bookings AS
SELECT
    id,
    room_id,
    guest_id,
    reservierungsnummer as reservation_number,
    checkin_date,
    checkout_date,
    anzahl_gaeste as guest_count,
    status,
    gesamtpreis as total_price,
    bemerkungen as remarks,
    created_at,
    anzahl_begleitpersonen as companion_count,
    grundpreis as base_price,
    services_preis as services_price,
    rabatt_preis as discount_amount,
    anzahl_naechte as nights_count,
    updated_at,
    bezahlt as is_paid,
    bezahlt_am as paid_at,
    zahlungsmethode as payment_method,
    mahnung_gesendet_am as reminder_sent_at,
    rechnung_versendet_am as invoice_sent_at,
    rechnung_versendet_an as invoice_sent_to,
    ist_stiftungsfall as is_foundation_case,
    payment_recipient_id,
    putzplan_checkout_date as cleaning_checkout_date,
    ist_dpolg_mitglied as is_dpolg_member
FROM bookings;

-- VIEW 4: v_payment_recipients - Bereits englisch, aber für Konsistenz
DROP VIEW IF EXISTS v_payment_recipients;
CREATE VIEW v_payment_recipients AS
SELECT
    id,
    name,
    company,
    street,
    plz as postal_code,
    city,
    country,
    contact_person,
    notes,
    created_at,
    updated_at
FROM payment_recipients;

-- VIEW 5: v_accompanying_guests - Konsistente Namen für Begleitpersonen
DROP VIEW IF EXISTS v_accompanying_guests;
CREATE VIEW v_accompanying_guests AS
SELECT
    id,
    booking_id,
    vorname as first_name,
    nachname as last_name,
    geburtsdatum as date_of_birth,
    companion_id
FROM accompanying_guests;

-- VIEW 6: v_guest_companions - Konsistente Namen für Gast-Begleiter
DROP VIEW IF EXISTS v_guest_companions;
CREATE VIEW v_guest_companions AS
SELECT
    id,
    guest_id,
    vorname as first_name,
    nachname as last_name,
    geburtsdatum as date_of_birth,
    beziehung as relationship,
    notizen as notes,
    created_at
FROM guest_companions;

-- VIEW 7: v_reminders - Bereits englisch
DROP VIEW IF EXISTS v_reminders;
CREATE VIEW v_reminders AS
SELECT * FROM reminders;

-- VIEW 8: v_services - Services mit konsistenten Namen
DROP VIEW IF EXISTS v_services;
CREATE VIEW v_services AS
SELECT
    id,
    booking_id,
    service_name,
    service_price,
    created_at,
    template_id,
    price_type,
    original_value,
    applies_to
FROM additional_services;

-- VIEW 9: v_discounts - Bereits englisch
DROP VIEW IF EXISTS v_discounts;
CREATE VIEW v_discounts AS
SELECT * FROM discounts;

-- VIEW 10: v_service_templates - Bereits englisch
DROP VIEW IF EXISTS v_service_templates;
CREATE VIEW v_service_templates AS
SELECT * FROM service_templates;

-- VIEW 11: v_discount_templates - Bereits englisch
DROP VIEW IF EXISTS v_discount_templates;
CREATE VIEW v_discount_templates AS
SELECT * FROM discount_templates;

-- ========================================================================
-- TEST QUERIES FÜR VIEWS
-- ========================================================================

-- Test 1: Simple SELECT
-- SELECT * FROM v_rooms LIMIT 5;
-- SELECT * FROM v_guests LIMIT 5;
-- SELECT * FROM v_bookings LIMIT 5;

-- Test 2: JOIN über Views
-- SELECT
--     b.reservation_number,
--     g.first_name || ' ' || g.last_name as guest_name,
--     r.name as room_name,
--     b.checkin_date,
--     b.checkout_date,
--     b.total_price
-- FROM v_bookings b
-- JOIN v_guests g ON b.guest_id = g.id
-- JOIN v_rooms r ON b.room_id = r.id
-- LIMIT 10;

-- Test 3: Performance-Vergleich
-- EXPLAIN QUERY PLAN
-- SELECT * FROM bookings WHERE checkin_date >= '2024-01-01';

-- EXPLAIN QUERY PLAN
-- SELECT * FROM v_bookings WHERE checkin_date >= '2024-01-01';

-- ========================================================================
-- PHASE 2: PERFORMANCE INDIZES
-- ========================================================================

-- Index 1: Booking Date Range (häufigste Query)
CREATE INDEX IF NOT EXISTS idx_bookings_dates
ON bookings(checkin_date, checkout_date);

-- Index 2: Booking Room + Guest Lookup
CREATE INDEX IF NOT EXISTS idx_bookings_room_guest
ON bookings(room_id, guest_id);

-- Index 3: Booking Status (für Filter)
CREATE INDEX IF NOT EXISTS idx_bookings_status
ON bookings(status)
WHERE status != 'storniert';

-- Index 4: Payment Status
CREATE INDEX IF NOT EXISTS idx_bookings_payment
ON bookings(bezahlt)
WHERE bezahlt = 0;

-- Index 5: Guest Name Search
CREATE INDEX IF NOT EXISTS idx_guests_name
ON guests(nachname, vorname);

-- Index 6: Guest Email (unique lookups)
CREATE INDEX IF NOT EXISTS idx_guests_email
ON guests(email);

-- Index 7: DPolG Members
CREATE INDEX IF NOT EXISTS idx_guests_member
ON guests(dpolg_mitglied)
WHERE dpolg_mitglied = 1;

-- Index 8: Room Building Type
CREATE INDEX IF NOT EXISTS idx_rooms_type
ON rooms(gebaeude_typ);

-- Index 9: Room Location
CREATE INDEX IF NOT EXISTS idx_rooms_location
ON rooms(ort);

-- Index 10: Reminders Due Date
CREATE INDEX IF NOT EXISTS idx_reminders_due
ON reminders(due_date, is_completed)
WHERE is_completed = 0;

-- Index 11: Reminders by Booking
CREATE INDEX IF NOT EXISTS idx_reminders_booking
ON reminders(booking_id);

-- Index 12: Email Logs by Booking
CREATE INDEX IF NOT EXISTS idx_email_logs_booking
ON email_logs(booking_id);

-- Index 13: Email Logs by Date
CREATE INDEX IF NOT EXISTS idx_email_logs_sent
ON email_logs(sent_at);

-- Index 14: Accompanying Guests by Booking
CREATE INDEX IF NOT EXISTS idx_accompanying_booking
ON accompanying_guests(booking_id);

-- Index 15: Guest Companions by Guest
CREATE INDEX IF NOT EXISTS idx_companions_guest
ON guest_companions(guest_id);

-- Index 16: Services by Booking
CREATE INDEX IF NOT EXISTS idx_services_booking
ON additional_services(booking_id);

-- Index 17: Discounts by Booking
CREATE INDEX IF NOT EXISTS idx_discounts_booking
ON discounts(booking_id);

-- ========================================================================
-- PERFORMANCE TEST QUERIES
-- ========================================================================

-- Vorher (ohne Index):
-- EXPLAIN QUERY PLAN
-- SELECT * FROM bookings
-- WHERE checkin_date >= '2024-01-01'
-- AND checkout_date <= '2024-12-31';

-- Nachher (mit Index):
-- EXPLAIN QUERY PLAN
-- SELECT * FROM bookings
-- WHERE checkin_date >= '2024-01-01'
-- AND checkout_date <= '2024-12-31';

-- ========================================================================
-- VACUUM UND ANALYZE
-- ========================================================================

-- Nach Index-Erstellung optimieren
VACUUM;
ANALYZE;