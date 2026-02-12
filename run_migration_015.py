#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Migration 015: Add auto-reminder flags to notification_settings
Adds 4 boolean columns for auto-reminder configuration
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
    """Execute migration 015"""
    print("[Migration 015] Starting: Add auto-reminder flags to notification_settings")
    print("=" * 80)

    try:
        # Connect to database
        conn = psycopg2.connect(**DB_CONFIG)
        conn.autocommit = False  # Use transaction
        cur = conn.cursor()

        # Read migration file
        migration_file = Path(__file__).parent / 'migrations' / '015_add_auto_reminder_flags.sql'
        if not migration_file.exists():
            raise FileNotFoundError(f"Migration file not found: {migration_file}")

        migration_sql = migration_file.read_text(encoding='utf-8')
        print(f"[OK] Loaded migration from: {migration_file}")
        print()

        # Execute migration
        print("[...] Executing migration...")
        cur.execute(migration_sql)

        # Verify columns were added
        cur.execute("""
            SELECT column_name, data_type, column_default
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
        print("\n[OK] Migration executed successfully!")
        print("\nColumns added:")
        for col_name, data_type, default in columns:
            print(f"   - {col_name}: {data_type}, default: {default}")

        if len(columns) != 4:
            raise Exception(f"Expected 4 columns, but found {len(columns)}")

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
            print("\nCurrent values in notification_settings (id=1):")
            print(f"   - auto_reminder_incomplete_data: {row[0]}")
            print(f"   - auto_reminder_payment: {row[1]}")
            print(f"   - auto_reminder_checkin: {row[2]}")
            print(f"   - auto_reminder_invoice: {row[3]}")

        # Commit transaction
        conn.commit()
        print("\n[OK] Transaction committed successfully!")
        print("=" * 80)
        print("[SUCCESS] Migration 015 completed successfully!")

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
