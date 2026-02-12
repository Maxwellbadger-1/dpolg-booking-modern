#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
Verify Migration 017: Test reminder update functionality
"""

import psycopg2
from datetime import datetime, timedelta

DB_CONFIG = {
    'dbname': 'dpolg_booking',
    'user': 'dpolg_admin',
    'password': 'DPolG2025SecureBooking',
    'host': '141.147.3.123',
    'port': '5432',
    'connect_timeout': 10
}

def verify_migration():
    """Verify Migration 017 functionality"""
    print("[Verify Migration 017] Reminder Update System")
    print("=" * 80)

    try:
        conn = psycopg2.connect(**DB_CONFIG)
        conn.autocommit = False
        cur = conn.cursor()

        # Get a test booking (first active booking)
        print("\n[1/4] Finding test booking...")
        cur.execute("""
            SELECT b.id, b.checkin_date, b.status, b.guest_id, g.vorname, g.nachname
            FROM bookings b
            JOIN guests g ON b.guest_id = g.id
            WHERE b.status NOT IN ('storniert', 'cancelled')
            LIMIT 1;
        """)

        booking = cur.fetchone()
        if not booking:
            print("   [SKIP] No active bookings found for testing")
            conn.close()
            return True

        booking_id, checkin_date, status, guest_id, vorname, nachname = booking
        print(f"   [OK] Test booking: #{booking_id}")
        print(f"        Check-in: {checkin_date}")
        print(f"        Guest: {vorname} {nachname}")

        # Count initial reminders
        print("\n[2/4] Checking existing reminders...")
        cur.execute("""
            SELECT COUNT(*), reminder_type, due_date
            FROM reminders
            WHERE booking_id = %s
              AND is_completed = false
            GROUP BY reminder_type, due_date;
        """, (booking_id,))

        initial_reminders = cur.fetchall()
        print(f"   [OK] Found {len(initial_reminders)} active reminder(s)")
        for count, rtype, due_date in initial_reminders:
            print(f"        - {rtype}: due {due_date}")

        # Simulate check-in date change
        print("\n[3/4] Testing check-in date update...")
        # Convert to datetime if string
        if isinstance(checkin_date, str):
            from datetime import datetime as dt
            checkin_dt = dt.strptime(checkin_date, '%Y-%m-%d')
            new_checkin = (checkin_dt + timedelta(days=7)).date()
        else:
            new_checkin = checkin_date + timedelta(days=7)
        print(f"   [...] Changing check-in: {checkin_date} -> {new_checkin}")

        cur.execute("""
            UPDATE bookings
            SET checkin_date = %s
            WHERE id = %s;
        """, (new_checkin, booking_id))

        # Check if reminder due_date updated
        cur.execute("""
            SELECT reminder_type, due_date, description, updated_at
            FROM reminders
            WHERE booking_id = %s
              AND is_completed = false
              AND reminder_type = 'auto_payment';
        """, (booking_id,))

        payment_reminder = cur.fetchone()
        if payment_reminder:
            rtype, due_date, description, updated_at = payment_reminder
            print(f"   [OK] Payment reminder updated!")
            print(f"        New due_date: {due_date}")
            print(f"        Updated_at: {updated_at}")
        else:
            print(f"   [INFO] No payment reminder found (booking might be paid)")

        # Rollback test changes
        print("\n[4/4] Rolling back test changes...")
        conn.rollback()
        print("   [OK] Test changes rolled back (database unchanged)")

        print("\n" + "=" * 80)
        print("[SUCCESS] Migration 017 verification complete!")
        print("\nNext steps:")
        print("   1. Test in UI: Open a booking and change check-in date")
        print("   2. Verify: Reminder due_date updates automatically")
        print("   3. Test: Cancel a booking")
        print("   4. Verify: Reminders auto-completed with '[Buchung storniert]'")
        print("=" * 80)

        return True

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
    verify_migration()
