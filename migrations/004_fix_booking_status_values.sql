-- Migration 004: Fix Booking Status Values
-- Migrate old SQLite status values to new PostgreSQL standard values
-- Old: 'gebucht', 'aktiv', 'abgeschlossen'
-- New: 'reserviert', 'bestaetigt', 'eingecheckt', 'ausgecheckt', 'storniert'

-- Update old status values to new ones
UPDATE bookings
SET status = 'reserviert'
WHERE status = 'gebucht';

UPDATE bookings
SET status = 'eingecheckt'
WHERE status = 'aktiv';

UPDATE bookings
SET status = 'ausgecheckt'
WHERE status = 'abgeschlossen';

-- Add CHECK constraint to ensure only valid status values
-- First drop any existing constraint if it exists
ALTER TABLE bookings DROP CONSTRAINT IF EXISTS bookings_status_check;

-- Add new constraint with correct values
ALTER TABLE bookings
ADD CONSTRAINT bookings_status_check
CHECK (status IN ('reserviert', 'bestaetigt', 'eingecheckt', 'ausgecheckt', 'storniert'));

-- Log migration completion
DO $$
BEGIN
    RAISE NOTICE '✅ Migration 004 completed: Booking status values fixed';
END $$;
