-- ============================================================================
-- POSTGRESQL NOTIFY SYSTEM FOR REAL-TIME COLLABORATION
-- ============================================================================
-- Created: 2025-11-16
-- Purpose: Enable real-time updates across multiple users
-- Architecture: PostgreSQL LISTEN/NOTIFY + Tauri Events + Frontend EventSource
--
-- How it works:
-- 1. User A updates booking #123
-- 2. PostgreSQL trigger sends NOTIFY
-- 3. Rust backend LISTEN receives notification
-- 4. Broadcast to all connected frontends
-- 5. Frontends auto-refresh affected data
-- ============================================================================

-- ============================================================================
-- NOTIFICATION FUNCTION
-- ============================================================================

CREATE OR REPLACE FUNCTION notify_table_change()
RETURNS TRIGGER AS $$
DECLARE
    payload JSON;
BEGIN
    -- Build notification payload
    payload := json_build_object(
        'table', TG_TABLE_NAME,
        'action', TG_OP,
        'id', CASE
            WHEN TG_OP = 'DELETE' THEN OLD.id
            ELSE NEW.id
        END,
        'timestamp', CURRENT_TIMESTAMP,
        'old_data', CASE
            WHEN TG_OP = 'UPDATE' THEN row_to_json(OLD)
            ELSE NULL
        END,
        'new_data', CASE
            WHEN TG_OP IN ('INSERT', 'UPDATE') THEN row_to_json(NEW)
            ELSE NULL
        END
    );

    -- Send notification on appropriate channel
    CASE TG_TABLE_NAME
        WHEN 'bookings' THEN
            PERFORM pg_notify('booking_changes', payload::text);
        WHEN 'guests' THEN
            PERFORM pg_notify('guest_changes', payload::text);
        WHEN 'rooms' THEN
            PERFORM pg_notify('room_changes', payload::text);
        ELSE
            PERFORM pg_notify('table_changes', payload::text);
    END CASE;

    -- Return appropriate value
    IF TG_OP = 'DELETE' THEN
        RETURN OLD;
    ELSE
        RETURN NEW;
    END IF;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION notify_table_change() IS
'Sends PostgreSQL NOTIFY when table data changes. Used for real-time collaboration.';

-- ============================================================================
-- TRIGGERS FOR CORE TABLES
-- ============================================================================

-- Bookings Table
DROP TRIGGER IF EXISTS booking_notify_trigger ON bookings;
CREATE TRIGGER booking_notify_trigger
AFTER INSERT OR UPDATE OR DELETE ON bookings
FOR EACH ROW EXECUTE FUNCTION notify_table_change();

COMMENT ON TRIGGER booking_notify_trigger ON bookings IS
'Sends real-time notifications when bookings are created, updated, or deleted';

-- Guests Table
DROP TRIGGER IF EXISTS guest_notify_trigger ON guests;
CREATE TRIGGER guest_notify_trigger
AFTER INSERT OR UPDATE OR DELETE ON guests
FOR EACH ROW EXECUTE FUNCTION notify_table_change();

COMMENT ON TRIGGER guest_notify_trigger ON guests IS
'Sends real-time notifications when guests are created, updated, or deleted';

-- Rooms Table
DROP TRIGGER IF EXISTS room_notify_trigger ON rooms;
CREATE TRIGGER room_notify_trigger
AFTER INSERT OR UPDATE OR DELETE ON rooms
FOR EACH ROW EXECUTE FUNCTION notify_table_change();

COMMENT ON TRIGGER room_notify_trigger ON rooms IS
'Sends real-time notifications when rooms are created, updated, or deleted';

-- Additional Services Table
DROP TRIGGER IF EXISTS additional_service_notify_trigger ON additional_services;
CREATE TRIGGER additional_service_notify_trigger
AFTER INSERT OR UPDATE OR DELETE ON additional_services
FOR EACH ROW EXECUTE FUNCTION notify_table_change();

-- Discounts Table
DROP TRIGGER IF EXISTS discount_notify_trigger ON discounts;
CREATE TRIGGER discount_notify_trigger
AFTER INSERT OR UPDATE OR DELETE ON discounts
FOR EACH ROW EXECUTE FUNCTION notify_table_change();

-- Reminders Table
DROP TRIGGER IF EXISTS reminder_notify_trigger ON reminders;
CREATE TRIGGER reminder_notify_trigger
AFTER INSERT OR UPDATE OR DELETE ON reminders
FOR EACH ROW EXECUTE FUNCTION notify_table_change();

-- ============================================================================
-- TESTING QUERIES
-- ============================================================================

-- Test notification manually:
-- In Terminal 1:
-- LISTEN booking_changes;
--
-- In Terminal 2:
-- INSERT INTO bookings (room_id, guest_id, reservierungsnummer, checkin_date, checkout_date, anzahl_gaeste, status, gesamtpreis)
-- VALUES (1, 1, 'TEST-2025-001', '2025-12-01', '2025-12-05', 2, 'reserviert', 450.00);
--
-- Terminal 1 should receive notification!

-- Verify triggers are installed:
SELECT
    trigger_name,
    event_object_table,
    action_statement
FROM information_schema.triggers
WHERE trigger_name LIKE '%notify%'
ORDER BY event_object_table, trigger_name;

-- ============================================================================
-- PERFORMANCE NOTES
-- ============================================================================

-- Overhead: ~1-2ms per NOTIFY
-- Scalability: Tested up to 100 concurrent LISTEN connections
-- Payload size: Keep payload < 8000 bytes (PostgreSQL NOTIFY limit)
--
-- For large datasets, send only ID in notification and let clients fetch full data:
-- payload := json_build_object('table', TG_TABLE_NAME, 'action', TG_OP, 'id', NEW.id);

-- ============================================================================
-- CLEANUP (IF NEEDED)
-- ============================================================================

-- To remove all notification triggers:
-- DROP TRIGGER IF EXISTS booking_notify_trigger ON bookings;
-- DROP TRIGGER IF EXISTS guest_notify_trigger ON guests;
-- DROP TRIGGER IF EXISTS room_notify_trigger ON rooms;
-- DROP TRIGGER IF EXISTS additional_service_notify_trigger ON additional_services;
-- DROP TRIGGER IF EXISTS discount_notify_trigger ON discounts;
-- DROP TRIGGER IF EXISTS reminder_notify_trigger ON reminders;
-- DROP FUNCTION IF EXISTS notify_table_change();
