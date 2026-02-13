-- Migration: Guest Credit System
-- Date: 2025-11-22
-- Description: Implements complete guest credit/gift card transaction system

-- ============================================================================
-- 1. GUEST CREDIT TRANSACTIONS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS guest_credit_transactions (
    id SERIAL PRIMARY KEY,
    guest_id INTEGER NOT NULL,
    booking_id INTEGER,
    amount NUMERIC(10, 2) NOT NULL,
    transaction_type VARCHAR(20) NOT NULL CHECK (transaction_type IN ('credit', 'debit', 'refund')),
    description TEXT,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_by VARCHAR(100),

    -- Foreign Keys
    CONSTRAINT fk_guest_credit_guest
        FOREIGN KEY (guest_id) REFERENCES guests(id) ON DELETE CASCADE,
    CONSTRAINT fk_guest_credit_booking
        FOREIGN KEY (booking_id) REFERENCES bookings(id) ON DELETE SET NULL,

    -- Constraints
    CONSTRAINT chk_amount_positive
        CHECK (
            (transaction_type = 'credit' AND amount > 0) OR
            (transaction_type = 'debit' AND amount > 0) OR
            (transaction_type = 'refund' AND amount > 0)
        )
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_guest_credit_guest_id ON guest_credit_transactions(guest_id);
CREATE INDEX IF NOT EXISTS idx_guest_credit_booking_id ON guest_credit_transactions(booking_id);
CREATE INDEX IF NOT EXISTS idx_guest_credit_created_at ON guest_credit_transactions(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_guest_credit_type ON guest_credit_transactions(transaction_type);

-- Comment
COMMENT ON TABLE guest_credit_transactions IS 'Tracks all guest credit/gift card transactions';
COMMENT ON COLUMN guest_credit_transactions.transaction_type IS 'credit=add funds, debit=use funds, refund=return funds';
COMMENT ON COLUMN guest_credit_transactions.amount IS 'Always positive - direction is determined by transaction_type';

-- ============================================================================
-- 2. HELPER FUNCTION: Get Guest Credit Balance
-- ============================================================================

CREATE OR REPLACE FUNCTION get_guest_credit_balance(p_guest_id INTEGER)
RETURNS NUMERIC(10, 2) AS $$
DECLARE
    v_balance NUMERIC(10, 2);
BEGIN
    SELECT
        COALESCE(
            SUM(CASE
                WHEN transaction_type = 'credit' THEN amount
                WHEN transaction_type = 'debit' THEN -amount
                WHEN transaction_type = 'refund' THEN amount
                ELSE 0
            END),
            0.0
        )
    INTO v_balance
    FROM guest_credit_transactions
    WHERE guest_id = p_guest_id;

    RETURN v_balance;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION get_guest_credit_balance IS 'Calculates current credit balance for a guest';

-- ============================================================================
-- 3. HELPER FUNCTION: Get Total Credit Used for Booking
-- ============================================================================

CREATE OR REPLACE FUNCTION get_booking_credit_usage(p_booking_id INTEGER)
RETURNS NUMERIC(10, 2) AS $$
DECLARE
    v_total NUMERIC(10, 2);
BEGIN
    SELECT COALESCE(SUM(amount), 0.0)
    INTO v_total
    FROM guest_credit_transactions
    WHERE booking_id = p_booking_id AND transaction_type = 'debit';

    RETURN v_total;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION get_booking_credit_usage IS 'Returns total credit used for a specific booking';

-- ============================================================================
-- 4. TRIGGER: Validate Credit Balance Before Debit
-- ============================================================================

CREATE OR REPLACE FUNCTION validate_credit_balance()
RETURNS TRIGGER AS $$
DECLARE
    v_current_balance NUMERIC(10, 2);
BEGIN
    -- Only validate for debit transactions
    IF NEW.transaction_type = 'debit' THEN
        -- Get current balance
        v_current_balance := get_guest_credit_balance(NEW.guest_id);

        -- Check if sufficient balance exists
        IF v_current_balance < NEW.amount THEN
            RAISE EXCEPTION 'Insufficient credit balance. Available: %, Required: %',
                v_current_balance, NEW.amount;
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_validate_credit_balance
    BEFORE INSERT ON guest_credit_transactions
    FOR EACH ROW
    EXECUTE FUNCTION validate_credit_balance();

COMMENT ON FUNCTION validate_credit_balance IS 'Prevents debit transactions that would result in negative balance';

-- ============================================================================
-- 5. SYNC bookings.credit_used with transactions
-- ============================================================================

CREATE OR REPLACE FUNCTION sync_booking_credit_used()
RETURNS TRIGGER AS $$
BEGIN
    -- Update bookings.credit_used when transaction relates to a booking
    IF NEW.booking_id IS NOT NULL AND NEW.transaction_type = 'debit' THEN
        UPDATE bookings
        SET credit_used = get_booking_credit_usage(NEW.booking_id)
        WHERE id = NEW.booking_id;
    END IF;

    -- Handle deletions/refunds
    IF TG_OP = 'DELETE' AND OLD.booking_id IS NOT NULL THEN
        UPDATE bookings
        SET credit_used = get_booking_credit_usage(OLD.booking_id)
        WHERE id = OLD.booking_id;
    END IF;

    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_sync_booking_credit_used
    AFTER INSERT OR DELETE ON guest_credit_transactions
    FOR EACH ROW
    EXECUTE FUNCTION sync_booking_credit_used();

COMMENT ON FUNCTION sync_booking_credit_used IS 'Automatically updates bookings.credit_used when transactions change';

-- ============================================================================
-- 6. VERIFICATION QUERIES
-- ============================================================================

-- Verify table was created
SELECT table_name, column_name, data_type
FROM information_schema.columns
WHERE table_name = 'guest_credit_transactions'
ORDER BY ordinal_position;

-- Verify functions were created
SELECT proname, pg_get_functiondef(oid) as definition
FROM pg_proc
WHERE proname IN ('get_guest_credit_balance', 'get_booking_credit_usage', 'validate_credit_balance', 'sync_booking_credit_used');

-- Verify triggers were created
SELECT trigger_name, event_manipulation, event_object_table
FROM information_schema.triggers
WHERE trigger_name IN ('trg_validate_credit_balance', 'trg_sync_booking_credit_used');

-- ============================================================================
-- 7. SAMPLE DATA (for testing)
-- ============================================================================

/*
-- Example: Add 100€ credit to guest 1
INSERT INTO guest_credit_transactions (guest_id, amount, transaction_type, description)
VALUES (1, 100.00, 'credit', 'Gift card purchase');

-- Example: Use 50€ for booking 123
INSERT INTO guest_credit_transactions (guest_id, booking_id, amount, transaction_type, description)
VALUES (1, 123, 50.00, 'debit', 'Applied to booking #123');

-- Example: Check balance
SELECT get_guest_credit_balance(1); -- Should return 50.00
*/
