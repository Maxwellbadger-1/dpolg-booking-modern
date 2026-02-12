import psycopg2

conn = psycopg2.connect('postgresql://dpolg_admin:DPolG2025SecureBooking@141.147.3.123:5432/dpolg_booking')
cur = conn.cursor()

print('Aktualisiere notify_table_change() Funktion...')

cur.execute("""
CREATE OR REPLACE FUNCTION notify_table_change()
RETURNS TRIGGER AS $$
DECLARE
    notification json;
BEGIN
    -- Build JSON notification payload
    notification = json_build_object(
        'table', TG_TABLE_NAME,
        'action', TG_OP,
        'id', COALESCE(NEW.id, OLD.id),
        'timestamp', NOW()
    );

    -- Send notification on appropriate channel
    IF TG_TABLE_NAME = 'bookings' THEN
        PERFORM pg_notify('booking_changes', notification::text);
    ELSIF TG_TABLE_NAME = 'guests' THEN
        PERFORM pg_notify('guest_changes', notification::text);
    ELSIF TG_TABLE_NAME = 'rooms' THEN
        PERFORM pg_notify('room_changes', notification::text);
    ELSIF TG_TABLE_NAME = 'reminders' THEN
        PERFORM pg_notify('reminder_changes', notification::text);
    ELSE
        PERFORM pg_notify('table_changes', notification::text);
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
""")

conn.commit()
print('OK: notify_table_change() Funktion erfolgreich aktualisiert!')
print('Der reminders Case wurde hinzugefuegt.')

cur.close()
conn.close()
