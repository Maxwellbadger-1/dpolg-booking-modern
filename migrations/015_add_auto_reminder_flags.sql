-- Migration 015: Add auto-reminder flags to notification_settings
-- These flags control which reminder types are automatically created

-- Add 4 auto-reminder boolean columns
ALTER TABLE notification_settings
ADD COLUMN IF NOT EXISTS auto_reminder_incomplete_data BOOLEAN DEFAULT true,
ADD COLUMN IF NOT EXISTS auto_reminder_payment BOOLEAN DEFAULT true,
ADD COLUMN IF NOT EXISTS auto_reminder_checkin BOOLEAN DEFAULT true,
ADD COLUMN IF NOT EXISTS auto_reminder_invoice BOOLEAN DEFAULT true;

-- Set defaults for existing row (singleton table, id=1)
UPDATE notification_settings
SET
  auto_reminder_incomplete_data = COALESCE(auto_reminder_incomplete_data, true),
  auto_reminder_payment = COALESCE(auto_reminder_payment, true),
  auto_reminder_checkin = COALESCE(auto_reminder_checkin, true),
  auto_reminder_invoice = COALESCE(auto_reminder_invoice, true)
WHERE id = 1;

-- Add comments
COMMENT ON COLUMN notification_settings.auto_reminder_incomplete_data IS 'Auto-create reminder when guest data incomplete';
COMMENT ON COLUMN notification_settings.auto_reminder_payment IS 'Auto-create reminder for unpaid bookings (7 days before check-in)';
COMMENT ON COLUMN notification_settings.auto_reminder_checkin IS 'Auto-create reminder for check-in preparation (1 day before)';
COMMENT ON COLUMN notification_settings.auto_reminder_invoice IS 'Auto-create reminder to send invoice after booking confirmation';
