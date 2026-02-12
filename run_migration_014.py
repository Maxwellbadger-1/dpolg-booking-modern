#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Migration 014: Add checkin_reminder_before_days column to notification_settings
This column was referenced in migration 013 but never created.
"""
import psycopg2
import os
import sys

# Fix Windows console encoding
if sys.platform == "win32":
    sys.stdout.reconfigure(encoding='utf-8')

def run_migration():
    # Database connection parameters (Oracle Cloud Server)
    # Using direct connection (5432) instead of pgBouncer (6432) for DDL operations
    conn = psycopg2.connect(
        host="141.147.3.123",
        port=5432,  # Direct PostgreSQL connection
        database="dpolg_booking",
        user="dpolg_admin",
        password="DPolG2025SecureBooking"
    )

    try:
        with conn:
            with conn.cursor() as cur:
                print("[*] Starting Migration 014...")

                # Read migration file
                migration_path = os.path.join('migrations', '014_add_checkin_reminder_days.sql')
                with open(migration_path, 'r', encoding='utf-8') as f:
                    sql = f.read()

                # Execute migration
                cur.execute(sql)
                print("[OK] Migration 014 executed successfully")

                # Verify column exists
                cur.execute("""
                    SELECT column_name, data_type, column_default
                    FROM information_schema.columns
                    WHERE table_name = 'notification_settings'
                    AND column_name = 'checkin_reminder_before_days'
                """)
                result = cur.fetchone()

                if result:
                    print(f"[OK] Column verified:")
                    print(f"     - Name: {result[0]}")
                    print(f"     - Type: {result[1]}")
                    print(f"     - Default: {result[2]}")
                else:
                    print("[ERROR] Column not found after migration!")
                    return False

                # Check current value
                cur.execute("SELECT checkin_reminder_before_days FROM notification_settings LIMIT 1")
                value = cur.fetchone()
                if value:
                    print(f"[OK] Current value: {value[0]} days")
                else:
                    print("[WARN] No settings row found")

                print("\n[SUCCESS] Migration 014 completed successfully!")
                return True

    except Exception as e:
        print(f"[ERROR] Migration failed: {e}")
        import traceback
        traceback.print_exc()
        return False
    finally:
        conn.close()

if __name__ == "__main__":
    success = run_migration()
    exit(0 if success else 1)
