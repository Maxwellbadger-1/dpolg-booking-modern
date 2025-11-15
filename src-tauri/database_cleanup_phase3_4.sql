-- ========================================================================
-- DATABASE CLEANUP PHASE 3 & 4: DATENVALIDIERUNG & NORMALISIERUNG
-- ========================================================================
-- Status: IN PROGRESS
-- Datum: 2024-11-14
--
-- WICHTIG: SQLite unterstützt keine ALTER TABLE ADD CONSTRAINT
-- Lösung: Trigger für Validierung + Neue Tabellen für Normalisierung
-- ========================================================================

-- ========================================================================
-- PHASE 3: DATENVALIDIERUNG MIT TRIGGERN
-- ========================================================================

-- Trigger 1: Validierung Booking Dates (checkout muss nach checkin sein)
DROP TRIGGER IF EXISTS validate_booking_dates_insert;
CREATE TRIGGER validate_booking_dates_insert
BEFORE INSERT ON bookings
FOR EACH ROW
WHEN NEW.checkout_date <= NEW.checkin_date
BEGIN
  SELECT RAISE(ABORT, 'Checkout-Datum muss nach Check-in-Datum liegen');
END;

DROP TRIGGER IF EXISTS validate_booking_dates_update;
CREATE TRIGGER validate_booking_dates_update
BEFORE UPDATE ON bookings
FOR EACH ROW
WHEN NEW.checkout_date <= NEW.checkin_date
BEGIN
  SELECT RAISE(ABORT, 'Checkout-Datum muss nach Check-in-Datum liegen');
END;

-- Trigger 2: Validierung Positive Preise
DROP TRIGGER IF EXISTS validate_booking_price_insert;
CREATE TRIGGER validate_booking_price_insert
BEFORE INSERT ON bookings
FOR EACH ROW
WHEN NEW.gesamtpreis < 0 OR NEW.grundpreis < 0
BEGIN
  SELECT RAISE(ABORT, 'Preise müssen positiv sein');
END;

DROP TRIGGER IF EXISTS validate_booking_price_update;
CREATE TRIGGER validate_booking_price_update
BEFORE UPDATE ON bookings
FOR EACH ROW
WHEN NEW.gesamtpreis < 0 OR NEW.grundpreis < 0
BEGIN
  SELECT RAISE(ABORT, 'Preise müssen positiv sein');
END;

-- Trigger 3: Validierung Guest Count > 0
DROP TRIGGER IF EXISTS validate_guest_count_insert;
CREATE TRIGGER validate_guest_count_insert
BEFORE INSERT ON bookings
FOR EACH ROW
WHEN NEW.anzahl_gaeste <= 0
BEGIN
  SELECT RAISE(ABORT, 'Anzahl Gäste muss mindestens 1 sein');
END;

DROP TRIGGER IF EXISTS validate_guest_count_update;
CREATE TRIGGER validate_guest_count_update
BEFORE UPDATE ON bookings
FOR EACH ROW
WHEN NEW.anzahl_gaeste <= 0
BEGIN
  SELECT RAISE(ABORT, 'Anzahl Gäste muss mindestens 1 sein');
END;

-- Trigger 4: Validierung Room Capacity
DROP TRIGGER IF EXISTS validate_room_capacity_insert;
CREATE TRIGGER validate_room_capacity_insert
BEFORE INSERT ON rooms
FOR EACH ROW
WHEN NEW.capacity <= 0
BEGIN
  SELECT RAISE(ABORT, 'Zimmerkapazität muss mindestens 1 sein');
END;

DROP TRIGGER IF EXISTS validate_room_capacity_update;
CREATE TRIGGER validate_room_capacity_update
BEFORE UPDATE ON rooms
FOR EACH ROW
WHEN NEW.capacity <= 0
BEGIN
  SELECT RAISE(ABORT, 'Zimmerkapazität muss mindestens 1 sein');
END;

-- Trigger 5: Email Format Validierung (Basic)
DROP TRIGGER IF EXISTS validate_guest_email_insert;
CREATE TRIGGER validate_guest_email_insert
BEFORE INSERT ON guests
FOR EACH ROW
WHEN NEW.email NOT LIKE '%@%.%'
BEGIN
  SELECT RAISE(ABORT, 'Ungültiges E-Mail-Format');
END;

DROP TRIGGER IF EXISTS validate_guest_email_update;
CREATE TRIGGER validate_guest_email_update
BEFORE UPDATE ON guests
FOR EACH ROW
WHEN NEW.email NOT LIKE '%@%.%'
BEGIN
  SELECT RAISE(ABORT, 'Ungültiges E-Mail-Format');
END;

-- Trigger 6: Status Validierung für Bookings
DROP TRIGGER IF EXISTS validate_booking_status_insert;
CREATE TRIGGER validate_booking_status_insert
BEFORE INSERT ON bookings
FOR EACH ROW
WHEN NEW.status NOT IN ('bestätigt', 'vorläufig', 'storniert', 'angefragt')
BEGIN
  SELECT RAISE(ABORT, 'Ungültiger Buchungsstatus. Erlaubt: bestätigt, vorläufig, storniert, angefragt');
END;

DROP TRIGGER IF EXISTS validate_booking_status_update;
CREATE TRIGGER validate_booking_status_update
BEFORE UPDATE ON bookings
FOR EACH ROW
WHEN NEW.status NOT IN ('bestätigt', 'vorläufig', 'storniert', 'angefragt')
BEGIN
  SELECT RAISE(ABORT, 'Ungültiger Buchungsstatus. Erlaubt: bestätigt, vorläufig, storniert, angefragt');
END;

-- ========================================================================
-- TEST VALIDATION TRIGGERS
-- ========================================================================

-- Diese Tests sollten FEHLER werfen:
-- INSERT INTO bookings (room_id, guest_id, reservierungsnummer, checkin_date, checkout_date, anzahl_gaeste, status, gesamtpreis)
-- VALUES (1, 1, 'TEST-001', '2024-01-15', '2024-01-10', 2, 'bestätigt', 100);
-- -- Fehler: Checkout vor Checkin

-- INSERT INTO bookings (room_id, guest_id, reservierungsnummer, checkin_date, checkout_date, anzahl_gaeste, status, gesamtpreis)
-- VALUES (1, 1, 'TEST-002', '2024-01-10', '2024-01-15', 0, 'bestätigt', 100);
-- -- Fehler: Gäste = 0

-- INSERT INTO bookings (room_id, guest_id, reservierungsnummer, checkin_date, checkout_date, anzahl_gaeste, status, gesamtpreis)
-- VALUES (1, 1, 'TEST-003', '2024-01-10', '2024-01-15', 2, 'bestätigt', -50);
-- -- Fehler: Negativer Preis

-- ========================================================================
-- PHASE 4: NORMALISIERUNG - NEUE TABELLEN
-- ========================================================================

-- Tabelle 1: Seasons (Saisonzeiten)
CREATE TABLE IF NOT EXISTS seasons (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    start_month INTEGER NOT NULL CHECK(start_month >= 1 AND start_month <= 12),
    start_day INTEGER NOT NULL CHECK(start_day >= 1 AND start_day <= 31),
    end_month INTEGER NOT NULL CHECK(end_month >= 1 AND end_month <= 12),
    end_day INTEGER NOT NULL CHECK(end_day >= 1 AND end_day <= 31),
    price_multiplier REAL NOT NULL DEFAULT 1.0 CHECK(price_multiplier > 0),
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP
);

-- Standard-Saisons einfügen
INSERT OR IGNORE INTO seasons (name, display_name, start_month, start_day, end_month, end_day, price_multiplier, sort_order)
VALUES
    ('nebensaison', 'Nebensaison', 11, 1, 3, 31, 0.9, 1),
    ('hauptsaison', 'Hauptsaison', 4, 1, 10, 31, 1.0, 2),
    ('hochsaison', 'Hochsaison', 7, 1, 8, 31, 1.2, 3);

-- Tabelle 2: Room Prices (Zimmerpreise pro Saison)
CREATE TABLE IF NOT EXISTS room_prices (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    room_id INTEGER NOT NULL,
    season_id INTEGER NOT NULL,
    price_per_night REAL NOT NULL CHECK(price_per_night >= 0),
    price_member REAL CHECK(price_member >= 0),
    price_non_member REAL CHECK(price_non_member >= 0),
    valid_from TEXT NOT NULL DEFAULT (date('now')),
    valid_until TEXT,
    is_active INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (room_id) REFERENCES rooms(id) ON DELETE CASCADE,
    FOREIGN KEY (season_id) REFERENCES seasons(id),
    UNIQUE(room_id, season_id, valid_from)
);

-- Migration: Aktuelle Preise aus rooms Tabelle in room_prices überführen
-- Nebensaison Preise
INSERT OR IGNORE INTO room_prices (room_id, season_id, price_per_night, price_member, price_non_member)
SELECT
    r.id,
    (SELECT id FROM seasons WHERE name = 'nebensaison'),
    r.nebensaison_preis,
    r.price_member * 0.9,  -- 10% Rabatt in Nebensaison
    r.price_non_member * 0.9
FROM rooms r
WHERE r.nebensaison_preis > 0;

-- Hauptsaison Preise
INSERT OR IGNORE INTO room_prices (room_id, season_id, price_per_night, price_member, price_non_member)
SELECT
    r.id,
    (SELECT id FROM seasons WHERE name = 'hauptsaison'),
    r.hauptsaison_preis,
    r.price_member,
    r.price_non_member
FROM rooms r
WHERE r.hauptsaison_preis > 0;

-- Tabelle 3: Price History (Preis-Historie für Nachvollziehbarkeit)
CREATE TABLE IF NOT EXISTS price_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    room_id INTEGER NOT NULL,
    season_id INTEGER,
    old_price REAL,
    new_price REAL NOT NULL,
    changed_by TEXT,
    change_reason TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (room_id) REFERENCES rooms(id) ON DELETE CASCADE,
    FOREIGN KEY (season_id) REFERENCES seasons(id)
);

-- Trigger für automatische Price History
DROP TRIGGER IF EXISTS track_price_changes;
CREATE TRIGGER track_price_changes
AFTER UPDATE ON room_prices
FOR EACH ROW
WHEN OLD.price_per_night != NEW.price_per_night
BEGIN
    INSERT INTO price_history (room_id, season_id, old_price, new_price, change_reason)
    VALUES (NEW.room_id, NEW.season_id, OLD.price_per_night, NEW.price_per_night, 'Price Update');
END;

-- ========================================================================
-- PHASE 5: AUDIT TRAIL
-- ========================================================================

-- Audit Log Tabelle
CREATE TABLE IF NOT EXISTS audit_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    table_name TEXT NOT NULL,
    record_id INTEGER NOT NULL,
    action TEXT NOT NULL CHECK(action IN ('INSERT', 'UPDATE', 'DELETE')),
    old_values TEXT,  -- JSON Format
    new_values TEXT,  -- JSON Format
    user_id INTEGER,
    user_name TEXT,
    ip_address TEXT,
    user_agent TEXT,
    timestamp TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    session_id TEXT
);

-- Index für Audit Log Performance
CREATE INDEX IF NOT EXISTS idx_audit_log_table ON audit_log(table_name, record_id);
CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON audit_log(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_log_user ON audit_log(user_id);

-- Trigger 1: Audit für Booking INSERT
DROP TRIGGER IF EXISTS audit_booking_insert;
CREATE TRIGGER audit_booking_insert
AFTER INSERT ON bookings
FOR EACH ROW
BEGIN
    INSERT INTO audit_log (table_name, record_id, action, new_values)
    VALUES (
        'bookings',
        NEW.id,
        'INSERT',
        json_object(
            'room_id', NEW.room_id,
            'guest_id', NEW.guest_id,
            'reservierungsnummer', NEW.reservierungsnummer,
            'checkin_date', NEW.checkin_date,
            'checkout_date', NEW.checkout_date,
            'anzahl_gaeste', NEW.anzahl_gaeste,
            'status', NEW.status,
            'gesamtpreis', NEW.gesamtpreis
        )
    );
END;

-- Trigger 2: Audit für Booking UPDATE
DROP TRIGGER IF EXISTS audit_booking_update;
CREATE TRIGGER audit_booking_update
AFTER UPDATE ON bookings
FOR EACH ROW
WHEN OLD.status != NEW.status OR OLD.gesamtpreis != NEW.gesamtpreis OR OLD.bezahlt != NEW.bezahlt
BEGIN
    INSERT INTO audit_log (table_name, record_id, action, old_values, new_values)
    VALUES (
        'bookings',
        NEW.id,
        'UPDATE',
        json_object(
            'status', OLD.status,
            'gesamtpreis', OLD.gesamtpreis,
            'bezahlt', OLD.bezahlt,
            'checkin_date', OLD.checkin_date,
            'checkout_date', OLD.checkout_date
        ),
        json_object(
            'status', NEW.status,
            'gesamtpreis', NEW.gesamtpreis,
            'bezahlt', NEW.bezahlt,
            'checkin_date', NEW.checkin_date,
            'checkout_date', NEW.checkout_date
        )
    );
END;

-- Trigger 3: Audit für Booking DELETE
DROP TRIGGER IF EXISTS audit_booking_delete;
CREATE TRIGGER audit_booking_delete
AFTER DELETE ON bookings
FOR EACH ROW
BEGIN
    INSERT INTO audit_log (table_name, record_id, action, old_values)
    VALUES (
        'bookings',
        OLD.id,
        'DELETE',
        json_object(
            'room_id', OLD.room_id,
            'guest_id', OLD.guest_id,
            'reservierungsnummer', OLD.reservierungsnummer,
            'checkin_date', OLD.checkin_date,
            'checkout_date', OLD.checkout_date,
            'status', OLD.status,
            'gesamtpreis', OLD.gesamtpreis
        )
    );
END;

-- Trigger 4: Audit für Guest UPDATE
DROP TRIGGER IF EXISTS audit_guest_update;
CREATE TRIGGER audit_guest_update
AFTER UPDATE ON guests
FOR EACH ROW
WHEN OLD.email != NEW.email OR OLD.telefon != NEW.telefon
BEGIN
    INSERT INTO audit_log (table_name, record_id, action, old_values, new_values)
    VALUES (
        'guests',
        NEW.id,
        'UPDATE',
        json_object(
            'vorname', OLD.vorname,
            'nachname', OLD.nachname,
            'email', OLD.email,
            'telefon', OLD.telefon
        ),
        json_object(
            'vorname', NEW.vorname,
            'nachname', NEW.nachname,
            'email', NEW.email,
            'telefon', NEW.telefon
        )
    );
END;

-- Trigger 5: Audit für Payment Updates
DROP TRIGGER IF EXISTS audit_payment_update;
CREATE TRIGGER audit_payment_update
AFTER UPDATE ON bookings
FOR EACH ROW
WHEN OLD.bezahlt != NEW.bezahlt
BEGIN
    INSERT INTO audit_log (table_name, record_id, action, old_values, new_values)
    VALUES (
        'bookings',
        NEW.id,
        'PAYMENT_UPDATE',
        json_object(
            'bezahlt', OLD.bezahlt,
            'bezahlt_am', OLD.bezahlt_am,
            'zahlungsmethode', OLD.zahlungsmethode
        ),
        json_object(
            'bezahlt', NEW.bezahlt,
            'bezahlt_am', NEW.bezahlt_am,
            'zahlungsmethode', NEW.zahlungsmethode
        )
    );
END;

-- ========================================================================
-- HILFS-VIEWS FÜR AUDIT TRAIL
-- ========================================================================

-- View für lesbare Audit-Einträge
DROP VIEW IF EXISTS v_audit_log_readable;
CREATE VIEW v_audit_log_readable AS
SELECT
    al.id,
    al.table_name,
    al.record_id,
    al.action,
    al.old_values,
    al.new_values,
    al.user_name,
    al.timestamp,
    CASE
        WHEN al.table_name = 'bookings' THEN
            (SELECT reservierungsnummer FROM bookings WHERE id = al.record_id)
        WHEN al.table_name = 'guests' THEN
            (SELECT nachname || ', ' || vorname FROM guests WHERE id = al.record_id)
        WHEN al.table_name = 'rooms' THEN
            (SELECT name FROM rooms WHERE id = al.record_id)
        ELSE NULL
    END as record_identifier
FROM audit_log al
ORDER BY al.timestamp DESC;

-- View für Payment Audit Trail
DROP VIEW IF EXISTS v_payment_audit;
CREATE VIEW v_payment_audit AS
SELECT
    al.id,
    al.record_id as booking_id,
    b.reservierungsnummer,
    al.old_values,
    al.new_values,
    al.user_name,
    al.timestamp
FROM audit_log al
JOIN bookings b ON al.record_id = b.id
WHERE al.action = 'PAYMENT_UPDATE'
ORDER BY al.timestamp DESC;

-- ========================================================================
-- STATISTIKEN FÜR AUDIT LOG
-- ========================================================================

-- View für Audit Statistiken
DROP VIEW IF EXISTS v_audit_statistics;
CREATE VIEW v_audit_statistics AS
SELECT
    table_name,
    action,
    COUNT(*) as count,
    DATE(MIN(timestamp)) as first_action,
    DATE(MAX(timestamp)) as last_action
FROM audit_log
GROUP BY table_name, action
ORDER BY table_name, action;

-- ========================================================================
-- CLEANUP & OPTIMIZATION
-- ========================================================================

-- Nach allen Änderungen
VACUUM;
ANALYZE;