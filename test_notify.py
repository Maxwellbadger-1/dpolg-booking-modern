import psycopg2
import json
import time

conn = psycopg2.connect('postgresql://dpolg_admin:DPolG2025SecureBooking@141.147.3.123:5432/dpolg_booking')
cur = conn.cursor()

# Send a manual NOTIFY on reminder_changes channel
notification = {
    'table': 'reminders',
    'action': 'UPDATE',
    'id': 999,
    'timestamp': '2026-02-12T19:00:00Z'
}

print('Sende manuelles NOTIFY auf reminder_changes Channel...')
print(f'Payload: {json.dumps(notification)}')

cur.execute("SELECT pg_notify('reminder_changes', %s)", (json.dumps(notification),))
conn.commit()

print('\nNOTIFY gesendet!')
print('Pruefe jetzt deine App Console - sollte zeigen:')
print('  - [DEBUG PgListener] REMINDER EVENT DETECTED!')
print('  - [Real-Time] UPDATE Reminder #999')

cur.close()
conn.close()
