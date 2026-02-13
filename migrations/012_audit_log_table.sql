-- Migration: Add audit triggers to existing audit_log table
-- Purpose: Log booking/guest/room changes to existing audit_log table
-- Date: 2026-02-11
-- Note: Uses existing audit_log table with schema: old_values, new_values, user_name, timestamp

-- Trigger function to log booking changes (adapted for existing schema)
CREATE OR REPLACE FUNCTION log_booking_changes()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'UPDATE') THEN
        INSERT INTO audit_log (table_name, record_id, action, old_values, new_values, user_name)
        VALUES (
            'bookings',
            OLD.id,
            'UPDATE',
            to_jsonb(OLD),
            to_jsonb(NEW),
            COALESCE(NEW.updated_by, 'system')
        );
        RETURN NEW;
    ELSIF (TG_OP = 'DELETE') THEN
        INSERT INTO audit_log (table_name, record_id, action, old_values, new_values, user_name)
        VALUES (
            'bookings',
            OLD.id,
            'DELETE',
            to_jsonb(OLD),
            NULL,
            COALESCE(OLD.updated_by, 'system')
        );
        RETURN OLD;
    ELSIF (TG_OP = 'INSERT') THEN
        INSERT INTO audit_log (table_name, record_id, action, old_values, new_values, user_name)
        VALUES (
            'bookings',
            NEW.id,
            'INSERT',
            NULL,
            to_jsonb(NEW),
            COALESCE(NEW.created_by, 'system')
        );
        RETURN NEW;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Attach trigger to bookings table (drop first if exists)
DROP TRIGGER IF EXISTS trigger_audit_bookings ON bookings;
CREATE TRIGGER trigger_audit_bookings
    AFTER INSERT OR UPDATE OR DELETE ON bookings
    FOR EACH ROW
    EXECUTE FUNCTION log_booking_changes();

-- Trigger for guests
CREATE OR REPLACE FUNCTION log_guest_changes()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'UPDATE') THEN
        INSERT INTO audit_log (table_name, record_id, action, old_values, new_values, user_name)
        VALUES ('guests', OLD.id, 'UPDATE', to_jsonb(OLD), to_jsonb(NEW), COALESCE(NEW.updated_by, 'system'));
        RETURN NEW;
    ELSIF (TG_OP = 'DELETE') THEN
        INSERT INTO audit_log (table_name, record_id, action, old_values, new_values, user_name)
        VALUES ('guests', OLD.id, 'DELETE', to_jsonb(OLD), NULL, COALESCE(OLD.updated_by, 'system'));
        RETURN OLD;
    ELSIF (TG_OP = 'INSERT') THEN
        INSERT INTO audit_log (table_name, record_id, action, old_values, new_values, user_name)
        VALUES ('guests', NEW.id, 'INSERT', NULL, to_jsonb(NEW), COALESCE(NEW.created_by, 'system'));
        RETURN NEW;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trigger_audit_guests ON guests;
CREATE TRIGGER trigger_audit_guests
    AFTER INSERT OR UPDATE OR DELETE ON guests
    FOR EACH ROW
    EXECUTE FUNCTION log_guest_changes();

-- Trigger for rooms
CREATE OR REPLACE FUNCTION log_room_changes()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'UPDATE') THEN
        INSERT INTO audit_log (table_name, record_id, action, old_values, new_values, user_name)
        VALUES ('rooms', OLD.id, 'UPDATE', to_jsonb(OLD), to_jsonb(NEW), COALESCE(NEW.updated_by, 'system'));
        RETURN NEW;
    ELSIF (TG_OP = 'DELETE') THEN
        INSERT INTO audit_log (table_name, record_id, action, old_values, new_values, user_name)
        VALUES ('rooms', OLD.id, 'DELETE', to_jsonb(OLD), NULL, COALESCE(OLD.updated_by, 'system'));
        RETURN OLD;
    ELSIF (TG_OP = 'INSERT') THEN
        INSERT INTO audit_log (table_name, record_id, action, old_values, new_values, user_name)
        VALUES ('rooms', NEW.id, 'INSERT', NULL, to_jsonb(NEW), COALESCE(NEW.created_by, 'system'));
        RETURN NEW;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trigger_audit_rooms ON rooms;
CREATE TRIGGER trigger_audit_rooms
    AFTER INSERT OR UPDATE OR DELETE ON rooms
    FOR EACH ROW
    EXECUTE FUNCTION log_room_changes();
