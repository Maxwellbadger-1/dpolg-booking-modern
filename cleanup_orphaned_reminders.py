#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Cleanup: Remove orphaned reminders (booking_id references non-existent bookings)
This is necessary before Migration 017 can add ON DELETE CASCADE constraint
"""

import psycopg2
import sys

# Database connection parameters
DB_CONFIG = {
    'dbname': 'dpolg_booking',
    'user': 'dpolg_admin',
    'password': 'DPolG2025SecureBooking',
    'host': '141.147.3.123',
    'port': '5432',
    'connect_timeout': 10
}

def cleanup_orphaned_reminders():
    """Find and delete orphaned reminders"""
    print("[Cleanup] Starting: Remove orphaned reminders")
    print("=" * 80)

    try:
        # Connect to database
        conn = psycopg2.connect(**DB_CONFIG)
        conn.autocommit = False
        cur = conn.cursor()

        # Find orphaned reminders
        print("\n[1/3] Finding orphaned reminders...")
        cur.execute("""
            SELECT r.id, r.booking_id, r.reminder_type, r.title
            FROM reminders r
            LEFT JOIN bookings b ON r.booking_id = b.id
            WHERE b.id IS NULL;
        """)

        orphaned = cur.fetchall()

        if not orphaned:
            print("   [OK] No orphaned reminders found!")
            conn.close()
            return True

        print(f"   [FOUND] {len(orphaned)} orphaned reminder(s):")
        for reminder_id, booking_id, reminder_type, title in orphaned:
            print(f"      - Reminder #{reminder_id}: booking_id={booking_id}, type={reminder_type}")
            print(f"        Title: {title}")

        # Delete orphaned reminders
        print(f"\n[2/3] Deleting {len(orphaned)} orphaned reminder(s)...")
        cur.execute("""
            DELETE FROM reminders
            WHERE id IN (
                SELECT r.id
                FROM reminders r
                LEFT JOIN bookings b ON r.booking_id = b.id
                WHERE b.id IS NULL
            );
        """)

        deleted_count = cur.rowcount
        print(f"   [OK] Deleted {deleted_count} orphaned reminder(s)")

        # Verify cleanup
        print("\n[3/3] Verifying cleanup...")
        cur.execute("""
            SELECT COUNT(*)
            FROM reminders r
            LEFT JOIN bookings b ON r.booking_id = b.id
            WHERE b.id IS NULL;
        """)

        remaining = cur.fetchone()[0]
        if remaining > 0:
            raise Exception(f"Cleanup failed! {remaining} orphaned reminders still exist")

        print("   [OK] No orphaned reminders remaining")

        # Commit
        conn.commit()
        print("\n" + "=" * 80)
        print("[SUCCESS] Cleanup completed!")
        print(f"   • {deleted_count} orphaned reminders deleted")
        print("\nYou can now run Migration 017:")
        print("   python run_migration_017.py")
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
    success = cleanup_orphaned_reminders()
    sys.exit(0 if success else 1)
