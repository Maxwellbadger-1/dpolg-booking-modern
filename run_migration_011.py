#!/usr/bin/env python3
"""
Run database migration 011: Add active_locks table (Presence System)
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
with open('migrations/011_active_locks_table.sql', 'r', encoding='utf-8') as f:
    migration_sql = f.read()

print("Connecting to database...")
try:
    conn = psycopg2.connect(**DB_CONFIG)
    conn.autocommit = True
    cursor = conn.cursor()

    print("Connected to database")
    print("Executing migration 011 (active_locks table)...")

    # Execute migration
    cursor.execute(migration_sql)

    print("Migration executed successfully!")

    # Verify table was created
    print("\nVerifying migration...")
    cursor.execute("""
        SELECT column_name, data_type
        FROM information_schema.columns
        WHERE table_name = 'active_locks'
        ORDER BY ordinal_position;
    """)

    results = cursor.fetchall()
    print(f"\nFound {len(results)} columns in active_locks:")
    for col_name, col_type in results:
        print(f"   - {col_name}: {col_type}")

    # Verify indexes
    cursor.execute("""
        SELECT indexname
        FROM pg_indexes
        WHERE tablename = 'active_locks'
        ORDER BY indexname;
    """)

    indexes = cursor.fetchall()
    print(f"\nFound {len(indexes)} indexes:")
    for idx in indexes:
        print(f"   - {idx[0]}")

    # Verify functions
    cursor.execute("""
        SELECT proname
        FROM pg_proc
        WHERE proname IN ('cleanup_stale_locks', 'notify_lock_change')
        ORDER BY proname;
    """)

    functions = cursor.fetchall()
    print(f"\nFound {len(functions)} functions:")
    for func in functions:
        print(f"   - {func[0]}()")

    # Verify triggers
    cursor.execute("""
        SELECT tgname
        FROM pg_trigger
        WHERE tgname = 'trigger_lock_changes';
    """)

    triggers = cursor.fetchall()
    print(f"\nFound {len(triggers)} triggers:")
    for trig in triggers:
        print(f"   - {trig[0]}")

    cursor.close()
    conn.close()

    print("\nMigration 011 completed successfully!")
    print("Lock changes will now be broadcast via PostgreSQL NOTIFY")

except psycopg2.Error as e:
    print(f"Database error: {e}")
    sys.exit(1)
except Exception as e:
    print(f"Error: {e}")
    sys.exit(1)
