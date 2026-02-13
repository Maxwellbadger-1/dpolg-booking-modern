-- Migration: Email Automation System
-- Date: 2025-11-22
-- Description: Implements scheduled email system for automatic reminders

-- ============================================================================
-- 1. SCHEDULED EMAILS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS scheduled_emails (
    id SERIAL PRIMARY KEY,
    booking_id INTEGER,
    guest_id INTEGER,
    template_name VARCHAR(100) NOT NULL,
    recipient_email VARCHAR(255) NOT NULL,
    subject VARCHAR(255) NOT NULL,
    scheduled_for TIMESTAMP NOT NULL,
    status VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('pending', 'sent', 'failed', 'cancelled')),
    sent_at TIMESTAMP,
    error_message TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    -- Foreign Keys
    CONSTRAINT fk_scheduled_email_booking
        FOREIGN KEY (booking_id) REFERENCES bookings(id) ON DELETE CASCADE,
    CONSTRAINT fk_scheduled_email_guest
        FOREIGN KEY (guest_id) REFERENCES guests(id) ON DELETE CASCADE
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_scheduled_emails_status ON scheduled_emails(status);
CREATE INDEX IF NOT EXISTS idx_scheduled_emails_scheduled_for ON scheduled_emails(scheduled_for);
CREATE INDEX IF NOT EXISTS idx_scheduled_emails_booking ON scheduled_emails(booking_id);
CREATE INDEX IF NOT EXISTS idx_scheduled_emails_guest ON scheduled_emails(guest_id);

-- Comments
COMMENT ON TABLE scheduled_emails IS 'Scheduled emails for automatic reminders and notifications';
COMMENT ON COLUMN scheduled_emails.status IS 'pending=waiting to send, sent=successfully sent, failed=error occurred, cancelled=user cancelled';

-- ============================================================================
-- 2. AUTO-SCHEDULE REMINDER EMAILS
-- ============================================================================

CREATE OR REPLACE FUNCTION schedule_reminder_emails_for_booking()
RETURNS TRIGGER AS $$
DECLARE
    v_guest_email VARCHAR(255);
    v_guest_name VARCHAR(200);
    v_reminder_days INTEGER;
    v_reminder_enabled BOOLEAN;
BEGIN
    -- Only for non-cancelled bookings
    IF NEW.status IN ('cancelled', 'storniert') THEN
        RETURN NEW;
    END IF;

    -- Get notification settings (using existing columns)
    SELECT checkin_reminders_enabled
    INTO v_reminder_enabled
    FROM notification_settings
    WHERE id = 1;

    -- Default: 3 days before check-in
    v_reminder_days := 3;

    -- Skip if reminders disabled
    IF NOT COALESCE(v_reminder_enabled, FALSE) THEN
        RETURN NEW;
    END IF;

    -- Get guest info
    SELECT email, CONCAT(vorname, ' ', nachname)
    INTO v_guest_email, v_guest_name
    FROM guests WHERE id = NEW.guest_id;

    -- Skip if no email
    IF v_guest_email IS NULL OR v_guest_email = '' THEN
        RETURN NEW;
    END IF;

    -- Schedule reminder email
    INSERT INTO scheduled_emails (
        booking_id,
        guest_id,
        template_name,
        recipient_email,
        subject,
        scheduled_for,
        status
    ) VALUES (
        NEW.id,
        NEW.guest_id,
        'booking_reminder',
        v_guest_email,
        format('Erinnerung: Ihre Buchung #%s', NEW.id),
        (NEW.checkin_date::date - INTERVAL '1 day' * COALESCE(v_reminder_days, 3))::TIMESTAMP,
        'pending'
    )
    ON CONFLICT DO NOTHING;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_schedule_reminder_emails
    AFTER INSERT OR UPDATE OF checkin_date, guest_id ON bookings
    FOR EACH ROW
    EXECUTE FUNCTION schedule_reminder_emails_for_booking();

-- ============================================================================
-- 3. HELPER FUNCTIONS
-- ============================================================================

-- Get pending emails that should be sent now
CREATE OR REPLACE FUNCTION get_pending_scheduled_emails()
RETURNS TABLE (
    id INTEGER,
    booking_id INTEGER,
    guest_id INTEGER,
    template_name VARCHAR,
    recipient_email VARCHAR,
    subject VARCHAR,
    scheduled_for TIMESTAMP
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        se.id,
        se.booking_id,
        se.guest_id,
        se.template_name,
        se.recipient_email,
        se.subject,
        se.scheduled_for
    FROM scheduled_emails se
    WHERE se.status = 'pending'
      AND se.scheduled_for <= CURRENT_TIMESTAMP
    ORDER BY se.scheduled_for
    LIMIT 100;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- 4. VERIFICATION
-- ============================================================================

SELECT table_name FROM information_schema.tables WHERE table_name = 'scheduled_emails';
SELECT proname FROM pg_proc WHERE proname IN ('schedule_reminder_emails_for_booking', 'get_pending_scheduled_emails');
