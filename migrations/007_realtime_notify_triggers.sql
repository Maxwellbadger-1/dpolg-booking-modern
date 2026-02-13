-- ============================================================================
-- Migration 007: Real-Time NOTIFY Triggers for Multi-User Support
-- ============================================================================
-- Created: 2025-11-30
-- Purpose: Enable PostgreSQL LISTEN/NOTIFY for real-time updates across clients
-- ============================================================================

-- ============================================================================
-- NOTIFY FUNCTION
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
    ELSE
        PERFORM pg_notify('table_changes', notification::text);
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- BOOKINGS TRIGGERS
-- ============================================================================

DROP TRIGGER IF EXISTS trg_notify_booking_insert ON bookings;
CREATE TRIGGER trg_notify_booking_insert
    AFTER INSERT ON bookings
    FOR EACH ROW
    EXECUTE FUNCTION notify_table_change();

DROP TRIGGER IF EXISTS trg_notify_booking_update ON bookings;
CREATE TRIGGER trg_notify_booking_update
    AFTER UPDATE ON bookings
    FOR EACH ROW
    EXECUTE FUNCTION notify_table_change();

DROP TRIGGER IF EXISTS trg_notify_booking_delete ON bookings;
CREATE TRIGGER trg_notify_booking_delete
    AFTER DELETE ON bookings
    FOR EACH ROW
    EXECUTE FUNCTION notify_table_change();

-- ============================================================================
-- GUESTS TRIGGERS
-- ============================================================================

DROP TRIGGER IF EXISTS trg_notify_guest_insert ON guests;
CREATE TRIGGER trg_notify_guest_insert
    AFTER INSERT ON guests
    FOR EACH ROW
    EXECUTE FUNCTION notify_table_change();

DROP TRIGGER IF EXISTS trg_notify_guest_update ON guests;
CREATE TRIGGER trg_notify_guest_update
    AFTER UPDATE ON guests
    FOR EACH ROW
    EXECUTE FUNCTION notify_table_change();

DROP TRIGGER IF EXISTS trg_notify_guest_delete ON guests;
CREATE TRIGGER trg_notify_guest_delete
    AFTER DELETE ON guests
    FOR EACH ROW
    EXECUTE FUNCTION notify_table_change();

-- ============================================================================
-- ROOMS TRIGGERS
-- ============================================================================

DROP TRIGGER IF EXISTS trg_notify_room_insert ON rooms;
CREATE TRIGGER trg_notify_room_insert
    AFTER INSERT ON rooms
    FOR EACH ROW
    EXECUTE FUNCTION notify_table_change();

DROP TRIGGER IF EXISTS trg_notify_room_update ON rooms;
CREATE TRIGGER trg_notify_room_update
    AFTER UPDATE ON rooms
    FOR EACH ROW
    EXECUTE FUNCTION notify_table_change();

DROP TRIGGER IF EXISTS trg_notify_room_delete ON rooms;
CREATE TRIGGER trg_notify_room_delete
    AFTER DELETE ON rooms
    FOR EACH ROW
    EXECUTE FUNCTION notify_table_change();

-- ============================================================================
-- ADDITIONAL SERVICES TRIGGERS
-- ============================================================================

DROP TRIGGER IF EXISTS trg_notify_service_insert ON additional_services;
CREATE TRIGGER trg_notify_service_insert
    AFTER INSERT ON additional_services
    FOR EACH ROW
    EXECUTE FUNCTION notify_table_change();

DROP TRIGGER IF EXISTS trg_notify_service_update ON additional_services;
CREATE TRIGGER trg_notify_service_update
    AFTER UPDATE ON additional_services
    FOR EACH ROW
    EXECUTE FUNCTION notify_table_change();

DROP TRIGGER IF EXISTS trg_notify_service_delete ON additional_services;
CREATE TRIGGER trg_notify_service_delete
    AFTER DELETE ON additional_services
    FOR EACH ROW
    EXECUTE FUNCTION notify_table_change();

-- ============================================================================
-- DISCOUNTS TRIGGERS
-- ============================================================================

DROP TRIGGER IF EXISTS trg_notify_discount_insert ON discounts;
CREATE TRIGGER trg_notify_discount_insert
    AFTER INSERT ON discounts
    FOR EACH ROW
    EXECUTE FUNCTION notify_table_change();

DROP TRIGGER IF EXISTS trg_notify_discount_update ON discounts;
CREATE TRIGGER trg_notify_discount_update
    AFTER UPDATE ON discounts
    FOR EACH ROW
    EXECUTE FUNCTION notify_table_change();

DROP TRIGGER IF EXISTS trg_notify_discount_delete ON discounts;
CREATE TRIGGER trg_notify_discount_delete
    AFTER DELETE ON discounts
    FOR EACH ROW
    EXECUTE FUNCTION notify_table_change();

-- ============================================================================
-- VERIFICATION
-- ============================================================================

-- List all NOTIFY triggers
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
WHERE tgname LIKE 'trg_notify%'
ORDER BY tgrelid::regclass::text, tgname;

-- ============================================================================
-- DONE
-- ============================================================================

COMMENT ON FUNCTION notify_table_change() IS 'Sends PostgreSQL NOTIFY events on table changes for real-time multi-user updates';
