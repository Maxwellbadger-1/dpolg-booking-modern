#!/usr/bin/env python3
"""
Run database migration 012: Add audit_log table (Change History)
"""
import sys

try:
    import psycopg2
except ImportError:
    print("psycopg2 not installed. Installing...")
    import subprocess
    subprocess.check_call([sys.executable, "-m", "pip", "install", "psycopg2-binary"])
    import psycopg2

# Database connection parameters
DB_CONFIG = {
    'host': '141.147.3.123',
    'database': 'dpolg_booking',
    'user': 'dpolg_admin',
    'password': 'DPolG2025SecureBooking',
    'port': 5432
}

# Read migration SQL
with open('migrations/012_audit_log_table.sql', 'r', encoding='utf-8') as f:
    migration_sql = f.read()

print("Connecting to database...")
try:
    conn = psycopg2.connect(**DB_CONFIG)
    conn.autocommit = True
    cursor = conn.cursor()

    print("Connected to database")
    print("Executing migration 012 (audit_log table)...")

    # Execute migration
    cursor.execute(migration_sql)

    print("Migration executed successfully!")

    # Verify table was created
    print("\nVerifying migration...")
    cursor.execute("""
        SELECT column_name, data_type
        FROM information_schema.columns
        WHERE table_name = 'audit_log'
        ORDER BY ordinal_position;
    """)

    results = cursor.fetchall()
    print(f"\nFound {len(results)} columns in audit_log:")
    for col_name, col_type in results:
        print(f"   - {col_name}: {col_type}")

    # Verify indexes
    cursor.execute("""
        SELECT indexname
        FROM pg_indexes
        WHERE tablename = 'audit_log'
        ORDER BY indexname;
    """)

    indexes = cursor.fetchall()
    print(f"\nFound {len(indexes)} indexes:")
    for idx in indexes:
        print(f"   - {idx[0]}")

    # Verify trigger functions
    cursor.execute("""
        SELECT proname
        FROM pg_proc
        WHERE proname IN ('log_booking_changes', 'log_guest_changes', 'log_room_changes')
        ORDER BY proname;
    """)

    functions = cursor.fetchall()
    print(f"\nFound {len(functions)} audit trigger functions:")
    for func in functions:
        print(f"   - {func[0]}()")

    # Verify triggers are attached
    cursor.execute("""
        SELECT t.tgname, c.relname as table_name
        FROM pg_trigger t
        JOIN pg_class c ON t.tgrelid = c.oid
        WHERE t.tgname IN ('trigger_audit_bookings', 'trigger_audit_guests', 'trigger_audit_rooms')
        ORDER BY c.relname, t.tgname;
    """)

    triggers = cursor.fetchall()
    print(f"\nFound {len(triggers)} audit triggers:")
    for trig_name, table_name in triggers:
        print(f"   - {trig_name} on {table_name}")

    # Test: Count existing audit logs
    cursor.execute("SELECT COUNT(*) FROM audit_log;")
    count = cursor.fetchone()[0]
    print(f"\nCurrent audit log entries: {count}")

    cursor.close()
    conn.close()

    print("\nMigration 012 completed successfully!")
    print("All changes to bookings, guests, and rooms will now be logged")

except psycopg2.Error as e:
    print(f"Database error: {e}")
    sys.exit(1)
except Exception as e:
    print(f"Error: {e}")
    sys.exit(1)
