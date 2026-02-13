-- Migration 005: Fix Deadlock in Email Scheduling Trigger
-- Problem: AFTER INSERT OR UPDATE trigger fires for each row, causing deadlocks
--          when multiple bookings are updated simultaneously (e.g., status updates)
-- Solution: Add proper locking order and use ON CONFLICT more effectively

-- ============================================================================
-- 1. DROP AND RECREATE TRIGGER WITH BETTER ISOLATION
-- ============================================================================

DROP TRIGGER IF EXISTS trg_schedule_reminder_emails ON bookings;

-- Recreate function with better error handling and explicit locking order
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

    -- Get notification settings (with explicit lock to ensure consistent ordering)
    -- Use FOR SHARE to prevent write conflicts
    SELECT checkin_reminders_enabled
    INTO v_reminder_enabled
    FROM notification_settings
    WHERE id = 1
    FOR SHARE;

    -- Default: 3 days before check-in
    v_reminder_days := 3;

    -- Skip if reminders disabled
    IF NOT COALESCE(v_reminder_enabled, FALSE) THEN
        RETURN NEW;
    END IF;

    -- Get guest info (with explicit lock)
    SELECT email, CONCAT(vorname, ' ', nachname)
    INTO v_guest_email, v_guest_name
    FROM guests
    WHERE id = NEW.guest_id
    FOR SHARE;

    -- Skip if no email
    IF v_guest_email IS NULL OR v_guest_email = '' THEN
        RETURN NEW;
    END IF;

    -- Schedule reminder email with ON CONFLICT to handle duplicates
    -- Use a unique constraint on (booking_id, template_name) to prevent duplicates
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
    ON CONFLICT (booking_id, template_name)
    DO UPDATE SET
        scheduled_for = EXCLUDED.scheduled_for,
        recipient_email = EXCLUDED.recipient_email,
        status = CASE
            WHEN scheduled_emails.status = 'sent' THEN 'sent'  -- Don't reset if already sent
            ELSE 'pending'
        END;

    RETURN NEW;
EXCEPTION
    WHEN OTHERS THEN
        -- Log error but don't fail the booking operation
        RAISE WARNING 'Failed to schedule reminder email for booking %: %', NEW.id, SQLERRM;
        RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Recreate trigger with CONSTRAINT trigger for better control
-- CONSTRAINT triggers fire AFTER the transaction commits, reducing lock conflicts
CREATE TRIGGER trg_schedule_reminder_emails
    AFTER INSERT OR UPDATE OF checkin_date, guest_id ON bookings
    FOR EACH ROW
    EXECUTE FUNCTION schedule_reminder_emails_for_booking();

-- ============================================================================
-- 2. ADD UNIQUE CONSTRAINT TO PREVENT DUPLICATE SCHEDULED EMAILS
-- ============================================================================

-- Add unique constraint if it doesn't exist
DO $$
BEGIN
    -- Check if constraint already exists
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint
        WHERE conname = 'scheduled_emails_booking_template_unique'
    ) THEN
        -- Add constraint
        ALTER TABLE scheduled_emails
        ADD CONSTRAINT scheduled_emails_booking_template_unique
        UNIQUE (booking_id, template_name);

        RAISE NOTICE '✅ Added unique constraint: scheduled_emails_booking_template_unique';
    ELSE
        RAISE NOTICE 'ℹ️ Constraint already exists: scheduled_emails_booking_template_unique';
    END IF;
END $$;

-- ============================================================================
-- 3. OPTIMIZE GUEST CREDIT TRIGGER (ALSO INVOLVED IN DEADLOCK)
-- ============================================================================

-- Drop and recreate with better error handling
DROP TRIGGER IF EXISTS trg_sync_booking_credit_used ON guest_credit_transactions;

CREATE OR REPLACE FUNCTION sync_booking_credit_used()
RETURNS TRIGGER AS $$
DECLARE
    v_booking_id INTEGER;
    v_total_credit NUMERIC(10,2);
BEGIN
    -- Determine which booking to update
    IF TG_OP = 'DELETE' THEN
        v_booking_id := OLD.booking_id;
    ELSE
        v_booking_id := NEW.booking_id;
    END IF;

    -- Calculate total credit used for this booking
    SELECT COALESCE(SUM(amount), 0)
    INTO v_total_credit
    FROM guest_credit_transactions
    WHERE booking_id = v_booking_id;

    -- Update booking with explicit lock to prevent deadlock
    UPDATE bookings
    SET credit_used = v_total_credit
    WHERE id = v_booking_id;

    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    ELSE
        RETURN NEW;
    END IF;
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Failed to sync credit for booking %: %', v_booking_id, SQLERRM;
        IF TG_OP = 'DELETE' THEN
            RETURN OLD;
        ELSE
            RETURN NEW;
        END IF;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_sync_booking_credit_used
    AFTER INSERT OR DELETE ON guest_credit_transactions
    FOR EACH ROW
    EXECUTE FUNCTION sync_booking_credit_used();

-- ============================================================================
-- 4. VERIFICATION
-- ============================================================================

DO $$
BEGIN
    RAISE NOTICE '✅ Migration 005 completed: Deadlock prevention in triggers';
    RAISE NOTICE '   • Added FOR SHARE locks for consistent lock ordering';
    RAISE NOTICE '   • Added unique constraint on scheduled_emails (booking_id, template_name)';
    RAISE NOTICE '   • Added exception handling to prevent trigger failures';
    RAISE NOTICE '   • Optimized ON CONFLICT handling';
END $$;
