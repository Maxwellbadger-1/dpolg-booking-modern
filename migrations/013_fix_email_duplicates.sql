-- Migration 013: Fix E-Mail Duplikate
-- Fügt Unique Constraint hinzu und bereinigt existierende Duplikate
-- Fixes: Doppelte Einträge in scheduled_emails + Template-Fehler 'booking_reminder'

BEGIN;

-- ============================================================================
-- 1. DUPLIKATE BEREINIGEN
-- ============================================================================

-- Lösche Duplikate (behalte nur den ältesten Eintrag pro Kombination)
DELETE FROM scheduled_emails
WHERE id NOT IN (
    SELECT MIN(id)
    FROM scheduled_emails
    GROUP BY booking_id, template_name, status
);

-- ============================================================================
-- 2. UNIQUE CONSTRAINT HINZUFÜGEN
-- ============================================================================

-- Füge Unique Constraint hinzu um zukünftige Duplikate zu verhindern
ALTER TABLE scheduled_emails
ADD CONSTRAINT unique_scheduled_email
UNIQUE (booking_id, template_name, status);

-- ============================================================================
-- 3. VERWAISTE EINTRÄGE LÖSCHEN
-- ============================================================================

-- Lösche verwaiste 'booking_reminder' Einträge (Template existiert nicht)
-- Das korrekte Template heißt 'reminder'
DELETE FROM scheduled_emails
WHERE template_name = 'booking_reminder';

-- ============================================================================
-- 4. TRIGGER KORRIGIEREN
-- ============================================================================

-- Update Trigger: Korrigiere Template-Namen und verwende ON CONFLICT DO UPDATE
CREATE OR REPLACE FUNCTION schedule_reminder_emails_for_booking()
RETURNS TRIGGER AS $$
DECLARE
    v_reminder_days INTEGER;
    v_guest_email VARCHAR;
    v_reminder_enabled BOOLEAN;
    reminder_date TIMESTAMP;
BEGIN
    -- Lade Notification Settings
    SELECT
        checkin_reminders_enabled,
        checkin_reminder_before_days,
        g.email
    INTO
        v_reminder_enabled,
        v_reminder_days,
        v_guest_email
    FROM notification_settings ns
    CROSS JOIN guests g
    WHERE g.id = NEW.guest_id
    LIMIT 1;

    -- Prüfe ob Check-in Erinnerungen aktiviert sind
    IF COALESCE(v_reminder_enabled, false) = false THEN
        RETURN NEW;
    END IF;

    -- Prüfe ob Gast E-Mail hat
    IF v_guest_email IS NULL OR v_guest_email = '' THEN
        RETURN NEW;
    END IF;

    -- Berechne Erinnerungs-Datum (N Tage vor Check-in)
    reminder_date := (NEW.checkin_date::date - INTERVAL '1 day' * COALESCE(v_reminder_days, 3))::TIMESTAMP;

    -- Nur einfügen wenn in der Zukunft
    IF reminder_date > NOW() THEN
        -- INSERT mit ON CONFLICT DO UPDATE (statt DO NOTHING)
        -- Template 'reminder' statt 'booking_reminder' verwenden
        INSERT INTO scheduled_emails (
            booking_id,
            guest_id,
            template_name,
            recipient_email,
            subject,
            scheduled_for,
            status
        ) VALUES (
            NEW.id,
            NEW.guest_id,
            'reminder',  -- KORRIGIERT: 'reminder' statt 'booking_reminder'
            v_guest_email,
            format('Erinnerung: Ihre Buchung #%s', NEW.id),
            reminder_date,
            'pending'
        )
        ON CONFLICT (booking_id, template_name, status)
        DO UPDATE SET
            scheduled_for = EXCLUDED.scheduled_for,
            recipient_email = EXCLUDED.recipient_email,
            subject = EXCLUDED.subject;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Trigger muss nicht neu erstellt werden, da nur die Funktion geändert wurde

COMMIT;
