-- Migration 009: Store logo as Base64 data in DB instead of file path
-- This allows multi-machine setups to share the logo via PostgreSQL

DO $$
BEGIN
    -- Add logo_data column (Base64-encoded image data)
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'company_settings' AND column_name = 'logo_data'
    ) THEN
        ALTER TABLE company_settings ADD COLUMN logo_data TEXT;
    END IF;

    -- Add logo_mime_type column
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'company_settings' AND column_name = 'logo_mime_type'
    ) THEN
        ALTER TABLE company_settings ADD COLUMN logo_mime_type VARCHAR(50);
    END IF;
END $$;
