-- Migration: Cleaning Tasks System
-- Date: 2025-11-22
-- Description: Implements cleaning tasks for mobile app integration

-- ============================================================================
-- 1. CLEANING TASKS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS cleaning_tasks (
    id SERIAL PRIMARY KEY,
    booking_id INTEGER NOT NULL,
    room_id INTEGER NOT NULL,
    task_date DATE NOT NULL,
    checkout_time TIME,
    checkin_time TIME,
    priority VARCHAR(20) DEFAULT 'NORMAL' CHECK (priority IN ('HIGH', 'NORMAL', 'LOW')),
    has_dog BOOLEAN DEFAULT FALSE,
    change_bedding BOOLEAN DEFAULT FALSE,
    guest_count INTEGER,
    guest_name VARCHAR(200),
    status VARCHAR(20) DEFAULT 'pending' CHECK (status IN ('pending', 'in_progress', 'done', 'cancelled')),
    completed_at TIMESTAMP,
    completed_by VARCHAR(100),
    notes TEXT,
    synced_to_turso BOOLEAN DEFAULT FALSE,
    turso_sync_at TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,

    -- Foreign Keys
    CONSTRAINT fk_cleaning_booking
        FOREIGN KEY (booking_id) REFERENCES bookings(id) ON DELETE CASCADE,
    CONSTRAINT fk_cleaning_room
        FOREIGN KEY (room_id) REFERENCES rooms(id) ON DELETE CASCADE
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_cleaning_task_date ON cleaning_tasks(task_date DESC);
CREATE INDEX IF NOT EXISTS idx_cleaning_booking_id ON cleaning_tasks(booking_id);
CREATE INDEX IF NOT EXISTS idx_cleaning_room_id ON cleaning_tasks(room_id);
CREATE INDEX IF NOT EXISTS idx_cleaning_status ON cleaning_tasks(status);
CREATE INDEX IF NOT EXISTS idx_cleaning_priority ON cleaning_tasks(priority);
CREATE INDEX IF NOT EXISTS idx_cleaning_sync ON cleaning_tasks(synced_to_turso, turso_sync_at);

-- Comments
COMMENT ON TABLE cleaning_tasks IS 'Cleaning tasks generated from bookings for mobile app';
COMMENT ON COLUMN cleaning_tasks.priority IS 'HIGH=same-day check-in/out, NORMAL=standard, LOW=optional';
COMMENT ON COLUMN cleaning_tasks.synced_to_turso IS 'Whether task has been synced to Turso cloud database';

-- ============================================================================
-- 2. AUTO-UPDATE TIMESTAMP TRIGGER
-- ============================================================================

CREATE OR REPLACE FUNCTION update_cleaning_task_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_update_cleaning_task_timestamp
    BEFORE UPDATE ON cleaning_tasks
    FOR EACH ROW
    EXECUTE FUNCTION update_cleaning_task_timestamp();

-- ============================================================================
-- 3. AUTO-GENERATE CLEANING TASKS FROM BOOKINGS
-- ============================================================================

CREATE OR REPLACE FUNCTION generate_cleaning_task_for_booking()
RETURNS TRIGGER AS $$
DECLARE
    v_priority VARCHAR(20);
    v_has_dog BOOLEAN;
    v_guest_name VARCHAR(200);
    v_guest_count INTEGER;
BEGIN
    -- Only generate for non-cancelled bookings
    IF NEW.status IN ('cancelled', 'storniert') THEN
        RETURN NEW;
    END IF;

    -- Determine priority (HIGH if same-day checkout/checkin)
    v_priority := 'NORMAL';

    -- Check for dog (via additional_services)
    SELECT EXISTS(
        SELECT 1 FROM additional_services
        WHERE booking_id = NEW.id AND service_name ILIKE '%hund%'
    ) INTO v_has_dog;

    -- Get guest name
    SELECT CONCAT(vorname, ' ', nachname) INTO v_guest_name
    FROM guests WHERE id = NEW.guest_id;

    -- Get guest count (main guest + accompanying guests)
    SELECT 1 + COUNT(*) INTO v_guest_count
    FROM accompanying_guests WHERE booking_id = NEW.id;

    -- Create cleaning task for checkout day
    INSERT INTO cleaning_tasks (
        booking_id,
        room_id,
        task_date,
        checkout_time,
        priority,
        has_dog,
        change_bedding,
        guest_count,
        guest_name,
        status
    ) VALUES (
        NEW.id,
        NEW.room_id,
        NEW.checkout_date::date,
        NULL, -- checkout_time (not stored in bookings)
        v_priority,
        v_has_dog,
        TRUE, -- Always change bedding on checkout
        v_guest_count,
        v_guest_name,
        'pending'
    )
    ON CONFLICT DO NOTHING; -- Prevent duplicates

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger on booking insert/update
CREATE TRIGGER trg_generate_cleaning_task_insert
    AFTER INSERT ON bookings
    FOR EACH ROW
    EXECUTE FUNCTION generate_cleaning_task_for_booking();

CREATE TRIGGER trg_generate_cleaning_task_update
    AFTER UPDATE OF checkout_date, room_id, status ON bookings
    FOR EACH ROW
    EXECUTE FUNCTION generate_cleaning_task_for_booking();

-- ============================================================================
-- 4. DELETE CLEANING TASKS WHEN BOOKING IS CANCELLED
-- ============================================================================

CREATE OR REPLACE FUNCTION delete_cleaning_task_on_cancel()
RETURNS TRIGGER AS $$
BEGIN
    -- If booking is cancelled, delete its cleaning tasks
    IF NEW.status IN ('cancelled', 'storniert') AND OLD.status NOT IN ('cancelled', 'storniert') THEN
        DELETE FROM cleaning_tasks WHERE booking_id = NEW.id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_delete_cleaning_task_on_cancel
    AFTER UPDATE OF status ON bookings
    FOR EACH ROW
    EXECUTE FUNCTION delete_cleaning_task_on_cancel();

-- ============================================================================
-- 5. HELPER FUNCTIONS
-- ============================================================================

-- Get tasks for a date range
CREATE OR REPLACE FUNCTION get_cleaning_tasks_for_period(
    p_start_date DATE,
    p_end_date DATE
)
RETURNS TABLE (
    id INTEGER,
    booking_id INTEGER,
    room_number VARCHAR,
    task_date DATE,
    checkout_time TIME,
    checkin_time TIME,
    priority VARCHAR,
    has_dog BOOLEAN,
    change_bedding BOOLEAN,
    guest_count INTEGER,
    guest_name VARCHAR,
    status VARCHAR,
    completed_at TIMESTAMP
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        ct.id,
        ct.booking_id,
        r.room_number,
        ct.task_date,
        ct.checkout_time,
        ct.checkin_time,
        ct.priority,
        ct.has_dog,
        ct.change_bedding,
        ct.guest_count,
        ct.guest_name,
        ct.status,
        ct.completed_at
    FROM cleaning_tasks ct
    JOIN rooms r ON r.id = ct.room_id
    WHERE ct.task_date BETWEEN p_start_date AND p_end_date
    ORDER BY ct.task_date, ct.priority DESC, r.room_number;
END;
$$ LANGUAGE plpgsql;

-- Get cleaning statistics
CREATE OR REPLACE FUNCTION get_cleaning_stats()
RETURNS TABLE (
    today INTEGER,
    tomorrow INTEGER,
    this_week INTEGER,
    total INTEGER
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        COUNT(*) FILTER (WHERE task_date = CURRENT_DATE)::INTEGER as today,
        COUNT(*) FILTER (WHERE task_date = CURRENT_DATE + INTERVAL '1 day')::INTEGER as tomorrow,
        COUNT(*) FILTER (WHERE task_date BETWEEN CURRENT_DATE AND CURRENT_DATE + INTERVAL '7 days')::INTEGER as this_week,
        COUNT(*)::INTEGER as total
    FROM cleaning_tasks
    WHERE status != 'cancelled';
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- 6. VERIFICATION
-- ============================================================================

-- Verify table was created
SELECT table_name, column_name, data_type
FROM information_schema.columns
WHERE table_name = 'cleaning_tasks'
ORDER BY ordinal_position;

-- Verify functions were created
SELECT proname FROM pg_proc
WHERE proname IN ('generate_cleaning_task_for_booking', 'delete_cleaning_task_on_cancel', 'get_cleaning_tasks_for_period', 'get_cleaning_stats');

-- Verify triggers were created
SELECT trigger_name, event_manipulation, event_object_table
FROM information_schema.triggers
WHERE event_object_table IN ('cleaning_tasks', 'bookings')
  AND trigger_name LIKE '%cleaning%';

-- ============================================================================
-- 7. GENERATE TASKS FOR EXISTING BOOKINGS (One-time)
-- ============================================================================

-- Uncomment to generate tasks for all existing future bookings:
/*
INSERT INTO cleaning_tasks (
    booking_id, room_id, task_date, checkout_time, priority,
    has_dog, change_bedding, guest_count, guest_name, status
)
SELECT
    b.id,
    b.room_id,
    b.checkout_date::date,
    NULL, -- checkout_time
    'NORMAL',
    EXISTS(SELECT 1 FROM additional_services WHERE booking_id = b.id AND service_name ILIKE '%hund%'),
    TRUE,
    1 + (SELECT COUNT(*) FROM accompanying_guests WHERE booking_id = b.id),
    (SELECT CONCAT(vorname, ' ', nachname) FROM guests WHERE id = b.guest_id),
    'pending'
FROM bookings b
WHERE b.status NOT IN ('cancelled', 'storniert')
  AND b.checkout_date >= CURRENT_DATE
ON CONFLICT DO NOTHING;
*/
