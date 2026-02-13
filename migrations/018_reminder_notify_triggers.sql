-- ============================================================================
-- Migration 018: Real-Time NOTIFY Triggers for Reminders
-- ============================================================================
-- Created: 2026-02-12
-- Purpose: Enable PostgreSQL LISTEN/NOTIFY for reminders table
-- Pattern: Same as Migration 007 (bookings, guests, rooms)
-- ============================================================================

-- ============================================================================
-- PART 1: Update notify_table_change() function to handle 'reminders' table
-- ============================================================================

CREATE OR REPLACE FUNCTION notify_table_change()
RETURNS TRIGGER AS $$
DECLARE
    notification json;
BEGIN
    -- Build JSON notification payload
    notification = json_build_object(
        'table', TG_TABLE_NAME,
        'action', TG_OP,
        'id', COALESCE(NEW.id, OLD.id),
        'timestamp', NOW()
    );

    -- Send notification on appropriate channel
    IF TG_TABLE_NAME = 'bookings' THEN
        PERFORM pg_notify('booking_changes', notification::text);
    ELSIF TG_TABLE_NAME = 'guests' THEN
        PERFORM pg_notify('guest_changes', notification::text);
    ELSIF TG_TABLE_NAME = 'rooms' THEN
        PERFORM pg_notify('room_changes', notification::text);
    ELSIF TG_TABLE_NAME = 'reminders' THEN
        PERFORM pg_notify('reminder_changes', notification::text);  -- NEW
    ELSE
        PERFORM pg_notify('table_changes', notification::text);
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- PART 2: Create NOTIFY triggers for reminders table
-- ============================================================================

DROP TRIGGER IF EXISTS trg_notify_reminder_insert ON reminders;
CREATE TRIGGER trg_notify_reminder_insert
    AFTER INSERT ON reminders
    FOR EACH ROW
    EXECUTE FUNCTION notify_table_change();

DROP TRIGGER IF EXISTS trg_notify_reminder_update ON reminders;
CREATE TRIGGER trg_notify_reminder_update
    AFTER UPDATE ON reminders
    FOR EACH ROW
    EXECUTE FUNCTION notify_table_change();

DROP TRIGGER IF EXISTS trg_notify_reminder_delete ON reminders;
CREATE TRIGGER trg_notify_reminder_delete
    AFTER DELETE ON reminders
    FOR EACH ROW
    EXECUTE FUNCTION notify_table_change();

-- ============================================================================
-- PART 3: Verification
-- ============================================================================

-- List all reminder NOTIFY triggers
SELECT
    tgname AS trigger_name,
    tgrelid::regclass AS table_name,
    CASE tgtype & 66
        WHEN 2 THEN 'BEFORE'
        WHEN 64 THEN 'INSTEAD OF'
        ELSE 'AFTER'
    END AS trigger_timing,
    CASE tgtype & 28
        WHEN 4 THEN 'INSERT'
        WHEN 8 THEN 'DELETE'
        WHEN 16 THEN 'UPDATE'
        WHEN 12 THEN 'INSERT OR DELETE'
        WHEN 20 THEN 'INSERT OR UPDATE'
        WHEN 24 THEN 'DELETE OR UPDATE'
        WHEN 28 THEN 'INSERT OR UPDATE OR DELETE'
    END AS trigger_event
FROM pg_trigger
WHERE tgname LIKE 'trg_notify_reminder%'
ORDER BY tgrelid::regclass::text, tgname;

-- ============================================================================
-- DONE
-- ============================================================================

COMMENT ON TRIGGER trg_notify_reminder_insert ON reminders IS 'Sends real-time NOTIFY events when reminders are inserted';
COMMENT ON TRIGGER trg_notify_reminder_update ON reminders IS 'Sends real-time NOTIFY events when reminders are updated';
COMMENT ON TRIGGER trg_notify_reminder_delete ON reminders IS 'Sends real-time NOTIFY events when reminders are deleted';
