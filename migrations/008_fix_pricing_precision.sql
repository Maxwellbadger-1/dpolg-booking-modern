-- Migration 008: Fix pricing precision (f32 -> f64)
-- Problem: additional_services and discounts use 'real' (f32) causing precision loss
-- Solution: Convert to 'double precision' (f64) for consistency with all other price fields

-- Additional Services: service_price and original_value
ALTER TABLE additional_services
    ALTER COLUMN service_price TYPE double precision,
    ALTER COLUMN original_value TYPE double precision;

-- Discounts: discount_value
ALTER TABLE discounts
    ALTER COLUMN discount_value TYPE double precision;

-- Booking Attributes: price_modifier_value (if exists)
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'booking_attributes'
        AND column_name = 'price_modifier_value'
    ) THEN
        ALTER TABLE booking_attributes
            ALTER COLUMN price_modifier_value TYPE double precision;
    END IF;
END $$;

-- Verify changes
DO $$
DECLARE
    col_type text;
BEGIN
    SELECT data_type INTO col_type
    FROM information_schema.columns
    WHERE table_name = 'additional_services' AND column_name = 'service_price';

    IF col_type != 'double precision' THEN
        RAISE EXCEPTION 'Migration failed: additional_services.service_price is still %', col_type;
    END IF;

    RAISE NOTICE 'Migration 008 completed: All pricing columns now use double precision';
END $$;
