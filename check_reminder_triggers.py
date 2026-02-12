import psycopg2

conn = psycopg2.connect('postgresql://dpolg_admin:DPolG2025SecureBooking@141.147.3.123:5432/dpolg_booking')
cur = conn.cursor()

cur.execute("""
    SELECT tgname
    FROM pg_trigger
    WHERE tgname LIKE 'trg_notify_reminder%'
    ORDER BY tgname
""")

triggers = cur.fetchall()
if triggers:
    print('OK: Reminder NOTIFY Triggers gefunden:')
    for t in triggers:
        print(f'   - {t[0]}')
else:
    print('FEHLER: KEINE Reminder NOTIFY Triggers gefunden!')
    print('Migration 018 wurde noch nicht ausgefuehrt!')

cur.close()
conn.close()
