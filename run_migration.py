#!/usr/bin/env python3
"""
Run database migration: Add audit trail columns
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
with open('migrations/010_audit_trail_columns.sql', 'r', encoding='utf-8') as f:
    migration_sql = f.read()

print("Connecting to database...")
try:
    conn = psycopg2.connect(**DB_CONFIG)
    conn.autocommit = True
    cursor = conn.cursor()

    print("Connected to database")
    print("Executing migration...")

    # Execute migration
    cursor.execute(migration_sql)

    print("Migration executed successfully!")

    # Verify columns were added
    print("\nVerifying migration...")
    cursor.execute("""
        SELECT column_name, data_type
        FROM information_schema.columns
        WHERE table_name IN ('bookings', 'guests', 'rooms')
          AND column_name IN ('created_by', 'updated_by')
        ORDER BY table_name, column_name;
    """)

    results = cursor.fetchall()
    print(f"\nFound {len(results)} audit columns:")
    for col_name, col_type in results:
        print(f"   - {col_name}: {col_type}")

    # Verify indexes
    cursor.execute("""
        SELECT indexname
        FROM pg_indexes
        WHERE indexname LIKE 'idx_%_created_by'
           OR indexname LIKE 'idx_%_updated_by'
        ORDER BY indexname;
    """)

    indexes = cursor.fetchall()
    print(f"\nFound {len(indexes)} audit indexes:")
    for idx in indexes:
        print(f"   - {idx[0]}")

    cursor.close()
    conn.close()

    print("\nMigration completed successfully!")

except psycopg2.Error as e:
    print(f"Database error: {e}")
    sys.exit(1)
except Exception as e:
    print(f"Error: {e}")
    sys.exit(1)
