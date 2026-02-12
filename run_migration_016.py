#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Migration 016: Auto-create reminders for bookings
- Adds payment_reminder_before_days column to notification_settings
- Creates auto_create_reminders_for_booking() function and trigger
- Creates auto_complete_reminders_on_booking_update() function and trigger
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
    """Execute migration 016"""
    print("[Migration 016] Starting: Auto-create reminders for bookings")
    print("=" * 80)

    try:
        # Connect to database
        conn = psycopg2.connect(**DB_CONFIG)
        conn.autocommit = False  # Use transaction
        cur = conn.cursor()

        # Read migration file
        migration_file = Path(__file__).parent / 'migrations' / '016_auto_create_reminders.sql'
        if not migration_file.exists():
            raise FileNotFoundError(f"Migration file not found: {migration_file}")

        migration_sql = migration_file.read_text(encoding='utf-8')
        print(f"[OK] Loaded migration from: {migration_file}")
        print()

        # Execute migration
        print("[...] Executing migration...")
        cur.execute(migration_sql)

        # PART 1: Verify column was added
        print("\n[PART 1] Verifying payment_reminder_before_days column...")
        cur.execute("""
            SELECT column_name, data_type, column_default
            FROM information_schema.columns
            WHERE table_name = 'notification_settings'
            AND column_name = 'payment_reminder_before_days';
        """)

        column = cur.fetchone()
        if column:
            col_name, data_type, default = column
            print(f"   [OK] Column added: {col_name} ({data_type}, default: {default})")
        else:
            raise Exception("Column payment_reminder_before_days not found!")

        # Check current value
        cur.execute("""
            SELECT payment_reminder_before_days
            FROM notification_settings
            WHERE id = 1;
        """)
        row = cur.fetchone()
        if row:
            print(f"   [OK] Current value: {row[0]} days")

        # PART 2: Verify auto-creation function and trigger
        print("\n[PART 2] Verifying auto-creation function and trigger...")
        cur.execute("""
            SELECT p.proname, pg_get_functiondef(p.oid)::text
            FROM pg_proc p
            JOIN pg_namespace n ON p.pronamespace = n.oid
            WHERE n.nspname = 'public'
            AND p.proname = 'auto_create_reminders_for_booking';
        """)
        func = cur.fetchone()
        if func:
            print(f"   [OK] Function created: {func[0]}()")
        else:
            raise Exception("Function auto_create_reminders_for_booking not found!")

        cur.execute("""
            SELECT tgname, tgrelid::regclass::text, tgtype
            FROM pg_trigger
            WHERE tgname = 'trg_auto_create_reminders';
        """)
        trigger = cur.fetchone()
        if trigger:
            print(f"   [OK] Trigger created: {trigger[0]} on {trigger[1]}")
        else:
            raise Exception("Trigger trg_auto_create_reminders not found!")

        # PART 3: Verify auto-completion function and trigger
        print("\n[PART 3] Verifying auto-completion function and trigger...")
        cur.execute("""
            SELECT p.proname
            FROM pg_proc p
            JOIN pg_namespace n ON p.pronamespace = n.oid
            WHERE n.nspname = 'public'
            AND p.proname = 'auto_complete_reminders_on_booking_update';
        """)
        func = cur.fetchone()
        if func:
            print(f"   [OK] Function created: {func[0]}()")
        else:
            raise Exception("Function auto_complete_reminders_on_booking_update not found!")

        cur.execute("""
            SELECT tgname, tgrelid::regclass::text
            FROM pg_trigger
            WHERE tgname = 'trg_auto_complete_reminders';
        """)
        trigger = cur.fetchone()
        if trigger:
            print(f"   [OK] Trigger created: {trigger[0]} on {trigger[1]}")
        else:
            raise Exception("Trigger trg_auto_complete_reminders not found!")

        # Commit transaction
        conn.commit()
        print("\n" + "=" * 80)
        print("[SUCCESS] Migration 016 completed successfully!")
        print("\nNext steps:")
        print("   1. Restart the Tauri app to load new database changes")
        print("   2. Go to Settings → Benachrichtigungen")
        print("   3. Enable auto-reminders and configure 'payment_reminder_before_days'")
        print("   4. Create a new booking to test auto-reminder creation")
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
