-- Migration 014: Add checkin_reminder_before_days column
-- This column was referenced in migration 013 but never created
-- It controls how many days before check-in the reminder email is sent

-- Add the missing column with default value of 3 days
ALTER TABLE notification_settings
ADD COLUMN IF NOT EXISTS checkin_reminder_before_days INTEGER DEFAULT 3;

-- Set default value for existing row (there should only be one settings row)
UPDATE notification_settings
SET checkin_reminder_before_days = 3
WHERE checkin_reminder_before_days IS NULL;

-- Add comment for documentation
COMMENT ON COLUMN notification_settings.checkin_reminder_before_days IS
'Number of days before check-in to send reminder email. Default: 3 days.';
