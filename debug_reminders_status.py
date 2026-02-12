#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""Debug script to check reminder status"""

import psycopg2

DB_CONFIG = {
    'dbname': 'dpolg_booking',
    'user': 'dpolg_admin',
    'password': 'DPolG2025SecureBooking',
    'host': '141.147.3.123',
    'port': '5432',
    'connect_timeout': 10
}

try:
    conn = psycopg2.connect(**DB_CONFIG)
    cur = conn.cursor()

    # Get total reminders count
    cur.execute("SELECT COUNT(*) FROM reminders;")
    total = cur.fetchone()[0]
    print(f"Total reminders: {total}")

    # Get active reminders count (same as badge count)
    cur.execute("SELECT COUNT(*) FROM reminders WHERE is_completed = FALSE AND is_snoozed = FALSE;")
    active = cur.fetchone()[0]
    print(f"Active reminders (Badge Count): {active}")

    # Get completed reminders count
    cur.execute("SELECT COUNT(*) FROM reminders WHERE is_completed = TRUE;")
    completed = cur.fetchone()[0]
    print(f"Completed reminders: {completed}")

    # Get latest 5 reminders with details
    print("\nLatest 5 reminders:")
    cur.execute("""
        SELECT id, title, is_completed, is_snoozed, created_at::date
        FROM reminders
        ORDER BY id DESC
        LIMIT 5;
    """)

    for row in cur.fetchall():
        reminder_id, title, is_completed, is_snoozed, created_at = row
        status = "COMPLETED" if is_completed else ("SNOOZED" if is_snoozed else "ACTIVE")
        print(f"  #{reminder_id}: {title[:50]} - {status} (created: {created_at})")

    cur.close()
    conn.close()

except Exception as e:
    print(f"Error: {e}")
