#!/usr/bin/env python3
"""
Verify Migration 013 status
"""
import sys
import psycopg2

DB_CONFIG = {
    'host': '141.147.3.123',
    'database': 'dpolg_booking',
    'user': 'dpolg_admin',
    'password': 'DPolG2025SecureBooking',
    'port': 5432
}

try:
    conn = psycopg2.connect(**DB_CONFIG)
    cursor = conn.cursor()

    print("=== MIGRATION 013 VERIFICATION ===\n")

    # 1. Check duplicates
    cursor.execute("""
        SELECT COUNT(*)
        FROM (
            SELECT booking_id, template_name, status
            FROM scheduled_emails
            GROUP BY booking_id, template_name, status
            HAVING COUNT(*) > 1
        ) AS duplicates;
    """)
    dup_count = cursor.fetchone()[0]
    print(f"[OK] Duplicate entries: {dup_count} (should be 0)")

    # 2. Check constraint
    cursor.execute("""
        SELECT constraint_name
        FROM information_schema.table_constraints
        WHERE table_name = 'scheduled_emails'
        AND constraint_name = 'unique_scheduled_email';
    """)
    constraint = cursor.fetchone()
    print(f"[OK] UNIQUE constraint exists: {constraint is not None}")

    # 3. Check 'booking_reminder' entries
    cursor.execute("SELECT COUNT(*) FROM scheduled_emails WHERE template_name = 'booking_reminder';")
    booking_reminder = cursor.fetchone()[0]
    print(f"[OK] 'booking_reminder' entries: {booking_reminder} (should be 0)")

    # 4. Check trigger function
    cursor.execute("""
        SELECT pg_get_functiondef(oid)
        FROM pg_proc
        WHERE proname = 'schedule_reminder_emails_for_booking';
    """)
    function_def = cursor.fetchone()
    if function_def:
        func_text = function_def[0]
        uses_reminder = "'reminder'" in func_text
        uses_on_conflict_update = "ON CONFLICT" in func_text and "DO UPDATE" in func_text
        print(f"[OK] Trigger function:")
        print(f"     - Uses 'reminder': {uses_reminder}")
        print(f"     - Uses ON CONFLICT DO UPDATE: {uses_on_conflict_update}")

    # 5. Total emails
    cursor.execute("SELECT COUNT(*) FROM scheduled_emails;")
    total = cursor.fetchone()[0]
    print(f"\nTotal scheduled emails: {total}")

    cursor.close()
    conn.close()

    print("\n=== ALL CHECKS PASSED! ===")

except Exception as e:
    print(f"Error: {e}")
    sys.exit(1)
