-- Migration: Add audit trail columns (created_by, updated_by) to main tables
-- Purpose: Track which user created or last modified bookings, guests, and rooms
-- Date: 2026-02-11

-- Add audit columns to bookings table
ALTER TABLE bookings ADD COLUMN created_by VARCHAR(100);
ALTER TABLE bookings ADD COLUMN updated_by VARCHAR(100);

-- Add audit columns to guests table
ALTER TABLE guests ADD COLUMN created_by VARCHAR(100);
ALTER TABLE guests ADD COLUMN updated_by VARCHAR(100);

-- Add audit columns to rooms table
ALTER TABLE rooms ADD COLUMN created_by VARCHAR(100);
ALTER TABLE rooms ADD COLUMN updated_by VARCHAR(100);

-- Create indexes for performance (useful for filtering/reporting by user)
CREATE INDEX idx_bookings_created_by ON bookings(created_by);
CREATE INDEX idx_bookings_updated_by ON bookings(updated_by);

CREATE INDEX idx_guests_created_by ON guests(created_by);
CREATE INDEX idx_guests_updated_by ON guests(updated_by);

CREATE INDEX idx_rooms_created_by ON rooms(created_by);
CREATE INDEX idx_rooms_updated_by ON rooms(updated_by);

-- Note: Columns are nullable to avoid breaking existing data
-- Existing records will have NULL for these fields
