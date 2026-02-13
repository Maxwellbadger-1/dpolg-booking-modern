-- Migration 016: Auto-create reminders for bookings based on notification_settings
-- This implements the core logic for automatic reminder creation
--
-- Part 1: Add configurable payment reminder days setting
-- Part 2: Create auto-creation trigger function
-- Part 3: Create auto-completion trigger function

-- =============================================================================
-- PART 1: Add Setting for Payment Reminder Days
-- =============================================================================

-- Add configurable payment reminder days (default 7)
ALTER TABLE notification_settings
ADD COLUMN IF NOT EXISTS payment_reminder_before_days INTEGER DEFAULT 7;

COMMENT ON COLUMN notification_settings.payment_reminder_before_days IS 'Days before check-in to create payment reminder (if unpaid)';

-- Update existing row
UPDATE notification_settings
SET payment_reminder_before_days = COALESCE(payment_reminder_before_days, 7)
WHERE id = 1;

-- =============================================================================
-- PART 2: Auto-Creation Logic
-- =============================================================================

-- =============================================================================
-- FUNCTION: auto_create_reminders_for_booking()
-- =============================================================================
-- Purpose: Automatically creates reminders based on booking state and settings
-- Trigger: AFTER INSERT OR UPDATE ON bookings
-- Logic:
--   1. Check if auto-reminder flags are enabled in notification_settings
--   2. Check booking/guest state to determine which reminders to create
--   3. Use duplicate prevention to avoid creating same reminder twice
-- =============================================================================

CREATE OR REPLACE FUNCTION auto_create_reminders_for_booking()
RETURNS TRIGGER AS $$
DECLARE
    v_settings RECORD;
    v_guest RECORD;
    v_guest_incomplete BOOLEAN;
    v_payment_due_date DATE;
    v_checkin_due_date DATE;
    v_existing_reminder_count INTEGER;
BEGIN
    -- Skip cancelled/storniert bookings
    IF NEW.status IN ('cancelled', 'storniert') THEN
        RETURN NEW;
    END IF;

    -- Get notification settings (singleton table)
    SELECT * INTO v_settings FROM notification_settings WHERE id = 1;

    -- If no settings found, skip (should never happen)
    IF v_settings IS NULL THEN
        RETURN NEW;
    END IF;

    -- Get guest data for incomplete data check
    SELECT * INTO v_guest FROM guests WHERE id = NEW.guest_id;

    IF v_guest IS NULL THEN
        RETURN NEW;
    END IF;

    -- =============================================================================
    -- 1. AUTO-REMINDER: Incomplete Guest Data
    -- =============================================================================
    -- Purpose: Notify when guest is missing critical information
    -- Trigger: When email, phone, or address is missing
    -- Priority: HIGH (blocks other operations)
    -- =============================================================================

    IF v_settings.auto_reminder_incomplete_data IS TRUE THEN
        -- Check if guest data is incomplete
        v_guest_incomplete := (
            v_guest.email IS NULL OR v_guest.email = '' OR
            v_guest.telefon IS NULL OR v_guest.telefon = '' OR
            v_guest.strasse IS NULL OR v_guest.strasse = '' OR
            v_guest.plz IS NULL OR v_guest.plz = '' OR
            v_guest.ort IS NULL OR v_guest.ort = ''
        );

        IF v_guest_incomplete THEN
            -- Check if reminder already exists (avoid duplicates)
            SELECT COUNT(*) INTO v_existing_reminder_count
            FROM reminders
            WHERE booking_id = NEW.id
              AND reminder_type = 'auto_incomplete_data'
              AND is_completed = false;

            -- Only create if no active reminder exists
            IF v_existing_reminder_count = 0 THEN
                INSERT INTO reminders (
                    booking_id, reminder_type, title, description, due_date, priority,
                    is_completed, is_snoozed, created_at, updated_at
                )
                VALUES (
                    NEW.id,
                    'auto_incomplete_data',
                    format('Unvollständige Gästedaten - Buchung #%s', NEW.id),
                    format('Gast: %s %s - Fehlende Daten: Email, Telefon oder Adresse vervollständigen', v_guest.vorname, v_guest.nachname),
                    CURRENT_DATE,  -- Due immediately
                    'high',
                    false,
                    false,
                    CURRENT_TIMESTAMP,
                    CURRENT_TIMESTAMP
                );
            END IF;
        END IF;
    END IF;

    -- =============================================================================
    -- 2. AUTO-REMINDER: Payment Overdue
    -- =============================================================================
    -- Purpose: Remind to follow up on unpaid bookings before check-in
    -- Trigger: When booking is unpaid (bezahlt = false or NULL)
    -- Due Date: X days before check-in (configurable via payment_reminder_before_days)
    -- Priority: HIGH (payment critical)
    -- Skip: If payment reminder email already sent (mahnung_gesendet_am)
    -- =============================================================================

    IF v_settings.auto_reminder_payment IS TRUE THEN
        -- Only create if booking is unpaid
        IF (NEW.bezahlt IS FALSE OR NEW.bezahlt IS NULL) THEN
            -- Calculate due date (X days before check-in)
            v_payment_due_date := (NEW.checkin_date::date -
                COALESCE(v_settings.payment_reminder_before_days, 7) * INTERVAL '1 day')::date;

            -- Check if reminder already exists
            SELECT COUNT(*) INTO v_existing_reminder_count
            FROM reminders
            WHERE booking_id = NEW.id
              AND reminder_type = 'auto_payment'
              AND is_completed = false;

            -- Only create if:
            -- 1. No active reminder exists
            -- 2. Payment reminder email NOT already sent (mahnung_gesendet_am IS NULL)
            IF v_existing_reminder_count = 0 AND NEW.mahnung_gesendet_am IS NULL THEN
                INSERT INTO reminders (
                    booking_id, reminder_type, title, description, due_date, priority,
                    is_completed, is_snoozed, created_at, updated_at
                )
                VALUES (
                    NEW.id,
                    'auto_payment',
                    format('Zahlungserinnerung - Buchung #%s', NEW.id),
                    format('Buchung noch nicht bezahlt - Check-in: %s - Gast: %s %s',
                        TO_CHAR(NEW.checkin_date::date, 'DD.MM.YYYY'),
                        v_guest.vorname,
                        v_guest.nachname
                    ),
                    v_payment_due_date,
                    'high',
                    false,
                    false,
                    CURRENT_TIMESTAMP,
                    CURRENT_TIMESTAMP
                );
            END IF;
        END IF;
    END IF;

    -- =============================================================================
    -- 3. AUTO-REMINDER: Check-in Preparation
    -- =============================================================================
    -- Purpose: Remind to prepare for guest arrival (keys, room check, etc.)
    -- Trigger: Always (for all confirmed bookings)
    -- Due Date: X days before check-in (configurable via checkin_reminder_before_days)
    -- Priority: MEDIUM
    -- =============================================================================

    IF v_settings.auto_reminder_checkin IS TRUE THEN
        -- Calculate due date (X days before check-in)
        v_checkin_due_date := (NEW.checkin_date::date -
            COALESCE(v_settings.checkin_reminder_before_days, 1) * INTERVAL '1 day')::date;

        -- Check if reminder already exists
        SELECT COUNT(*) INTO v_existing_reminder_count
        FROM reminders
        WHERE booking_id = NEW.id
          AND reminder_type = 'auto_checkin'
          AND is_completed = false;

        IF v_existing_reminder_count = 0 THEN
            INSERT INTO reminders (
                booking_id, reminder_type, title, description, due_date, priority,
                is_completed, is_snoozed, created_at, updated_at
            )
            VALUES (
                NEW.id,
                'auto_checkin',
                format('Check-in Vorbereitung - Buchung #%s', NEW.id),
                format('Check-in: %s - Zimmer vorbereiten, Schlüssel bereitstellen - Gast: %s %s',
                    TO_CHAR(NEW.checkin_date::date, 'DD.MM.YYYY'),
                    v_guest.vorname,
                    v_guest.nachname
                ),
                v_checkin_due_date,
                'medium',
                false,
                false,
                CURRENT_TIMESTAMP,
                CURRENT_TIMESTAMP
            );
        END IF;
    END IF;

    -- =============================================================================
    -- 4. AUTO-REMINDER: Invoice Not Sent
    -- =============================================================================
    -- Purpose: Remind to send invoice after booking confirmation
    -- Trigger: When invoice not yet sent (rechnung_versendet_am IS NULL)
    -- Due Date: Tomorrow (1 day after booking creation)
    -- Priority: MEDIUM
    -- Skip: If invoice already sent
    -- =============================================================================

    IF v_settings.auto_reminder_invoice IS TRUE THEN
        -- Only create if invoice not sent yet
        IF NEW.rechnung_versendet_am IS NULL THEN
            -- Check if reminder already exists
            SELECT COUNT(*) INTO v_existing_reminder_count
            FROM reminders
            WHERE booking_id = NEW.id
              AND reminder_type = 'auto_invoice'
              AND is_completed = false;

            IF v_existing_reminder_count = 0 THEN
                INSERT INTO reminders (
                    booking_id, reminder_type, title, description, due_date, priority,
                    is_completed, is_snoozed, created_at, updated_at
                )
                VALUES (
                    NEW.id,
                    'auto_invoice',
                    format('Rechnung versenden - Buchung #%s', NEW.id),
                    format('Rechnung noch nicht versendet an %s - Gast: %s %s',
                        COALESCE(v_guest.email, 'keine Email'),
                        v_guest.vorname,
                        v_guest.nachname
                    ),
                    CURRENT_DATE + INTERVAL '1 day',  -- Due tomorrow
                    'medium',
                    false,
                    false,
                    CURRENT_TIMESTAMP,
                    CURRENT_TIMESTAMP
                );
            END IF;
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- =============================================================================
-- TRIGGER: trg_auto_create_reminders
-- =============================================================================
-- Purpose: Call auto_create_reminders_for_booking() after booking changes
-- Fires: AFTER INSERT OR UPDATE ON bookings
-- =============================================================================

CREATE TRIGGER trg_auto_create_reminders
    AFTER INSERT OR UPDATE ON bookings
    FOR EACH ROW
    EXECUTE FUNCTION auto_create_reminders_for_booking();

-- =============================================================================
-- PART 3: Auto-Completion Logic
-- =============================================================================

-- =============================================================================
-- FUNCTION: auto_complete_reminders_on_booking_update()
-- =============================================================================
-- Purpose: Automatically mark reminders as completed when conditions are met
-- Examples:
--   - Payment reminder completed when bezahlt = true
--   - Invoice reminder completed when rechnung_versendet_am is set
--   - Incomplete data reminder completed when guest data is complete
-- =============================================================================

CREATE OR REPLACE FUNCTION auto_complete_reminders_on_booking_update()
RETURNS TRIGGER AS $$
DECLARE
    v_guest RECORD;
    v_guest_complete BOOLEAN;
BEGIN
    -- Get guest data
    SELECT * INTO v_guest FROM guests WHERE id = NEW.guest_id;

    -- 1. Auto-complete payment reminder if paid
    IF NEW.bezahlt = true AND OLD.bezahlt IS DISTINCT FROM NEW.bezahlt THEN
        UPDATE reminders
        SET is_completed = true,
            completed_at = CURRENT_TIMESTAMP,
            updated_at = CURRENT_TIMESTAMP
        WHERE booking_id = NEW.id
          AND reminder_type = 'auto_payment'
          AND is_completed = false;
    END IF;

    -- 2. Auto-complete invoice reminder if sent
    IF NEW.rechnung_versendet_am IS NOT NULL AND OLD.rechnung_versendet_am IS DISTINCT FROM NEW.rechnung_versendet_am THEN
        UPDATE reminders
        SET is_completed = true,
            completed_at = CURRENT_TIMESTAMP,
            updated_at = CURRENT_TIMESTAMP
        WHERE booking_id = NEW.id
          AND reminder_type = 'auto_invoice'
          AND is_completed = false;
    END IF;

    -- 3. Auto-complete incomplete data reminder if guest data is now complete
    IF v_guest IS NOT NULL THEN
        v_guest_complete := (
            v_guest.email IS NOT NULL AND v_guest.email != '' AND
            v_guest.telefon IS NOT NULL AND v_guest.telefon != '' AND
            v_guest.strasse IS NOT NULL AND v_guest.strasse != '' AND
            v_guest.plz IS NOT NULL AND v_guest.plz != '' AND
            v_guest.ort IS NOT NULL AND v_guest.ort != ''
        );

        IF v_guest_complete THEN
            UPDATE reminders
            SET is_completed = true,
                completed_at = CURRENT_TIMESTAMP,
                updated_at = CURRENT_TIMESTAMP
            WHERE booking_id = NEW.id
              AND reminder_type = 'auto_incomplete_data'
              AND is_completed = false;
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_auto_complete_reminders
    AFTER UPDATE ON bookings
    FOR EACH ROW
    EXECUTE FUNCTION auto_complete_reminders_on_booking_update();

-- =============================================================================
-- COMMENTS for Documentation
-- =============================================================================

COMMENT ON FUNCTION auto_create_reminders_for_booking() IS
'Automatically creates reminders for bookings based on notification_settings flags. Checks for duplicates and skips if reminders already exist.';

COMMENT ON TRIGGER trg_auto_create_reminders ON bookings IS
'Automatically creates reminders after booking creation/update based on notification_settings configuration.';

COMMENT ON FUNCTION auto_complete_reminders_on_booking_update() IS
'Automatically marks reminders as completed when booking conditions are met (payment received, invoice sent, etc.)';

COMMENT ON TRIGGER trg_auto_complete_reminders ON bookings IS
'Automatically completes reminders when their conditions are met (e.g., payment received, invoice sent).';
