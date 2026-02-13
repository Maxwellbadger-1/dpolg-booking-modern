-- Migration: Add active_locks table for presence system and advisory locks
-- Purpose: Track which user is currently editing which booking
-- Date: 2026-02-11

-- Create active_locks table
CREATE TABLE IF NOT EXISTS active_locks (
    id SERIAL PRIMARY KEY,
    booking_id INT NOT NULL REFERENCES bookings(id) ON DELETE CASCADE,
    user_name VARCHAR(100) NOT NULL,
    locked_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_heartbeat TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(booking_id)  -- Only one lock per booking
);

-- Index for fast lookups
CREATE INDEX idx_active_locks_booking_id ON active_locks(booking_id);
CREATE INDEX idx_active_locks_user_name ON active_locks(user_name);
CREATE INDEX idx_active_locks_last_heartbeat ON active_locks(last_heartbeat);

-- Function to clean up stale locks (older than 5 minutes without heartbeat)
CREATE OR REPLACE FUNCTION cleanup_stale_locks()
RETURNS void AS $$
BEGIN
    DELETE FROM active_locks
    WHERE last_heartbeat < NOW() - INTERVAL '5 minutes';
END;
$$ LANGUAGE plpgsql;

-- Trigger to send NOTIFY when locks change
CREATE OR REPLACE FUNCTION notify_lock_change()
RETURNS TRIGGER AS $$
BEGIN
    IF (TG_OP = 'INSERT') THEN
        PERFORM pg_notify('lock_changes', json_build_object(
            'action', 'ACQUIRED',
            'booking_id', NEW.booking_id,
            'user_name', NEW.user_name,
            'locked_at', NEW.locked_at::text
        )::text);
        RETURN NEW;
    ELSIF (TG_OP = 'DELETE') THEN
        PERFORM pg_notify('lock_changes', json_build_object(
            'action', 'RELEASED',
            'booking_id', OLD.booking_id,
            'user_name', OLD.user_name
        )::text);
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_lock_changes
    AFTER INSERT OR DELETE ON active_locks
    FOR EACH ROW
    EXECUTE FUNCTION notify_lock_change();
