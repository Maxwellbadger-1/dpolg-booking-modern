import psycopg2

conn = psycopg2.connect('postgresql://dpolg_admin:DPolG2025SecureBooking@141.147.3.123:5432/dpolg_booking')
cur = conn.cursor()

cur.execute("""
    SELECT pg_get_functiondef(oid)
    FROM pg_proc
    WHERE proname = 'notify_table_change'
""")

result = cur.fetchone()
if result:
    print('notify_table_change() Funktion gefunden:')
    print('=' * 60)
    print(result[0])
    print('=' * 60)

    # Check if 'reminders' is in the function
    if 'reminders' in result[0]:
        print('\nOK: Function enthaelt reminders Case!')
    else:
        print('\nFEHLER: Function enthaelt KEINEN reminders Case!')
        print('Die Funktion muss aktualisiert werden!')
else:
    print('FEHLER: notify_table_change() Funktion nicht gefunden!')

cur.close()
conn.close()
