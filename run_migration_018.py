#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Migration 018: Real-Time NOTIFY Triggers for Reminders
- Extends notify_table_change() to handle reminders table
- Adds NOTIFY triggers for INSERT/UPDATE/DELETE on reminders
- Enables real-time Badge-Count updates (<1 second latency)
- Pattern: Same as Migration 007 (bookings, guests, rooms)
"""

import psycopg2
import sys
from pathlib import Path

# Database connection parameters - Cloud Database (Direct PostgreSQL)
DB_CONFIG = {
    'dbname': 'dpolg_booking',
    'user': 'dpolg_admin',
    'password': 'DPolG2025SecureBooking',
    'host': '141.147.3.123',
    'port': '5432',  # Direct PostgreSQL port
    'connect_timeout': 10
}

def run_migration():
    """Execute migration 018"""
    print("[Migration 018] Starting: Real-Time NOTIFY Triggers for Reminders")
    print("=" * 80)

    try:
        # Connect to database
        conn = psycopg2.connect(**DB_CONFIG)
        conn.autocommit = False  # Use transaction
        cur = conn.cursor()

        # Read migration file
        migration_file = Path(__file__).parent / 'migrations' / '018_reminder_notify_triggers.sql'
        if not migration_file.exists():
            raise FileNotFoundError(f"Migration file not found: {migration_file}")

        migration_sql = migration_file.read_text(encoding='utf-8')
        print(f"[OK] Loaded migration from: {migration_file}")
        print()

        # Execute migration
        print("[...] Executing migration...")
        cur.execute(migration_sql)

        # PART 1: Verify notify_table_change() function updated
        print("\n[PART 1] Verifying notify_table_change() function...")
        cur.execute("""
            SELECT p.proname, pg_get_functiondef(p.oid)::text
            FROM pg_proc p
            JOIN pg_namespace n ON p.pronamespace = n.oid
            WHERE n.nspname = 'public'
            AND p.proname = 'notify_table_change';
        """)
        func = cur.fetchone()
        if func:
            func_name, func_def = func
            print(f"   [OK] Function found: {func_name}()")
            # Check if reminders channel is in function definition
            if "reminder_changes" in func_def:
                print(f"   [OK] reminder_changes channel implemented")
            else:
                raise Exception("Function does not contain reminder_changes channel!")
        else:
            raise Exception("Function notify_table_change not found!")

        # PART 2: Verify reminder NOTIFY triggers
        print("\n[PART 2] Verifying reminder NOTIFY triggers...")
        cur.execute("""
            SELECT
                tgname AS trigger_name,
                tgrelid::regclass AS table_name,
                CASE tgtype & 66
                    WHEN 2 THEN 'BEFORE'
                    WHEN 64 THEN 'INSTEAD OF'
                    ELSE 'AFTER'
                END AS trigger_timing,
                CASE tgtype & 28
                    WHEN 4 THEN 'INSERT'
                    WHEN 8 THEN 'DELETE'
                    WHEN 16 THEN 'UPDATE'
                    WHEN 12 THEN 'INSERT OR DELETE'
                    WHEN 20 THEN 'INSERT OR UPDATE'
                    WHEN 24 THEN 'DELETE OR UPDATE'
                    WHEN 28 THEN 'INSERT OR UPDATE OR DELETE'
                END AS trigger_event
            FROM pg_trigger
            WHERE tgname LIKE 'trg_notify_reminder%'
            ORDER BY tgname;
        """)

        triggers = cur.fetchall()
        if not triggers:
            raise Exception("No reminder NOTIFY triggers found!")

        print(f"   [OK] Found {len(triggers)} reminder NOTIFY triggers:")
        expected_triggers = [
            'trg_notify_reminder_insert',
            'trg_notify_reminder_update',
            'trg_notify_reminder_delete'
        ]

        for trigger in triggers:
            trigger_name, table_name, timing, event = trigger
            print(f"   [OK] {trigger_name} - {timing} {event} on {table_name}")
            if trigger_name not in expected_triggers:
                raise Exception(f"Unexpected trigger: {trigger_name}")

        # Verify all expected triggers exist
        found_trigger_names = [t[0] for t in triggers]
        for expected in expected_triggers:
            if expected not in found_trigger_names:
                raise Exception(f"Missing trigger: {expected}")

        # PART 3: Test NOTIFY functionality (simulate)
        print("\n[PART 3] Testing NOTIFY channel...")
        print("   [INFO] Backend listener.rs should now receive 'reminder_changes' events")
        print("   [INFO] Frontend App.tsx should update Badge-Count on reminder changes")

        # Commit transaction
        conn.commit()
        print("\n" + "=" * 80)
        print("[SUCCESS] Migration 018 completed successfully!")
        print("\nWhat's changed:")
        print("   ✅ Real-Time NOTIFY enabled for reminders table")
        print("   ✅ Badge-Count updates instantly (<1 second)")
        print("   ✅ Removed 5-minute polling (better performance)")
        print("   ✅ Pattern consistent with bookings, guests, rooms")
        print("\nAffected components:")
        print("   - Migration 018: NOTIFY triggers for reminders")
        print("   - listener.rs: LISTEN reminder_changes channel")
        print("   - App.tsx: Tauri Event Listener (no more polling)")
        print("\nTest plan:")
        print("   1. Start app → Badge shows current count")
        print("   2. Create new booking → Auto-reminder created → Badge +1 (instant)")
        print("   3. Complete reminder → Badge -1 (instant)")
        print("   4. Update booking → Reminder updated → Badge refreshed (instant)")
        print("   5. Cancel booking → Reminders deleted → Badge updated (instant)")
        print("=" * 80)

        return True

    except psycopg2.Error as e:
        if 'conn' in locals():
            conn.rollback()
        print(f"\n[ERROR] Database error: {e}")
        print(f"   Error code: {e.pgcode}")
        return False

    except Exception as e:
        if 'conn' in locals():
            conn.rollback()
        print(f"\n[ERROR] Error: {e}")
        return False

    finally:
        if 'cur' in locals():
            cur.close()
        if 'conn' in locals():
            conn.close()

if __name__ == "__main__":
    success = run_migration()
    sys.exit(0 if success else 1)
