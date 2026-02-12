#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Check if Migration 015 columns already exist
"""

import psycopg2
import sys

# Database connection parameters - Cloud Database (Direct PostgreSQL)
DB_CONFIG = {
    'dbname': 'dpolg_booking',
    'user': 'dpolg_admin',
    'password': 'DPolG2025SecureBooking',
    'host': '141.147.3.123',
    'port': '5432',  # Direct PostgreSQL port
    'connect_timeout': 10
}

def check_columns():
    """Check if auto-reminder columns exist"""
    print("[Check] Checking Migration 015 status...")
    print("=" * 80)

    try:
        # Connect to database
        conn = psycopg2.connect(**DB_CONFIG)
        cur = conn.cursor()

        # Check if columns exist
        cur.execute("""
            SELECT column_name, data_type, column_default, is_nullable
            FROM information_schema.columns
            WHERE table_name = 'notification_settings'
            AND column_name IN (
                'auto_reminder_incomplete_data',
                'auto_reminder_payment',
                'auto_reminder_checkin',
                'auto_reminder_invoice'
            )
            ORDER BY column_name;
        """)

        columns = cur.fetchall()

        if len(columns) == 0:
            print("[INFO] No auto-reminder columns found. Migration needs to be run.")
            return False
        elif len(columns) == 4:
            print(f"[OK] All 4 auto-reminder columns exist!")
            print("\nColumn details:")
            for col_name, data_type, default, nullable in columns:
                print(f"   - {col_name}:")
                print(f"      Type: {data_type}")
                print(f"      Default: {default}")
                print(f"      Nullable: {nullable}")

            # Check current values
            cur.execute("""
                SELECT
                    auto_reminder_incomplete_data,
                    auto_reminder_payment,
                    auto_reminder_checkin,
                    auto_reminder_invoice
                FROM notification_settings
                WHERE id = 1;
            """)

            row = cur.fetchone()
            if row:
                print("\n[OK] Current values in notification_settings (id=1):")
                print(f"   - auto_reminder_incomplete_data: {row[0]}")
                print(f"   - auto_reminder_payment: {row[1]}")
                print(f"   - auto_reminder_checkin: {row[2]}")
                print(f"   - auto_reminder_invoice: {row[3]}")

            print("\n[SUCCESS] Migration 015 is already applied!")
            return True
        else:
            print(f"[WARNING] Only {len(columns)} of 4 columns found:")
            for col_name, data_type, default, nullable in columns:
                print(f"   - {col_name} ({data_type})")
            return False

    except psycopg2.Error as e:
        print(f"\n[ERROR] Database error: {e}")
        return False

    except Exception as e:
        print(f"\n[ERROR] Error: {e}")
        return False

    finally:
        if 'cur' in locals():
            cur.close()
        if 'conn' in locals():
            conn.close()

if __name__ == "__main__":
    success = check_columns()
    sys.exit(0 if success else 1)
