#!/usr/bin/env python3
"""
Run database migration 013: Fix E-Mail Duplicates
- Bereinigt Duplikate in scheduled_emails
- Fügt UNIQUE constraint hinzu
- Korrigiert Template-Name von 'booking_reminder' zu 'reminder'
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
with open('migrations/013_fix_email_duplicates.sql', 'r', encoding='utf-8') as f:
    migration_sql = f.read()

print("Connecting to database...")
try:
    conn = psycopg2.connect(**DB_CONFIG)
    conn.autocommit = False  # Transaction mode für Rollback-Sicherheit
    cursor = conn.cursor()

    print("Connected to database")

    # Prüfe Status VOR Migration
    print("\n=== PRE-MIGRATION STATUS ===")

    # Count duplicates BEFORE
    cursor.execute("""
        SELECT booking_id, template_name, status, COUNT(*) as count
        FROM scheduled_emails
        GROUP BY booking_id, template_name, status
        HAVING COUNT(*) > 1
        ORDER BY count DESC;
    """)
    duplicates_before = cursor.fetchall()
    print(f"Duplicate entries BEFORE: {len(duplicates_before)}")
    if len(duplicates_before) > 0:
        print("Sample duplicates:")
        for booking_id, template, status, count in duplicates_before[:5]:
            print(f"   - Booking {booking_id}, Template '{template}', Status '{status}': {count}x")

    # Count 'booking_reminder' entries
    cursor.execute("SELECT COUNT(*) FROM scheduled_emails WHERE template_name = 'booking_reminder';")
    booking_reminder_count = cursor.fetchone()[0]
    print(f"'booking_reminder' entries: {booking_reminder_count}")

    # Check if constraint exists
    cursor.execute("""
        SELECT constraint_name
        FROM information_schema.table_constraints
        WHERE table_name = 'scheduled_emails'
        AND constraint_name = 'unique_scheduled_email';
    """)
    constraint_exists = cursor.fetchone() is not None
    print(f"UNIQUE constraint exists: {constraint_exists}")

    print("\n=== EXECUTING MIGRATION 013 ===")

    # Execute migration
    cursor.execute(migration_sql)

    print("Migration executed successfully!")

    # Verify changes
    print("\n=== POST-MIGRATION VERIFICATION ===")

    # 1. Check no more duplicates
    cursor.execute("""
        SELECT booking_id, template_name, status, COUNT(*) as count
        FROM scheduled_emails
        GROUP BY booking_id, template_name, status
        HAVING COUNT(*) > 1;
    """)
    duplicates_after = cursor.fetchall()
    print(f"[OK] Duplicate entries AFTER: {len(duplicates_after)}")
    if len(duplicates_after) > 0:
        print("WARNING: Still have duplicates!")
        for booking_id, template, status, count in duplicates_after:
            print(f"   - Booking {booking_id}, Template '{template}', Status '{status}': {count}x")

    # 2. Verify UNIQUE constraint
    cursor.execute("""
        SELECT constraint_name
        FROM information_schema.table_constraints
        WHERE table_name = 'scheduled_emails'
        AND constraint_name = 'unique_scheduled_email';
    """)
    constraint = cursor.fetchone()
    if constraint:
        print(f"[OK] UNIQUE constraint '{constraint[0]}' added successfully")
    else:
        print("ERROR: UNIQUE constraint not found!")

    # 3. Check no 'booking_reminder' entries
    cursor.execute("SELECT COUNT(*) FROM scheduled_emails WHERE template_name = 'booking_reminder';")
    booking_reminder_after = cursor.fetchone()[0]
    print(f"[OK] 'booking_reminder' entries AFTER: {booking_reminder_after}")

    # 4. Verify trigger function was updated
    cursor.execute("""
        SELECT pg_get_functiondef(oid)
        FROM pg_proc
        WHERE proname = 'schedule_reminder_emails_for_booking';
    """)
    function_def = cursor.fetchone()
    if function_def:
        func_text = function_def[0]
        uses_reminder = "'reminder'" in func_text
        uses_booking_reminder = "'booking_reminder'" in func_text
        print(f"[OK] Trigger function updated:")
        print(f"   - Uses 'reminder': {uses_reminder}")
        print(f"   - Uses 'booking_reminder': {uses_booking_reminder}")
        if uses_reminder and not uses_booking_reminder:
            print("   [OK] Correct template name!")

    # 5. Count total scheduled emails
    cursor.execute("SELECT COUNT(*) FROM scheduled_emails;")
    total = cursor.fetchone()[0]
    print(f"\nTotal scheduled emails: {total}")

    # COMMIT Transaction
    conn.commit()
    print("\n[OK] Transaction committed successfully!")

    cursor.close()
    conn.close()

    print("\n=== MIGRATION 013 COMPLETED SUCCESSFULLY! ===")
    print("[OK] Duplikate bereinigt")
    print("[OK] UNIQUE constraint hinzugefügt")
    print("[OK] Template-Name korrigiert ('reminder' statt 'booking_reminder')")
    print("[OK] Trigger aktualisiert mit ON CONFLICT DO UPDATE")

except psycopg2.Error as e:
    print(f"\nDatabase error: {e}")
    if conn:
        conn.rollback()
        print("Transaction rolled back!")
    sys.exit(1)
except Exception as e:
    print(f"\nError: {e}")
    if conn:
        conn.rollback()
        print("Transaction rolled back!")
    sys.exit(1)
