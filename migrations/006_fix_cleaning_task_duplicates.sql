-- Migration: Fix Cleaning Task Duplicates
-- Date: 2025-11-27
-- Description: Adds UNIQUE constraint and fixes trigger to prevent duplicate cleaning tasks

-- ============================================================================
-- 1. ADD UNIQUE CONSTRAINT TO PREVENT DUPLICATES
-- ============================================================================

-- Add UNIQUE index on (booking_id, room_id, task_date)
-- This ensures one cleaning task per booking/room/date combination
CREATE UNIQUE INDEX IF NOT EXISTS idx_cleaning_tasks_unique_booking_room_date
    ON cleaning_tasks (booking_id, room_id, task_date);

COMMENT ON INDEX idx_cleaning_tasks_unique_booking_room_date IS
    'Prevents duplicate cleaning tasks for the same booking/room/date';

-- ============================================================================
-- 2. FIX TRIGGER FUNCTION TO USE UPSERT
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

    -- UPSERT: Insert or update existing cleaning task
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
    ON CONFLICT (booking_id, room_id, task_date)
    DO UPDATE SET
        checkout_time = EXCLUDED.checkout_time,
        priority = EXCLUDED.priority,
        has_dog = EXCLUDED.has_dog,
        change_bedding = EXCLUDED.change_bedding,
        guest_count = EXCLUDED.guest_count,
        guest_name = EXCLUDED.guest_name,
        updated_at = CURRENT_TIMESTAMP;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- 3. VERIFICATION
-- ============================================================================

-- Show any remaining duplicates (should be none after cleaning)
SELECT
    booking_id,
    room_id,
    task_date,
    COUNT(*) as count
FROM cleaning_tasks
GROUP BY booking_id, room_id, task_date
HAVING COUNT(*) > 1;
